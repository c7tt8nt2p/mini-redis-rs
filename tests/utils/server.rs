use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;

use server::core::broker::MyBrokerService;
use server::core::cache::reader::MyCacheReader;
use server::core::cache::writer::MyCacheWriter;
use server::core::handler::MyHandlerService;
use server::core::redis::MyRedisService;
use server::core::server::{MyNonSecureServerService, ServerService};

pub fn start_server(address: &str, cache_folder: &str) -> Receiver<u16> {
    let (started_signal_tx, started_signal_rx) = oneshot::channel::<u16>();
    let server_service = new_server(address, cache_folder);
    tokio::spawn(async move {
        server_service.start(started_signal_tx).await.unwrap();
    });
    started_signal_rx
}

fn new_server(address: &str, cache_folder: &str) -> MyNonSecureServerService {
    let cache_reader_service = Arc::new(MyCacheReader::new(cache_folder));
    let cache_writer_service = Arc::new(MyCacheWriter::new(cache_folder));
    let redis_service = Arc::new(MyRedisService::new(
        cache_reader_service,
        cache_writer_service,
    ));
    let broker_service = Arc::new(MyBrokerService::new());
    let handler_service = Arc::new(MyHandlerService::new(redis_service, broker_service));

    MyNonSecureServerService::new(address, handler_service, )
}

pub async fn write_message(writer: &mut WriteHalf<'_>, message: &str) {
    let _ = writer.write_all(message.as_bytes()).await;
}

pub async fn read_message(mut reader: ReadHalf<'_>) -> Vec<u8> {
    let mut buffer = [0u8; 1024];
    let _ = reader
        .read(&mut buffer)
        .await
        .expect("failed to read a message from server");
    buffer.into_iter().filter(|&byte| byte != 0).collect()
}
