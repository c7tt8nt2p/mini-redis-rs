#[cfg(test)]
use mockall::{automock, mock, predicate::*};
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::core::broker::BrokerService;
use crate::core::redis::RedisService;
use crate::core::tlv::{from_tlv, to_tlv, TLVType};

#[async_trait]
pub trait HandlerService: Send + Sync {
    async fn handle_cache_recovering(&self) -> io::Result<()>;

    async fn handle_exit_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>);
    async fn handle_ping_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>);
    async fn handle_ping_value_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>, value: Vec<u8>);
    async fn handle_get_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>, key: &str);
    async fn handle_set_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>, key: String, value: Vec<u8>);
    async fn handle_subscribe_cmd(
        &self,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        sender: UnboundedSender<Vec<u8>>,
        topic: String,
    );
    async fn handle_other_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>);

    async fn handle_publish_cmd(&self, publisher_addr: SocketAddr, message: Vec<u8>);
    async fn handle_unsubscribe_cmd(
        &self,
        socket_addr: SocketAddr,
        writer_cloned: Arc<Mutex<OwnedWriteHalf>>,
    );

    async fn is_subscription_connection(&self, socket_addr: SocketAddr) -> bool;
}

pub struct MyHandlerService {
    redis_service: Arc<dyn RedisService>,
    broker_service: Arc<dyn BrokerService>,
}

impl MyHandlerService {
    pub fn new(
        redis_service: Arc<dyn RedisService>,
        broker_service: Arc<dyn BrokerService>,
    ) -> Self {
        Self {
            redis_service,
            broker_service,
        }
    }
}

#[async_trait]
impl HandlerService for MyHandlerService {
    async fn handle_cache_recovering(&self) -> io::Result<()> {
        self.redis_service.read_cache().await
    }

    async fn handle_exit_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>) {
        let _ = writer.lock().await.shutdown().await;
    }

    async fn handle_ping_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>) {
        writer.lock().await.write_all(b"pong\n").await.unwrap();
    }

    async fn handle_ping_value_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>, mut value: Vec<u8>) {
        value.push(b'\n');
        writer.lock().await.write_all(&value).await.unwrap();
    }

    async fn handle_get_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>, key: &str) {
        let tlv = self.redis_service.get(key).await;
        if let Some(tlv) = tlv {
            let mut value = from_tlv(tlv);
            value.push(b'\n');
            writer.lock().await.write_all(&value).await.unwrap();
        } else {
            writer.lock().await.write_all(b"not found\n").await.unwrap();
        }
    }

    async fn handle_set_cmd(
        &self,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        key: String,
        value: Vec<u8>,
    ) {
        let tlv = to_tlv(value, TLVType::String);
        self.redis_service.set(key.clone(), tlv.clone()).await;
        let cache_result = self.redis_service.write_cache(key.clone(), tlv).await;
        if cache_result.is_err() {
            self.redis_service.remove(&key).await;
            eprintln!(
                "error during writing cache: {}",
                cache_result.err().unwrap()
            );
        }

        writer.lock().await.write_all(b"set ok\n").await.unwrap();
    }

    async fn handle_subscribe_cmd(
        &self,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        sender: UnboundedSender<Vec<u8>>,
        topic: String,
    ) {
        let socket_addr = writer.lock().await.peer_addr().unwrap();
        self.broker_service
            .subscribe(socket_addr, sender, topic)
            .await;
        writer
            .lock()
            .await
            .write_all(b"subscribed ok\n")
            .await
            .unwrap();
    }

    async fn handle_other_cmd(&self, writer: Arc<Mutex<OwnedWriteHalf>>) {
        writer.lock().await.write_all(b"unknown\n").await.unwrap();
    }

    async fn handle_publish_cmd(&self, publisher_addr: SocketAddr, message: Vec<u8>) {
        self.broker_service.publish(publisher_addr, message).await;
    }

    async fn handle_unsubscribe_cmd(
        &self,
        socket_addr: SocketAddr,
        writer: Arc<Mutex<OwnedWriteHalf>>,
    ) {
        self.broker_service.unsubscribe(socket_addr).await;
        let _ = writer.lock().await.write_all(b"ubnsubscribed ok\n").await;
    }

    async fn is_subscription_connection(&self, socket_addr: SocketAddr) -> bool {
        self.broker_service
            .is_subscription_connection(socket_addr)
            .await
    }
}
