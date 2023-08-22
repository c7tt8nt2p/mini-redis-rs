use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::WriteHalf;

use crate::core::redis::RedisService;
use crate::core::tlv::{from_tlv, to_tlv, TLVType};

#[async_trait]
pub trait HandlerService: Send + Sync {
    async fn handle_exit(&self, writer: WriteHalf<'_>);
    async fn handle_ping(&self, writer: WriteHalf<'_>, value: Vec<u8>);
    async fn handle_get(&self, writer: WriteHalf<'_>, key: &str);
    async fn handle_set(&self, writer: WriteHalf<'_>, key: String, value: Vec<u8>);
    async fn handle_other(&self, writer: WriteHalf<'_>);
}

pub struct MyHandlerService {
    redis_service: Arc<dyn RedisService>,
}

impl MyHandlerService {
    pub fn new(redis_service: Arc<dyn RedisService>) -> Self {
        Self { redis_service }
    }
}

#[async_trait]
impl HandlerService for MyHandlerService {
    async fn handle_exit(&self, mut writer: WriteHalf<'_>) {
        writer.shutdown().await.unwrap();
    }

    async fn handle_ping(&self, mut writer: WriteHalf<'_>, value: Vec<u8>) {
        let format = format!(">>> {:?}\n", value);
        writer.write_all(format.as_bytes()).await.unwrap();
    }

    async fn handle_get(&self, mut writer: WriteHalf<'_>, key: &str) {
        let tlv = self.redis_service.get(key).await;
        if let Some(tlv) = tlv {
            let value = from_tlv(tlv);
            let format = format!(">>> {:?}\n", value);
            writer.write_all(format.as_bytes()).await.unwrap();
        }
    }

    async fn handle_set(&self, mut writer: WriteHalf<'_>, key: String, value: Vec<u8>) {
        println!("[get] value = {:?}", value);
        let tlv = to_tlv(value, TLVType::String);
        println!("[set] tlv = {:?}", tlv);
        self.redis_service.set(key, tlv).await;
        writer.write_all(b">>> set ok\n").await.unwrap();
    }

    async fn handle_other(&self, mut writer: WriteHalf<'_>) {
        writer.write_all(b">>> unknown\n").await.unwrap();
    }
}
