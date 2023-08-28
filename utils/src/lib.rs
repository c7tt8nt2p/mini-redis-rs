pub mod cert {

    use rustls_pemfile::{certs, pkcs8_private_keys};
    use std::fs::File;
    use std::io;
    use std::io::BufReader;
    use std::path::Path;
    use tokio_rustls::rustls::{Certificate, PrivateKey};

    pub fn load_cert(path: &Path) -> io::Result<Vec<Certificate>> {
        certs(&mut BufReader::new(File::open(path)?))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
            .map(|mut certs| certs.drain(..).map(Certificate).collect())
    }

    pub fn load_key(path: &Path) -> io::Result<Vec<PrivateKey>> {
        pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
            .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
    }
}
