use serde::{Deserialize, Serialize};
use super::ai::AIInstruction;

/// 多因子选股策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub weights: FactorWeights,
    #[serde(default)]
    pub filters: StockFilters,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_top_n")]
    pub top_n: usize,
    /// 自选股代码列表（可选，空=全市场扫描）
    #[serde(default)]
    pub watch_codes: Vec<String>,
}

fn default_top_n() -> usize { 50 }
fn default_true() -> bool { true }

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "多因子择时选股".to_string(),
            description: "基于价值、质量、动量时机、资金流向、风险控制五维度的量化择时选股策略".to_string(),
            weights: FactorWeights::default(),
            filters: StockFilters::default(),
            enabled: true,
            top_n: 50,
            watch_codes: vec![],
        }
    }
}

/// 六大因子权重（总和 = 100）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorWeights {
    #[serde(default = "default_15w")]
    pub value: u32,     // 价值因子：PE + PB 综合
    #[serde(default = "default_15w")]
    pub quality: u32,   // 质量因子：ROE + 营收增长
    #[serde(default = "default_25w")]
    pub momentum: u32,  // 动量因子：买入时机 + 趋势 + 量比
    #[serde(default = "default_20w")]
    pub capital: u32,   // 资金因子：主力净流入 + 量价配合
    #[serde(default = "default_10w")]
    pub risk: u32,      // 风险因子：市值适中 + 波动率适中
    #[serde(default = "default_15w")]
    pub sentiment: u32, // 消息因子：政策/主题催化 + 概念映射热度
}

fn default_25w() -> u32 { 25 }
fn default_20w() -> u32 { 20 }
fn default_15w() -> u32 { 15 }
fn default_10w() -> u32 { 10 }

impl Default for FactorWeights {
    fn default() -> Self {
        Self {
            value: 15,
            quality: 15,
            momentum: 25,
            capital: 20,
            risk: 10,
            sentiment: 15,
        }
    }
}

/// 股票筛选过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockFilters {
    #[serde(default = "default_true")]
    pub exclude_st: bool,            // 排除 ST
    #[serde(default = "default_60")]
    pub exclude_new_stock_days: u32, // 排除上市不足 N 天的次新股
    #[serde(default = "default_30f")]
    pub min_market_cap: f64,         // 最低市值（亿元）
    #[serde(default)]
    pub max_market_cap: f64,         // 最高市值（亿元），0=不限
    #[serde(default = "default_3f_filter")]
    pub min_price: f64,              // 最低股价
    #[serde(default = "default_5000f")]
    pub min_amount: f64,             // 最低成交额（万元）
    #[serde(default = "default_100f")]
    pub pe_max: f64,                 // PE 上限，0=不限
    #[serde(default)]
    pub pe_min: f64,                 // PE 下限（排除负值/亏损）
    #[serde(default = "default_20f")]
    pub pb_max: f64,                 // PB 上限，0=不限
    #[serde(default = "default_5f_filter")]
    pub roe_min: f64,                // ROE 下限 %
}

fn default_60() -> u32 { 60 }
fn default_30f() -> f64 { 30.0 }
fn default_3f_filter() -> f64 { 3.0 }
fn default_5000f() -> f64 { 5000.0 }
fn default_100f() -> f64 { 100.0 }
fn default_20f() -> f64 { 20.0 }
fn default_5f_filter() -> f64 { 5.0 }

impl Default for StockFilters {
    fn default() -> Self {
        Self {
            exclude_st: true,
            exclude_new_stock_days: 60,
            min_market_cap: 30.0,
            max_market_cap: 0.0,
            min_price: 3.0,
            min_amount: 5000.0,
            pe_max: 100.0,
            pe_min: 0.0,
            pb_max: 20.0,
            roe_min: 5.0,
        }
    }
}

/// 各因子分项得分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorScoreDetail {
    pub value_score: f64,     // 0-1
    pub quality_score: f64,
    pub momentum_score: f64,
    pub capital_score: f64,
    pub risk_score: f64,
    #[serde(default)]
    pub sentiment_score: f64, // 0-1 消息面因子得分
}

/// 策略筛选结果行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResultRow {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,        // 今日涨跌幅
    pub pe_ttm: f64,
    pub pb: f64,
    pub roe: f64,               // %
    pub revenue_yoy: f64,       // 营收同比 %
    pub profit_yoy: f64,        // 利润同比 %
    pub total_market_cap: f64,  // 总市值（亿元）
    pub float_market_cap: f64,  // 流通市值（亿元）
    pub turnover_rate: f64,     // 换手率 %
    pub volume_ratio: f64,      // 量比
    pub amount: f64,            // 成交额（万元）
    pub main_net_inflow: f64,   // 主力净流入（万元）
    pub main_net_pct: f64,      // 主力净占比 %
    pub pct_5d: f64,            // 5日涨幅 %
    pub pct_20d: f64,           // 20日涨幅 %
    pub pct_60d: f64,           // 60日涨幅 %
    pub score: u32,             // 综合得分 0-100
    pub score_detail: FactorScoreDetail,
    #[serde(default)]
    pub sentiment_score: f64,   // 消息面得分 0-1
    #[serde(default)]
    pub news_heat: f64,         // 消息热度（越高越热）
    #[serde(default)]
    pub matched_themes: Vec<String>,
    pub labels: Vec<StockLabel>,
    pub instruction: Option<AIInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLabel {
    pub text: String,
    pub color: String,
    pub icon: Option<String>,
}

// === 旧结构兼容（ScoreWeights/ScoreThresholds/ScoreDetail）===
// 保留以便 serde 反序列化旧数据不报错

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    #[serde(default = "default_25")]
    pub bid_amount: u32,
    #[serde(default = "default_20")]
    pub volume_ratio: u32,
    #[serde(default = "default_20")]
    pub streak_days: u32,
    #[serde(default = "default_15")]
    pub turnover_rate: u32,
    #[serde(default = "default_10")]
    pub relative_strength: u32,
    #[serde(default = "default_10")]
    pub bid_pattern: u32,
}

fn default_25() -> u32 { 25 }
fn default_20() -> u32 { 20 }
fn default_15() -> u32 { 15 }
fn default_10() -> u32 { 10 }

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            bid_amount: 25,
            volume_ratio: 20,
            streak_days: 20,
            turnover_rate: 15,
            relative_strength: 10,
            bid_pattern: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreThresholds {
    #[serde(default = "default_50m")]
    pub bid_amount_full: f64,
    #[serde(default = "default_3f")]
    pub volume_ratio_full: f64,
    #[serde(default = "default_3f")]
    pub turnover_best_low: f64,
    #[serde(default = "default_8f")]
    pub turnover_best_high: f64,
    #[serde(default = "default_5f")]
    pub open_pct_strong: f64,
}

fn default_50m() -> f64 { 50_000_000.0 }
fn default_3f() -> f64 { 3.0 }
fn default_8f() -> f64 { 8.0 }
fn default_5f() -> f64 { 5.0 }

impl Default for ScoreThresholds {
    fn default() -> Self {
        Self {
            bid_amount_full: 50_000_000.0,
            volume_ratio_full: 3.0,
            turnover_best_low: 3.0,
            turnover_best_high: 8.0,
            open_pct_strong: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDetail {
    #[serde(default)]
    pub bid_amount_score: f64,
    #[serde(default)]
    pub volume_ratio_score: f64,
    #[serde(default)]
    pub streak_days_score: f64,
    #[serde(default)]
    pub turnover_rate_score: f64,
    #[serde(default)]
    pub rs_score: f64,
    #[serde(default)]
    pub bid_pattern_score: f64,
}
