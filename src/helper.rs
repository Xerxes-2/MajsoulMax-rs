use crate::{
    parser::{decode_action, LiqiMessage, Parser},
    settings::SETTINGS,
    ARBITRARY_MD5,
};
use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Map, Value as JsonValue};
use std::{future::Future, sync::LazyLock};
use tokio::{spawn, sync::mpsc::Receiver, time::sleep};
use tracing::{debug, error, info};

#[derive(Serialize, Debug)]
struct Action {
    pub name: String,
    pub data: JsonValue,
}

pub async fn helper_worker(mut receiver: Receiver<(Bytes, char)>, mut parser: Parser) {
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
        debug!("{direction_char} {hex}");
        let parsed = parser.parse(buf.clone());
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("Failed to parse message: {e}");
                continue;
            }
        };
        debug!(
            "Method: {direction_char}, {}, {:?}, {}",
            parsed.id, parsed.msg_type, parsed.method_name
        );
        if direction_char == '\u{2191}' {
            continue;
        }
        if let Err(e) = process_message(parsed, &mut parser) {
            error!("Failed to process message: {e}");
        }
    }
}

fn process_message(mut parsed: LiqiMessage, parser: &mut Parser) -> Result<()> {
    static CLIENT: LazyLock<Client> = LazyLock::new(|| {
        reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create reqwest client")
    });
    if !SETTINGS.is_method(&parsed.method_name) {
        return Ok(());
    }
    let json_data = match parsed.method_name.as_ref() {
        ".lq.ActionPrototype" => {
            let name = parsed.data["name"].as_str().context("name field invalid")?;
            if !SETTINGS.is_action(name) {
                return Ok(());
            }
            if name == "ActionNewRound" {
                parsed.data["data"]
                    .as_object_mut()
                    .context("data field invalid")?
                    .insert("md5".to_string(), json!(ARBITRARY_MD5));
            }
            parsed.data.get_mut("data").context("No data field")?.take()
        }
        ".lq.FastTest.syncGame" => {
            let game_restore = parsed.data["game_restore"]["actions"]
                .as_array()
                .context("actions field invalid")?;
            let mut actions: Vec<Action> = vec![];
            for item in game_restore.iter() {
                let action_name = item["name"].as_str().context("name field invalid")?;
                let action_data = item["data"].as_str().unwrap_or_default();
                if action_data.is_empty() {
                    let action = Action {
                        name: action_name.to_string(),
                        data: JsonValue::Object(Map::new()),
                    };
                    actions.push(action);
                } else {
                    let mut value = decode_action(action_name, action_data, parser.pool)?;
                    if action_name == "ActionNewRound" {
                        value
                            .as_object_mut()
                            .context("data is not an object")?
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
            JsonValue::Object(map)
        }
        _ => parsed.data,
    };

    // post data to API, no verification
    let res = CLIENT.post(&SETTINGS.api_url).json(&json_data).send();

    spawn(handle_response(res));
    info!("发送至助手……");

    if let Some(liqi_data) = json_data.get("liqi") {
        let res = CLIENT.post(&SETTINGS.api_url).json(liqi_data).send();
        spawn(handle_response(res));
        info!("发送立直至助手……");
    }

    Ok(())
}

async fn handle_response(
    res: impl Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + 'static,
) {
    match res.await {
        Ok(_) => {
            info!("请求小助手已接收");
        }
        Err(e) => {
            error!("请求小助手失败: {e}");
        }
    }
}
