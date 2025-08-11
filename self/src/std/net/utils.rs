use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use std::net::TcpStream;
use std::sync::Arc;

pub fn tls(host_with_port: &str) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
    let (domain, _) = host_with_port
        .rsplit_once(':')
        .ok_or("host must include the port")?;

    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let cfg = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );

    // tcp + tls
    let tcp = TcpStream::connect(host_with_port).map_err(|e| e.to_string())?;
    let sn = ServerName::try_from(domain.to_owned()).map_err(|e| e.to_string())?;
    let conn = ClientConnection::new(cfg, sn).map_err(|e| e.to_string())?;
    Ok(StreamOwned::new(conn, tcp))
}
