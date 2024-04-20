use base64::prelude::*;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, KeyPair},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use prost_reflect::{DynamicMessage, SerializeOptions, Value};
use serde_json::{json, Map, Value as JsonValue};
use std::{
    error::Error,
    future::Future,
    io::Read,
    sync::{Arc, Mutex},
};
use std::{format, net::SocketAddr};
use tracing::*;
mod parser;
mod settings;
use parser::{Action, LiqiMessage};

use crate::parser::my_serialize;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct ActionHandler {
    parser: Arc<Mutex<parser::Parser>>,
    settings: Arc<settings::Settings>,
    client: reqwest::Client,
}

pub const SERIALIZE_OPTIONS: SerializeOptions = SerializeOptions::new()
    .skip_default_fields(false)
    .use_proto_field_name(true);

pub const RANDOM_MD5: &str = "0123456789abcdef0123456789abcdef";

impl WebSocketHandler for ActionHandler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let direction_char = match _ctx {
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
            debug!("{} {}", direction_char, hex);
            let mut parser = self.parser.lock().unwrap();
            let parsed = parser.parse(buf);
            let parsed = match parsed {
                Ok(parsed) => parsed,
                Err(e) => {
                    error!("Failed to parse message: {:?}", e);
                    return Some(msg);
                }
            };
            info!(
                "监听到: {}, {}, {:?}, {}",
                direction_char, parsed.id, parsed.msg_type, parsed.method_name
            );
            if direction_char == '\u{2193}' {
                return Some(msg);
            }
            if let Err(e) = self.send_message(parsed) {
                error!("Failed to send message: {:?}", e);
            }
        }
        Some(msg)
    }
}

impl ActionHandler {
    fn send_message(&self, mut parsed: LiqiMessage) -> Result<(), Box<dyn Error>> {
        let settings = self.settings.clone();
        let json_data: JsonValue;
        if !settings.is_method(&parsed.method_name) {
            return Ok(());
        }
        if parsed.method_name == ".lq.ActionPrototype" {
            let name = parsed
                .data
                .get("name")
                .ok_or("No name field")?
                .as_str()
                .ok_or("name is not a string")?
                .to_owned();
            if !settings.is_action(&name) {
                return Ok(());
            }
            let data = parsed.data.get_mut("data").ok_or("No data field")?;
            if name == "ActionNewRound" {
                data.as_object_mut()
                    .ok_or("data is not an object")?
                    .insert("md5".to_string(), json!(RANDOM_MD5));
            }
            json_data = data.take();
        } else if parsed.method_name == ".lq.FastTest.syncGame" {
            let game_restore = parsed
                .data
                .get("game_restore")
                .ok_or("No game_restore field")?
                .get("actions")
                .ok_or("No actions field")?
                .as_array()
                .ok_or("actions is not an array")?;
            let mut actions: Vec<Action> = vec![];
            for item in game_restore.iter() {
                let action_name = item
                    .get("name")
                    .ok_or("No name field")?
                    .as_str()
                    .ok_or("name is not a string")?;
                let action_data = item
                    .get("data")
                    .ok_or("No data field")?
                    .as_str()
                    .unwrap_or("data is not a string");
                if action_data.is_empty() {
                    let action = Action {
                        name: action_name.to_string(),
                        data: JsonValue::Object(Map::new()),
                    };
                    actions.push(action);
                } else {
                    let b64 = BASE64_STANDARD.decode(action_data)?;
                    let parser = self.parser.lock().unwrap();
                    let action_type = parser
                        .pool
                        .get_message_by_name(action_name)
                        .ok_or("Invalid action type")?;
                    let mut action_obj = DynamicMessage::decode(action_type, b64.as_ref())?;
                    if action_name == ".lq.ActionNewRound" {
                        action_obj.set_field_by_name("md5", Value::String(RANDOM_MD5.to_string()));
                    }
                    let value: JsonValue = my_serialize(action_obj)?;
                    let action = Action {
                        name: action_name.to_string(),
                        data: value,
                    };
                    actions.push(action);
                }
            }
            let mut map = Map::new();
            map.insert(
                "sync_game_actions".to_string(),
                serde_json::to_value(actions)?,
            );
            json_data = JsonValue::Object(map);
        } else {
            json_data = parsed.data;
        }

        // post data to API, no verification
        let client = self.client.clone();
        let future = client.post(&settings.api_url).json(&json_data).send();

        handle_future(future);
        info!("已发送: {}", json_data);

        if let Some(liqi_data) = json_data.get("liqi") {
            let res = client.post(&settings.api_url).json(liqi_data).send();
            handle_future(res);
            info!("已发送: {:?}", liqi_data);
        }

        Ok(())
    }
}

fn handle_future(
    future: impl Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + 'static,
) {
    tokio::spawn(async move {
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
    let parser = parser::Parser::new();
    let settings = settings::Settings::new();
    let settings = match settings {
        Ok(settings) => settings,
        Err(e) => {
            error!("{}", e);
            // press any key to exit
            println!("按任意键退出");
            let mut stdin = std::io::stdin();
            let _ = stdin.read(&mut [0u8]).unwrap_or_default();
            return;
        }
    };
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create reqwest client");

    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], 23410)))
        .with_rustls_client()
        .with_ca(ca)
        .with_websocket_handler(ActionHandler {
            parser: Arc::new(Mutex::new(parser)),
            settings: Arc::new(settings),
            client,
        })
        .with_graceful_shutdown(shutdown_signal())
        .build();

    if let Err(e) = proxy.start().await {
        error!("{}", e);
    }
}
