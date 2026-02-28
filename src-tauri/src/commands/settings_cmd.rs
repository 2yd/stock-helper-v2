use tauri::State;
use crate::AppState;
use crate::models::settings::AppSettings;
use crate::models::ai::AIConfig;
use crate::models::strategy::StrategyConfig;
use crate::services::ai_service::AIService;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state.db.load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    state.db.save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_ai_config(
    state: State<'_, AppState>,
    config: AIConfig,
) -> Result<AppSettings, String> {
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    settings.ai_configs.push(config);
    if settings.active_ai_config_id.is_none() {
        settings.active_ai_config_id = settings.ai_configs.first().map(|c| c.id.clone());
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn remove_ai_config(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<AppSettings, String> {
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    settings.ai_configs.retain(|c| c.id != config_id);
    if settings.active_ai_config_id.as_deref() == Some(&config_id) {
        settings.active_ai_config_id = settings.ai_configs.first().map(|c| c.id.clone());
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn update_ai_config(
    state: State<'_, AppState>,
    config: AIConfig,
) -> Result<AppSettings, String> {
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    if let Some(existing) = settings.ai_configs.iter_mut().find(|c| c.id == config.id) {
        *existing = config;
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn set_active_ai_config(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<AppSettings, String> {
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    settings.active_ai_config_id = Some(config_id);
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn update_strategy_config(
    state: State<'_, AppState>,
    strategy: StrategyConfig,
) -> Result<AppSettings, String> {
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    if let Some(existing) = settings.strategies.iter_mut().find(|s| s.id == strategy.id) {
        *existing = strategy;
    } else {
        settings.strategies.push(strategy);
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

/// 测试 AI 模型配置是否可用
#[tauri::command]
pub async fn test_ai_config(
    config: AIConfig,
) -> Result<String, String> {
    AIService::test_ai_connection(&config).await.map_err(|e| e.to_string())
}
