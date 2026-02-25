use tauri::State;
use crate::AppState;
use crate::services::market_pool::{MarketPoolService, PoolStock};

/// 获取涨停池
#[tauri::command]
pub async fn fetch_limit_up_pool(date: Option<String>) -> Result<Vec<PoolStock>, String> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let service = MarketPoolService::new().map_err(|e| e.to_string())?;
    service.fetch_limit_up_pool(&date).await.map_err(|e| e.to_string())
}

/// 获取连板池
#[tauri::command]
pub async fn fetch_streak_pool(date: Option<String>) -> Result<Vec<PoolStock>, String> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let service = MarketPoolService::new().map_err(|e| e.to_string())?;
    service.fetch_streak_pool(&date).await.map_err(|e| e.to_string())
}

/// 一键获取高标池（连板+涨停去重合并）并自动更新 watch_codes
#[tauri::command]
pub async fn fetch_and_apply_high_pool(
    state: State<'_, AppState>,
    strategy_id: Option<String>,
) -> Result<Vec<PoolStock>, String> {
    let service = MarketPoolService::new().map_err(|e| e.to_string())?;
    let pool = service.fetch_high_pool().await.map_err(|e| e.to_string())?;

    let codes: Vec<String> = pool.iter().map(|s| s.code.clone()).collect();

    // Update runtime state
    {
        let mut watch = state.watch_codes.lock().unwrap();
        *watch = codes.clone();
    }

    // Persist to DB
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    let sid = strategy_id.unwrap_or_else(|| settings.active_strategy_id.clone());
    if let Some(s) = settings.strategies.iter_mut().find(|s| s.id == sid) {
        s.watch_codes = codes;
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;

    Ok(pool)
}
