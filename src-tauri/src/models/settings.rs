use serde::{Deserialize, Serialize};
use super::ai::AIConfig;
use super::strategy::StrategyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
    #[serde(default = "default_true")]
    pub auto_refresh: bool,
    #[serde(default = "default_true")]
    pub ai_instruction_enabled: bool,
    #[serde(default)]
    pub data_source_primary: DataSource,
    #[serde(default)]
    pub ai_configs: Vec<AIConfig>,
    #[serde(default)]
    pub active_ai_config_id: Option<String>,
    #[serde(default)]
    pub strategies: Vec<StrategyConfig>,
    #[serde(default)]
    pub active_strategy_id: String,
    #[serde(default)]
    pub token_usage_today: u32,
    #[serde(default)]
    pub qgqp_b_id: String,
}

fn default_refresh_interval() -> u64 { 30 }
fn default_true() -> bool { true }

impl Default for AppSettings {
    fn default() -> Self {
        let default_strategy = StrategyConfig::default();
        let strategy_id = default_strategy.id.clone();
        Self {
            refresh_interval_secs: 30,
            auto_refresh: true,
            ai_instruction_enabled: true,
            data_source_primary: DataSource::Sina,
            ai_configs: vec![],
            active_ai_config_id: None,
            strategies: vec![default_strategy],
            active_strategy_id: strategy_id,
            token_usage_today: 0,
            qgqp_b_id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DataSource {
    #[default]
    #[serde(rename = "sina")]
    Sina,
    #[serde(rename = "tencent")]
    Tencent,
}
