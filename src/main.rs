use base64::prelude::*;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    rcgen::{CertificateParams, KeyPair},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use prost_reflect::{DynamicMessage, SerializeOptions, Value};
use serde_json::{Map, Value as JsonValue};
use std::sync::{Arc, Mutex};
use std::{format, net::SocketAddr};
use tracing::*;
mod lq;
mod parser;
mod settings;
use parser::Action;

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

impl WebSocketHandler for ActionHandler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let direction_char = match _ctx {
            WebSocketContext::ClientToServer { .. } => '\u{2191}',
            WebSocketContext::ServerToClient { .. } => '\u{2193}',
        };
        let is_downstream = direction_char == '\u{2193}';
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
            let settings = self.1.clone();
            let parsed = parser.parse(&buf);
            if parsed.is_none() {
                return Some(msg);
            }
            let parsed = parsed.unwrap();
            event!(
                Level::INFO,
                "接收到: {} {:?} {}",
                parsed.id,
                parsed.msg_type,
                parsed.method_name
            );
            let json_body: String;
            if !settings.send_method.contains(&parsed.method_name) || is_downstream {
                return Some(msg);
            }
            if parsed.method_name == ".lq.ActionPrototype" {
                let name = parsed.data.get_field_by_name("name").unwrap().to_string();
                if !settings.send_action.contains(&name) {
                    return Some(msg);
                }
                let data_val = parsed.data.get_field_by_name("data").unwrap();
                let mut data_msg = data_val.as_message().unwrap().to_owned();

                if name == "ActionNewRound" {
                    // give a fake md5, 32 bytes
                    data_msg.set_field_by_name(
                        "md5",
                        Value::String(
                            "
                             0123456789ABCDEF0123456789ABCDEF
                         "
                            .to_string(),
                        ),
                    );
                }
                let mut serializer = serde_json::Serializer::new(vec![]);
                data_msg
                    .serialize_with_options(&mut serializer, &SERIALIZE_OPTIONS)
                    .unwrap();
                json_body = String::from_utf8(serializer.into_inner()).unwrap();
            } else if parsed.method_name == ".lq.FastTest.syncGame" {
                let game_restore = parsed
                    .data
                    .get_field_by_name("game_restore")
                    .unwrap()
                    .into_owned();
                let list_value = game_restore
                    .as_message()
                    .unwrap()
                    .get_field_by_name("actions")
                    .unwrap();
                let list = list_value.as_list().unwrap();
                let mut actions: Vec<Action> = vec![];
                for item in list.iter() {
                    let action = item.as_message().unwrap();
                    let action_name = action.get_field_by_name("name").unwrap().to_string();
                    let action_data = action.get_field_by_name("data").unwrap().to_string();
                    if action_data.len() == 0 {
                        let action = Action {
                            name: action_name,
                            data: JsonValue::Object(Map::new()),
                        };
                        actions.push(action);
                    } else {
                        let b64 = BASE64_STANDARD.decode(action_data.as_bytes()).unwrap();
                        let action_type = parser.pool.get_message_by_name(&action_name).unwrap();
                        let mut action_obj =
                            DynamicMessage::decode(action_type, b64.as_ref()).unwrap();
                        if action_name == ".lq.ActionNewRound" {
                            action_obj.set_field_by_name(
                                "md5",
                                Value::String(
                                    "
                                     0123456789ABCDEF0123456789ABCDEF
                                 "
                                    .to_string(),
                                ),
                            );
                        }
                        let value = Value::Message(action_obj).to_string();
                        let action = Action {
                            name: action_name,
                            data: value.into(),
                        };
                        actions.push(action);
                    }
                }
                let mut map = Map::new();
                map.insert(
                    "sync_game_actions".to_string(),
                    serde_json::to_value(actions).unwrap(),
                );
                json_body = serde_json::to_string(&map).unwrap();
            } else {
                let mut serializer = serde_json::Serializer::new(vec![]);
                parsed
                    .data
                    .serialize_with_options(&mut serializer, &SERIALIZE_OPTIONS)
                    .unwrap();
                json_body = String::from_utf8(serializer.into_inner()).unwrap();
            }
            // post data to API, no verification
            let client = self.2.clone();
            let future = client
                .post(&settings.api_url)
                .header("Content-Type", "application/json")
                .body(json_body.to_owned())
                .send();

            handle_future(future);
            event!(Level::INFO, "已发送: {}", json_body);

            let json_obj: JsonValue = serde_json::from_str(&json_body).unwrap();
            if let Some(liqi_data) = json_obj.get("liqi") {
                let res = client.post(&settings.api_url).json(liqi_data).send();
                handle_future(res);
                event!(Level::INFO, "已发送: {:?}", liqi_data);
            }
        }
        Some(msg)
    }
}

fn handle_future(
    future: impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>
        + Send
        + 'static,
) {
    tokio::spawn(async move {
        match future.await {
            Ok(res) => {
                let body = res.text().await.unwrap();
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
