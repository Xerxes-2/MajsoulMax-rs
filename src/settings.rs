use crate::proto::lq::ViewSlot;
use anyhow::{Context, Result, bail};
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DescriptorPool, prost_types::FileDescriptorSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tokio::spawn;
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub send_method: Vec<String>,
    pub send_action: Vec<String>,
    pub proxy_addr: String,
    pub api_url: String,
    helper_switch: bool,
    mod_switch: bool,
    auto_update: bool,
    liqi_version: String,
    github_token: String,
    #[serde(skip)]
    methods_set: HashSet<String>,
    #[serde(skip)]
    actions_set: HashSet<String>,
    #[serde(skip)]
    pub desc: DescriptorPool,
    #[serde(skip)]
    pub proto_json: Value,
    #[serde(skip)]
    dir: PathBuf,
}

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static REQUEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("An error occured in building request client.")
});

impl Settings {
    pub fn new(arg_dir: &Path) -> Result<Self> {
        let exe = std::env::current_exe().context("无法获取当前可执行文件路径")?;
        let dir = if arg_dir.is_dir() {
            arg_dir.to_path_buf()
        } else {
            // current executable path
            exe.parent()
                .context("无法获取当前可执行文件路径的父目录")?
                .join("liqi_config")
        };
        let settings =
            std::fs::read_to_string(dir.join("settings.json")).context("无法读取settings.json")?;
        let mut settings: Settings =
            serde_json::from_str(&settings).context("无法解析settings.json")?;
        info!("已载入配置");
        settings.methods_set = settings.send_method.iter().cloned().collect();
        settings.actions_set = settings.send_action.iter().cloned().collect();

        // read desc from file
        let file_descriptor_set_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/liqi_desc.bin"));
        let file_descriptor_set =
            FileDescriptorSet::decode(&file_descriptor_set_bytes[..]).unwrap();
        settings.desc = DescriptorPool::from_file_descriptor_set(file_descriptor_set)
            .context("无法解析liqi.desc")?;

        // read liqi.json from file
        settings.proto_json = serde_json::from_str(
            &std::fs::read_to_string(dir.join("liqi.json")).context("无法读取liqi.json")?,
        )
        .context("无法解析liqi.json")?;
        settings.dir = dir;
        Ok(settings)
    }

    pub fn is_method(&self, method: &str) -> bool {
        self.methods_set.contains(method)
    }

    pub fn is_action(&self, action: &str) -> bool {
        self.actions_set.contains(action)
    }

    pub fn helper_on(&self) -> bool {
        self.helper_switch
    }

    pub fn mod_on(&self) -> bool {
        self.mod_switch
    }

    pub fn auto_update(&self) -> bool {
        self.auto_update
    }

    pub async fn update(&mut self) -> Result<bool> {
        let version = get_version().await?;
        let prefix = get_proto_prefix(&version).await?;
        if self.liqi_version == prefix {
            info!("无需更新liqi, 当前版本: {version}");
            return Ok(false);
        }
        info!(
            "liqi需要更新, 当前版本: {}, 最新版本: {prefix}",
            self.liqi_version
        );

        let resp = if self.github_token.is_empty() {
            REQUEST_CLIENT
                .get("https://api.github.com/repos/Xerxes-2/AutoLiqi/releases/latest")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .context("Failed to get latest release")?
        } else {
            REQUEST_CLIENT
                .get("https://api.github.com/repos/Xerxes-2/AutoLiqi/releases/latest")
                .header("Authorization", format!("Bearer {}", self.github_token))
                .header("X-GitHub-Api-Version", "2022-11-28")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .context("Failed to get latest release")?
        };
        if resp
            .headers()
            .get("X-RateLimit-Remaining")
            .context("GitHub API rate limit header not found")?
            == "0"
        {
            bail!("GitHub API rate limit exceeded");
        }
        let json: Value = resp.json().await?;
        if json["tag_name"] == self.liqi_version {
            info!("liqi需要更新, 但是AutoLiqi尚未更新, 稍晚再试");
            return Ok(false);
        }
        let assets = json["assets"]
            .as_array()
            .context("No assets found in latest release")?;
        for asset_item in assets {
            self.download_asset(asset_item).await?;
        }
        // write settings.json
        self.liqi_version = prefix;
        let dir = self.dir.join("settings.json");
        std::fs::write(dir, serde_json::to_string_pretty(self)?)?;
        Ok(true)
    }

    pub async fn download_asset(&self, asset_item: &Value) -> Result<()> {
        const ASSET_NAMES: [&str; 2] = ["liqi.desc", "liqi.json"];
        let name = asset_item["name"]
            .as_str()
            .context("No name found in asset")?;
        if !ASSET_NAMES.contains(&name) {
            return Ok(());
        }
        let url = asset_item["browser_download_url"]
            .as_str()
            .context("No download url found in asset")?;
        let resp = if self.github_token.is_empty() {
            REQUEST_CLIENT
                .get(url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .context("Failed to download asset")?
        } else {
            REQUEST_CLIENT
                .get(url)
                .header("Authorization", format!("Bearer {}", self.github_token))
                .header("X-GitHub-Api-Version", "2022-11-28")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .context("Failed to download asset")?
        };

        let resp = resp
            .error_for_status()
            .context("Failed to download asset")?;

        let bytes = resp.bytes().await?;
        let file_dir = self.dir.join(name);
        std::fs::write(file_dir, bytes)?;
        info!("下载完成: {name}");
        Ok(())
    }
}

async fn get_version() -> Result<String> {
    let resp = REQUEST_CLIENT
        .get("https://game.maj-soul.com/1/version.json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .context("Failed to get version")?;
    let json: Value = resp.json().await?;
    let version = json["version"].as_str().context("No version found")?;
    Ok(version.to_string())
}

async fn get_proto_prefix(version: &str) -> Result<String> {
    let resp = REQUEST_CLIENT
        .get(format!(
            "https://game.maj-soul.com/1/resversion{version}.json",
        ))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .context("Failed to get prefix")?;
    let json: Value = resp.json().await?;
    let prefix = json["res"]["res/proto/liqi.json"]["prefix"]
        .as_str()
        .context("No prefix found")?;
    Ok(prefix.to_string())
}

pub async fn get_lqbin_prefix(version: &str) -> Result<String> {
    let resp = REQUEST_CLIENT
        .get(format!(
            "https://game.maj-soul.com/1/resversion{version}.json"
        ))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .context("Failed to get prefix")?;
    let json: Value = resp.json().await?;
    let prefix = json["res"]["res/config/lqc.lqbin"]["prefix"]
        .as_str()
        .context("No prefix found")?;
    Ok(prefix.to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModSettings {
    pub main_char: u32,
    pub char_skin: HashMap<u32, u32>,
    pub nickname: String,
    pub star_character: Vec<u32>,
    hint_switch: bool,
    pub title: u32,
    pub loading_bg: Vec<u32>,
    emoji_switch: bool,
    pub views_presets: [Vec<ViewSlot>; 10],
    pub preset_index: u32,
    show_server: bool,
    anti_nickname_censorship: bool,
    auto_update: bool,
    version: String,
    pub random_char_switch: bool,
    pub random_char_pool: Vec<(u32, u32)>,
    pub verified: u32,
    #[serde(skip)]
    pub resource: Bytes,
    #[serde(skip)]
    dir: PathBuf,
}

impl Default for ModSettings {
    fn default() -> Self {
        ModSettings {
            main_char: 200001,
            char_skin: HashMap::new(),
            nickname: String::new(),
            star_character: Vec::new(),
            hint_switch: true,
            title: 0,
            loading_bg: Vec::new(),
            emoji_switch: false,
            views_presets: Default::default(),
            preset_index: 0,
            show_server: true,
            anti_nickname_censorship: true,
            auto_update: true,
            verified: 0,
            random_char_switch: false,
            random_char_pool: Vec::new(),
            version: String::new(),
            resource: Bytes::new(),
            dir: PathBuf::new(),
        }
    }
}

impl ModSettings {
    pub fn new(general_settings: &Settings) -> Result<Self> {
        // read settings.mod.json, if not exist, create a new one
        let dir = general_settings.dir.join("settings.mod.json");
        // read res from lqc.lqbin
        let res =
            std::fs::read(general_settings.dir.join("lqc.lqbin")).context("无法读取lqc.lqbin")?;
        let settings = std::fs::read_to_string(dir);
        let settings = match settings {
            Ok(settings) => settings,
            Err(_) => {
                let default = ModSettings {
                    dir: general_settings.dir.clone(),
                    resource: Bytes::from(res),
                    ..Default::default()
                };
                default.write();
                return Ok(default);
            }
        };
        let mut settings: ModSettings =
            serde_json::from_str(&settings).context("无法解析settings.mod.json")?;
        info!("已载入Mod配置");
        settings.resource = Bytes::from(res);
        settings.dir = general_settings.dir.clone();
        Ok(settings)
    }

    pub fn hint_on(&self) -> bool {
        self.hint_switch
    }

    pub fn emoji_on(&self) -> bool {
        self.emoji_switch
    }

    pub fn show_server(&self) -> bool {
        self.show_server
    }

    pub fn auto_update(&self) -> bool {
        self.auto_update
    }

    pub fn anti_nickname_censorship(&self) -> bool {
        self.anti_nickname_censorship
    }

    pub async fn get_lqc(&mut self) -> Result<bool> {
        // get lqc.lqbin prefix from https://game.maj-soul.com/1/{prefix}/res/config/lqc.lqbin
        let version = get_version().await?;
        let prefix = get_lqbin_prefix(&version).await?;

        if self.version == prefix {
            info!("无需更新lqc.lqbin, 当前版本: {version}");
            return Ok(false);
        }
        info!(
            "lqc.lqbin需要更新, 当前版本: {}, 最新版本: {prefix}",
            self.version
        );

        let resp = REQUEST_CLIENT
            .get(format!(
                "https://game.maj-soul.com/1/{prefix}/res/config/lqc.lqbin",
            ))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context("Failed to get lqc.lqbin")?;

        let bytes = resp.bytes().await?;
        let file_dir = self.dir.join("lqc.lqbin");
        std::fs::write(file_dir, bytes)?;
        info!("lqc.lqbin更新完成");
        self.version = prefix;
        // write settings.mod.json
        let dir = self.dir.join("settings.mod.json");
        std::fs::write(dir, serde_json::to_string_pretty(self)?)?;
        Ok(true)
    }

    pub fn write(&self) {
        let dir = self.dir.join("settings.mod.json");
        let Ok(contend) = serde_json::to_string_pretty(self) else {
            error!("Failed to serialize settings.mod.json");
            return;
        };
        spawn(async move {
            tokio::fs::write(dir, contend)
                .await
                .inspect_err(|e| error!("Failed to write settings.mod.json: {e}"))
        });
    }
}
