use bytes::Bytes;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, KeyPair},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use once_cell::sync::Lazy;
use std::{net::SocketAddr, str::FromStr};
use tokio::sync::mpsc::{channel, Sender};
use tracing::*;

mod helper;
mod parser;
mod settings;

use helper::helper_worker;
use parser::Parser;
use settings::Settings;

const ARBITRARY_MD5: &str = "0123456789abcdef0123456789abcdef";
pub static SETTINGS: Lazy<Settings> = Lazy::new(Settings::new);

#[derive(Clone)]
struct Handler(Sender<(Bytes, char)>);

impl WebSocketHandler for Handler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let (direction_char, uri) = match _ctx {
            WebSocketContext::ClientToServer { dst, .. } => ('\u{2193}', dst),
            WebSocketContext::ServerToClient { src, .. } => ('\u{2191}', src),
        };

        debug!("{} {}", direction_char, uri);

        if let Message::Binary(ref buf) = msg {
            if let Err(e) = self
                .0
                .send((Bytes::copy_from_slice(buf), direction_char))
                .await
            {
                error!("Failed to send message to channel: {:?}", e);
            }
        }

        // TODO: MajSoul Mod

        Some(msg)
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let key_pair = include_str!("./ca/hudsucker.key");
    let ca_cert = include_str!("./ca/hudsucker.cer");
    let key_pair = KeyPair::from_pem(key_pair).expect("Failed to parse private key");
    let ca_cert = CertificateParams::from_ca_cert_pem(ca_cert)
        .expect("Failed to parse CA certificate")
        .self_signed(&key_pair)
        .expect("Failed to sign CA certificate");

    let ca = RcgenAuthority::new(key_pair, ca_cert, 1_000);

    // print red declaimer text
    println!(
        "\x1b[31m
    本项目完全免费开源，如果您购买了此程序，请立即退款！
    项目地址: https://github.com/Xerxes-2/mahjong_helper_majsoul_hudsucker/
    
    本程序仅供学习交流使用，严禁用于商业用途！
    请遵守当地法律法规，对于使用本程序所产生的任何后果，作者概不负责！
    \x1b[0m"
    );
    let proxy_addr = match SocketAddr::from_str(SETTINGS.proxy_addr.as_str()) {
        Ok(addr) => addr,
        Err(e) => {
            error!(
                "Failed to parse proxy address: {:?}, url: {}",
                e, SETTINGS.proxy_addr
            );
            return;
        }
    };
    let (tx, rx) = channel::<(Bytes, char)>(100);
    let parser = Parser::new();
    let proxy = Proxy::builder()
        .with_addr(proxy_addr)
        .with_rustls_client()
        .with_ca(ca)
        .with_websocket_handler(Handler(tx.clone()))
        .with_graceful_shutdown(shutdown_signal())
        .build();

    tokio::spawn(helper_worker(rx, parser));

    if let Err(e) = proxy.start().await {
        error!("{}", e);
    }
}
