use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::models::settings::AppSettings;
use crate::models::ai::AIConfig;
use crate::models::ai::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage};
use crate::services::stock_data::StockDataService;
use crate::services::scheduler::TradingScheduler;
use crate::services::stock_tools;
use crate::utils::http::{build_stock_client, build_ai_client};

// ============================================================
// 数据结构定义
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketOverview {
    pub market_status: String,
    pub indexes: Vec<IndexQuote>,
    pub market_stats: MarketStats,
    pub sentiment: SentimentInfo,
    pub sector_top: Vec<SectorInfo>,
    pub sector_bottom: Vec<SectorInfo>,
    pub global_indexes: Vec<GlobalIndex>,
    pub total_amount: f64,
    pub volume_compare: VolumeCompare,
    pub update_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexQuote {
    pub name: String,
    pub code: String,
    pub price: f64,
    pub change_pct: f64,
    pub change_amount: f64,
    pub amount: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketStats {
    pub rise_count: u32,
    pub fall_count: u32,
    pub flat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SentimentInfo {
    pub score: f64,
    pub level: String,
    pub money_effect: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SectorInfo {
    pub name: String,
    pub change_pct: f64,
    pub lead_stock: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeCompare {
    pub today_amount: f64,
    pub yesterday_amount: f64,
    pub diff: f64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalIndex {
    pub name: String,
    pub code: String,
    pub price: String,
    pub change_pct: String,
    pub region: String,
}

// ============================================================
// 聚合入口
// ============================================================

pub async fn fetch_overview(settings: &AppSettings) -> Result<MarketOverview> {
    let use_sina = matches!(settings.data_source_primary, crate::models::settings::DataSource::Sina);

    let (indexes_res, stats_res, sectors_res, global_res, kline_res) =
        tokio::join!(
            fetch_index_quotes(use_sina),
            fetch_market_stats(),
            fetch_sector_ranking_sina(),
            fetch_global_indexes(),
            fetch_prev_day_volume(),
        );

    let indexes = indexes_res.unwrap_or_default();
    let market_stats = stats_res.unwrap_or_default();
    let (sector_top, sector_bottom) = sectors_res.unwrap_or_default();
    let global_indexes = global_res.unwrap_or_default();
    let yesterday_amount = kline_res.unwrap_or(0.0);

    // 今日两市总成交额 = 三大指数成交额之和
    let total_amount: f64 = indexes.iter().map(|idx| idx.amount).sum();

    // 量能对比
    let volume_compare = VolumeCompare {
        today_amount: total_amount,
        yesterday_amount,
        diff: total_amount - yesterday_amount,
        ratio: if yesterday_amount > 0.0 { total_amount / yesterday_amount } else { 1.0 },
    };

    // 情绪评分
    let sentiment = calculate_sentiment(&market_stats, volume_compare.ratio);

    let update_time = chrono::Local::now().format("%H:%M:%S").to_string();
    let market_status = TradingScheduler::market_status();

    Ok(MarketOverview {
        market_status,
        indexes,
        market_stats,
        sentiment,
        sector_top,
        sector_bottom,
        global_indexes,
        total_amount,
        volume_compare,
        update_time,
    })
}

// ============================================================
// 三大指数实时行情
// ============================================================

async fn fetch_index_quotes(use_sina: bool) -> Result<Vec<IndexQuote>> {
    let svc = StockDataService::new()?;
    let codes = vec![
        "sh000001".to_string(), // 上证指数
        "sz399001".to_string(), // 深证成指
        "sz399006".to_string(), // 创业板指
    ];

    let stocks = svc.get_realtime_batch(&codes, use_sina).await?;

    let names = ["上证指数", "深证成指", "创业板指"];
    let quotes: Vec<IndexQuote> = stocks.into_iter().enumerate().map(|(i, s)| {
        let name = names.get(i).unwrap_or(&"未知");
        IndexQuote {
            name: name.to_string(),
            code: s.code.clone(),
            price: s.price,
            change_pct: s.change_percent(),
            change_amount: s.change_price(),
            amount: s.amount,
            open: s.open,
            high: s.high,
            low: s.low,
            pre_close: s.pre_close,
        }
    }).collect();

    Ok(quotes)
}

// ============================================================
// 涨跌家数统计 — 通过指数 f104/f105/f106 字段轻量获取
// ============================================================

async fn fetch_market_stats() -> Result<MarketStats> {
    let client = build_stock_client()?;
    let url = "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&invt=2&fields=f104,f105,f106&secids=1.000001";

    let text = client.get(url)
        .header("Referer", "https://quote.eastmoney.com/")
        .send()
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow!("涨跌家数数据解析失败: {}", e))?;

    let diff = json.get("data")
        .and_then(|d| d.get("diff"))
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| anyhow!("涨跌家数数据格式错误"))?;

    let rise = diff.get("f104").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let fall = diff.get("f105").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let flat = diff.get("f106").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    Ok(MarketStats {
        rise_count: rise,
        fall_count: fall,
        flat_count: flat,
    })
}

// ============================================================
// 板块涨跌排行 — 新浪行业板块接口
// ============================================================

async fn fetch_sector_ranking_sina() -> Result<(Vec<SectorInfo>, Vec<SectorInfo>)> {
    let client = build_stock_client()?;
    let url = "https://vip.stock.finance.sina.com.cn/q/view/newSinaHy.php";

    let text = client.get(url)
        .header("Referer", "https://finance.sina.com.cn/")
        .send()
        .await?
        .text()
        .await?;

    // 接口返回格式: var S_Finance_bankuai_sinaindustry = {"key":"val1,val2,...", ...}
    // 提取 JSON 对象部分
    let start = text.find('{').ok_or_else(|| anyhow!("新浪板块数据格式错误: 未找到{{"))?;
    let end = text.rfind('}').ok_or_else(|| anyhow!("新浪板块数据格式错误: 未找到}}"))?;
    let json_str = &text[start..=end];

    let map: std::collections::HashMap<String, String> = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("新浪板块JSON解析失败: {}", e))?;

    // 解析每个板块数据
    // value 格式: "板块代码,板块名称,股票数量,平均价格,平均涨跌额,平均涨跌幅(%),成交量,成交额,领涨股代码,领涨股涨幅,领涨股价格,领涨股涨跌额,领涨股名称"
    let mut sectors: Vec<SectorInfo> = Vec::new();
    for (_key, value) in &map {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() >= 13 {
            let name = parts[1].to_string();
            let change_pct: f64 = parts[5].parse().unwrap_or(0.0);
            let lead_stock = parts[12].to_string();
            sectors.push(SectorInfo {
                name,
                change_pct,
                lead_stock,
            });
        }
    }

    // 按涨跌幅降序排列
    sectors.sort_by(|a, b| b.change_pct.partial_cmp(&a.change_pct).unwrap_or(std::cmp::Ordering::Equal));

    let top5: Vec<SectorInfo> = sectors.iter().take(5).cloned().collect();
    let bottom5: Vec<SectorInfo> = sectors.iter().rev().take(5).cloned().collect();

    Ok((top5, bottom5))
}

// ============================================================
// 全球指数 — 复用 stock_tools
// ============================================================

async fn fetch_global_indexes() -> Result<Vec<GlobalIndex>> {
    let raw = stock_tools::get_global_indexes().await?;
    let json: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| anyhow!("全球指数解析失败: {}", e))?;

    let indexes = json.get("indexes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("全球指数数据格式错误"))?;

    let result: Vec<GlobalIndex> = indexes.iter().map(|item| {
        GlobalIndex {
            name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            code: item.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            price: item.get("price").and_then(|v| v.as_str()).unwrap_or("0").to_string(),
            change_pct: item.get("change_pct").and_then(|v| v.as_str()).unwrap_or("0%").to_string(),
            region: item.get("region").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        }
    }).collect();

    Ok(result)
}

// ============================================================
// 昨日成交额 — K线差值
// ============================================================

async fn fetch_prev_day_volume() -> Result<f64> {
    let svc = StockDataService::new()?;
    let klines = svc.get_kline_data("sh000001", "240", 2).await?;

    if klines.len() >= 2 {
        Ok(klines[klines.len() - 2].amount)
    } else if !klines.is_empty() {
        Ok(klines[0].amount)
    } else {
        Ok(0.0)
    }
}

// ============================================================
// 情绪评分算法 — 2维: 涨跌比(55%) + 量能变化(45%)
// ============================================================

fn calculate_sentiment(stats: &MarketStats, volume_ratio: f64) -> SentimentInfo {
    let total = (stats.rise_count + stats.fall_count + stats.flat_count) as f64;
    if total == 0.0 {
        return SentimentInfo {
            score: 50.0,
            level: "中性".to_string(),
            money_effect: 50.0,
        };
    }

    // 1. 涨跌比 (55%) — 上涨占比直接反映多空力量
    let rise_ratio = stats.rise_count as f64 / total;
    let rise_score = rise_ratio * 100.0;

    // 2. 量能变化 (45%) — ratio>1放量加分, <1缩量减分
    let vol_score = (volume_ratio * 50.0).min(100.0);

    let score = rise_score * 0.55 + vol_score * 0.45;
    let score = score.clamp(0.0, 100.0);

    let level = if score >= 75.0 {
        "极强"
    } else if score >= 60.0 {
        "偏强"
    } else if score >= 40.0 {
        "中性"
    } else if score >= 25.0 {
        "偏弱"
    } else {
        "极弱"
    };

    // 赚钱效应 = 上涨家数占比
    let money_effect = rise_ratio * 100.0;

    SentimentInfo {
        score: (score * 10.0).round() / 10.0,
        level: level.to_string(),
        money_effect: (money_effect * 10.0).round() / 10.0,
    }
}

// ============================================================
// AI 盘面解说 — 构造 Prompt + 非流式 LLM 调用
// ============================================================

pub async fn generate_market_comment(config: &AIConfig, overview_json: &str) -> Result<String> {
    let client = build_ai_client(config.timeout_secs)?;
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let system_prompt = r#"你是一位资深 A 股盘面解说员。请根据提供的实时大盘数据，生成精炼的盘面点评。

要求：
1. 第一句话用一句话定性当日行情（如"今日市场震荡走弱，创业板领跌"）
2. 随后用 2-3 句话补充分析关键信号（成交量变化、涨跌家数分布、热点板块等）
3. 语言专业简洁，避免废话，总字数控制在 80-150 字
4. 不要给投资建议，只做客观描述"#;

    let user_msg = format!("以下是当前A股大盘实时数据（JSON格式），请据此生成盘面点评：\n\n{}", overview_json);

    let req = ChatCompletionRequest {
        model: config.model_name.clone(),
        messages: vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(&user_msg),
        ],
        max_tokens: Some(300),
        temperature: Some(0.3),
        stream: Some(false),
        tools: None,
        tool_choice: None,
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| anyhow!("AI 盘面解说请求失败: {}", e))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| anyhow!("读取AI响应失败: {}", e))?;

    if !status.is_success() {
        return Err(anyhow!("AI 返回错误 ({}): {}", status.as_u16(), &body[..200.min(body.len())]));
    }

    let response: ChatCompletionResponse = serde_json::from_str(&body)
        .map_err(|e| anyhow!("AI 响应解析失败: {}", e))?;

    let reply = response.choices.first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m| m.content.clone())
        .unwrap_or_else(|| "暂无解说".to_string());

    Ok(reply.trim().to_string())
}
