use std::io;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::core::handler::{HandlerService, MyHandlerService};
use crate::core::redis::MyRedisService;
use crate::core::server::ServerService;

mod client;
#[path = "../config/mod.rs"]
mod config;
#[path = "../core/mod.rs"]
mod core;

#[tokio::main]
async fn main() -> io::Result<()> {
    let redis_service = Arc::new(MyRedisService::new());
    let handler_service = Arc::new(MyHandlerService::new(redis_service));

    let server_service = ServerService::new(handler_service);

    server_service.start().await
}
// =================================================================
// =================================================================

#[cfg(test)]
mod tests {
    // #[tokio::test]
    // async fn ping_pong_without_message() {
    //     let mut client = Client::connect("localhost:6973").await;
    //     let response = client.ping(None).await.unwrap();
    //     assert_eq!(response.as_bytes(), config"PONG");
    // }

    // #[tokio::test]
    // async fn ping_pong_with_message() {
    //     let mut client = Client::connect("localhost:6973").await;
    //     let response = client.ping(Some("你好世界")).await.unwrap();
    //     assert_eq!(response.as_bytes(), "你好世界".as_bytes());
    // }
}
