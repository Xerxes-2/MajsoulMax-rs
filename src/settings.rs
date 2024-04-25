use crate::ARG;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use prost_reflect::DescriptorPool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, path::PathBuf};
use tracing::info;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Settings {
    #[serde(rename = "sendMethod")]
    pub send_method: Vec<String>,
    #[serde(rename = "sendAction")]
    pub send_action: Vec<String>,
    #[serde(rename = "proxyAddr")]
    pub proxy_addr: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
    #[serde(rename = "helperSwitch")]
    helper_switch: i32,
    #[serde(rename = "modSwitch")]
    mod_switch: i32,
    #[serde(rename = "autoUpdate")]
    auto_update: i32,
    #[serde(rename = "liqiVersion")]
    liqi_version: String,
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

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static REQUEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("An error occured in building request client.")
});

impl Settings {
    pub fn new() -> Self {
        let arg_dir = std::path::Path::new(&ARG.config_dir);
        let exe = std::env::current_exe().expect("无法获取当前可执行文件路径");
        let dir = if arg_dir.is_dir() {
            arg_dir.to_path_buf()
        } else {
            // current executable path
            exe.parent()
                .expect("无法获取当前可执行文件路径的父目录")
                .join("liqi_config")
        };
        let settings =
            std::fs::read_to_string(dir.join("settings.json")).expect("无法读取settings.json");
        let mut settings: Settings =
            serde_json::from_str(&settings).expect("无法解析settings.json");
        info!("已载入配置: {:?}", settings);
        settings.methods_set = settings.send_method.iter().cloned().collect();
        settings.actions_set = settings.send_action.iter().cloned().collect();

        // read desc from file
        let bytes = std::fs::read(dir.join("liqi.desc")).expect("无法读取liqi.desc");

        settings.desc = DescriptorPool::decode(bytes.as_slice()).expect("无法解析liqi.desc");

        // read liqi.json from file
        settings.proto_json = serde_json::from_str(
            &std::fs::read_to_string(dir.join("liqi.json")).expect("无法读取liqi.json"),
        )
        .expect("无法解析liqi.json");
        settings.dir = dir;
        settings
    }

    pub fn is_method(&self, method: &str) -> bool {
        self.methods_set.contains(method)
    }

    pub fn is_action(&self, action: &str) -> bool {
        self.actions_set.contains(action)
    }

    pub fn helper_on(&self) -> bool {
        self.helper_switch != 0
    }

    pub fn mod_on(&self) -> bool {
        self.mod_switch != 0
    }

    pub fn auto_update(&self) -> bool {
        self.auto_update != 0
    }

    pub async fn update(&mut self) -> Result<bool> {
        let version = get_version().await?;
        let prefix = get_proto_prefix(&version).await?;
        if self.liqi_version == prefix {
            info!("无需更新liqi, 当前版本: {}", version);
            return Ok(false);
        }
        info!(
            "liqi需要更新, 当前版本: {}, 最新版本: {}",
            self.liqi_version, prefix
        );

        let req = REQUEST_CLIENT
            .get("https://api.github.com/repos/Xerxes-2/AutoLiqi/releases/latest")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;
        match req {
            Ok(resp) => {
                if resp
                    .headers()
                    .get("x-ratelimit-remaining")
                    .ok_or(anyhow!("GitHub API rate limit header not found"))?
                    == "0"
                {
                    return Err(anyhow!("GitHub API rate limit exceeded"));
                }
                let json: Value = resp.json().await?;
                if json["tag_name"] == self.liqi_version {
                    info!("liqi需要更新, 但是AutoLiqi尚未更新, 稍晚再试");
                    return Ok(false);
                }
                let assets = json["assets"]
                    .as_array()
                    .ok_or(anyhow!("No assets found in latest release"))?;
                for asset_item in assets {
                    self.download_asset(asset_item).await?;
                }
            }
            Err(e) => return Err(anyhow!("Failed to get latest release: {:?}", e)),
        }
        // write settings.json
        self.liqi_version = prefix;
        let dir = self.dir.join("settings.json");
        std::fs::write(dir, serde_json::to_string_pretty(self)?)?;
        Ok(true)
    }

    pub async fn download_asset(&self, asset_item: &Value) -> Result<()> {
        const ASSET_NAMES: [&str; 3] = ["liqi.desc", "liqi.json", "liqi.proto"];
        let name = asset_item["name"]
            .as_str()
            .ok_or(anyhow!("No name found in asset"))?;
        if !ASSET_NAMES.contains(&name) {
            return Ok(());
        }
        let url = asset_item["browser_download_url"]
            .as_str()
            .ok_or(anyhow!("No download url found in asset"))?;
        let req = REQUEST_CLIENT
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;
        match req {
            Ok(resp) => {
                let bytes = resp.bytes().await?;
                let file_dir = self.dir.join(name);
                std::fs::write(file_dir, bytes).expect("无法写入文件");
                info!("下载完成: {}", name);
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to download asset: {:?}", e)),
        }
    }
}

async fn get_version() -> Result<String> {
    let req = REQUEST_CLIENT
        .get("https://game.maj-soul.com/1/version.json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    match req {
        Ok(resp) => {
            let json: Value = resp.json().await?;
            let version = json["version"]
                .as_str()
                .ok_or(anyhow!("No version found"))?;
            Ok(version.to_string())
        }
        Err(e) => Err(anyhow!("Failed to get version: {:?}", e)),
    }
}

async fn get_proto_prefix(version: &str) -> Result<String> {
    let req = REQUEST_CLIENT
        .get(format!("https://game.maj-soul.com/1/resversion{}.json", version).as_str())
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    match req {
        Ok(resp) => {
            let json: Value = resp.json().await?;
            let prefix = json["res"]["res/proto/liqi.json"]["prefix"]
                .as_str()
                .ok_or(anyhow!("No prefix found"))?;
            Ok(prefix.to_string())
        }
        Err(e) => Err(anyhow!("Failed to get prefix: {:?}", e)),
    }
}

pub async fn get_lqbin_prefix(version: &str) -> Result<String> {
    let req = REQUEST_CLIENT
        .get(format!("https://game.maj-soul.com/1/resversion{}.json", version).as_str())
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    match req {
        Ok(resp) => {
            let json: Value = resp.json().await?;
            let prefix = json["res"]["res/config/lqc.lqbin"]["prefix"]
                .as_str()
                .ok_or(anyhow!("No prefix found"))?;
            Ok(prefix.to_string())
        }
        Err(e) => Err(anyhow!("Failed to get prefix: {:?}", e)),
    }
}
