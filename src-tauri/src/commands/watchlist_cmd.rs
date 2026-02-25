use tauri::{State, Emitter, AppHandle};
use crate::AppState;
use crate::models::watchlist::*;
use crate::models::ai::{AIAnalysisResult, AIStreamEvent};
use crate::models::stock::StockDailyHistory;
use crate::services::history_kline::HistoryKlineService;
use crate::services::technical_indicators;
use crate::services::ai_service::AIService;

#[tauri::command]
pub async fn add_watchlist_stock(
    state: State<'_, AppState>,
    code: String,
    name: String,
) -> Result<(), String> {
    let stock = WatchlistStock {
        code,
        name,
        sort_order: 0,
        group_name: String::new(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    state.db.add_watchlist_stock(&stock).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_watchlist_stock(
    state: State<'_, AppState>,
    code: String,
) -> Result<(), String> {
    state.db.remove_watchlist_stock(&code).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_watchlist_stocks(
    state: State<'_, AppState>,
) -> Result<Vec<WatchlistStock>, String> {
    state.db.get_watchlist_stocks().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reorder_watchlist(
    state: State<'_, AppState>,
    codes: Vec<String>,
) -> Result<(), String> {
    state.db.reorder_watchlist(&codes).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stock_technical_analysis(
    state: State<'_, AppState>,
    code: String,
    name: String,
    period: String,
) -> Result<StockTechnicalAnalysis, String> {
    let period = if period.is_empty() { "day".to_string() } else { period };
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Check cached data
    let latest_date = state.db.get_latest_history_date(&code).map_err(|e| e.to_string())?;

    // Fetch from remote if needed
    let kline_service = HistoryKlineService::new().map_err(|e| e.to_string())?;

    let start_date = "2023-01-01".to_string();
    let new_items = if let Some(ref latest) = latest_date {
        kline_service.fetch_kline_incremental(&code, &period, latest, &today)
            .await.map_err(|e| e.to_string())?
    } else {
        kline_service.fetch_kline_full(&code, &period, &start_date, &today)
            .await.map_err(|e| e.to_string())?
    };

    // Save new data to DB
    if !new_items.is_empty() {
        let history_records: Vec<StockDailyHistory> = new_items.iter().map(|k| StockDailyHistory {
            code: code.clone(),
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

    // Load all cached data
    let cached = state.db.get_daily_history_asc(&code, 500).map_err(|e| e.to_string())?;

    let kline_data: Vec<KlineItem> = cached.iter().map(|h| KlineItem {
        date: h.date.clone(),
        open: h.open,
        close: h.close,
        high: h.high,
        low: h.low,
        volume: h.volume,
        amount: h.amount,
        change_pct: h.change_pct,
        turnover_rate: h.turnover_rate,
    }).collect();

    if kline_data.is_empty() {
        return Err("无K线数据".to_string());
    }

    // Compute indicators
    let indicators = technical_indicators::compute_indicators(&kline_data);
    let signals = technical_indicators::detect_signals(&kline_data, &indicators);
    let ma_alignment = technical_indicators::determine_ma_alignment(&indicators);
    let volume_price_relation = technical_indicators::determine_volume_price_relation(&kline_data);
    let summary = technical_indicators::generate_summary(&ma_alignment, &volume_price_relation, &signals);

    Ok(StockTechnicalAnalysis {
        code,
        name,
        kline_data,
        indicators,
        signals,
        ma_alignment,
        volume_price_relation,
        summary,
    })
}

/// AI 诊断股票（Agent 模式：AI 自主调用工具获取真实数据后分析）
#[tauri::command]
pub async fn ai_diagnose_stock(
    state: State<'_, AppState>,
    app: AppHandle,
    code: String,
    name: String,
    #[allow(unused_variables)]
    technical_summary: String,
) -> Result<(), String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;

    let ai_config = settings.ai_configs.iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .cloned()
        .ok_or("未配置AI模型".to_string())?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AIStreamEvent>(100);

    let app_clone = app.clone();
    let code_clone = code.clone();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app_clone.emit(&format!("ai-diagnose-{}", code_clone), &event);
        }
    });

    let result = AIService::diagnose_stock_with_tools(
        &ai_config,
        &code,
        &name,
        tx,
    ).await.map_err(|e| e.to_string())?;

    let analysis = AIAnalysisResult {
        id: uuid::Uuid::new_v4().to_string(),
        code: code.clone(),
        name: name.clone(),
        model_name: ai_config.model_name.clone(),
        question: "AI技术诊断(Agent)".to_string(),
        content: result.0,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    let _ = state.db.save_ai_analysis(&analysis);

    if let Some(usage) = result.1 {
        let _ = state.db.record_token_usage(&ai_config.model_name, usage.prompt_tokens, usage.completion_tokens);
    }

    Ok(())
}
