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
struct ActionHandler(
    Arc<Mutex<parser::Parser>>,
    Arc<settings::Settings>,
    reqwest::Client,
);

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
            event!(Level::DEBUG, "{} {}", direction_char, hex);
            let mut parser = self.0.lock().unwrap();
            let parsed = parser.parse(&buf);
            let parsed = match parsed {
                Ok(parsed) => parsed,
                Err(e) => {
                    event!(Level::ERROR, "Failed to parse message: {:?}", e);
                    return Some(msg);
                }
            };
            event!(
                Level::INFO,
                "拦截到: {}, {}, {:?}, {}",
                direction_char,
                parsed.id,
                parsed.msg_type,
                parsed.method_name
            );
            if let Err(e) = self.send_message(parsed) {
                event!(Level::ERROR, "Failed to send message: {:?}", e);
            }
        }
        Some(msg)
    }
}

impl ActionHandler {
    fn send_message(&self, mut parsed: LiqiMessage) -> Result<(), Box<dyn Error>> {
        let settings = self.1.clone();
        let json_body: String;
        event!(Level::INFO, "Method: {}", parsed.method_name);
        if settings
            .send_method
            .iter()
            .all(|x| !parsed.method_name.contains(x))
        {
            return Ok(());
        }
        if parsed.method_name.contains(".lq.ActionPrototype") {
            let name = parsed.data.get("name").ok_or("No name field")?.to_string();
            event!(Level::INFO, "Action: {}", name);
            if settings.send_action.iter().all(|x| !name.contains(x)) {
                event!(Level::INFO, "Action {} not in send_action", name);
                return Ok(());
            }
            let data = parsed.data.get_mut("data").ok_or("No data field")?;
            if name.contains("ActionNewRound") {
                data.as_object_mut()
                    .ok_or("data is not an object")?
                    .insert("md5".to_string(), json!(RANDOM_MD5));
            }
            json_body = serde_json::to_string(data)?;
        } else if parsed.method_name.contains(".lq.FastTest.syncGame") {
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
                let action_name = item.get("name").ok_or("No name field")?.as_str().ok_or(
                    "
                        name is not a string
                    ",
                )?;
                let action_data = item.get("data").ok_or("No data field")?.as_str().unwrap_or(
                    "
                        data is not a string",
                );
                if action_data.len() == 0 {
                    let action = Action {
                        name: action_name.to_string(),
                        data: JsonValue::Object(Map::new()),
                    };
                    actions.push(action);
                } else {
                    let b64 = BASE64_STANDARD.decode(action_data)?;
                    let parser = self.0.lock().unwrap();
                    let action_type = parser
                        .pool
                        .get_message_by_name(&action_name)
                        .ok_or("Invalid action type")?;
                    let mut action_obj = DynamicMessage::decode(action_type, b64.as_ref())?;
                    if action_name.contains(".lq.ActionNewRound") {
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
            json_body = serde_json::to_string(&map)?;
        } else {
            json_body = serde_json::to_string(&parsed.data)?;
        }

        // post data to API, no verification
        let client = self.2.clone();
        let future = client
            .post(&settings.api_url)
            .body(json_body.to_owned())
            .send();

        handle_future(future);
        event!(Level::INFO, "已发送: {}", json_body);

        let json_obj: JsonValue = serde_json::from_str(&json_body)?;
        if let Some(liqi_data) = json_obj.get("liqi") {
            let res = client.post(&settings.api_url).json(liqi_data).send();
            handle_future(res);
            event!(Level::INFO, "已发送: {:?}", liqi_data);
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
                event!(Level::INFO, "小助手已接收: {}", body);
            }
            Err(e) => {
                event!(Level::ERROR, "请求失败: {:?}", e);
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

    let parser = parser::Parser::new();
    let settings = settings::Settings::new();
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create reqwest client");

    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], 23410)))
        .with_rustls_client()
        .with_ca(ca)
        .with_websocket_handler(ActionHandler(
            Arc::new(Mutex::new(parser)),
            Arc::new(settings),
            client,
        ))
        .with_graceful_shutdown(shutdown_signal())
        .build();

    if let Err(e) = proxy.start().await {
        error!("{}", e);
    }
}
