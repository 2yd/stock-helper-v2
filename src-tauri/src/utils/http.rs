use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, ORIGIN, USER_AGENT, REFERER, HOST};
use std::time::Duration;

pub fn build_stock_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_static("https://finance.sina.com.cn/"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .gzip(true)
        .build()?;
    Ok(client)
}

pub fn build_ai_client(timeout_secs: u64) -> Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;
    Ok(client)
}

/// 东方财富 NLP 选股器专用 HTTP client
/// Origin/Referer 指向 xuangu.eastmoney.com，超时30秒
pub fn build_xuangu_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("np-tjxg-g.eastmoney.com"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://xuangu.eastmoney.com"));
    headers.insert(REFERER, HeaderValue::from_static("https://xuangu.eastmoney.com/"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .gzip(true)
        .build()?;
    Ok(client)
}

/// 东方财富数据中心 HTTP client（宏观经济数据）
pub fn build_datacenter_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("datacenter-web.eastmoney.com"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:140.0) Gecko/20100101 Firefox/140.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://datacenter.eastmoney.com"));
    headers.insert(REFERER, HeaderValue::from_static("https://data.eastmoney.com/cjsj/gdp.html"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(8))
        .gzip(true)
        .build()?;
    Ok(client)
}

/// 腾讯财经 HTTP client（全球指数）
pub fn build_qq_finance_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(REFERER, HeaderValue::from_static("https://stockapp.finance.qq.com/mstats"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(8))
        .gzip(true)
        .build()?;
    Ok(client)
}

/// 财联社 HTTP client（财经日历等）
pub fn build_cls_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://www.cls.cn"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.cls.cn/"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(8))
        .gzip(true)
        .build()?;
    Ok(client)
}
