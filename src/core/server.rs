use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};

use crate::config::app_config::BINDING_ADDRESS;
use crate::core::handler::HandlerService;
use crate::core::parser::{parse_non_subscription_command, NonSubscriptionCmdType};

const DEFAULT_BUFFER_SIZE: usize = 1024;
type DataBuffer = [u8; DEFAULT_BUFFER_SIZE];

pub struct ServerService {
    handler_service: Arc<Mutex<dyn HandlerService + Send>>,
}

impl ServerService {
    pub fn new(handler_service: Arc<Mutex<dyn HandlerService + Send>>) -> Self {
        Self { handler_service }
    }

    pub async fn start(&self) -> io::Result<()> {
        let listener = TcpListener::bind(BINDING_ADDRESS).await?;
        println!("[Server] started...");
        loop {
            let (socket, address) = listener.accept().await?;
            println!("[Server] accepted a new connection: {}", address);

            let handler_service = Arc::clone(&self.handler_service);
            tokio::spawn(async move { handle_connection(handler_service, socket, address).await });
        }
    }
}

async fn handle_connection(
    handler_service: Arc<Mutex<dyn HandlerService + Send>>,
    mut socket: TcpStream,
    address: SocketAddr,
) {
    loop {
        let (mut reader, writer) = socket.split();
        let Some(raw_data) = read(&mut reader).await else { break; };

        // writer.write_all(&data).await.unwrap()
        let handler_service = Arc::clone(&handler_service);
        let data: Vec<u8> = raw_data.into_iter().filter(|&byte| byte != 0).collect();
        println!("data: {:?}", data);
        handle_non_subscription_connection(handler_service, writer, data).await;
        // print!("\t[{}]: {}", address, String::from_utf8(data).unwrap())
    }
}

async fn read(reader: &mut ReadHalf<'_>) -> Option<DataBuffer> {
    let mut buffer: DataBuffer = [0u8; DEFAULT_BUFFER_SIZE];
    let size = reader.read(&mut buffer).await.unwrap();
    if size == 0 {
        // client left
        return None;
    }
    Some(buffer)
}

async fn handle_non_subscription_connection(
    handler_service: Arc<Mutex<dyn HandlerService + Send>>,
    mut writer: WriteHalf<'_>,
    data: Vec<u8>,
) {
    let cmd_type = parse_non_subscription_command(data);
    match cmd_type {
        NonSubscriptionCmdType::Exit => {
            writer.shutdown().await.unwrap();
        }
        NonSubscriptionCmdType::Ping(value) => {}
        NonSubscriptionCmdType::Set => {}
        NonSubscriptionCmdType::Get => {}
        NonSubscriptionCmdType::Subscribe => {}
        NonSubscriptionCmdType::Other => {}
    }
}
