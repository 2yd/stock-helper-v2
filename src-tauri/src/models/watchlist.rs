use serde::{Deserialize, Serialize};

/// 自选股条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistStock {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub group_name: String,
    #[serde(default)]
    pub created_at: String,
}

/// 技术指标计算结果（Rust端 -> 前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub dates: Vec<String>,
    pub ma5: Vec<Option<f64>>,
    pub ma10: Vec<Option<f64>>,
    pub ma20: Vec<Option<f64>>,
    pub ma60: Vec<Option<f64>>,
    pub ema12: Vec<Option<f64>>,
    pub ema26: Vec<Option<f64>>,
    pub macd_dif: Vec<Option<f64>>,
    pub macd_dea: Vec<Option<f64>>,
    pub macd_hist: Vec<Option<f64>>,
    pub kdj_k: Vec<Option<f64>>,
    pub kdj_d: Vec<Option<f64>>,
    pub kdj_j: Vec<Option<f64>>,
    pub rsi6: Vec<Option<f64>>,
    pub rsi12: Vec<Option<f64>>,
    pub rsi24: Vec<Option<f64>>,
    pub boll_upper: Vec<Option<f64>>,
    pub boll_middle: Vec<Option<f64>>,
    pub boll_lower: Vec<Option<f64>>,
}

/// 技术信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSignal {
    pub signal_type: String,
    pub direction: String,
    pub description: String,
    pub strength: u8,
    pub date: String,
}

/// 均线排列状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaAlignment {
    #[serde(rename = "bullish")]
    Bullish,
    #[serde(rename = "bearish")]
    Bearish,
    #[serde(rename = "tangled")]
    Tangled,
}

/// 量价关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumePriceRelation {
    #[serde(rename = "volume_up_price_up")]
    VolumeUpPriceUp,
    #[serde(rename = "volume_down_price_up")]
    VolumeDownPriceUp,
    #[serde(rename = "volume_up_price_down")]
    VolumeUpPriceDown,
    #[serde(rename = "volume_down_price_down")]
    VolumeDownPriceDown,
    #[serde(rename = "normal")]
    Normal,
}

/// 股票技术分析聚合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTechnicalAnalysis {
    pub code: String,
    pub name: String,
    pub kline_data: Vec<KlineItem>,
    pub indicators: TechnicalIndicators,
    pub signals: Vec<TechnicalSignal>,
    pub ma_alignment: MaAlignment,
    pub volume_price_relation: VolumePriceRelation,
    pub summary: String,
}

/// K线单条数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineItem {
    pub date: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub change_pct: f64,
    #[serde(default)]
    pub turnover_rate: f64,
}
