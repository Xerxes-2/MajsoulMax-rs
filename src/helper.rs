use crate::{
    ARBITRARY_MD5,
    parser::{LiqiMessage, decode_action},
    settings::Settings,
};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use serde_json::{Map, Value as JsonValue, json};
use std::{future::Future, sync::LazyLock};
use tokio::{spawn, sync::mpsc::Receiver, time::sleep};
use tracing::{debug, error, info};

#[derive(Serialize, Debug)]
struct Action {
    pub name: String,
    pub data: JsonValue,
}

pub async fn helper_worker(mut receiver: Receiver<(LiqiMessage, char)>, settings: &Settings) {
    loop {
        let (parsed, direction_char) = match receiver.recv().await {
            Some((b, c)) => (b, c),
            None => {
                error!("Failed to receive message from channel, retrying...");
                sleep(std::time::Duration::from_secs(1)).await;
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
        if let Err(e) = process_message(parsed, settings) {
            error!("Failed to process message: {e}");
        }
    }
}

fn process_message(mut parsed: LiqiMessage, settings: &Settings) -> Result<()> {
    static CLIENT: LazyLock<Client> = LazyLock::new(|| {
        reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create reqwest client")
    });
    if !settings.is_method(&parsed.method_name) {
        return Ok(());
    }
    let json_data: JsonValue = match parsed.method_name.as_ref() {
        ".lq.ActionPrototype" => {
            let name = parsed.data["name"].as_str().context("name field invalid")?;
            if !settings.is_action(name) {
                return Ok(());
            }
            if name == "ActionNewRound" {
                info!("New Round: {}", parsed);
                parsed.data["data"]["md5"] = ARBITRARY_MD5.into();
            }
            parsed.data["data"].take()
        }
        ".lq.FastTest.syncGame" => {
            let game_restore = parsed.data["game_restore"]["actions"]
                .as_array()
                .context("actions field invalid")?;
            let mut actions = Vec::with_capacity(game_restore.len());
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
                    let mut value = decode_action(action_name, action_data, &settings.desc)?;
                    if action_name == "ActionNewRound" {
                        value["md5"] = ARBITRARY_MD5.into();
                    }
                    let action = Action {
                        name: action_name.to_string(),
                        data: value,
                    };
                    actions.push(action);
                }
            }
            json!({
                "sync_game_actions": actions,
            })
        }
        _ => parsed.data,
    };

    // post data to API, no verification
    let res = CLIENT.post(&settings.api_url).json(&json_data).send();

    spawn(handle_response(res));
    info!("发送至助手……");

    if let Some(liqi_data) = json_data.get("liqi") {
        let res = CLIENT.post(&settings.api_url).json(liqi_data).send();
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
