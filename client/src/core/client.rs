use std::io;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio_rustls::{rustls, TlsConnector};
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore};

#[async_trait]
pub trait ClientService: Send + Sync {
    async fn connect(&self) -> TcpStream;
}

pub struct MyClientService {
    connection_host: String,
    connection_port: String,
    cert_file_path: String,
    key_file_path: String,
}

impl MyClientService {
    pub fn new(
        connection_host: &str,
        connection_port: &str,
        cert_file_path: &str,
        key_file_path: &str,
    ) -> Self {
        Self {
            connection_host: connection_host.to_owned(),
            connection_port: connection_port.to_owned(),
            cert_file_path: cert_file_path.to_owned(),
            key_file_path: key_file_path.to_owned(),
        }
    }

    fn load_tls_config(cert_file_path: &str, key_file_path: &str) -> ClientConfig {
        let root_cert_store = Self::get_default_root_ca();
        let certs = utils::cert::load_cert(Path::new(cert_file_path)).unwrap();
        let mut keys = utils::cert::load_key(Path::new(key_file_path)).unwrap();
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_client_auth_cert(certs, keys.remove(0))
            .unwrap()
    }

    fn get_default_root_ca() -> RootCertStore {
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));
        root_cert_store
    }
}

pub struct MyNonSecureClientService {
    connection_host: String,
    connection_port: String,
}

impl MyNonSecureClientService {
    pub fn new(connection_host: &str, connection_port: &str) -> Self {
        Self {
            connection_host: connection_host.to_owned(),
            connection_port: connection_port.to_owned(),
        }
    }
}

#[async_trait]
impl ClientService for MyClientService {
    async fn connect(&self) -> TcpStream {
        let config = Self::load_tls_config(&self.cert_file_path, &self.key_file_path);
        let connector = TlsConnector::from(Arc::new(config));

        let address = format!("{}:{}", self.connection_host, self.connection_port);
        let socket = TcpStream::connect(address.clone())
            .await
            .unwrap_or_else(|_| panic!("unable to connect to: {}", address));

        let domain = rustls::ServerName::try_from(self.connection_host.as_str())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))
            .unwrap();
        let tls_socket = connector.connect(domain, socket).await.unwrap();
        let (stream, _client_connection) = tls_socket.into_inner();
        stream
    }
}

#[async_trait]
impl ClientService for MyNonSecureClientService {
    async fn connect(&self) -> TcpStream {
        let address = format!("{}:{}", self.connection_host, self.connection_port);
        TcpStream::connect(address.clone())
            .await
            .unwrap_or_else(|_| panic!("unable to connect to: {}", address))
    }
}
