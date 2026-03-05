use serde::{Deserialize, Serialize};

pub const BUILTIN_DEFAULT_PROMPT_ID: &str = "builtin_default";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPrompt {
    pub id: String,
    pub name: String,
    pub strategy_prompt: String,
    pub is_builtin: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
