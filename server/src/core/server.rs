
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::{oneshot, Mutex};

use crate::core::handler::HandlerService;
use crate::core::parser::{
    parse_non_subscription_command, parse_subscription_command, NonSubscriptionCmdType,
    SubscriptionCmdType,
};

const DEFAULT_BUFFER_SIZE: usize = 1024;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ServerService: Send + Sync {
    /// started_notify is a oneshot channel tx to notify the receiver with a port that the server is successfully started.
    async fn start(&self, started_signal_tx: oneshot::Sender<u16>) -> io::Result<()>;
}

pub struct MyServerService {
    connection_address: String,
    handler_service: Arc<dyn HandlerService>,
}

impl MyServerService {
    pub fn new(connection_address: &str, handler_service: Arc<dyn HandlerService>) -> Self {
        Self {
            connection_address: connection_address.to_owned(),
            handler_service,
        }
    }
}

#[async_trait]
impl ServerService for MyServerService {
    async fn start(&self, started_signal_tx: oneshot::Sender<u16>) -> io::Result<()> {
        let listener = TcpListener::bind(self.connection_address.to_owned()).await?;
        println!("===============================================================================================");
        self.handler_service.handle_cache_recovering().await?;
        println!("===============================================================================================");
        println!("server started...",);

        let port = listener.local_addr().unwrap().port();
        started_signal_tx.send(port).unwrap();

        loop {
            let (socket, address) = listener.accept().await?;
            println!("[{}] has connected", address);

            let handler_service = Arc::clone(&self.handler_service);
            let (reader, writer) = socket.into_split();
            let reader = Arc::new(Mutex::new(reader));
            let writer = Arc::new(Mutex::new(writer));

            tokio::spawn(async move {
                handle_connection(handler_service, reader, writer, address).await
            });
        }
    }
}

async fn handle_connection(
    handler_service: Arc<dyn HandlerService>,
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    address: SocketAddr,
) {
    let (tx, mut rx) = unbounded_channel::<Vec<u8>>();
    let subscription_writer = Arc::clone(&writer);
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let _ = subscription_writer.lock().await.write_all(&message).await;
        }
        // channel closed
    });

    loop {
        let writer_cloned = Arc::clone(&writer);
        let reader_cloned = Arc::clone(&reader);
        let tx_cloned = tx.clone();

        let Some(read_data) = read(reader_cloned, address.to_string()).await else {
            handler_service.handle_exit_cmd(writer).await;
            handler_service.handle_unsubscribe_cmd(address, writer_cloned).await;
            break;
        };

        let handler_service = Arc::clone(&handler_service);

        let subscription_connection = handler_service.is_subscription_connection(address).await;
        if subscription_connection {
            handle_subscription_connection(handler_service, address, writer_cloned, read_data)
                .await;
        } else {
            handle_non_subscription_connection(
                handler_service,
                tx_cloned,
                writer_cloned,
                read_data,
            )
            .await;
        }
        // print!("\t[{}]: {}", address, String::from_utf8(data).unwrap())
    }
}

async fn read(reader: Arc<Mutex<OwnedReadHalf>>, address: String) -> Option<Vec<u8>> {
    let mut buffer = [0u8; DEFAULT_BUFFER_SIZE];
    let size = reader.lock().await.read(&mut buffer).await.unwrap();
    if size == 0 {
        // client disconnected
        println!("[{}] disconnected", address);
        return None;
    }
    let read_data: Vec<u8> = buffer.into_iter().filter(|&byte| byte != 0).collect();
    Some(read_data)
}

async fn handle_subscription_connection(
    handler_service: Arc<dyn HandlerService>,
    address: SocketAddr,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    data: Vec<u8>,
) {
    let cmd_type = parse_subscription_command(data);
    match cmd_type {
        SubscriptionCmdType::Publish(message) => {
            handler_service.handle_publish_cmd(address, message).await;
        }
        SubscriptionCmdType::Unsubscribe => {
            handler_service
                .handle_unsubscribe_cmd(address, writer)
                .await;
        }
    }
}

async fn handle_non_subscription_connection(
    handler_service: Arc<dyn HandlerService>,
    sender: UnboundedSender<Vec<u8>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    data: Vec<u8>,
) {
    let cmd_type = parse_non_subscription_command(data);
    match cmd_type {
        NonSubscriptionCmdType::Exit => {
            handler_service.handle_exit_cmd(writer).await;
        }
        NonSubscriptionCmdType::Ping => {
            handler_service.handle_ping_cmd(writer).await;
        }
        NonSubscriptionCmdType::PingValue(value) => {
            handler_service.handle_ping_value_cmd(writer, value).await;
        }
        NonSubscriptionCmdType::Get(key) => {
            handler_service.handle_get_cmd(writer, key.as_str()).await;
        }
        NonSubscriptionCmdType::Set(key, value) => {
            handler_service.handle_set_cmd(writer, key, value).await;
        }
        NonSubscriptionCmdType::Subscribe(topic) => {
            handler_service
                .handle_subscribe_cmd(writer, sender, topic)
                .await;
        }
        NonSubscriptionCmdType::Other => {
            handler_service.handle_other_cmd(writer).await;
        }
    }
}
