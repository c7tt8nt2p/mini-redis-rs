use std::io;
use std::sync::Arc;
use tokio::sync::oneshot;

use server::core::broker::MyBrokerService;
use server::core::cache::reader::MyCacheReader;
use server::core::cache::writer::MyCacheWriter;
use server::core::handler::MyHandlerService;
use server::core::redis::MyRedisService;
use server::core::server::{MyNonSecureServerService, ServerService};

const BINDING_HOST: &str = "localhost";
const BINDING_PORT: &str = "6973";
const _CERT_FILE_PATH: &str =
    "/Users/chantapat.t/CLionProjects/mini-redis-rs/server/src/config/ssl/server.crt";
const _KEY_FILE_PATH: &str =
    "/Users/chantapat.t/CLionProjects/mini-redis-rs/server/src/config/ssl/server.key";

const CACHE_FOLDER: &str = "/Users/chantapat.t/CLionProjects/mini-redis-rs/cache";

#[tokio::main]
async fn main() -> io::Result<()> {
    let cache_reader_service = Arc::new(MyCacheReader::new(CACHE_FOLDER));
    let cache_writer_service = Arc::new(MyCacheWriter::new(CACHE_FOLDER));
    let redis_service = Arc::new(MyRedisService::new(
        cache_reader_service,
        cache_writer_service,
    ));
    let broker_service = Arc::new(MyBrokerService::new());
    let handler_service = Arc::new(MyHandlerService::new(redis_service, broker_service));

    let server_service = MyNonSecureServerService::new(BINDING_HOST, BINDING_PORT, handler_service);

    let (tx, _rx) = oneshot::channel::<u16>();
    server_service.start(tx).await
}
