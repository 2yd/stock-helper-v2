use serde::{Deserialize, Serialize};

/// 全市场股票快照数据（来自东方财富 clist API）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStockSnapshot {
    pub code: String,          // "sz000001"
    pub name: String,
    pub price: f64,            // 最新价
    pub change_pct: f64,       // 涨跌幅 %
    pub change_amount: f64,    // 涨跌额
    pub volume: f64,           // 成交量（手）
    pub amount: f64,           // 成交额（元）
    pub amplitude: f64,        // 振幅 %
    pub turnover_rate: f64,    // 换手率 %
    pub pe_ttm: f64,           // 市盈率(动态)
    pub pb: f64,               // 市净率
    pub total_market_cap: f64, // 总市值（元）
    pub float_market_cap: f64, // 流通市值（元）
    pub volume_ratio: f64,     // 量比
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub pre_close: f64,
    pub pct_5d: f64,           // 5日涨幅 %
    pub pct_20d: f64,          // 20日涨幅 %
    pub pct_60d: f64,          // 60日涨幅 %
    pub roe: f64,              // 净资产收益率 %（来自财报）
    pub gross_margin: f64,     // 毛利率 %
    pub revenue_yoy: f64,      // 营收同比增长 %
    pub profit_yoy: f64,       // 净利润同比增长 %
    pub main_net_inflow: f64,  // 主力净流入（元）
    pub main_net_pct: f64,     // 主力净占比 %
    #[serde(default)]
    pub list_date: String,     // 上市日期 "YYYYMMDD"（来自东财 f26）
}

/// 实时行情数据（用于已选股票的详细盘口）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub open: f64,
    pub pre_close: f64,
    pub price: f64,
    pub high: f64,
    pub low: f64,
    pub bid: f64,
    pub ask: f64,
    pub volume: f64,
    pub amount: f64,
    pub buy1_vol: f64,
    pub buy1_price: f64,
    pub buy2_vol: f64,
    pub buy2_price: f64,
    pub buy3_vol: f64,
    pub buy3_price: f64,
    pub buy4_vol: f64,
    pub buy4_price: f64,
    pub buy5_vol: f64,
    pub buy5_price: f64,
    pub sell1_vol: f64,
    pub sell1_price: f64,
    pub sell2_vol: f64,
    pub sell2_price: f64,
    pub sell3_vol: f64,
    pub sell3_price: f64,
    pub sell4_vol: f64,
    pub sell4_price: f64,
    pub sell5_vol: f64,
    pub sell5_price: f64,
    pub date: String,
    pub time: String,
}

impl StockInfo {
    pub fn change_percent(&self) -> f64 {
        if self.pre_close == 0.0 {
            return 0.0;
        }
        (self.price - self.pre_close) / self.pre_close * 100.0
    }

    pub fn open_percent(&self) -> f64 {
        if self.pre_close == 0.0 {
            return 0.0;
        }
        (self.open - self.pre_close) / self.pre_close * 100.0
    }

    pub fn change_price(&self) -> f64 {
        self.price - self.pre_close
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBasic {
    pub ts_code: String,
    pub symbol: String,
    pub name: String,
    pub area: String,
    pub industry: String,
    pub market: String,
    pub list_date: String,
    pub total_share: f64,
    pub float_share: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLineData {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockDailyHistory {
    pub code: String,
    pub date: String,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub volume: f64,
    pub amount: f64,
    pub change_pct: f64,
    pub is_limit_up: bool,
    pub turnover_rate: f64,
}

/// 股票搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSearchResult {
    pub code: String,
    pub name: String,
    pub market: String,
}
