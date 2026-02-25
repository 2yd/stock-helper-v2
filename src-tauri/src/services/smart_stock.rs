use anyhow::{Result, anyhow};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT, HOST};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::time::Duration;

/// 东财 API 的 code 字段可能是字符串 "100" 或数字 100，统一反序列化为 i32
fn deserialize_string_or_i32<'de, D>(deserializer: D) -> std::result::Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::Number(n) => n.as_i64().map(|i| i as i32).ok_or_else(|| serde::de::Error::custom("invalid number")),
        Value::String(s) => s.parse::<i32>().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("expected string or number")),
    }
}

/// 东方财富智能选股 NLP API
/// 对标 go-stock 的 search_stock_api.go

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartStockColumn {
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(rename = "dateMsg", default)]
    pub date_msg: Option<String>,
    #[serde(rename = "hiddenNeed", default)]
    pub hidden_need: bool,
    #[serde(default)]
    pub children: Option<Vec<SmartStockColumn>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartStockResult {
    pub columns: Vec<SmartStockColumn>,
    #[serde(rename = "dataList")]
    pub data_list: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceInfo {
    #[serde(rename = "showText", default)]
    pub show_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartStockData {
    pub result: SmartStockResult,
    #[serde(rename = "traceInfo", default)]
    pub trace_info: Option<TraceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartStockResponse {
    #[serde(deserialize_with = "deserialize_string_or_i32")]
    pub code: i32,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub data: Option<SmartStockData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStrategyItem {
    pub rank: i32,
    pub question: String,
    #[serde(default)]
    pub chg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStrategyResponse {
    #[serde(deserialize_with = "deserialize_string_or_i32")]
    pub code: i32,
    #[serde(default)]
    pub data: Vec<HotStrategyItem>,
}

pub struct SmartStockService;

impl SmartStockService {
    fn build_client(host: &'static str) -> Result<reqwest::Client> {
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_static(host));
        headers.insert(ORIGIN, HeaderValue::from_static("https://xuangu.eastmoney.com"));
        headers.insert(REFERER, HeaderValue::from_static("https://xuangu.eastmoney.com/"));
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0"
        ));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .build()?;
        Ok(client)
    }

    fn build_body(keyword: &str, page_size: usize, qgqp_b_id: &str) -> Value {
        let timestamp = chrono::Utc::now().timestamp();
        serde_json::json!({
            "keyWord": keyword,
            "pageSize": page_size,
            "pageNo": 1,
            "fingerprint": qgqp_b_id,
            "gids": [],
            "matchWord": "",
            "timestamp": timestamp.to_string(),
            "shareToGuba": false,
            "requestId": "",
            "needCorrect": true,
            "removedConditionIdList": [],
            "xcId": "",
            "ownSelectAll": false,
            "dxInfo": [],
            "extraCondition": ""
        })
    }

    /// 智能选股 - 自然语言条件搜索A股
    pub async fn search_stock(keyword: &str, page_size: usize, qgqp_b_id: &str) -> Result<SmartStockResponse> {
        if qgqp_b_id.is_empty() {
            return Err(anyhow!(
                "请先配置东财用户标识（qgqp_b_id）：\n\
                 1. 打开浏览器访问 https://xuangu.eastmoney.com\n\
                 2. 按 F12 打开开发者工具 → 网络面板\n\
                 3. 随便点开一个请求，复制 Cookie 中 qgqp_b_id 的值\n\
                 4. 在设置中粘贴保存"
            ));
        }

        let client = Self::build_client("np-tjxg-g.eastmoney.com")?;
        let body = Self::build_body(keyword, page_size, qgqp_b_id);

        let url = "https://np-tjxg-g.eastmoney.com/api/smart-tag/stock/v3/pw/search-code";
        let resp = client.post(url).json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("东财选股API请求失败 ({}): {}", status, &text[..200.min(text.len())]));
        }

        let response: SmartStockResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("解析东财选股响应失败: {} body: {}", e, &text[..300.min(text.len())]))?;

        Ok(response)
    }

    /// 智能板块搜索 - 自然语言搜索概念板块/行业板块
    pub async fn search_board(keyword: &str, page_size: usize, qgqp_b_id: &str) -> Result<SmartStockResponse> {
        if qgqp_b_id.is_empty() {
            return Err(anyhow!(
                "请先配置东财用户标识（qgqp_b_id）"
            ));
        }

        let client = Self::build_client("np-tjxg-b.eastmoney.com")?;
        let body = Self::build_body(keyword, page_size, qgqp_b_id);

        let url = "https://np-tjxg-b.eastmoney.com/api/smart-tag/bkc/v3/pw/search-code";
        let resp = client.post(url).json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("东财板块搜索API请求失败 ({}): {}", status, &text[..200.min(text.len())]));
        }

        let response: SmartStockResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("解析东财板块搜索响应失败: {} body: {}", e, &text[..300.min(text.len())]))?;

        Ok(response)
    }

    /// 获取热门选股策略列表
    pub async fn get_hot_strategies() -> Result<Vec<HotStrategyItem>> {
        let timestamp = chrono::Utc::now().timestamp();
        let url = format!(
            "https://np-ipick.eastmoney.com/recommend/stock/heat/ranking?count=20&trace={}&client=web&biz=web_smart_tag",
            timestamp
        );

        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_static("np-ipick.eastmoney.com"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://xuangu.eastmoney.com"));
        headers.insert(REFERER, HeaderValue::from_static("https://xuangu.eastmoney.com/"));
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0"
        ));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .gzip(true)
            .build()?;

        let resp = client.get(&url).send().await?;
        let text = resp.text().await?;

        let response: HotStrategyResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("解析热门策略失败: {} body: {}", e, &text[..200.min(text.len())]))?;

        if response.code == 1 {
            Ok(response.data)
        } else {
            Ok(vec![])
        }
    }
}
