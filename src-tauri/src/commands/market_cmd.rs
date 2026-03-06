use tauri::State;
use crate::AppState;
use crate::services::market_overview::{self, MarketOverview};

#[tauri::command]
pub async fn get_market_overview(
    state: State<'_, AppState>,
) -> Result<MarketOverview, String> {
    log::info!("[market_cmd] get_market_overview");
    let settings = state.db.load_settings().map_err(|e| {
        log::error!("[market_cmd] load_settings failed: {}", e);
        e.to_string()
    })?;

    market_overview::fetch_overview(&settings).await.map_err(|e| {
        log::error!("[market_cmd] fetch_overview failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn generate_market_comment(
    state: State<'_, AppState>,
    overview_json: String,
) -> Result<String, String> {
    log::info!("[market_cmd] generate_market_comment");
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;

    let config = settings.ai_configs.iter()
        .find(|c| {
            settings.active_ai_config_id.as_deref() == Some(&c.id) && c.enabled
        })
        .or_else(|| settings.ai_configs.iter().find(|c| c.enabled))
        .ok_or_else(|| "未配置 AI 模型，无法生成盘面解说".to_string())?;

    market_overview::generate_market_comment(config, &overview_json).await.map_err(|e| {
        log::error!("[market_cmd] generate_market_comment failed: {}", e);
        e.to_string()
    })
}
