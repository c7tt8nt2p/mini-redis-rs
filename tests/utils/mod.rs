use tempdir::TempDir;
use tokio::net::TcpStream;

use ::client::core::client::ClientService;

use crate::utils::client::new_client;

pub mod client;
pub mod file;
pub mod server;

pub async fn start_server(address: &str, temp_dir: &TempDir) -> u16 {
    let temp_dir = temp_dir.path().display().to_string();
    let rx = server::start_server(address, &temp_dir);
    rx.await.unwrap()
}

pub async fn start_client(port: u16) -> TcpStream {
    let client = new_client(&format!("localhost:{}", port));
    client.connect().await
}
