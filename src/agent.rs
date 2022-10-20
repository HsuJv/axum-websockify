use axum::extract::ws::{Message, WebSocket};
use log::*;
#[cfg(feature = "ssl")]
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
#[cfg(feature = "ssl")]
use tokio_rustls::{
    rustls::{
        client::{self, ServerCertVerifier},
        Certificate, ClientConfig, RootCertStore, ServerName,
    },
    TlsConnector,
};

use crate::config::get_dst_addr;

#[cfg(feature = "ssl")]
struct NoCertVerifier {}
#[cfg(feature = "ssl")]
impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<client::ServerCertVerified, tokio_rustls::rustls::Error> {
        Ok(client::ServerCertVerified::assertion())
    }
}

#[cfg(feature = "ssl")]
async fn tls_setup() -> TlsConnector {
    let root_store = RootCertStore::empty();
    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoCertVerifier {}));

    let rc_config = Arc::new(config);
    TlsConnector::from(rc_config)
}

pub async fn handle_client(mut client: WebSocket) {
    let dst = get_dst_addr();
    let server_stream = TcpStream::connect(dst).await;
    info!("Connect to remote {:#?}", server_stream);

    if server_stream.is_err() {
        error!("Connect to remote failed {:#?}", server_stream);
        let _ = client
            .send(Message::Text(format!("{:#?}", server_stream)))
            .await;
        return;
    }

    let mut server_stream = server_stream.unwrap();

    let mut buf = [0u8; 16384]; // the max ssl record should be 16384 by default

    loop {
        tokio::select! {
            res  = client.recv() => {
                if let Some(msg) = res {
                    #[cfg(not(feature = "ssl"))]
                    if let Ok(Message::Binary(msg)) = msg {
                        let _ = server_stream.write_all(&msg).await;
                    }
                    #[cfg(feature = "ssl")]
                    match msg {
                        Ok(Message::Binary(msg)) => {
                            let _ = server_stream.write_all(&msg).await;
                        }
                        Ok(Message::Text(msg)) => {
                            info!("Get {}", msg);
                            if msg == "SSL" {
                                break;
                            } else {
                                error!("Unknow client msg {}", msg);
                            }
                        }
                        _ => {
                            error!("Error type");
                            return
                        }
                    }
                } else {
                    info!("Client close");
                    return;
                }
            },
            res  = server_stream.read(&mut buf) => {
                match res {
                    Ok(n) => {
                        info!("Recv {}", n);
                        if 0 != n {
                            debug!("Recv {}", n);
                            let _ = client.send(Message::Binary(buf[..n].to_vec())).await;
                        } else {
                            return ;
                        }
                    },
                    Err(e) => {
                        info!("Server close with err {:?}", e);
                        return;
                    }
                }

            },
        }
    }
    #[cfg(feature = "ssl")]
    {
        let tls = tls_setup().await;
        let stream = tls
            .connect("not.in.use.google.com".try_into().unwrap(), server_stream)
            .await
            .unwrap();
        let peer_cert = stream.get_ref().1.peer_certificates().unwrap();
        let _ = client
            .send(Message::Binary(peer_cert[0].as_ref().to_vec()))
            .await;

        let (mut reader, mut writer) = tokio::io::split(stream);
        loop {
            tokio::select! {
                res  = client.recv() => {
                    if let Some(msg) = res {
                        if let Ok(Message::Binary(msg)) = msg {
                            let _ = writer.write_all(&msg).await;
                        }
                    } else {
                        info!("Client close");
                        return;
                    }
                },
                res  = reader.read(&mut buf) => {
                    match res {
                        Ok(n) => {
                            info!("Recv {}", n);
                            if 0 != n {
                                debug!("Recv {}", n);
                                let _ = client.send(Message::Binary(buf[..n].to_vec())).await;
                            } else {
                                return ;
                            }
                        },
                        Err(e) => {
                            info!("Server close with err {:?}", e);
                            return;
                        }
                    }

                },
            }
        }
    }
}
