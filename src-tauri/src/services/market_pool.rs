use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::utils::http::build_stock_client;

/// 东方财富涨停池/连板池数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStock {
    pub code: String,
    pub name: String,
    pub change_pct: f64,
    pub streak_days: u32,
    pub limit_up_type: String, // 一字/T字/换手
    pub amount: f64,           // 成交额
    pub turnover_rate: f64,
    pub industry: String,
}

pub struct MarketPoolService {
    client: reqwest::Client,
}

/// 东方财富涨停板池接口返回结构
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct EastMoneyResp {
    result: Option<EastMoneyResult>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct EastMoneyResult {
    data: Option<Vec<serde_json::Value>>,
}

impl MarketPoolService {
    pub fn new() -> Result<Self> {
        let client = build_stock_client()?;
        Ok(Self { client })
    }

    /// 获取东方财富涨停池（当日涨停的股票列表）
    /// API: https://push2ex.eastmoney.com/getTopicZTPool
    pub async fn fetch_limit_up_pool(&self, date: &str) -> Result<Vec<PoolStock>> {
        let url = format!(
            "https://push2ex.eastmoney.com/getTopicZTPool?ut=7eea3edcaed734bea9cb3f4cbb3b8f09&dpt=wz.ztzt&Ession_Id=1&date={}&_={}",
            date,
            chrono::Utc::now().timestamp_millis()
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        let pool = body.get("data").and_then(|d| d.get("pool"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| anyhow!("涨停池数据格式错误"))?;

        let mut stocks = Vec::new();
        for item in pool {
            let code_raw = item.get("c").and_then(|v| v.as_str()).unwrap_or("");
            let name = item.get("n").and_then(|v| v.as_str()).unwrap_or("");
            let zdp = item.get("zdp").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let days = item.get("days").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let amount = item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let hs = item.get("hs").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fund = item.get("fund").and_then(|v| v.as_str()).unwrap_or("");
            let zttj = item.get("zttj").and_then(|v| v.as_object());
            let limit_type = zttj.and_then(|t| t.get("ct").and_then(|v| v.as_str()))
                .unwrap_or("换手");

            let code = normalize_eastmoney_code(code_raw);
            if code.is_empty() { continue; }

            stocks.push(PoolStock {
                code,
                name: name.to_string(),
                change_pct: zdp,
                streak_days: days,
                limit_up_type: limit_type.to_string(),
                amount,
                turnover_rate: hs,
                industry: fund.to_string(),
            });
        }

        // Sort by streak days descending (高标优先)
        stocks.sort_by(|a, b| b.streak_days.cmp(&a.streak_days));
        Ok(stocks)
    }

    /// 获取连板股池（>=2板的股票）
    /// API: https://push2ex.eastmoney.com/getTopicLBPool
    pub async fn fetch_streak_pool(&self, date: &str) -> Result<Vec<PoolStock>> {
        let url = format!(
            "https://push2ex.eastmoney.com/getTopicLBPool?ut=7eea3edcaed734bea9cb3f4cbb3b8f09&dpt=wz.ztzt&Ession_Id=1&date={}&_={}",
            date,
            chrono::Utc::now().timestamp_millis()
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        let pool = body.get("data").and_then(|d| d.get("pool"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| anyhow!("连板池数据格式错误"))?;

        let mut stocks = Vec::new();
        for item in pool {
            let code_raw = item.get("c").and_then(|v| v.as_str()).unwrap_or("");
            let name = item.get("n").and_then(|v| v.as_str()).unwrap_or("");
            let zdp = item.get("zdp").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let days = item.get("lbs").and_then(|v| v.as_u64()).unwrap_or(2) as u32;
            let amount = item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let hs = item.get("hs").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fund = item.get("fund").and_then(|v| v.as_str()).unwrap_or("");

            let code = normalize_eastmoney_code(code_raw);
            if code.is_empty() { continue; }

            stocks.push(PoolStock {
                code,
                name: name.to_string(),
                change_pct: zdp,
                streak_days: days,
                limit_up_type: "连板".to_string(),
                amount,
                turnover_rate: hs,
                industry: fund.to_string(),
            });
        }

        stocks.sort_by(|a, b| b.streak_days.cmp(&a.streak_days));
        Ok(stocks)
    }

    /// 一键获取高标池：连板池(>=2板) + 当日涨停池，去重合并
    pub async fn fetch_high_pool(&self) -> Result<Vec<PoolStock>> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let (streak_result, zt_result) = tokio::join!(
            self.fetch_streak_pool(&today),
            self.fetch_limit_up_pool(&today),
        );

        let mut stocks = streak_result.unwrap_or_default();
        let zt_stocks = zt_result.unwrap_or_default();

        // Merge: add limit-up stocks not already in streak pool
        let existing_codes: std::collections::HashSet<String> =
            stocks.iter().map(|s| s.code.clone()).collect();
        for s in zt_stocks {
            if !existing_codes.contains(&s.code) {
                stocks.push(s);
            }
        }

        stocks.sort_by(|a, b| b.streak_days.cmp(&a.streak_days));
        Ok(stocks)
    }
}

/// 东方财富代码 -> sh/sz 前缀格式
fn normalize_eastmoney_code(raw: &str) -> String {
    let raw = raw.trim();
    // 东方财富格式可能是 "000890" 或 "0.000890" 或带前缀
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 6 {
        return String::new();
    }
    // Take last 6 digits
    let code = &digits[digits.len() - 6..];
    match code.chars().next() {
        Some('6') => format!("sh{}", code),
        Some('0') | Some('3') => format!("sz{}", code),
        Some('8') | Some('4') => format!("bj{}", code),
        _ => format!("sz{}", code),
    }
}
