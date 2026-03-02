use tauri::{AppHandle, Emitter, State};
use crate::AppState;
use crate::models::tracking::{AIPickTracking, LossStock};
use crate::models::ai::AIStreamEvent;
use crate::services::ai_service::AIService;

#[tauri::command]
pub async fn add_tracking_stock(
    state: State<'_, AppState>,
    code: String,
    name: String,
    added_price: f64,
    rating: String,
    reason: String,
    sector: String,
) -> Result<(), String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tracking = AIPickTracking {
        code,
        name,
        added_date: today,
        added_price,
        rating,
        reason,
        sector,
        created_at: now,
    };
    state.db.add_tracking_stock(&tracking).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tracking_stock(
    state: State<'_, AppState>,
    code: String,
    added_date: String,
) -> Result<(), String> {
    state.db.remove_tracking_stock(&code, &added_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tracking_stocks(
    state: State<'_, AppState>,
) -> Result<Vec<AIPickTracking>, String> {
    state.db.get_tracking_stocks().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_tracking_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<(), String> {
    state.db.clear_tracking_by_date(&date).map_err(|e| e.to_string())
}

/// AI 败因分析命令
#[tauri::command]
pub async fn analyze_loss_reasons(
    app: AppHandle,
    state: State<'_, AppState>,
    date: String,
    loss_stocks: Vec<LossStock>,
) -> Result<(), String> {
    if loss_stocks.is_empty() {
        return Err("没有亏损股票需要分析".to_string());
    }

    let settings = state.db.load_settings().map_err(|e| e.to_string())?;
    let config = settings
        .ai_configs
        .iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .or_else(|| settings.ai_configs.iter().find(|c| c.enabled))
        .ok_or_else(|| "未配置可用的 AI 模型，请在设置中添加".to_string())?
        .clone();

    let qgqp_b_id = settings.qgqp_b_id.clone();

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<AIStreamEvent>(100);

    let event_name = format!("ai-loss-analysis-{}", date);
    let app_clone = app.clone();
    let event_name_clone = event_name.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let _ = app_clone.emit(&event_name_clone, &event);
        }
    });

    tokio::spawn(async move {
        match AIService::analyze_loss_reasons_with_tools(&config, &date, &loss_stocks, &qgqp_b_id, sender.clone()).await {
            Ok((content, usage)) => {
                let _ = sender.send(AIStreamEvent {
                    event_type: "done".to_string(),
                    content: Some(content),
                    done: true,
                    usage,
                    tool_name: None,
                }).await;
            }
            Err(e) => {
                let _ = sender.send(AIStreamEvent {
                    event_type: "error".to_string(),
                    content: Some(format!("败因分析失败: {}", e)),
                    done: true,
                    usage: None,
                    tool_name: None,
                }).await;
            }
        }
    });

    Ok(())
}
