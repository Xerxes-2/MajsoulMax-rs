use crate::{
    base::BaseMessage, lq, lq_config::ConfigTables, parser::Parser, settings::ModSettings, sheets,
};
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use prost::Message;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::error;

pub static MOD_SETTINGS: Lazy<RwLock<ModSettings>> = Lazy::new(|| RwLock::new(ModSettings::new()));
pub static SAFE: Lazy<RwLock<Safe>> = Lazy::new(|| RwLock::new(Safe::default()));

#[derive(Debug, Default)]
pub struct Safe {
    pub account_id: u32,
    pub characters: Vec<lq::Character>,
    pub main_character_id: u32,
    pub nickname: String,
    pub skin: u32,
    pub title: sheets::ItemDefinitionTitle,
    pub loading_image: sheets::ItemDefinitionLoadingImage,
}

#[derive(Debug, Default)]
pub struct Modder {
    characters: Vec<sheets::ItemDefinitionCharacter>,
    skins: Vec<sheets::ItemDefinitionSkin>,
    titles: Vec<sheets::ItemDefinitionTitle>,
    items: Vec<sheets::ItemDefinitionItem>,
    loading_images: Vec<sheets::ItemDefinitionLoadingImage>,
    emojis: HashMap<u32, Vec<sheets::CharacterEmoji>>,
    endings: Vec<sheets::SpotRewards>,
    parser: Parser,
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn to_vec<T: Message + std::default::Default>(buf: &[Vec<u8>]) -> Vec<T> {
    buf.iter()
        .map(|d| {
            T::decode(d.as_ref())
                .unwrap_or_else(|_| panic!("Failed to decode {}", std::any::type_name::<T>()))
        })
        .collect()
}

pub struct ModifyResult {
    pub msg: Option<Vec<u8>>,
    pub inject_msg: Option<Vec<u8>>,
}

impl Modder {
    pub async fn new() -> Self {
        let mod_settings = MOD_SETTINGS.read().await;
        let config_tables = ConfigTables::decode(mod_settings.res.as_ref())
            .expect("Failed to decode config tables");
        let mut modder = Modder::default();
        for data in config_tables.datas {
            // get '_' splitted words in data.table and data.sheet, turn into CamelCase then join by ""
            let class_name = data
                .table
                .split('_')
                .chain(data.sheet.split('_'))
                .map(capitalize)
                .collect::<String>();
            match class_name.as_str() {
                "ItemDefinitionCharacter" => {
                    modder.characters = to_vec(data.data.as_ref());
                }
                "ItemDefinitionSkin" => {
                    modder.skins = to_vec(data.data.as_ref());
                }
                "ItemDefinitionTitle" => {
                    modder.titles = to_vec(data.data.as_ref());
                }
                "ItemDefinitionItem" => {
                    modder.items = to_vec(data.data.as_ref());
                }
                "ItemDefinitionLoadingImage" => {
                    modder.loading_images = to_vec(data.data.as_ref());
                }
                "CharacterEmoji" => {
                    // one character can have multiple emojis
                    data.data.iter().for_each(|d| {
                        let emoji = sheets::CharacterEmoji::decode(d.as_ref())
                            .expect("Failed to decode CharacterEmoji");
                        modder
                            .emojis
                            .entry(emoji.charid)
                            .or_insert_with(Vec::new)
                            .push(emoji);
                    });
                }
                "SpotRewards" => {
                    modder.endings = to_vec(data.data.as_ref());
                }
                _ => {}
            }
        }
        modder
    }

    pub async fn modify(&self, buf: Vec<u8>, from_client: bool) -> ModifyResult {
        let msg_type = buf[0];
        let res = match msg_type {
            0x01 => self.modify_notify(&buf).await,
            _ => Err(anyhow!("Unimplemented message type: {}", msg_type)),
        };
        match res {
            Ok(r) => r,
            Err(_) => ModifyResult {
                msg: Some(buf),
                inject_msg: None,
            },
        }
    }

    pub async fn modify_notify(&self, buf: &[u8]) -> Result<ModifyResult> {
        let mut msg_block = BaseMessage::decode(&buf[1..])?;
        let method_name = &msg_block.method_name;
        let mut modified_data: Option<Vec<u8>> = None;
        match method_name.as_str() {
            ".lq.NotifyAccountUpdate" => {
                let msg = lq::NotifyAccountUpdate::decode(msg_block.data.as_ref())?;
                if let Some(ref update) = msg.update {
                    if update.character.is_some() {
                        // drop message if character is updated
                        return Ok(ModifyResult {
                            msg: None,
                            inject_msg: None,
                        });
                    }
                }
            }
            ".lq.NotifyRoomPlayerUpdate" => {
                let mut msg = lq::NotifyRoomPlayerUpdate::decode(msg_block.data.as_ref())?;
                for player in msg.player_list.iter_mut().chain(msg.update_list.iter_mut()) {
                    if player.account_id == SAFE.read().await.account_id {
                        player.avatar_id = MOD_SETTINGS.read().await.characters
                            [&MOD_SETTINGS.read().await.character];
                        if !MOD_SETTINGS.read().await.nickname.is_empty() {
                            player.nickname = MOD_SETTINGS.read().await.nickname.to_owned();
                        }
                        player.title = MOD_SETTINGS.read().await.title;
                    }
                    if MOD_SETTINGS.read().await.show_server() {
                        player.nickname = add_zone_id(player.account_id, &player.nickname);
                    }
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.NotifyGameFinishRewardV2" => {
                let mut msg = Box::new(lq::NotifyGameFinishRewardV2::decode(
                    msg_block.data.as_ref(),
                )?);
                let main = SAFE.read().await.main_character_id;
                for char in SAFE.write().await.characters.iter_mut() {
                    if char.charid == main {
                        if let Some(ref main_char) = msg.main_character {
                            char.exp = main_char.exp;
                            char.level = main_char.level;
                        }
                        break;
                    }
                }
                if let Some(ref mut main_char) = msg.main_character {
                    main_char.add = 0;
                    main_char.exp = 0;
                    main_char.level = 5;
                }
                modified_data = Some(msg.encode_to_vec());
            }
            _ => {}
        }
        if let Some(data) = modified_data {
            // add 0x01 to the beginning of the message
            msg_block.data = data;
            let mut buf = vec![0x01];
            buf.extend(msg_block.encode_to_vec());
            Ok(ModifyResult {
                msg: Some(buf),
                inject_msg: None,
            })
        } else {
            Ok(ModifyResult {
                msg: Some(buf.to_owned()),
                inject_msg: None,
            })
        }
    }
}

fn add_zone_id(id: u32, name: &str) -> String {
    let zone_code = id >> 23;
    let zone = match zone_code {
        code if code <= 6 => "[CN]",
        code if (7..=12).contains(&code) => "[JP]",
        code if (13..=15).contains(&code) => "[EN]",
        _ => "[??]",
    }
    .to_string();
    zone + name
}
