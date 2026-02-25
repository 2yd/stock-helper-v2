use serde::{Deserialize, Serialize};
use super::strategy::FactorWeights;

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub codes: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    #[serde(default = "default_initial_capital")]
    pub initial_capital: f64,
    #[serde(default = "default_commission_rate")]
    pub commission_rate: f64,
    #[serde(default = "default_slippage")]
    pub slippage: f64,
    #[serde(default = "default_buy_threshold")]
    pub buy_threshold: f64,
    #[serde(default = "default_sell_threshold")]
    pub sell_threshold: f64,
    #[serde(default = "default_stop_loss")]
    pub stop_loss: f64,
    #[serde(default = "default_take_profit")]
    pub take_profit: f64,
    #[serde(default)]
    pub factor_weights: FactorWeights,
    #[serde(default = "default_max_position_pct")]
    pub max_position_pct: f64,
}

fn default_initial_capital() -> f64 { 1_000_000.0 }
fn default_commission_rate() -> f64 { 0.0003 }
fn default_slippage() -> f64 { 0.001 }
fn default_buy_threshold() -> f64 { 70.0 }
fn default_sell_threshold() -> f64 { 40.0 }
fn default_stop_loss() -> f64 { 0.08 }
fn default_take_profit() -> f64 { 0.20 }
fn default_max_position_pct() -> f64 { 0.25 }

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            codes: vec![],
            start_date: String::new(),
            end_date: String::new(),
            initial_capital: default_initial_capital(),
            commission_rate: default_commission_rate(),
            slippage: default_slippage(),
            buy_threshold: default_buy_threshold(),
            sell_threshold: default_sell_threshold(),
            stop_loss: default_stop_loss(),
            take_profit: default_take_profit(),
            factor_weights: FactorWeights::default(),
            max_position_pct: default_max_position_pct(),
        }
    }
}

/// 回测绩效指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestPerformance {
    pub total_return: f64,
    pub annual_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub profit_loss_ratio: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
    pub avg_holding_days: f64,
    pub benchmark_return: f64,
    pub alpha: f64,
}

impl Default for BacktestPerformance {
    fn default() -> Self {
        Self {
            total_return: 0.0,
            annual_return: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            profit_loss_ratio: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            avg_holding_days: 0.0,
            benchmark_return: 0.0,
            alpha: 0.0,
        }
    }
}

/// 权益曲线点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub date: String,
    pub equity: f64,
    pub benchmark: f64,
    pub drawdown: f64,
}

/// 回测交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub id: u32,
    pub code: String,
    pub name: String,
    pub direction: String,
    pub open_date: String,
    pub open_price: f64,
    pub close_date: String,
    pub close_price: f64,
    pub shares: f64,
    pub profit: f64,
    pub profit_pct: f64,
    pub holding_days: u32,
    pub commission: f64,
}

/// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub config: BacktestConfig,
    pub performance: BacktestPerformance,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<BacktestTrade>,
}

impl Default for BacktestResult {
    fn default() -> Self {
        Self {
            config: BacktestConfig::default(),
            performance: BacktestPerformance::default(),
            equity_curve: vec![],
            trades: vec![],
        }
    }
}
