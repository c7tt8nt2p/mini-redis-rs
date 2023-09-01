use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::core::broker::BrokerService;
use crate::core::redis::RedisService;
use crate::core::tlv::{from_tlv, TLVType, to_tlv};

#[async_trait]
pub trait HandlerService: Send + Sync {
    async fn handle_cache_recovering(&self) -> io::Result<()>;

    async fn handle_exit_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
    ) -> io::Result<()>;
    async fn handle_ping_cmd(&self, writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>);
    async fn handle_ping_value_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        value: Vec<u8>,
    );
    async fn handle_get_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        key: &str,
    );
    async fn handle_set_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        key: String,
        value: Vec<u8>,
    );
    async fn handle_subscribe_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        sender: UnboundedSender<Vec<u8>>,
        topic: SocketAddr,
        topic0: String,
    );
    async fn handle_other_cmd(&self, writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>);

    async fn handle_publish_cmd(&self, publisher_addr: SocketAddr, message: Vec<u8>);
    async fn handle_unsubscribe_cmd(
        &self,
        socket_addr: SocketAddr,
        writer_cloned: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
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

    async fn handle_exit_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
    ) -> io::Result<()> {
        writer.lock().await.shutdown().await
    }

    async fn handle_ping_cmd(&self, writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>) {
        writer.lock().await.write_all(b"pong\n").await.unwrap();
    }

    async fn handle_ping_value_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        mut value: Vec<u8>,
    ) {
        value.push(b'\n');
        writer.lock().await.write_all(&value).await.unwrap();
    }

    async fn handle_get_cmd(
        &self,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        key: &str,
    ) {
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
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
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
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
        sender: UnboundedSender<Vec<u8>>,
        socket_addr: SocketAddr,
        topic: String,
    ) {
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

    async fn handle_other_cmd(&self, writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>) {
        writer.lock().await.write_all(b"unknown\n").await.unwrap();
    }

    async fn handle_publish_cmd(&self, publisher_addr: SocketAddr, message: Vec<u8>) {
        self.broker_service.publish(publisher_addr, message).await;
    }

    async fn handle_unsubscribe_cmd(
        &self,
        socket_addr: SocketAddr,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
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

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::{Error, ErrorKind};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use mockall::mock;
    use mockall::predicate::eq;
    use tokio::io::AsyncWrite;
    use tokio::sync::Mutex;

    use crate::core::broker::MockBrokerService;
    use crate::core::handler::{HandlerService, MyHandlerService};
    use crate::core::redis::MockRedisService;

    fn mock_deps() -> (MockRedisService, MockBrokerService) {
        (MockRedisService::new(), MockBrokerService::new())
    }

    fn new_instance(
        redis_service: Arc<MockRedisService>,
        broker_service: Arc<MockBrokerService>,
    ) -> MyHandlerService {
        MyHandlerService::new(redis_service, broker_service)
    }

    mock! {
        pub MyAsyncWriter {}

        impl AsyncWrite for MyAsyncWriter {
            fn poll_write<'a>(self: Pin<&mut Self>,cx: &mut Context<'a>,buf: &[u8]) -> Poll<Result<usize, io::Error>>;
            fn poll_flush<'a>(self: Pin<&mut Self>,cx: &mut Context<'a>) -> Poll<Result<(), io::Error>>;
            fn poll_shutdown<'a>(self: Pin<&mut Self>,cx: &mut Context<'a>) -> Poll<Result<(), io::Error>>;
        }
    }

    #[tokio::test]
    async fn handle_cache_recovering_should_be_handled() {
        let (mut redis_service, broker_service) = mock_deps();
        redis_service
            .expect_read_cache()
            .once()
            .returning(|| Ok(()));
        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));

        let result = instance.handle_cache_recovering().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handle_exit_cmd_should_be_handled() {
        let (redis_service, broker_service) = mock_deps();
        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));
        let mut writer = MockMyAsyncWriter::new();

        writer
            .expect_poll_shutdown()
            .once()
            .returning(|_| Poll::Ready(Ok(())));

        let result = instance.handle_exit_cmd(Arc::new(Mutex::new(writer))).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handle_ping_cmd_should_be_handled() {
        let (redis_service, broker_service) = mock_deps();
        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));
        let mut writer = MockMyAsyncWriter::new();

        writer
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Ok(1usize)));

        instance.handle_ping_cmd(Arc::new(Mutex::new(writer))).await;
    }

    #[tokio::test]
    async fn handle_ping_value_cmd_should_be_handled() {
        let (redis_service, broker_service) = mock_deps();
        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));
        let mut writer = MockMyAsyncWriter::new();

        writer
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Ok(1usize)));
        let value = vec![111u8, 122u8];

        instance
            .handle_ping_value_cmd(Arc::new(Mutex::new(writer)), value)
            .await;
    }

    #[tokio::test]
    async fn handle_set_cmd_should_be_handled_when_cache_ok() {
        let (mut redis_service, broker_service) = mock_deps();

        redis_service
            .expect_set()
            .with(
                eq("key1".to_owned()),
                eq(vec![1, 0, 0, 0, 0, 0, 0, 0, 2, 55, 66]),
            )
            .once()
            .returning(|_, _| ());
        redis_service
            .expect_write_cache()
            .with(
                eq("key1".to_owned()),
                eq(vec![1, 0, 0, 0, 0, 0, 0, 0, 2, 55, 66]),
            )
            .once()
            .returning(|_, _| Ok(()));

        redis_service.expect_remove().never();

        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));
        let mut writer = MockMyAsyncWriter::new();
        writer
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Ok(1usize)));

        instance
            .handle_set_cmd(
                Arc::new(Mutex::new(writer)),
                "key1".to_owned(),
                vec![55u8, 66u8],
            )
            .await;
    }

    #[tokio::test]
    async fn handle_set_cmd_should_be_handled_when_cache_err() {
        let (mut redis_service, broker_service) = mock_deps();

        redis_service
            .expect_set()
            .with(
                eq("key1".to_owned()),
                eq(vec![1, 0, 0, 0, 0, 0, 0, 0, 2, 44, 45]),
            )
            .once()
            .returning(|_, _| ());
        redis_service
            .expect_write_cache()
            .with(
                eq("key1".to_owned()),
                eq(vec![1, 0, 0, 0, 0, 0, 0, 0, 2, 44, 45]),
            )
            .once()
            .returning(|_, _| Err(Error::new(ErrorKind::Other, "Other")));

        redis_service
            .expect_remove()
            .with(eq("key1".to_owned()))
            .once()
            .returning(|_| ());

        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));
        let mut writer = MockMyAsyncWriter::new();
        writer
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Ok(1usize)));

        instance
            .handle_set_cmd(
                Arc::new(Mutex::new(writer)),
                "key1".to_owned(),
                vec![44u8, 45u8],
            )
            .await;
    }

    #[tokio::test]
    async fn handle_publish_cmd_should_be_handled() {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
        let (redis_service, mut broker_service) = mock_deps();
        broker_service
            .expect_publish()
            .with(eq(socket_addr), eq(vec![77u8, 88u8]))
            .once()
            .returning(|_, _| ());

        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));

        instance
            .handle_publish_cmd(socket_addr, vec![77u8, 88u8])
            .await;
    }

    #[tokio::test]
    async fn handle_unsubscribe_cmd_should_be_handled() {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
        let (redis_service, mut broker_service) = mock_deps();
        broker_service
            .expect_unsubscribe()
            .with(eq(socket_addr))
            .once()
            .returning(|_| ());

        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));
        let mut writer = MockMyAsyncWriter::new();
        writer
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Ok(1usize)));

        instance
            .handle_unsubscribe_cmd(socket_addr, Arc::new(Mutex::new(writer)))
            .await;
    }

    #[tokio::test]
    async fn is_subscription_connection_should_be_returned() {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
        let (redis_service, mut broker_service) = mock_deps();
        broker_service
            .expect_is_subscription_connection()
            .with(eq(socket_addr))
            .once()
            .returning(|_| true);

        let instance = new_instance(Arc::new(redis_service), Arc::new(broker_service));

        let result = instance.is_subscription_connection(socket_addr).await;

        assert!(result)
    }
}
