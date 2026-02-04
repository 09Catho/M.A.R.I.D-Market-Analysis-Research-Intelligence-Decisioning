use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub preset: String,
    pub default_symbol: String,
    pub timeframe: String,
    pub providers: ProvidersConfig,
    pub brain: BrainConfig,
    pub ml: MlConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvidersConfig {
    pub ai: AiProviders,
    pub search: SearchProvider,
    pub market: MarketProvider,
    pub news: NewsProvider,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiProviders {
    pub chain: Vec<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchProvider {
    pub provider: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketProvider {
    pub provider: String,
    pub polling_interval_ms: u64,
    pub symbols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsProvider {
    pub feeds: Vec<String>,
    pub fetch_interval_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrainConfig {
    pub model: String,
    pub max_history_turns: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MlConfig {
    pub enabled_modules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preset: "crypto".to_string(),
            default_symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            providers: ProvidersConfig {
                ai: AiProviders {
                    chain: vec!["gemini-main".to_string(), "openai-backup".to_string()],
                    timeout_seconds: 30,
                },
                search: SearchProvider {
                    provider: "google_cse".to_string(),
                },
                market: MarketProvider {
                    provider: "binance".to_string(),
                    polling_interval_ms: 5000,
                    symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string(), "SOLUSDT".to_string()],
                },
                news: NewsProvider {
                    feeds: vec![
                        "https://cointelegraph.com/rss".to_string(),
                        "https://www.coindesk.com/arc/outboundfeeds/rss".to_string(),
                    ],
                    fetch_interval_seconds: 300,
                },
            },
            brain: BrainConfig {
                model: "gemini-1.5-pro".to_string(),
                max_history_turns: 10,
            },
            ml: MlConfig {
                enabled_modules: vec!["all".to_string()],
            },
            ui: UiConfig {
                theme: "default".to_string(),
            },
        }
    }
}

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Config file not found at {:?}", path));
    }
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(path: &Path, config: &Config) -> anyhow::Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

pub fn default_config_path() -> PathBuf {
    let home = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".termbrain").join("config.json")
}
