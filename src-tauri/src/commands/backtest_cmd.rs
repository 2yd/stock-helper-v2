use tauri::State;
use crate::AppState;
use crate::models::backtest::*;
use crate::models::watchlist::KlineItem;
use crate::models::stock::StockDailyHistory;
use crate::services::history_kline::HistoryKlineService;
use crate::services::backtest_engine;
use std::collections::HashMap;

#[tauri::command]
pub async fn run_backtest(
    state: State<'_, AppState>,
    config: BacktestConfig,
) -> Result<BacktestResult, String> {
    let kline_service = HistoryKlineService::new().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut kline_map: HashMap<String, Vec<KlineItem>> = HashMap::new();

    // Fetch kline data for each stock
    for code in &config.codes {
        let klines = fetch_and_cache_klines(
            &state, &kline_service, code, &config.start_date, &config.end_date, &today
        ).await?;
        kline_map.insert(code.clone(), klines);
    }

    // Fetch benchmark (沪深300)
    let benchmark_code = "sh000300";
    let benchmark_klines = fetch_and_cache_klines(
        &state, &kline_service, benchmark_code, &config.start_date, &config.end_date, &today
    ).await?;

    // Run backtest engine
    let result = backtest_engine::run_backtest(&config, &kline_map, &benchmark_klines);

    Ok(result)
}

#[tauri::command]
pub async fn fetch_history_kline(
    state: State<'_, AppState>,
    code: String,
    start_date: String,
    end_date: String,
) -> Result<Vec<KlineItem>, String> {
    let kline_service = HistoryKlineService::new().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    fetch_and_cache_klines(&state, &kline_service, &code, &start_date, &end_date, &today).await
}

async fn fetch_and_cache_klines(
    state: &State<'_, AppState>,
    kline_service: &HistoryKlineService,
    code: &str,
    start_date: &str,
    end_date: &str,
    today: &str,
) -> Result<Vec<KlineItem>, String> {
    // Check cache
    let latest_date = state.db.get_latest_history_date(code).map_err(|e| e.to_string())?;

    // Fetch new data if needed
    let need_fetch = match &latest_date {
        Some(latest) => latest.as_str() < today,
        None => true,
    };

    if need_fetch {
        let new_items = if let Some(ref latest) = latest_date {
            kline_service.fetch_kline_incremental(code, "day", latest, today)
                .await.map_err(|e| e.to_string())?
        } else {
            kline_service.fetch_kline_full(code, "day", start_date, today)
                .await.map_err(|e| e.to_string())?
        };

        if !new_items.is_empty() {
            let history_records: Vec<StockDailyHistory> = new_items.iter().map(|k| StockDailyHistory {
                code: code.to_string(),
                date: k.date.clone(),
                close: k.close,
                high: k.high,
                low: k.low,
                open: k.open,
                volume: k.volume,
                amount: k.amount,
                change_pct: k.change_pct,
                is_limit_up: false,
                turnover_rate: k.turnover_rate,
            }).collect();
            let _ = state.db.save_daily_history(&history_records);
        }
    }

    // Load from cache
    let cached = state.db.get_daily_history_range(code, start_date, end_date)
        .map_err(|e| e.to_string())?;

    Ok(cached.iter().map(|h| KlineItem {
        date: h.date.clone(),
        open: h.open,
        close: h.close,
        high: h.high,
        low: h.low,
        volume: h.volume,
        amount: h.amount,
        change_pct: h.change_pct,
        turnover_rate: h.turnover_rate,
    }).collect())
}
