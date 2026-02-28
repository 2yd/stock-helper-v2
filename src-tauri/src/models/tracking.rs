use serde::{Deserialize, Serialize};

/// AI 选股追踪条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPickTracking {
    pub code: String,
    pub name: String,
    pub added_date: String,
    pub added_price: f64,
    pub rating: String,
    pub reason: String,
    #[serde(default)]
    pub sector: String,
    pub created_at: String,
}
