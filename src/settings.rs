use crate::ARG;
use prost_reflect::DescriptorPool;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use tracing::info;

#[derive(Deserialize, Debug, Default)]
pub struct Settings {
    #[serde(rename(deserialize = "SEND_METHOD"))]
    pub send_method: Vec<String>,
    #[serde(rename(deserialize = "SEND_ACTION"))]
    pub send_action: Vec<String>,
    #[serde(rename(deserialize = "PROXY_ADDR"))]
    pub proxy_addr: String,
    #[serde(rename(deserialize = "API_URL"))]
    pub api_url: String,
    #[serde(rename(deserialize = "HELPER_SWITCH"))]
    helper_switch: i32,
    #[serde(rename(deserialize = "MOD_SWITCH"))]
    mod_switch: i32,
    #[serde(skip)]
    methods_set: HashSet<String>,
    #[serde(skip)]
    actions_set: HashSet<String>,
    #[serde(skip)]
    pub desc: DescriptorPool,
    #[serde(skip)]
    pub proto_json: Value,
}

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
}
