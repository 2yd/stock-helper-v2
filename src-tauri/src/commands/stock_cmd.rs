use tauri::State;
use crate::models::stock::{StockInfo, KLineData, StockSearchResult, MarketStockSnapshot};
use crate::services::stock_data::{StockDataService, format_stock_code};
use crate::services::market_scanner::MarketScanner;
use crate::utils::http::build_stock_client;
use crate::AppState;

#[tauri::command]
pub async fn get_realtime_data(
    state: State<'_, AppState>,
    codes: Vec<String>,
) -> Result<Vec<StockInfo>, String> {
    let formatted: Vec<String> = codes.iter().map(|c| format_stock_code(c)).collect();
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;
    let use_sina = matches!(settings.data_source_primary, crate::models::settings::DataSource::Sina);

    let service = StockDataService::new().map_err(|e| e.to_string())?;
    service.get_realtime_batch(&formatted, use_sina).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_kline_data(
    code: String,
    scale: String,
    days: u32,
) -> Result<Vec<KLineData>, String> {
    let service = StockDataService::new().map_err(|e| e.to_string())?;
    let formatted = format_stock_code(&code);
    service.get_kline_data(&formatted, &scale, days).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_stocks(keyword: String) -> Result<Vec<StockSearchResult>, String> {
    let keyword = keyword.trim().to_string();
    if keyword.is_empty() {
        return Ok(vec![]);
    }

    let client = build_stock_client().map_err(|e| e.to_string())?;
    let url = format!(
        "https://searchapi.eastmoney.com/api/suggest/get?input={}&type=14&token=D43BF722C8E33BDC906FB84D85E326E8&count=15",
        urlencoding::encode(&keyword)
    );

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    if let Some(data) = body.get("QuotationCodeTable")
        .and_then(|t| t.get("Data"))
        .and_then(|d| d.as_array())
    {
        for item in data {
            let classify = item.get("Classify").and_then(|v| v.as_str()).unwrap_or("");
            // Only include A-shares
            if classify != "AStock" {
                continue;
            }
            let code = item.get("Code").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let security_type_name = item.get("SecurityTypeName").and_then(|v| v.as_str()).unwrap_or("");

            // Convert to sh/sz prefix format
            let full_code = if security_type_name.contains("沪") {
                format!("sh{}", code)
            } else {
                format!("sz{}", code)
            };

            results.push(StockSearchResult {
                code: full_code,
                name,
                market: security_type_name.to_string(),
            });
        }
    }

    Ok(results)
}

/// 获取指定代码列表的多维度快照（PE/PB/ROE/市值/换手率/量比/主力净流入/5日%/20日%等）
#[tauri::command]
pub async fn get_watchlist_enriched(
    codes: Vec<String>,
) -> Result<Vec<MarketStockSnapshot>, String> {
    if codes.is_empty() {
        return Ok(vec![]);
    }
    let scanner = MarketScanner::new().map_err(|e| e.to_string())?;
    scanner.fetch_stocks_by_codes(&codes).await.map_err(|e| e.to_string())
}
