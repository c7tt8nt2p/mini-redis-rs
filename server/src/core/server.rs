use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{Mutex, oneshot};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use crate::core::handler::{HandlerService, MyHandlerService};
use crate::core::parser::{
    NonSubscriptionCmdType, parse_non_subscription_command, parse_subscription_command,
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
    binding_host: String,
    binding_port: String,
    cert_file_path: String,
    key_file_path: String,
    handler_service: Arc<dyn HandlerService>,
}

impl MyServerService {
    pub fn new(
        binding_host: &str,
        binding_port: &str,
        cert_file_path: &str,
        key_file_path: &str,
        handler_service: Arc<dyn HandlerService>,
    ) -> Self {
        Self {
            binding_host: binding_host.to_owned(),
            binding_port: binding_port.to_owned(),
            cert_file_path: cert_file_path.to_owned(),
            key_file_path: key_file_path.to_owned(),
            handler_service,
        }
    }

    fn load_tls_config(cert_file_path: &str, key_file_path: &str) -> io::Result<ServerConfig> {
        let certs = utils::cert::load_cert(Path::new(cert_file_path))?;
        let mut keys = utils::cert::load_key(Path::new(key_file_path))?;
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, keys.remove(0))
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
        Ok(config)
    }

    async fn accept_new_connection(
        listener: &TcpListener,
        tls_acceptor: TlsAcceptor,
    ) -> Option<(TcpStream, SocketAddr)> {
        let (tls_socket, address) = match Self::accept_as_tls(listener, tls_acceptor).await {
            Ok((tls_socket, address)) => (tls_socket, address),
            Err(err) => {
                println!("failed to accept a new connection: {:?}", err);
                return None;
            }
        };
        Some((tls_socket, address))
    }

    async fn accept_as_tls(
        listener: &TcpListener,
        acceptor: TlsAcceptor,
    ) -> io::Result<(TcpStream, SocketAddr)> {
        let (socket, address) = listener.accept().await?;
        let stream = acceptor.accept(socket).await?;
        let (tls_socket, _server_connection) = stream.into_inner();
        Ok((tls_socket, address))
    }
}

pub struct MyNonSecureServerService {
    binding_host: String,
    binding_port: String,
    handler_service: Arc<dyn HandlerService>,
}

impl MyNonSecureServerService {
    pub fn new(
        binding_host: &str,
        binding_port: &str,
        handler_service: Arc<MyHandlerService>,
    ) -> Self {
        Self {
            binding_host: binding_host.to_owned(),
            binding_port: binding_port.to_owned(),
            handler_service,
        }
    }

    async fn accept_new_connection(listener: &TcpListener) -> Option<(TcpStream, SocketAddr)> {
        let (socket, address) = match listener.accept().await {
            Ok((tls_socket, address)) => (tls_socket, address),
            Err(err) => {
                println!("failed to accept a new connection: {:?}", err);
                return None;
            }
        };
        Some((socket, address))
    }
}

#[async_trait]
impl ServerService for MyServerService {
    async fn start(&self, started_signal_tx: oneshot::Sender<u16>) -> io::Result<()> {
        let config = Self::load_tls_config(&self.cert_file_path, &self.key_file_path)?;
        let tls_acceptor = TlsAcceptor::from(Arc::new(config));

        let address = format!("{}:{}", self.binding_host, self.binding_port);
        let listener = TcpListener::bind(address.clone()).await?;
        println!("===============================================================================================");
        self.handler_service.handle_cache_recovering().await?;
        println!("===============================================================================================");
        println!("server started...",);

        let port = listener.local_addr().unwrap().port();
        started_signal_tx.send(port).unwrap();

        loop {
            let tls_acceptor = tls_acceptor.clone();
            let Some((tls_socket, address)) = Self::accept_new_connection(&listener, tls_acceptor).await else { continue} ;
            println!("[{}] has connected", address);

            let handler_service = Arc::clone(&self.handler_service);
            let (reader, writer) = tls_socket.into_split();
            let reader = Arc::new(Mutex::new(reader));
            let writer = Arc::new(Mutex::new(writer));

            tokio::spawn(async move {
                handle_connection(handler_service, reader, writer, address).await
            });
        }
    }
}

#[async_trait]
impl ServerService for MyNonSecureServerService {
    async fn start(&self, started_signal_tx: oneshot::Sender<u16>) -> io::Result<()> {
        let address = format!("{}:{}", self.binding_host, self.binding_port);
        let listener = TcpListener::bind(address.clone()).await?;
        println!("===============================================================================================");
        self.handler_service.handle_cache_recovering().await?;
        println!("===============================================================================================");
        println!("server started...",);

        let port = listener.local_addr().unwrap().port();
        started_signal_tx.send(port).unwrap();

        loop {
            let Some((socket, address)) = Self::accept_new_connection(&listener).await else { continue} ;
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
            let _ = handler_service.handle_exit_cmd(writer).await;
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
            let _ = handler_service.handle_exit_cmd(writer).await;
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
            let peer_addr = writer.lock().await.peer_addr().unwrap();
            handler_service
                .handle_subscribe_cmd(writer, sender, peer_addr, topic)
                .await;
        }
        NonSubscriptionCmdType::Other => {
            handler_service.handle_other_cmd(writer).await;
        }
    }
}
