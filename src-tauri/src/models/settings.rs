use serde::{Deserialize, Serialize};
use super::ai::AIConfig;
use super::agent_prompt::AgentPrompt;

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
    pub token_usage_today: u32,
    #[serde(default)]
    pub qgqp_b_id: String,
    #[serde(default = "default_max_pick_tool_rounds")]
    pub max_pick_tool_rounds: usize,
    #[serde(default = "default_max_pick_token_budget")]
    pub max_pick_token_budget: u32,
    #[serde(default)]
    pub agent_prompts: Vec<AgentPrompt>,
    #[serde(default)]
    pub active_pick_prompt_id: Option<String>,
}

fn default_refresh_interval() -> u64 { 30 }
fn default_true() -> bool { true }
fn default_max_pick_tool_rounds() -> usize { 10 }
fn default_max_pick_token_budget() -> u32 { 100_000 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            refresh_interval_secs: 30,
            auto_refresh: true,
            ai_instruction_enabled: true,
            data_source_primary: DataSource::Sina,
            ai_configs: vec![],
            active_ai_config_id: None,
            token_usage_today: 0,
            qgqp_b_id: String::new(),
            max_pick_tool_rounds: 10,
            max_pick_token_budget: 100_000,
            agent_prompts: vec![],
            active_pick_prompt_id: None,
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
