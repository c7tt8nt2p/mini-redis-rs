use tokio::io::{stdin, AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use client::core::client::{ClientService, MyNonSecureClientService};

const CONNECTION_HOST: &str = "localhost";
const CONNECTION_PORT: &str = "6973";
const _CERT_FILE_PATH: &str =
    "/Users/chantapat.t/CLionProjects/mini-redis-rs/client/src/config/ssl/client.crt";
const _KEY_FILE_PATH: &str =
    "/Users/chantapat.t/CLionProjects/mini-redis-rs/client/src/config/ssl/client.key";
const DEFAULT_BUFFER_SIZE: usize = 1024;

#[tokio::main]
async fn main() {
    // let client_service = MyClientService::new(CONNECTION_ADDRESS, CERT_FILE_PATH, KEY_FILE_PATH);
    let client_service = MyNonSecureClientService::new(CONNECTION_HOST, CONNECTION_PORT);
    let socket = client_service.connect().await;
    println!("connected to server");
    let (mut reader, mut writer) = socket.into_split();

    tokio::spawn(async move { handle_message_from_server(&mut reader).await });
    handle_message_from_client(&mut writer).await;
}

async fn handle_message_from_server(reader: &mut OwnedReadHalf) {
    loop {
        let mut buffer = [0u8; DEFAULT_BUFFER_SIZE];
        let size = reader.read(&mut buffer).await.unwrap();
        if size == 0 {
            println!("socket is closed");
            break;
        }
        let data: Vec<u8> = buffer.into_iter().filter(|&byte| byte != 0).collect();
        println!(">>> {:?}", data);
    }
}

async fn handle_message_from_client(writer: &mut OwnedWriteHalf) {
    loop {
        let mut buffer = [0u8; DEFAULT_BUFFER_SIZE];
        if let Err(error) = stdin().read(&mut buffer).await {
            eprintln!("error reading from stdin: {}", error);
            let _ = writer.shutdown().await;
        }
        if let Err(error) = writer.write_all(&buffer).await {
            eprintln!("error writing to server: {}", error);
            let _ = writer.shutdown().await;
        }
    }
}
