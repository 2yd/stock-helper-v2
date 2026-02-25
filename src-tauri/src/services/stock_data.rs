use anyhow::{Result, anyhow};
use crate::models::stock::{StockInfo, KLineData};
use crate::utils::encoding::gb18030_to_utf8;
use crate::utils::http::build_stock_client;

#[allow(dead_code)]
const SINA_STOCK_URL: &str = "http://hq.sinajs.cn/rn={}&list={}";
#[allow(dead_code)]
const TX_STOCK_URL: &str = "http://qt.gtimg.cn/?_={}&q={}";
const SINA_KLINE_URL: &str = "http://quotes.sina.cn/cn/api/json_v2.php/CN_MarketDataService.getKLineData";

pub struct StockDataService {
    client: reqwest::Client,
}

impl StockDataService {
    pub fn new() -> Result<Self> {
        let client = build_stock_client()?;
        Ok(Self { client })
    }

    pub async fn get_realtime_data_sina(&self, codes: &[String]) -> Result<Vec<StockInfo>> {
        if codes.is_empty() {
            return Ok(vec![]);
        }
        let code_list = codes.join(",");
        let ts = chrono::Utc::now().timestamp();
        let url = format!("http://hq.sinajs.cn/rn={}&list={}", ts, code_list);

        let resp = self.client.get(&url).send().await?;
        let bytes = resp.bytes().await?;
        let text = gb18030_to_utf8(&bytes);

        let mut results = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(stock) = parse_sina_shsz_line(line) {
                results.push(stock);
            }
        }
        Ok(results)
    }

    pub async fn get_realtime_data_tencent(&self, codes: &[String]) -> Result<Vec<StockInfo>> {
        if codes.is_empty() {
            return Ok(vec![]);
        }
        let code_list = codes.join(",");
        let ts = chrono::Utc::now().timestamp();
        let url = format!("http://qt.gtimg.cn/?_={}&q={}", ts, code_list);

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let mut results = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(stock) = parse_tencent_shsz_line(line) {
                results.push(stock);
            }
        }
        Ok(results)
    }

    pub async fn get_kline_data(&self, code: &str, scale: &str, days: u32) -> Result<Vec<KLineData>> {
        let url = format!(
            "{}?symbol={}&scale={}&ma=yes&datalen={}",
            SINA_KLINE_URL, code, scale, days
        );
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let data: Vec<SinaKLineItem> = serde_json::from_str(&text)
            .map_err(|e| anyhow!("K线数据解析失败: {}", e))?;

        Ok(data
            .into_iter()
            .map(|item| KLineData {
                date: item.day,
                open: item.open.parse().unwrap_or(0.0),
                high: item.high.parse().unwrap_or(0.0),
                low: item.low.parse().unwrap_or(0.0),
                close: item.close.parse().unwrap_or(0.0),
                volume: item.volume.parse().unwrap_or(0.0),
                amount: 0.0,
            })
            .collect())
    }

    pub async fn get_realtime_batch(&self, codes: &[String], use_sina: bool) -> Result<Vec<StockInfo>> {
        if use_sina {
            self.get_realtime_data_sina(codes).await
        } else {
            self.get_realtime_data_tencent(codes).await
        }
    }
}

#[derive(serde::Deserialize)]
struct SinaKLineItem {
    day: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

fn parse_float(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}

fn parse_sina_shsz_line(line: &str) -> Option<StockInfo> {
    // Format: var hq_str_sh601006="大秦铁路,27.55,27.25,26.91,...";
    let eq_pos = line.find('=')?;
    let prefix = &line[..eq_pos];

    // Extract stock code from prefix: "var hq_str_sh601006" -> "sh601006"
    let code = prefix
        .strip_prefix("var hq_str_")?
        .trim()
        .to_string();

    // Only handle A-share (sh/sz/bj)
    if !code.starts_with("sh") && !code.starts_with("sz") && !code.starts_with("bj") {
        return None;
    }

    let data_str = line[eq_pos + 1..].trim().trim_matches('"').trim_end_matches(';');
    if data_str.is_empty() {
        return None;
    }

    let parts: Vec<&str> = data_str.split(',').collect();
    if parts.len() < 32 {
        return None;
    }

    Some(StockInfo {
        code,
        name: parts[0].to_string(),
        open: parse_float(parts[1]),
        pre_close: parse_float(parts[2]),
        price: parse_float(parts[3]),
        high: parse_float(parts[4]),
        low: parse_float(parts[5]),
        bid: parse_float(parts[6]),
        ask: parse_float(parts[7]),
        volume: parse_float(parts[8]),
        amount: parse_float(parts[9]),
        buy1_vol: parse_float(parts[10]),
        buy1_price: parse_float(parts[11]),
        buy2_vol: parse_float(parts[12]),
        buy2_price: parse_float(parts[13]),
        buy3_vol: parse_float(parts[14]),
        buy3_price: parse_float(parts[15]),
        buy4_vol: parse_float(parts[16]),
        buy4_price: parse_float(parts[17]),
        buy5_vol: parse_float(parts[18]),
        buy5_price: parse_float(parts[19]),
        sell1_vol: parse_float(parts[20]),
        sell1_price: parse_float(parts[21]),
        sell2_vol: parse_float(parts[22]),
        sell2_price: parse_float(parts[23]),
        sell3_vol: parse_float(parts[24]),
        sell3_price: parse_float(parts[25]),
        sell4_vol: parse_float(parts[26]),
        sell4_price: parse_float(parts[27]),
        sell5_vol: parse_float(parts[28]),
        sell5_price: parse_float(parts[29]),
        date: parts[30].to_string(),
        time: parts[31].to_string(),
    })
}

fn parse_tencent_shsz_line(line: &str) -> Option<StockInfo> {
    // Format: v_sz002241="51~歌尔股份~002241~22.26~22.27~0.00~...";
    let eq_pos = line.find('=')?;
    let prefix = &line[..eq_pos];

    let code_raw = prefix
        .strip_prefix("v_")?
        .trim()
        .to_string();

    // Only handle sh/sz
    if !code_raw.starts_with("sh") && !code_raw.starts_with("sz") {
        return None;
    }

    let data_str = line[eq_pos + 1..].trim().trim_matches('"').trim_end_matches(';');
    if data_str.is_empty() {
        return None;
    }

    let parts: Vec<&str> = data_str.split('~').collect();
    if parts.len() < 34 {
        return None;
    }

    // Parse date/time from parts[30] format: "20250509092233" or contains "/"
    let (date, time) = if parts.len() > 30 {
        parse_tx_datetime(parts[30])
    } else {
        (String::new(), String::new())
    };

    // Determine high/low indices based on format
    let (high, low) = if parts.len() > 33 {
        (parse_float(parts[33]), parse_float(parts[34].split_whitespace().next().unwrap_or("0")))
    } else {
        (0.0, 0.0)
    };

    // For A-share via tencent: high=parts[33], low=parts[34] when parts[30] doesn't contain "/"
    let (final_high, final_low) = if parts.len() > 30 && !parts[30].contains('/') && parts.len() > 34 {
        // A-share format: high at 33, low at 34
        (parse_float(parts[33]), parse_float(parts[34]))
    } else {
        (high, low)
    };

    Some(StockInfo {
        code: code_raw,
        name: parts[1].to_string(),
        price: parse_float(parts[3]),
        pre_close: parse_float(parts[4]),
        open: parse_float(parts[5]),
        high: final_high,
        low: final_low,
        bid: 0.0,
        ask: 0.0,
        volume: 0.0,
        amount: 0.0,
        buy1_price: parse_float(parts[9]),
        buy1_vol: parse_float(parts[10]),
        buy2_price: parse_float(parts[11]),
        buy2_vol: parse_float(parts[12]),
        buy3_price: parse_float(parts[13]),
        buy3_vol: parse_float(parts[14]),
        buy4_price: parse_float(parts[15]),
        buy4_vol: parse_float(parts[16]),
        buy5_price: parse_float(parts[17]),
        buy5_vol: parse_float(parts[18]),
        sell1_price: parse_float(parts[19]),
        sell1_vol: parse_float(parts[20]),
        sell2_price: parse_float(parts[21]),
        sell2_vol: parse_float(parts[22]),
        sell3_price: parse_float(parts[23]),
        sell3_vol: parse_float(parts[24]),
        sell4_price: parse_float(parts[25]),
        sell4_vol: parse_float(parts[26]),
        sell5_price: parse_float(parts[27]),
        sell5_vol: parse_float(parts[28]),
        date,
        time,
    })
}

fn parse_tx_datetime(raw: &str) -> (String, String) {
    let s = raw.trim();
    if s.len() >= 14 && !s.contains('/') {
        // Format: 20250509092233
        let date = format!("{}-{}-{}", &s[..4], &s[4..6], &s[6..8]);
        let time = format!("{}:{}:{}", &s[8..10], &s[10..12], &s[12..14]);
        (date, time)
    } else if s.contains('/') {
        // HK format with /
        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() >= 2 {
            (parts[0].replace('/', "-"), parts[1].to_string())
        } else {
            (s.replace('/', "-"), String::new())
        }
    } else {
        (String::new(), String::new())
    }
}

pub fn format_stock_code(code: &str) -> String {
    let code = code.trim().to_lowercase();
    if code.starts_with("sh") || code.starts_with("sz") || code.starts_with("bj") {
        return code;
    }
    // Auto-detect exchange from pure number code
    let digits: String = code.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return code;
    }
    match digits.chars().next() {
        Some('6') => format!("sh{}", digits),
        Some('0') | Some('3') => format!("sz{}", digits),
        Some('8') | Some('9') => format!("bj{}", digits),
        _ => format!("sz{}", digits),
    }
}

pub fn code_to_pure(code: &str) -> String {
    code.chars().filter(|c| c.is_ascii_digit()).collect()
}
