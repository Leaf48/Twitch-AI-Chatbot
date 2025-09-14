use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "Twitch")]
    pub twitch: TwitchConfig,
    #[serde(rename = "Accounts")]
    pub accounts: Vec<Account>,
    #[serde(rename = "OpenAI")]
    pub openai: OpenAIConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitchConfig {
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub oauth: String,
    pub account_name: String,
    pub channel: String,
    pub instruction: String,
}

pub fn load_config() -> Config {
    let path = get_config_path();
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read config file {}: {}", path, e));
    serde_yml::from_str::<Config>(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse YAML in {}: {}", path, e))
}

fn get_config_path() -> String {
    std::env::var("CONFIG_PATH").expect("CONFIG_PATH should be specified")
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| load_config());
