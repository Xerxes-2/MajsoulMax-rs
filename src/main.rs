use anyhow::{Context, Result};
use bytes::Bytes;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    futures::{Sink, SinkExt, Stream, StreamExt},
    rcgen::{CertificateParams, KeyPair},
    tokio_tungstenite::tungstenite::{self, Message},
    *,
};
use metadata::LevelFilter;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::mpsc::{channel, Sender};
use tracing::*;
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

use majsoul_max_rs::{
    helper::helper_worker,
    modder::{Modder, MOD_SETTINGS},
    parser::Parser,
    settings::SETTINGS,
};

#[derive(Clone)]
struct Handler {
    sender: Sender<(Bytes, char)>,
    modder: Option<Arc<Modder>>,
    inject_msg: Option<Message>,
}

impl WebSocketHandler for Handler {
    async fn handle_websocket(
        mut self,
        ctx: WebSocketContext,
        mut stream: impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin + Send + 'static,
        mut sink: impl Sink<Message, Error = tungstenite::Error> + Unpin + Send + 'static,
    ) {
        if let WebSocketContext::ServerToClient { .. } = ctx {
            if let Some(msg) = self.inject_msg.take() {
                if let Err(e) = sink.send(msg).await {
                    error!("Failed to send injected message: {e}");
                }
            }
        }
        while let Some(message) = stream.next().await {
            match message {
                Ok(message) => {
                    let Some(message) = self.handle_message(&ctx, message).await else {
                        continue;
                    };

                    match sink.send(message).await {
                        Err(tungstenite::Error::ConnectionClosed) => (),
                        Err(e) => error!("WebSocket send error: {e}"),
                        _ => (),
                    }
                }
                Err(e) => {
                    error!("WebSocket message error: {e}");

                    match sink.send(Message::Close(None)).await {
                        Err(tungstenite::Error::ConnectionClosed) => (),
                        Err(e) => error!("WebSocket close error: {e}"),
                        _ => (),
                    };

                    break;
                }
            }
        }
    }

    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let (direction_char, uri) = match _ctx {
            WebSocketContext::ServerToClient { src, .. } => ('\u{2193}', src),
            WebSocketContext::ClientToServer { dst, .. } => ('\u{2191}', dst),
        };

        if uri.path() == "/ob" {
            // ignore ob messages
            return Some(msg);
        }

        debug!("{direction_char} {uri}");

        let Message::Binary(buf) = msg else {
            return Some(msg);
        };

        let buf: Bytes = buf.into();

        if SETTINGS.helper_on() {
            if let Err(e) = self.sender.send((buf.clone(), direction_char)).await {
                error!("Failed to send message to channel: {e}");
            }
        }
        if let Some(ref modder) = self.modder {
            let res = modder.modify(buf, direction_char == '\u{2191}').await;
            if let Some(inj) = res.inject_msg {
                self.inject_msg = Some(Message::Binary(inj.into()));
            }
            return res.msg.map(|msg| Message::Binary(msg.into()));
        }
        Some(Message::Binary(buf.into()))
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<()> {
    // chrono formatted timer
    let timer = ChronoLocal::new("%H:%M:%S%.3f".to_string());
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .unwrap_or_default()
        .add_directive("majsoul_max_rs=info".parse().unwrap_or_default());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_timer(timer)
        .compact()
        .init();

    let key_pair = include_str!("./ca/hudsucker.key");
    let ca_cert = include_str!("./ca/hudsucker.cer");
    let key_pair = KeyPair::from_pem(key_pair).expect("Failed to parse private key");
    let ca_cert = CertificateParams::from_ca_cert_pem(ca_cert)
        .context("Failed to parse CA certificate")?
        .self_signed(&key_pair)
        .context("Failed to sign CA certificate")?;

    let ca = RcgenAuthority::new(
        key_pair,
        ca_cert,
        1_000,
        rustls::crypto::aws_lc_rs::default_provider(),
    );

    // print red declaimer text
    println!(
        "
    MajsoulMax-rs {}
    \x1b[31m
    本项目完全免费开源，如果您购买了此程序，请立即退款！
    项目地址: https://github.com/Xerxes-2/MajsoulMax-rs

    本程序仅供学习交流使用，严禁用于商业用途！
    请遵守当地法律法规，对于使用本程序所产生的任何后果，作者概不负责！
    \x1b[0m",
        env!("CARGO_PKG_VERSION")
    );

    let proxy_addr = SocketAddr::from_str(SETTINGS.proxy_addr.as_str())
        .context("Failed to parse proxy address")?;

    if SETTINGS.auto_update() {
        info!("自动更新liqi已开启");
        let mut new_settings = SETTINGS.clone();
        match new_settings.update().await {
            Err(e) => warn!("更新liqi失败: {e}"),
            Ok(true) => {
                info!("liqi更新成功, 请重启程序");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                return Ok(());
            }
            _ => (),
        }
    }

    // show mod and helper switch status, green for on, red for off
    println!(
        "\n\x1b[{}mmod: {}\x1b[0m\n\x1b[{}mhelper: {}\x1b[0m\n",
        if SETTINGS.mod_on() { 32 } else { 31 },
        if SETTINGS.mod_on() { "on" } else { "off" },
        if SETTINGS.helper_on() { 32 } else { 31 },
        if SETTINGS.helper_on() { "on" } else { "off" }
    );

    let modder = if SETTINGS.mod_on() {
        // start mod worker
        info!("Mod worker started");
        if MOD_SETTINGS.read().await.auto_update() {
            info!("自动更新mod已开启");
            let mut new_mod_settings = MOD_SETTINGS.read().await.clone();
            match new_mod_settings.get_lqc().await {
                Err(e) => warn!("更新mod失败: {e}"),
                Ok(true) => {
                    info!("mod更新成功, 请重启程序");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    return Ok(());
                }
                Ok(false) => (),
            }
        }
        Some(Arc::new(Modder::new().await))
    } else {
        None
    };

    let (tx, rx) = channel::<(Bytes, char)>(100);
    let proxy = Proxy::builder()
        .with_addr(proxy_addr)
        .with_ca(ca)
        .with_rustls_client(rustls::crypto::aws_lc_rs::default_provider())
        .with_websocket_handler(Handler {
            sender: tx.clone(),
            modder,
            inject_msg: None,
        })
        .with_graceful_shutdown(shutdown_signal())
        .build()
        .context("Failed to build proxy")?;

    if SETTINGS.helper_on() {
        // start helper worker
        info!("Helper worker started");
        tokio::spawn(helper_worker(
            rx,
            Parser::new(&SETTINGS.proto_json, &SETTINGS.desc),
        ));
    }

    proxy.start().await.context("Failed to start proxy")
}
