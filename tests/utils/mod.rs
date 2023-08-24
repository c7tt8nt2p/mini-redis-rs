use tempdir::TempDir;
use tokio::net::TcpStream;

use ::client::core::client::ClientService;

use crate::utils::client::new_client;

pub mod client;
pub mod server;
pub mod file;

pub async fn start_server_client(address: &str, temp_dir: &TempDir) -> TcpStream {
    let temp_dir = temp_dir.path().display().to_string();
    let rx = server::start_server(address, &temp_dir);

    let port = rx.await.expect("server failed to start");
    let client = new_client(format!("localhost:{}", port).as_str());

    client.connect().await
}