use crate::{lq_config::ConfigTables, settings::ModSettings, sheets};
use once_cell::sync::Lazy;
use prost::Message;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub static MOD_SETTINGS: Lazy<RwLock<ModSettings>> = Lazy::new(|| RwLock::new(ModSettings::new()));

#[derive(Debug, Default)]
pub struct Modder {
    characters: Vec<sheets::ItemDefinitionCharacter>,
    skins: Vec<sheets::ItemDefinitionSkin>,
    titles: Vec<sheets::ItemDefinitionTitle>,
    items: Vec<sheets::ItemDefinitionItem>,
    loading_images: Vec<sheets::ItemDefinitionLoadingImage>,
    emojis: HashMap<u32, Vec<sheets::CharacterEmoji>>,
    endings: Vec<sheets::SpotRewards>,
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
}
