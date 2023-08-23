use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};

use crate::config::app_config::BINDING_ADDRESS;
use crate::core::handler::HandlerService;
use crate::core::parser::{parse_non_subscription_command, NonSubscriptionCmdType};

const DEFAULT_BUFFER_SIZE: usize = 1024;

pub struct ServerService {
    handler_service: Arc<dyn HandlerService>,
}

impl ServerService {
    pub fn new(handler_service: Arc<dyn HandlerService>) -> Self {
        Self { handler_service }
    }

    pub async fn start(&self) -> io::Result<()> {
        let listener = TcpListener::bind(BINDING_ADDRESS).await?;
        println!("===============================================================================================");
        self.handler_service.handle_cache_recovering().await?;
        println!("===============================================================================================");
        println!("server started...");
        loop {
            let (socket, address) = listener.accept().await?;
            println!("[{}] has connected", address);

            let handler_service = Arc::clone(&self.handler_service);
            tokio::spawn(async move { handle_connection(handler_service, socket, address).await });
        }
    }
}

async fn handle_connection(
    handler_service: Arc<dyn HandlerService>,
    mut socket: TcpStream,
    address: SocketAddr,
) {
    loop {
        let (mut reader, writer) = socket.split();
        let Some(raw_data) = read(&mut reader, address.to_string()).await else { break; };
        let read_data: Vec<u8> = raw_data.into_iter().filter(|&byte| byte != 0).collect();

        let handler_service = Arc::clone(&handler_service);
        handle_non_subscription_connection(handler_service, writer, read_data).await;
        // print!("\t[{}]: {}", address, String::from_utf8(data).unwrap())
    }
}

async fn read(reader: &mut ReadHalf<'_>, address: String) -> Option<[u8; DEFAULT_BUFFER_SIZE]> {
    let mut buffer = [0u8; DEFAULT_BUFFER_SIZE];
    let size = reader.read(&mut buffer).await.unwrap();
    if size == 0 {
        println!("[{}] disconnected", address);
        return None;
    }
    Some(buffer)
}

async fn handle_non_subscription_connection(
    handler_service: Arc<dyn HandlerService>,
    writer: WriteHalf<'_>,
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
        NonSubscriptionCmdType::Subscribe => {}
        NonSubscriptionCmdType::Other => {
            handler_service.handle_other_cmd(writer).await;
        }
    }
}
