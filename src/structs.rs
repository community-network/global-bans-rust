use std::collections::HashMap;

use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalTemp {
    pub _id: String,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "playerId")]
    pub player_id: String,
    #[serde(rename = "playerName")]
    pub player_name: String,
    #[serde(rename = "untilTimeStamp")]
    pub until_time_stamp: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalGroup {
    pub reason: String,
    pub user: String,
    pub id: i64,
    #[serde(rename = "timeStamp")]
    pub time_stamp: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Global {
    pub _id: String,
    pub groups: HashMap<String, GlobalGroup>,
    #[serde(rename = "playerName")]
    pub player_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserLogging {
    #[serde(rename = "timeStamp")]
    pub time_stamp: DateTime,
    pub action: String,
    #[serde(rename = "adminName")]
    pub admin_name: String,
    #[serde(rename = "toPlayer")]
    pub to_player: String,
    #[serde(rename = "toPlayerId")]
    pub to_player_id: String,
    #[serde(rename = "inGroup")]
    pub in_group: String,
    pub reason: String,
}
