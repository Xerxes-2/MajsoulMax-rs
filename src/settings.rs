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
    pub fn new() -> Self {
        // read settings from file
        let settings =
            std::fs::read_to_string("settings.json").expect("Failed to read settings.json");
        let settings: Settings =
            serde_json::from_str(&settings).expect("Failed to parse settings.json");
        event!(tracing::Level::INFO, "已载入配置: {:?}", settings);
        settings
    }
}
