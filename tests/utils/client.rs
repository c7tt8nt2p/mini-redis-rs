use tokio::io::AsyncReadExt;
use tokio::net::tcp::ReadHalf;

use client::core::client::MyNonSecureClientService;

pub fn new_client(address: &str) -> MyNonSecureClientService {
    MyNonSecureClientService::new(address)
}

pub async fn read_message(reader: &mut ReadHalf<'_>) -> Vec<u8> {
    let mut buffer = [0u8; 1024];
    let _ = reader
        .read(&mut buffer)
        .await
        .expect("failed to read a message from server");
    buffer.into_iter().filter(|&byte| byte != 0).collect()
}
