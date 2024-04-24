use serde::Deserialize;
use std::collections::HashSet;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct Settings {
    #[serde(rename(deserialize = "SEND_METHOD"))]
    pub send_method: Vec<String>,
    #[serde(rename(deserialize = "SEND_ACTION"))]
    pub send_action: Vec<String>,
    #[serde(rename(deserialize = "PROXY_ADDR"))]
    pub proxy_addr: String,
    #[serde(rename(deserialize = "API_URL"))]
    pub api_url: String,
    #[serde(skip)]
    methods_set: HashSet<String>,
    #[serde(skip)]
    actions_set: HashSet<String>,
}

impl Settings {
    pub fn new() -> Self {
        let cur_exe = std::env::current_exe()
            .expect("无法获取当前程序路径")
            .canonicalize()
            .expect("无法获取当前程序路径的绝对路径");
        let exe_dir = cur_exe
            .parent()
            .expect("无法获取当前程序路径的父目录")
            .to_str()
            .expect("无法将目录转换为UTF-8字符串");
        // read settings from file
        let settings = std::fs::read_to_string(std::path::Path::new(exe_dir).join("settings.json"))
            .or_else(
                // read pwd
                |_| std::fs::read_to_string("settings.json"),
            )
            .expect("无法读取settings.json");
        let mut settings: Settings =
            serde_json::from_str(&settings).expect("无法解析settings.json");
        info!("已载入配置: {:?}", settings);
        settings.methods_set = settings.send_method.iter().cloned().collect();
        settings.actions_set = settings.send_action.iter().cloned().collect();
        settings
    }

    pub fn is_method(&self, method: &str) -> bool {
        self.methods_set.contains(method)
    }

    pub fn is_action(&self, action: &str) -> bool {
        self.actions_set.contains(action)
    }
}
