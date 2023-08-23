use std::sync::Arc;

use mini_redis_rust::config;
use mini_redis_rust::core::cache::reader::MyCacheReader;
use mini_redis_rust::core::cache::writer::MyCacheWriter;
use mini_redis_rust::core::handler::MyHandlerService;
use mini_redis_rust::core::redis::MyRedisService;
use mini_redis_rust::core::server::{MyServerService, ServerService};

fn new_server() -> MyServerService {
    let cache_reader_service = Arc::new(MyCacheReader::new(
        config::app_config::CACHE_FOLDER.to_owned(),
    ));
    let cache_writer_service = Arc::new(MyCacheWriter::new(
        config::app_config::CACHE_FOLDER.to_owned(),
    ));
    let redis_service = Arc::new(MyRedisService::new(
        cache_reader_service,
        cache_writer_service,
    ));
    let handler_service = Arc::new(MyHandlerService::new(redis_service));

    MyServerService::new(handler_service)
}

#[tokio::test]
async fn ping_should_return_pong() {
    let server = new_server();
    server.start().await.unwrap();
}
