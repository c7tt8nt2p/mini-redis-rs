use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

#[derive(Debug)]
struct Subscriber {
    addr: SocketAddr,
    sender: UnboundedSender<Vec<u8>>,
}

impl Subscriber {
    pub fn new(addr: SocketAddr, sender: UnboundedSender<Vec<u8>>) -> Self {
        Self { addr, sender }
    }
}

impl PartialEq for Subscriber {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

#[async_trait]
pub trait BrokerService: Send + Sync {
    async fn is_subscription_connection(&self, socket_addr: SocketAddr) -> bool;
    async fn subscribe(
        &self,
        socket_addr: SocketAddr,
        sender: UnboundedSender<Vec<u8>>,
        topic: String,
    );
    async fn unsubscribe(&self, socket_addr: SocketAddr);
    async fn publish(&self, socket_addr: SocketAddr, message: Vec<u8>);
}

pub struct MyBrokerService {
    clients: Arc<RwLock<HashMap<SocketAddr, String>>>,
    subscribers: Arc<RwLock<HashMap<String, Vec<Subscriber>>>>,
}

impl MyBrokerService {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl BrokerService for MyBrokerService {
    async fn is_subscription_connection(&self, socket_addr: SocketAddr) -> bool {
        self.clients.read().await.contains_key(&socket_addr)
    }

    async fn subscribe(
        &self,
        socket_addr: SocketAddr,
        sender: UnboundedSender<Vec<u8>>,
        topic: String,
    ) {
        let subscriber = Subscriber::new(socket_addr, sender);
        self.subscribers
            .write()
            .await
            .entry(topic.clone())
            .or_default()
            .push(subscriber);

        self.clients.write().await.insert(socket_addr, topic);
    }

    async fn unsubscribe(&self, socket_addr: SocketAddr) {
        if let Some(topic) = self.clients.write().await.remove(&socket_addr) {
            if let Some(subscribers) = self.subscribers.write().await.get_mut(&topic) {
                // remove a subscriber from a topic
                subscribers.retain(|s| s.addr != socket_addr);
            }
        }
    }

    async fn publish(&self, socket_addr: SocketAddr, message: Vec<u8>) {
        if let Some(topic) = self.clients.read().await.get(&socket_addr) {
            if let Some(subscribers) = self.subscribers.write().await.get(topic) {
                for sub in subscribers.iter() {
                    let _ = sub.sender.send(message.clone());
                }
            }
        }
    }
}
