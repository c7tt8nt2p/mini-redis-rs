use async_trait::async_trait;
use tokio::net::TcpStream;

#[async_trait]
pub trait ClientService: Send + Sync {
    async fn connect(&self) -> TcpStream;
}

pub struct MyClientService {
    connection_address: String,
}

impl MyClientService {
    pub fn new(connection_address: &str) -> Self {
        Self {
            connection_address: connection_address.to_owned(),
        }
    }
}

#[async_trait]
impl ClientService for MyClientService {
    async fn connect(&self) -> TcpStream {
        TcpStream::connect(self.connection_address.clone())
            .await
            .unwrap_or_else(|_| panic!("unable to connect to: {}", self.connection_address))
    }
}
