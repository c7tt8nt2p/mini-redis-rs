use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::WriteHalf;
use tokio::sync::Mutex;

use crate::core::redis::RedisService;

#[async_trait]
pub trait HandlerService: Send + Sync {
    async fn handle_exit(&self, writer: WriteHalf<'_>);
    async fn handle_ping(&self, writer: WriteHalf<'_>, value: Vec<u8>);
}

pub struct MyHandlerService {
    redis_service: Arc<Mutex<dyn RedisService + Send>>,
}

impl MyHandlerService {
    pub fn new(redis_service: Arc<Mutex<dyn RedisService + Send>>) -> Self {
        Self { redis_service }
    }
}

#[async_trait]
impl HandlerService for MyHandlerService {
    async fn handle_exit(&self, mut writer: WriteHalf<'_>) {
        writer.shutdown().await.unwrap();
    }

    async fn handle_ping(&self, mut writer: WriteHalf<'_>, value: Vec<u8>) {
        writer.write_all(&value).await.unwrap();
    }
}
