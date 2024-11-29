FROM rust:1.80 AS builder
WORKDIR /usr/src/myapp
COPY . .
ARG github_token 
RUN git config --global credential.helper store && echo "https://zefanjajobse:${github_token}@github.com" > ~/.git-credentials && cargo install --path .

FROM debian:bookworm-slim

EXPOSE 3030

HEALTHCHECK --interval=5m --timeout=3s --start-period=5s \
    CMD curl -f http://127.0.0.1:3030/ || exit 1

COPY --from=builder /usr/local/cargo/bin/global-bans-rust /usr/local/bin/global-bans-rust
RUN apt-get update && apt-get install --assume-yes curl && apt-get clean
CMD ["global-bans-rust"]
