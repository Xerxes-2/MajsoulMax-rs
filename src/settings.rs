use serde::Deserialize;
use serde_json;
use tracing::event;

#[derive(Deserialize, Debug)]
pub struct Settings {
    #[serde(rename(deserialize = "SEND_METHOD"))]
    pub send_method: Vec<String>,
    #[serde(rename(deserialize = "SEND_ACTION"))]
    pub send_action: Vec<String>,
    #[serde(rename(deserialize = "API_URL"))]
    pub api_url: String,
}

impl Settings {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // read settings from file
        let settings = std::fs::read_to_string("settings.json")
            .map_err(|e| format!("无法读取settings.json: {}", e))?;
        let settings: Settings =
            serde_json::from_str(&settings).map_err(|e| format!("无法解析settings.json: {}", e))?;
        event!(tracing::Level::INFO, "已载入配置: {:?}", settings);
        Ok(settings)
    }
}
