use anyhow::{anyhow, Result};
use base64::prelude::*;
use bytes::Bytes;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, KeyPair},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use once_cell::sync::Lazy;
use prost_reflect::{DynamicMessage, SerializeOptions};
use reqwest::Client;
use serde_json::{json, Map, Value as JsonValue};
use std::{format, future::Future, net::SocketAddr};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::sleep,
};
use tracing::*;

mod parser;
mod settings;
use parser::{my_serialize, to_fqn, Action, LiqiMessage, Parser};
use settings::Settings;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct Handler(Sender<(Bytes, char)>);

pub const SERIALIZE_OPTIONS: SerializeOptions = SerializeOptions::new()
    .skip_default_fields(false)
    .use_proto_field_name(true);

const ARBITRARY_MD5: &str = "0123456789abcdef0123456789abcdef";

impl WebSocketHandler for Handler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let direction_char = match _ctx {
            WebSocketContext::ClientToServer { .. } => '\u{2191}',
            WebSocketContext::ServerToClient { .. } => '\u{2193}',
        };

        if let Message::Binary(buf) = &msg {
            if let Err(e) = self
                .0
                .send((Bytes::copy_from_slice(buf), direction_char))
                .await
            {
                error!("Failed to send message to channel: {:?}", e);
            }
        }

        Some(msg)
    }
}

async fn worker(mut receiver: Receiver<(Bytes, char)>, mut parser: Parser) {
    loop {
        let (buf, direction_char) = match receiver.recv().await {
            Some((b, c)) => (b, c),
            None => {
                error!("Failed to receive message from channel, retrying...");
                sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };
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
        debug!("{} {}", direction_char, hex);
        let parsed = parser.parse(&buf);
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("Failed to parse message: {:?}", e);
                continue;
            }
        };
        info!(
            "监听到: {}, {}, {:?}, {}",
            direction_char, parsed.id, parsed.msg_type, parsed.method_name
        );
        if direction_char == '\u{2193}' {
            continue;
        }
        if let Err(e) = process_message(parsed, &mut parser) {
            error!("Failed to process message: {:?}", e);
        }
    }
}

fn process_message(mut parsed: LiqiMessage, parser: &mut Parser) -> Result<()> {
    static SETTINGS: Lazy<Settings> = Lazy::new(Settings::new);
    static CLIENT: Lazy<Client> = Lazy::new(|| {
        reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create reqwest client")
    });
    let json_data: JsonValue;
    if !SETTINGS.is_method(&parsed.method_name) {
        return Ok(());
    }
    if parsed.method_name == ".lq.ActionPrototype" {
        let name = parsed
            .data
            .get("name")
            .ok_or(anyhow!("No name field"))?
            .as_str()
            .ok_or(anyhow!("name is not a string"))?
            .to_owned();
        if !SETTINGS.is_action(&name) {
            return Ok(());
        }
        let data = parsed
            .data
            .get_mut("data")
            .ok_or(anyhow!("No data field"))?;
        if name == "ActionNewRound" {
            data.as_object_mut()
                .ok_or(anyhow!("data is not an object"))?
                .insert("md5".to_string(), json!(ARBITRARY_MD5));
        }
        json_data = data.take();
    } else if parsed.method_name == ".lq.FastTest.syncGame" {
        let game_restore = parsed
            .data
            .get("game_restore")
            .ok_or(anyhow!("No game_restore field"))?
            .get("actions")
            .ok_or(anyhow!("No actions field"))?
            .as_array()
            .ok_or(anyhow!("actions is not an array"))?;
        let mut actions: Vec<Action> = vec![];
        for item in game_restore.iter() {
            let action_name = item
                .get("name")
                .ok_or(anyhow!("No name field"))?
                .as_str()
                .ok_or(anyhow!("name is not a string"))?;
            let action_data = item
                .get("data")
                .ok_or(anyhow!("No data field"))?
                .as_str()
                .unwrap_or_default();
            if action_data.is_empty() {
                let action = Action {
                    name: action_name.to_string(),
                    data: JsonValue::Object(Map::new()),
                };
                actions.push(action);
            } else {
                let b64 = BASE64_STANDARD.decode(action_data)?;
                let action_type = parser
                    .pool
                    .get_message_by_name(to_fqn(action_name).as_str())
                    .ok_or(anyhow!("Invalid action type: {}", action_name))?;
                let action_obj = DynamicMessage::decode(action_type, b64.as_ref())?;
                let mut value: JsonValue = my_serialize(action_obj)?;
                if action_name == "ActionNewRound" {
                    value
                        .as_object_mut()
                        .ok_or(anyhow!("data is not an object"))?
                        .insert("md5".to_string(), json!(ARBITRARY_MD5));
                }
                let action = Action {
                    name: action_name.to_string(),
                    data: value,
                };
                actions.push(action);
            }
        }
        let mut map = Map::with_capacity(1);
        map.insert(
            "sync_game_actions".to_string(),
            serde_json::to_value(actions)?,
        );
        json_data = JsonValue::Object(map);
    } else {
        json_data = parsed.data;
    }

    // post data to API, no verification
    let future = CLIENT.post(&SETTINGS.api_url).json(&json_data).send();

    handle_future(future);
    info!("已发送: {}", json_data);

    if let Some(liqi_data) = json_data.get("liqi") {
        let res = CLIENT.post(&SETTINGS.api_url).json(liqi_data).send();
        handle_future(res);
        info!("已发送: {:?}", liqi_data);
    }

    Ok(())
}

fn handle_future(
    future: impl Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + 'static,
) {
    tokio::spawn(async {
        match future.await {
            Ok(res) => {
                let body = res.text().await.unwrap_or_default();
                info!("小助手已接收: {}", body);
            }
            Err(e) => {
                error!("请求失败: {:?}", e);
            }
        }
    });
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

    let (tx, rx) = channel::<(Bytes, char)>(100);
    let parser = Parser::new();
    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], 23410)))
        .with_rustls_client()
        .with_ca(ca)
        .with_websocket_handler(Handler(tx.clone()))
        .with_graceful_shutdown(shutdown_signal())
        .build();

    tokio::spawn(worker(rx, parser));
    if let Err(e) = proxy.start().await {
        error!("{}", e);
    }
}
