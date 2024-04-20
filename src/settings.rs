use serde::Deserialize;
use std::collections::HashSet;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct Settings {
    #[serde(rename(deserialize = "SEND_METHOD"))]
    pub send_method: Vec<String>,
    #[serde(rename(deserialize = "SEND_ACTION"))]
    pub send_action: Vec<String>,
    #[serde(rename(deserialize = "API_URL"))]
    pub api_url: String,
    #[serde(skip)]
    methods_set: HashSet<String>,
    #[serde(skip)]
    actions_set: HashSet<String>,
}

impl Settings {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cur_exe =
            std::env::current_exe().map_err(|e| format!("无法获取当前程序路径: {}", e))?;
        let exe_dir = cur_exe
            .parent()
            .ok_or("无法获取当前程序路径")?
            .to_str()
            .ok_or("无法转换路径为字符串")?;
        // read settings from file
        let settings = std::fs::read_to_string(std::path::Path::new(exe_dir).join("settings.json"))
            .or_else(
                // read pwd
                |_| std::fs::read_to_string("settings.json"),
            )
            .map_err(|e| format!("无法读取settings.json: {}", e))?;
        let mut settings: Settings =
            serde_json::from_str(&settings).map_err(|e| format!("无法解析settings.json: {}", e))?;
        info!("已载入配置: {:?}", settings);
        settings.methods_set = settings.send_method.iter().cloned().collect();
        settings.actions_set = settings.send_action.iter().cloned().collect();
        Ok(settings)
    }

    pub fn is_method(&self, method: &str) -> bool {
        self.methods_set.contains(method)
    }

    pub fn is_action(&self, action: &str) -> bool {
        self.actions_set.contains(action)
    }
}
