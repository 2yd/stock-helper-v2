use tauri::{AppHandle, Emitter, Manager};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::AppState;
use crate::models::ai::AIStreamEvent;
use crate::services::ai_service::AIService;

/// AI 自主选股命令
/// 启动 AI Agent，让其自主获取新闻/板块/行情数据并做出选股决策
#[tauri::command]
pub async fn ai_pick_stocks(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 单飞控制：防止重复提交
    if state.ai_picking.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Err("AI 选股正在进行中，请等待当前任务完成".to_string());
    }

    // 重置取消信号
    state.ai_pick_cancel.store(false, Ordering::SeqCst);
    let cancel_token = Arc::clone(&state.ai_pick_cancel);

    let settings = state.db.load_settings().map_err(|e| {
        state.ai_picking.store(false, Ordering::SeqCst);
        e.to_string()
    })?;
    let config = settings
        .ai_configs
        .iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .or_else(|| settings.ai_configs.iter().find(|c| c.enabled))
        .ok_or_else(|| {
            state.ai_picking.store(false, Ordering::SeqCst);
            "未配置可用的 AI 模型，请在设置中添加".to_string()
        })?
        .clone();

    let qgqp_b_id = settings.qgqp_b_id.clone();

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<AIStreamEvent>(100);

    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let _ = app_clone.emit("ai-pick-stream", &event);
        }
    });

    let app_for_db = app.clone();
    tokio::spawn(async move {
        let result = AIService::ai_pick_stocks_with_tools(&config, &qgqp_b_id, sender.clone(), cancel_token).await;

        // 无论成功或失败，都重置标志位
        let app_state = app_for_db.state::<AppState>();
        app_state.ai_picking.store(false, Ordering::SeqCst);

        match result {
            Ok((content, usage)) => {
                let _ = app_state.db.save_ai_pick_cache(&content);

                let _ = sender.send(AIStreamEvent {
                    event_type: "done".to_string(),
                    content: Some(content),
                    done: true,
                    usage,
                    tool_name: None,
                }).await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                let event_type = if err_msg.contains("用户取消") {
                    "done" // 用户主动取消视为正常结束
                } else {
                    "error"
                };
                let _ = sender.send(AIStreamEvent {
                    event_type: event_type.to_string(),
                    content: Some(if event_type == "done" {
                        "AI 选股已停止".to_string()
                    } else {
                        format!("AI 选股失败: {}", err_msg)
                    }),
                    done: true,
                    usage: None,
                    tool_name: None,
                }).await;
            }
        }
    });

    Ok(())
}

/// 停止 AI 选股
#[tauri::command]
pub async fn stop_ai_pick(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if !state.ai_picking.load(Ordering::SeqCst) {
        return Err("当前没有进行中的 AI 选股任务".to_string());
    }
    state.ai_pick_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

/// 获取缓存的当日 AI 选股结果
#[tauri::command]
pub async fn get_cached_picks(
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    state.db.get_ai_pick_cache().map_err(|e| e.to_string())
}

/// AI 找相似股：给定一只股票，找出同板块/同概念中尚未大涨的补涨机会
#[tauri::command]
pub async fn find_similar_stocks(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    code: String,
    name: String,
    sector: String,
) -> Result<(), String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;
    let config = settings
        .ai_configs
        .iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .or_else(|| settings.ai_configs.iter().find(|c| c.enabled))
        .ok_or_else(|| "未配置可用的 AI 模型，请在设置中添加".to_string())?
        .clone();

    let qgqp_b_id = settings.qgqp_b_id.clone();

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<crate::models::ai::AIStreamEvent>(100);

    let event_name = format!("ai-similar-{}", code);
    let app_clone = app.clone();
    let event_name_clone = event_name.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let _ = app_clone.emit(&event_name_clone, &event);
        }
    });

    tokio::spawn(async move {
        match AIService::find_similar_stocks_with_tools(&config, &code, &name, &sector, &qgqp_b_id, sender.clone()).await {
            Ok((content, usage)) => {
                let _ = sender.send(crate::models::ai::AIStreamEvent {
                    event_type: "done".to_string(),
                    content: Some(content),
                    done: true,
                    usage,
                    tool_name: None,
                }).await;
            }
            Err(e) => {
                let _ = sender.send(crate::models::ai::AIStreamEvent {
                    event_type: "error".to_string(),
                    content: Some(format!("查找相似股失败: {}", e)),
                    done: true,
                    usage: None,
                    tool_name: None,
                }).await;
            }
        }
    });

    Ok(())
}
