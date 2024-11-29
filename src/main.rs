use chrono::Utc;
use futures::StreamExt;
use mongodb::{Client, Collection, Database};
use std::{
    env,
    sync::{atomic, Arc},
    time::Duration,
};
use tokio::time::sleep;
use warp::Filter;

mod structs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_str("info")?.start()?;
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => log::info!(".env not found, using env variables..."),
    };

    let last_update = Arc::new(atomic::AtomicI64::new(Utc::now().timestamp() / 60));
    let last_update_clone = Arc::clone(&last_update);
    let mongo_url = env::var("MONGO_DETAILS_STRING").expect("MONGO_DETAILS_STRING wasn't set"); //
    let client = Client::with_uri_str(mongo_url).await?;
    let db = &client.database("serverManager");

    log::info!("Starting...");
    last_update.store(Utc::now().timestamp() / 60, atomic::Ordering::Relaxed);

    tokio::spawn(async move {
        let hello = warp::any().map(move || {
            let last_update_i64 = last_update_clone.load(atomic::Ordering::Relaxed);
            let now_minutes = Utc::now().timestamp() / 60;

            // error if 10 minutes without updates
            if (now_minutes - last_update_i64) > 60 {
                warp::reply::with_status(
                    format!("{}", now_minutes - last_update_i64),
                    warp::http::StatusCode::SERVICE_UNAVAILABLE,
                )
            } else {
                warp::reply::with_status(
                    format!("{}", now_minutes - last_update_i64),
                    warp::http::StatusCode::OK,
                )
            }
        });
        warp::serve(hello).run(([0, 0, 0, 0], 3030)).await;
    });

    loop {
        // remove old autobans
        match remove_exclusions_or_ban(db, "tempGlobalBans", "globalBans", "tempautoban").await {
            Ok(_) => {}
            Err(e) => log::error!("tempGlobalBans failed: {:#?}", e),
        };
        match remove_exclusions_or_ban(
            db,
            "tempGlobalExclusions",
            "globalExclusions",
            "tempautoexclusion",
        )
        .await
        {
            Ok(_) => {}
            Err(e) => log::error!("tempGlobalExclusions failed: {:#?}", e),
        };

        last_update.store(Utc::now().timestamp() / 60, atomic::Ordering::Relaxed);
        sleep(Duration::from_secs(1800)).await;
    }
}

async fn remove_exclusions_or_ban(
    db: &Database,
    temp_db: &str,
    normal_db: &str,
    del_type: &str,
) -> anyhow::Result<()> {
    let temp_col: Collection<structs::GlobalTemp> = db.collection(temp_db);
    let mut players = temp_col.find(bson::doc! {}).await?;

    while let Some(player) = players.next().await {
        let player = player?;

        if player.until_time_stamp < Utc::now().into() {
            let normal_col: Collection<structs::Global> = db.collection(normal_db);
            if let Some(mut db_return) = normal_col
                .find_one(bson::doc! {"_id": &player.player_id})
                .await?
            {
                db_return.groups.remove(&player.group_id);

                // update the group if there are still groupid's left
                if !db_return.groups.is_empty() {
                    normal_col
                        .find_one_and_update(
                            bson::doc! {"_id": &player.player_id},
                            bson::doc! {"$set": bson::to_document(&db_return)?},
                        )
                        .await?;
                    // otherwise remove
                } else {
                    normal_col
                        .delete_one(bson::doc! {"_id": &player.player_id})
                        .await?;
                }

                let user_logging_col: Collection<structs::UserLogging> =
                    db.collection("userLogging");
                user_logging_col
                    .insert_one(structs::UserLogging {
                        time_stamp: Utc::now().into(),
                        action: format!("remove-{}", del_type),
                        admin_name: "system".into(),
                        to_player: player.player_name,
                        to_player_id: player.player_id.clone(),
                        in_group: player.group_id.clone(),
                        reason: "time is up for this ban".into(),
                    })
                    .await?;
            }
            temp_col
                .delete_one(bson::doc! { "playerId": player.player_id, "groupId": player.group_id })
                .await?;
        }
    }

    Ok(())
}
