use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, KeyPair},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use std::net::SocketAddr;
use tracing::*;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct ActionHandler;

impl WebSocketHandler for ActionHandler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let direction = match _ctx {
            WebSocketContext::ClientToServer { .. } => '\u{2191}',
            WebSocketContext::ServerToClient { .. } => '\u{2193}',
        };
        if let Message::Binary(buf) = &msg {
            // convert binary message to hex string
            let hex = buf
                .iter()
                .map(|b| {
                    if *b >= 0x20 && *b <= 0x7e {
                        format!("{}", *b as char)
                    } else {
                        format!("{:02x} ", b)
                    }
                })
                .collect::<String>();
            event!(Level::DEBUG, "{} {}", direction, hex);
        }
        Some(msg)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let key_pair = include_str!("ca/hudsucker.key");
    let ca_cert = include_str!("ca/hudsucker.cer");
    let key_pair = KeyPair::from_pem(key_pair).expect("Failed to parse private key");
    let ca_cert = CertificateParams::from_ca_cert_pem(ca_cert)
        .expect("Failed to parse CA certificate")
        .self_signed(&key_pair)
        .expect("Failed to sign CA certificate");

    let ca = RcgenAuthority::new(key_pair, ca_cert, 1_000);

    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], 23410)))
        .with_rustls_client()
        .with_ca(ca)
        .with_websocket_handler(ActionHandler)
        .with_graceful_shutdown(shutdown_signal())
        .build();

    if let Err(e) = proxy.start().await {
        error!("{}", e);
    }
}
