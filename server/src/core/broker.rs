use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
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

#[cfg_attr(test, automock)]
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
    async fn publish(&self, publisher_addr: SocketAddr, message: Vec<u8>);
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

    async fn publish(&self, publisher_addr: SocketAddr, message: Vec<u8>) {
        if let Some(topic) = self.clients.read().await.get(&publisher_addr) {
            if let Some(subscribers) = self.subscribers.write().await.get(topic) {
                for sub in subscribers.iter() {
                    if sub.addr == publisher_addr {
                        // skip publishing to the sender
                        continue;
                    }
                    let _ = sub.sender.send(message.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    #[tokio::test]
    async fn new_should_be_returned() {
        let service = MyBrokerService::new();
        assert!(service.clients.read().await.is_empty());
        assert!(service.subscribers.read().await.is_empty());
    }

    #[tokio::test]
    async fn subscribe_should_be_ok() {
        let service = MyBrokerService::new();

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
        let (tx, _) = unbounded_channel::<Vec<u8>>();
        let topic = "t1";
        service
            .subscribe(socket_addr, tx.clone(), topic.to_owned())
            .await;

        {
            let guard = service.subscribers.read().await;
            let subscriber = guard.get_key_value(topic);
            assert!(subscriber.is_some());
            let (topic, sender) = subscriber.unwrap();
            assert_eq!(topic, "t1");
            assert_eq!(sender.len(), 1);
        }
        {
            let guard = service.clients.read().await;
            let client = guard.get_key_value(&socket_addr);
            assert!(client.is_some());
            let (_, topic) = client.unwrap();
            assert_eq!(topic, "t1");
        }
    }

    #[tokio::test]
    async fn unsubscribe_should_be_ok() {
        let service = MyBrokerService::new();

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
        let (tx, _) = unbounded_channel::<Vec<u8>>();
        let topic = "t1";
        service
            .subscribe(socket_addr, tx.clone(), topic.to_owned())
            .await;

        service.unsubscribe(socket_addr).await;

        {
            let guard = service.subscribers.read().await;
            let subscribers = guard.get(topic);
            assert!(subscribers.is_some());
            assert!(subscribers.unwrap().is_empty());
        }
        assert!(service.clients.read().await.is_empty());
    }

    #[tokio::test]
    async fn publish_should_be_ok() {
        let service = MyBrokerService::new();

        let socket_addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
        let socket_addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1112);
        let (tx1, _) = unbounded_channel::<Vec<u8>>();
        let (tx2, mut rx2) = unbounded_channel::<Vec<u8>>();

        let topic = "t1";
        service
            .subscribe(socket_addr1, tx1.clone(), topic.to_owned())
            .await;
        service
            .subscribe(socket_addr2, tx2.clone(), topic.to_owned())
            .await;

        service.publish(socket_addr1, vec![100u8, 110u8]).await;

        let result = rx2.recv().await;
        assert_eq!(result, Some(vec![100u8, 110u8]));
    }
}
