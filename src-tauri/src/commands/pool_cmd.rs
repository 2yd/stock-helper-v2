use crate::services::market_pool::{MarketPoolService, PoolStock};

/// 获取涨停池
#[tauri::command]
pub async fn fetch_limit_up_pool(date: Option<String>) -> Result<Vec<PoolStock>, String> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    log::info!("[pool_cmd] fetch_limit_up_pool date={}", date);
    let service = MarketPoolService::new().map_err(|e| e.to_string())?;
    service.fetch_limit_up_pool(&date).await.map_err(|e| {
        log::error!("[pool_cmd] fetch_limit_up_pool failed: {}", e);
        e.to_string()
    })
}

/// 获取连板池
#[tauri::command]
pub async fn fetch_streak_pool(date: Option<String>) -> Result<Vec<PoolStock>, String> {
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    log::info!("[pool_cmd] fetch_streak_pool date={}", date);
    let service = MarketPoolService::new().map_err(|e| e.to_string())?;
    service.fetch_streak_pool(&date).await.map_err(|e| {
        log::error!("[pool_cmd] fetch_streak_pool failed: {}", e);
        e.to_string()
    })
}

/// 一键获取高标池（连板+涨停去重合并）
#[tauri::command]
pub async fn fetch_and_apply_high_pool() -> Result<Vec<PoolStock>, String> {
    log::info!("[pool_cmd] fetch_and_apply_high_pool");
    let service = MarketPoolService::new().map_err(|e| e.to_string())?;
    let pool = service.fetch_high_pool().await.map_err(|e| {
        log::error!("[pool_cmd] fetch_and_apply_high_pool failed: {}", e);
        e.to_string()
    })?;

    Ok(pool)
}
