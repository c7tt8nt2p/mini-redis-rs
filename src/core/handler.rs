use crate::core::redis::RedisService;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::WriteHalf;

#[async_trait]
pub trait HandlerService {
    async fn handle_ping(&self, value: Vec<u8>);
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

    async fn handle_ping(&self, value: Vec<u8>) {
        todo!()
    }
}
