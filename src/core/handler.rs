use std::sync::Arc;

use async_trait::async_trait;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::WriteHalf;

use crate::core::redis::RedisService;
use crate::core::tlv::{from_tlv, to_tlv, TLVType};

#[async_trait]
pub trait HandlerService: Send + Sync {
    async fn handle_cache_recovering(&self) -> io::Result<()>;

    async fn handle_exit_cmd(&self, writer: WriteHalf<'_>);
    async fn handle_ping_cmd(&self, writer: WriteHalf<'_>);
    async fn handle_ping_value_cmd(&self, writer: WriteHalf<'_>, value: Vec<u8>);
    async fn handle_get_cmd(&self, writer: WriteHalf<'_>, key: &str);
    async fn handle_set_cmd(&self, writer: WriteHalf<'_>, key: String, value: Vec<u8>);
    async fn handle_other_cmd(&self, writer: WriteHalf<'_>);
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
    async fn handle_cache_recovering(&self) -> io::Result<()> {
        self.redis_service.read_cache().await
    }

    async fn handle_exit_cmd(&self, mut writer: WriteHalf<'_>) {
        writer.shutdown().await.unwrap();
    }

    async fn handle_ping_cmd(&self, mut writer: WriteHalf<'_>) {
        writer.write_all(b">pong\n").await.unwrap();
    }

    async fn handle_ping_value_cmd(&self, mut writer: WriteHalf<'_>, value: Vec<u8>) {
        let format = format!(">>> {:?}\n", value);
        writer.write_all(format.as_bytes()).await.unwrap();
    }

    async fn handle_get_cmd(&self, mut writer: WriteHalf<'_>, key: &str) {
        let tlv = self.redis_service.get(key).await;
        if let Some(tlv) = tlv {
            let value = from_tlv(tlv);
            let format = format!(">>> {:?}\n", value);
            writer.write_all(format.as_bytes()).await.unwrap();
        } else {
            writer.write_all(b">>> not found\n").await.unwrap();
        }
    }

    async fn handle_set_cmd(&self, mut writer: WriteHalf<'_>, key: String, value: Vec<u8>) {
        println!("[get] value = {:?}", value);
        let tlv = to_tlv(value, TLVType::String);
        println!("[set] tlv = {:?}", tlv);
        self.redis_service.set(key.clone(), tlv.clone()).await;
        let cache_result = self.redis_service.write_cache(key.clone(), tlv).await;
        if cache_result.is_err() {
            self.redis_service.remove(&key).await;
            eprintln!(
                "error during writing cache: {}",
                cache_result.err().unwrap()
            );
        }

        writer.write_all(b">>> set ok\n").await.unwrap();
    }

    async fn handle_other_cmd(&self, mut writer: WriteHalf<'_>) {
        writer.write_all(b">>> unknown\n").await.unwrap();
    }
}
