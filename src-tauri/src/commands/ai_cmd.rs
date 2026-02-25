use tauri::{State, Emitter, AppHandle};
use crate::AppState;
use crate::models::ai::{AIAnalysisResult, AIStreamEvent};
use crate::services::ai_service::AIService;

#[tauri::command]
pub async fn analyze_stock(
    state: State<'_, AppState>,
    app: AppHandle,
    code: String,
    name: String,
    context_data: String,
) -> Result<(), String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;

    let ai_config = settings.ai_configs.iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .cloned()
        .ok_or("未配置AI模型".to_string())?;

    // Check if we have a cached analysis for today
    if let Ok(Some(cached)) = state.db.get_today_ai_analysis(&code) {
        // Send cached result
        let _ = app.emit("ai-analysis-cached", &cached);
        return Ok(());
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AIStreamEvent>(100);

    let app_clone = app.clone();
    let code_clone = code.clone();

    // Spawn receiver to forward events to frontend
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app_clone.emit(&format!("ai-stream-{}", code_clone), &event);
        }
    });

    // Run the stream
    let result = AIService::analyze_stock_stream(
        &ai_config,
        &code,
        &name,
        &context_data,
        tx,
    ).await.map_err(|e| e.to_string())?;

    // Save result to DB
    let analysis = AIAnalysisResult {
        id: uuid::Uuid::new_v4().to_string(),
        code: code.clone(),
        name: name.clone(),
        model_name: ai_config.model_name.clone(),
        question: "AI深度分析".to_string(),
        content: result.0,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    let _ = state.db.save_ai_analysis(&analysis);

    // Record token usage
    if let Some(usage) = result.1 {
        let _ = state.db.record_token_usage(&ai_config.model_name, usage.prompt_tokens, usage.completion_tokens);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_analysis_history(
    state: State<'_, AppState>,
    code: String,
    limit: usize,
) -> Result<Vec<AIAnalysisResult>, String> {
    state.db.get_ai_analysis_history(&code, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_today_token_usage(
    state: State<'_, AppState>,
) -> Result<u32, String> {
    state.db.get_today_token_usage().map_err(|e| e.to_string())
}
