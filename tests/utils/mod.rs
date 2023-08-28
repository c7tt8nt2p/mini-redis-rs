use tempdir::TempDir;
use tokio::net::TcpStream;

use ::client::core::client::ClientService;

use crate::utils::client::new_client;

pub mod client;
pub mod file;
pub mod server;

pub const TEST_CONNECTION_HOST: &str = "localhost";
pub const TEST_CONNECTION_PORT: &str = "0";

pub async fn start_server(host: &str, port: &str, temp_dir: &TempDir) -> u16 {
    let temp_dir = temp_dir.path().display().to_string();
    let rx = server::start_server(host, port, &temp_dir);
    rx.await.unwrap()
}

pub async fn start_client(port: u16) -> TcpStream {
    let client = new_client("localhost", &port.to_string());
    client.connect().await
}
