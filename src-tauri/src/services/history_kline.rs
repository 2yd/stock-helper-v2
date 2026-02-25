use anyhow::{Result, anyhow};
use crate::models::watchlist::KlineItem;
use crate::utils::http::build_stock_client;

const QQ_KLINE_URL: &str = "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get";

pub struct HistoryKlineService {
    client: reqwest::Client,
}

impl HistoryKlineService {
    pub fn new() -> Result<Self> {
        let client = build_stock_client()?;
        Ok(Self { client })
    }

    /// 从腾讯接口拉取前复权日K线数据
    /// code: sh600519 / sz000001 格式
    /// period: day / week / month
    /// start: 2024-01-01
    /// end: 2025-02-25
    /// count: 最大条数（单次最多约640条）
    pub async fn fetch_kline(
        &self,
        code: &str,
        period: &str,
        start: &str,
        end: &str,
        count: u32,
    ) -> Result<Vec<KlineItem>> {
        let fq = "qfq";
        let param = format!("{},{},{},{},{},{}", code, period, start, end, count, fq);
        let url = format!("{}?param={}", QQ_KLINE_URL, param);

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("腾讯K线数据JSON解析失败: {}", e))?;

        let code_key = code.to_lowercase();
        let data = json.get("data")
            .and_then(|d| d.get(&code_key))
            .ok_or_else(|| anyhow!("腾讯K线数据中未找到 {} 的数据", code))?;

        // 前复权数据在 qfqday/qfqweek/qfqmonth 字段
        // 指数数据在 day/week/month 字段
        let period_key = format!("qfq{}", period);
        let klines = data.get(&period_key)
            .or_else(|| data.get(period))
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("腾讯K线数据中未找到 {} 字段", period_key))?;

        let mut items = Vec::with_capacity(klines.len());
        for kline in klines {
            let arr = kline.as_array();
            if let Some(arr) = arr {
                if arr.len() >= 6 {
                    let item = KlineItem {
                        date: arr[0].as_str().unwrap_or("").to_string(),
                        open: parse_kline_f64(&arr[1]),
                        close: parse_kline_f64(&arr[2]),
                        high: parse_kline_f64(&arr[3]),
                        low: parse_kline_f64(&arr[4]),
                        volume: parse_kline_f64(&arr[5]),
                        amount: 0.0,
                        change_pct: 0.0,
                        turnover_rate: 0.0,
                    };
                    items.push(item);
                }
            }
        }

        // 计算涨跌幅
        for i in 1..items.len() {
            let prev_close = items[i - 1].close;
            if prev_close > 0.0 {
                items[i].change_pct = (items[i].close - prev_close) / prev_close * 100.0;
            }
        }

        Ok(items)
    }

    /// 分段拉取长周期K线数据（超过640条时分段）
    pub async fn fetch_kline_full(
        &self,
        code: &str,
        period: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<KlineItem>> {
        let mut all_items = Vec::new();
        let mut current_start = start.to_string();
        let max_per_request = 640u32;

        loop {
            let items = self.fetch_kline(code, period, &current_start, end, max_per_request).await?;
            if items.is_empty() {
                break;
            }

            let last_date = items.last().unwrap().date.clone();
            let is_last_batch = items.len() < max_per_request as usize;

            // 去重合并
            if !all_items.is_empty() {
                let existing_last: &KlineItem = &all_items[all_items.len() - 1];
                let skip = items.iter().position(|item| item.date > existing_last.date).unwrap_or(items.len());
                all_items.extend_from_slice(&items[skip..]);
            } else {
                all_items = items;
            }

            if is_last_batch {
                break;
            }

            // 下一段从最后日期的下一天开始
            if let Some(next_date) = next_day(&last_date) {
                current_start = next_date;
            } else {
                break;
            }
        }

        Ok(all_items)
    }

    /// 增量拉取：从指定日期之后开始拉取
    pub async fn fetch_kline_incremental(
        &self,
        code: &str,
        period: &str,
        latest_date: &str,
        end: &str,
    ) -> Result<Vec<KlineItem>> {
        let start = next_day(latest_date).unwrap_or_else(|| latest_date.to_string());
        if start > end.to_string() {
            return Ok(vec![]);
        }
        self.fetch_kline(code, period, &start, end, 640).await
    }
}

fn parse_kline_f64(val: &serde_json::Value) -> f64 {
    match val {
        serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn next_day(date_str: &str) -> Option<String> {
    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let next = date + chrono::Duration::days(1);
    Some(next.format("%Y-%m-%d").to_string())
}
