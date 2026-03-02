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

/// 败因分析入参 — 单只亏损股的信息（前端已计算好涨跌幅）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossStock {
    pub code: String,
    pub name: String,
    pub added_price: f64,
    pub current_price: f64,
    pub change_pct: f64,
    pub reason: String,
    pub sector: String,
}
