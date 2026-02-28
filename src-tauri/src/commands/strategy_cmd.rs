use tauri::{State, Emitter, AppHandle};
use crate::AppState;
use crate::models::strategy::StrategyResultRow;
use crate::models::ai::StockSummaryForAI;
use crate::services::market_scanner::MarketScanner;
use crate::services::scoring::MultiFactorEngine;
use crate::services::labeling::LabelingEngine;
use crate::services::ai_service::AIService;
use crate::services::scheduler::TradingScheduler;
use crate::services::thematic_scoring::ThematicScoringEngine;
use crate::models::strategy::StockLabel;

/// 全市场扫描选股：拉取沪深A股全量数据 → 多因子过滤打分 → Top N
#[tauri::command]
pub async fn scan_market(
    state: State<'_, AppState>,
    app: AppHandle,
    strategy_id: String,
) -> Result<Vec<StrategyResultRow>, String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;

    let strategy = settings.strategies.iter()
        .find(|s| s.id == strategy_id)
        .cloned()
        .unwrap_or_default();

    let scanner = MarketScanner::new().map_err(|e| e.to_string())?;

    // 拉取全市场数据
    let all_stocks = scanner.scan_full_market().await.map_err(|e| e.to_string())?;

    // 消息面主题识别（失败时降级，不影响技术面主流程）
    let thematic = match ThematicScoringEngine::new() {
        Ok(engine) => engine.build_sentiment_map().await.unwrap_or_default(),
        Err(_) => Default::default(),
    };

    // 六因子打分（技术面 + 消息面）
    let ranked = MultiFactorEngine::screen_and_rank_with_sentiment(
        &all_stocks,
        &strategy.weights,
        &strategy.filters,
        strategy.top_n,
        &thematic.stock_sentiment,
    );

    // 转换为前端结构
    let results: Vec<StrategyResultRow> = ranked.into_iter().map(|(stock, score, detail)| {
        let code = stock.code.clone();
        let mut labels = LabelingEngine::generate_labels(&stock);
        let news_heat = thematic.stock_heat.get(&code).copied().unwrap_or(0.0);
        let matched_themes = thematic.stock_themes.get(&code).cloned().unwrap_or_default();

        if news_heat >= 0.6 {
            labels.push(StockLabel {
                text: "消息热度高".to_string(),
                color: "orange".to_string(),
                icon: Some("Zap".to_string()),
            });
        }
        if let Some(theme) = matched_themes.first() {
            labels.push(StockLabel {
                text: format!("主题:{}", theme),
                color: "purple".to_string(),
                icon: Some("Sparkles".to_string()),
            });
        }

        StrategyResultRow {
            code,
            name: stock.name,
            price: stock.price,
            change_pct: stock.change_pct,
            pe_ttm: stock.pe_ttm,
            pb: stock.pb,
            roe: stock.roe,
            revenue_yoy: stock.revenue_yoy,
            profit_yoy: stock.profit_yoy,
            total_market_cap: stock.total_market_cap / 1e8,  // 转亿
            float_market_cap: stock.float_market_cap / 1e8,
            turnover_rate: stock.turnover_rate,
            volume_ratio: stock.volume_ratio,
            amount: stock.amount / 1e4,  // 转万
            main_net_inflow: stock.main_net_inflow / 1e4,  // 转万
            main_net_pct: stock.main_net_pct,
            pct_5d: stock.pct_5d,
            pct_20d: stock.pct_20d,
            pct_60d: stock.pct_60d,
            score,
            sentiment_score: detail.sentiment_score,
            news_heat,
            matched_themes,
            score_detail: detail,
            labels,
            instruction: None,
        }
    }).collect();

    let _ = app.emit("strategy-update", &results);

    Ok(results)
}

/// 保持旧命令兼容（内部转发到 scan_market 逻辑）
#[tauri::command]
pub async fn refresh_strategy(
    state: State<'_, AppState>,
    app: AppHandle,
    strategy_id: String,
) -> Result<Vec<StrategyResultRow>, String> {
    scan_market(state, app, strategy_id).await
}

/// AI 批量指令生成
#[tauri::command]
pub async fn generate_ai_instructions(
    state: State<'_, AppState>,
    results: Vec<StrategyResultRow>,
) -> Result<Vec<StrategyResultRow>, String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;

    if !settings.ai_instruction_enabled {
        return Ok(results);
    }

    let ai_config = settings.ai_configs.iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .cloned();

    let ai_config = match ai_config {
        Some(c) => c,
        None => return Ok(results),
    };

    let summaries: Vec<StockSummaryForAI> = results.iter().map(|r| {
        StockSummaryForAI {
            code: r.code.clone(),
            name: r.name.clone(),
            open_pct: r.change_pct,
            current_pct: r.change_pct,
            score: r.score,
            bid_amount: r.amount * 10000.0,
            streak_days: 0,
            turnover: r.turnover_rate,
            labels: r.labels.iter().map(|l| l.text.clone()).collect(),
        }
    }).collect();

    let (instructions, token_usage) = AIService::batch_generate_instructions(&ai_config, &summaries)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(usage) = token_usage {
        let _ = state.db.record_token_usage(
            &ai_config.model_name,
            usage.prompt_tokens,
            usage.completion_tokens,
        );
    }

    let mut updated = results;
    for inst in instructions {
        if let Some(row) = updated.iter_mut().find(|r| r.code == inst.code) {
            use crate::models::ai::{AIInstruction, InstructionAction};
            row.instruction = Some(AIInstruction {
                action: match inst.action.as_str() {
                    "buy" => InstructionAction::Buy,
                    "watch" => InstructionAction::Watch,
                    _ => InstructionAction::Eliminate,
                },
                label: inst.label,
                reason: inst.reason,
            });
        }
    }

    Ok(updated)
}

#[tauri::command]
pub async fn get_market_status() -> Result<String, String> {
    Ok(TradingScheduler::market_status())
}

#[tauri::command]
pub async fn is_trading_time() -> Result<bool, String> {
    Ok(TradingScheduler::is_trading_time())
}

#[tauri::command]
pub async fn update_watch_codes(
    state: State<'_, AppState>,
    codes: Vec<String>,
    strategy_id: Option<String>,
) -> Result<(), String> {
    {
        let mut watch = state.watch_codes.lock().unwrap();
        *watch = codes.clone();
    }
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    let sid = strategy_id.unwrap_or_else(|| settings.active_strategy_id.clone());
    if let Some(s) = settings.strategies.iter_mut().find(|s| s.id == sid) {
        s.watch_codes = codes;
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_watch_codes(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let watch = state.watch_codes.lock().unwrap();
    Ok(watch.clone())
}

/// 获取全市场股票总数（用于 UI 显示）
#[tauri::command]
pub async fn get_market_stock_count() -> Result<usize, String> {
    let scanner = MarketScanner::new().map_err(|e| e.to_string())?;
    let stocks = scanner.scan_full_market().await.map_err(|e| e.to_string())?;
    Ok(stocks.len())
}
