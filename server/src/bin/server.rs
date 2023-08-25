use std::io;
use std::sync::Arc;
use tokio::sync::oneshot;

use server::config::app_config as config;
use server::core::broker::MyBrokerService;
use server::core::cache::reader::MyCacheReader;
use server::core::cache::writer::MyCacheWriter;
use server::core::handler::MyHandlerService;
use server::core::redis::MyRedisService;
use server::core::server::{MyServerService, ServerService};

#[tokio::main]
async fn main() -> io::Result<()> {
    let cache_reader_service = Arc::new(MyCacheReader::new(config::CACHE_FOLDER));
    let cache_writer_service = Arc::new(MyCacheWriter::new(config::CACHE_FOLDER));
    let redis_service = Arc::new(MyRedisService::new(
        cache_reader_service,
        cache_writer_service,
    ));
    let broker_service = Arc::new(MyBrokerService::new());
    let handler_service = Arc::new(MyHandlerService::new(redis_service, broker_service));

    let server_service = MyServerService::new(config::BINDING_ADDRESS, handler_service);

    let (tx, _rx) = oneshot::channel::<u16>();
    server_service.start(tx).await
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
