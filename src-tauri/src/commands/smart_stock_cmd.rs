use tauri::{State, Emitter, AppHandle};
use crate::AppState;
use crate::services::smart_stock::SmartStockService;
use crate::models::ai::{AIStreamEvent, ChatCompletionRequest, ChatMessage};
use crate::utils::http::build_ai_client;
use serde_json::Value;
use futures::StreamExt;

/// 智能选股：调用东财 NLP API
#[tauri::command]
pub async fn smart_search_stock(
    state: State<'_, AppState>,
    keyword: String,
    page_size: Option<usize>,
) -> Result<Value, String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;
    let qgqp_b_id = &settings.qgqp_b_id;

    let size = page_size.unwrap_or(50);
    let resp = SmartStockService::search_stock(&keyword, size, qgqp_b_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return the full response as JSON for the frontend to handle dynamically
    serde_json::to_value(&resp).map_err(|e| e.to_string())
}

/// 获取热门选股策略
#[tauri::command]
pub async fn get_hot_strategies() -> Result<Value, String> {
    let strategies = SmartStockService::get_hot_strategies()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&strategies).map_err(|e| e.to_string())
}

/// AI 智能选股：AI 根据市场热点自动构造选股条件，调东财 API，分析结果推荐
#[tauri::command]
pub async fn ai_smart_pick(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let settings = state.db.load_settings().map_err(|e| e.to_string())?;

    let ai_config = settings.ai_configs.iter()
        .find(|c| Some(c.id.clone()) == settings.active_ai_config_id && c.enabled)
        .cloned()
        .ok_or("未配置AI模型，请先在设置中添加".to_string())?;

    if settings.qgqp_b_id.is_empty() {
        return Err("请先在设置中配置东财用户标识（qgqp_b_id）".to_string());
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AIStreamEvent>(100);

    let app_clone = app.clone();
    // Forward stream events to frontend
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app_clone.emit("ai-smart-pick-stream", &event);
        }
    });

    // Step 1: Get hot strategies for context
    let hot_strategies = SmartStockService::get_hot_strategies()
        .await
        .unwrap_or_default();

    let hot_text = if hot_strategies.is_empty() {
        "暂无热门策略数据".to_string()
    } else {
        hot_strategies.iter()
            .take(10)
            .map(|s| format!("#{}: {}", s.rank, s.question))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Step 2: Ask AI to generate stock screening conditions
    let system_prompt = "你是一位拥有20年实战经验的顶级A股投资大师，精通价值投资、趋势交易、量化分析。\
        你需要根据当前市场热点和你的专业判断，生成选股条件。\n\
        \n\
        重要规则：\n\
        1. 你的回答必须包含一个 <SEARCH_QUERY> 标签，里面是给东方财富智能选股器的自然语言查询条件\n\
        2. 查询条件要具体、专业、可执行，包含基本面和技术面\n\
        3. 必须排除ST股、退市股\n\
        4. 在 <SEARCH_QUERY> 标签之前，先简要说明你的选股逻辑（2-3句话）\n\
        5. 在 <SEARCH_QUERY> 标签之后，继续等待选股结果\n\
        \n\
        示例输出格式：\n\
        根据当前市场AI+半导体热点，我选择关注高ROE、资金流入的优质标的：\n\
        <SEARCH_QUERY>换手率大于3%小于25%.量比1以上.流通股本<100亿.当日净流入;股价在20日均线以上.沪深个股.近一年市盈率波动小于150%.MACD金叉;不要ST股及不要退市股，非北交所，每股收益>0</SEARCH_QUERY>";

    let user_prompt = format!(
        "当前时间：{}\n\
        当前市场热门选股策略：\n{}\n\n\
        请基于以上信息和你的专业判断，生成一个高质量的选股条件。要求：\n\
        1. 综合考虑当前市场热点\n\
        2. 注重基本面（PE、ROE、市值、营收）和技术面（均线、量能、资金流向）的结合\n\
        3. 风控优先，排除垃圾股\n\
        4. 适合短线到中线操作",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        hot_text
    );

    let client = build_ai_client(ai_config.timeout_secs).map_err(|e| e.to_string())?;

    let req = ChatCompletionRequest {
        model: ai_config.model_name.clone(),
        messages: vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(&user_prompt),
        ],
        max_tokens: Some(ai_config.max_tokens),
        temperature: Some(ai_config.temperature),
        stream: Some(true),
        tools: None,
        tool_choice: None,
    };

    let url = format!("{}/chat/completions", ai_config.base_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", ai_config.api_key))
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("AI API错误: {}", body));
    }

    let mut full_content = String::new();
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    // Phase 1: Stream AI's reasoning and get the search query
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(chunk_resp) = serde_json::from_str::<crate::models::ai::ChatCompletionResponse>(data) {
                    if let Some(choice) = chunk_resp.choices.first() {
                        if let Some(delta) = &choice.delta {
                            if let Some(content) = &delta.content {
                                full_content.push_str(content);
                                let _ = tx.send(AIStreamEvent {
                                    event_type: "content".to_string(),
                                    content: Some(content.clone()),
                                    done: false,
                                    usage: None,
                                    tool_name: None,
                                }).await;
                            }
                        }
                    }
                }
            }
        }
    }

    // Phase 2: Extract search query from AI response
    let search_query = extract_search_query(&full_content);

    if let Some(query) = search_query {
        // Notify frontend we're now searching
        let _ = tx.send(AIStreamEvent {
            event_type: "content".to_string(),
            content: Some(format!("\n\n---\n正在使用选股条件搜索...\n条件：`{}`\n\n", query)),
            done: false,
            usage: None,
        tool_name: None,
        }).await;

        // Phase 3: Call eastmoney search API
        match SmartStockService::search_stock(&query, 30, &settings.qgqp_b_id).await {
            Ok(search_resp) => {
                if search_resp.code == 100 {
                    if let Some(data) = &search_resp.data {
                        let stock_count = data.result.data_list.len();
                        let trace_text = data.trace_info.as_ref()
                            .map(|t| t.show_text.clone())
                            .unwrap_or_default();

                        // Send search result metadata
                        let _ = tx.send(AIStreamEvent {
                            event_type: "content".to_string(),
                            content: Some(format!(
                                "选股条件解析：{}\n共找到 **{}** 只符合条件的股票\n\n",
                                trace_text, stock_count
                            )),
                            done: false,
                            usage: None,
                        tool_name: None,
                        }).await;

                        // Send search results as special event for frontend table rendering
                        let result_json = serde_json::to_value(&search_resp).unwrap_or_default();
                        let _ = tx.send(AIStreamEvent {
                            event_type: "search_result".to_string(),
                            content: Some(serde_json::to_string(&result_json).unwrap_or_default()),
                            done: false,
                            usage: None,
                        tool_name: None,
                        }).await;

                        // Phase 4: Ask AI to analyze results
                        if stock_count > 0 {
                            let stocks_summary = build_stocks_summary(&data.result);

                            let analysis_prompt = format!(
                                "以下是根据你的选股条件筛选出的 {} 只股票数据：\n\n{}\n\n\
                                请对这些股票进行专业分析：\n\
                                1. 从中推荐最值得关注的 3-5 只，说明理由\n\
                                2. 标注需要回避的标的及原因\n\
                                3. 给出整体操作建议（仓位、时机）\n\
                                4. 风险提示\n\
                                \n请简洁专业地输出。",
                                stock_count, stocks_summary
                            );

                            // Second AI call for analysis (stream)
                            let req2 = ChatCompletionRequest {
                                model: ai_config.model_name.clone(),
                                messages: vec![
                                    ChatMessage::system("你是一位拥有20年实战经验的顶级A股投资大师。请对选股结果进行专业分析和推荐。"),
                                    ChatMessage::user(&analysis_prompt),
                                ],
                                max_tokens: Some(ai_config.max_tokens),
                                temperature: Some(ai_config.temperature),
                                stream: Some(true),
                                tools: None,
                                tool_choice: None,
                            };

                            let _ = tx.send(AIStreamEvent {
                                event_type: "content".to_string(),
                                content: Some("\n### AI 分析与推荐\n\n".to_string()),
                                done: false,
                                usage: None,
                            tool_name: None,
                            }).await;

                            let resp2 = client
                                .post(&url)
                                .header("Authorization", format!("Bearer {}", ai_config.api_key))
                                .header("Content-Type", "application/json")
                                .json(&req2)
                                .send()
                                .await;

                            if let Ok(resp2) = resp2 {
                                if resp2.status().is_success() {
                                    let mut stream2 = resp2.bytes_stream();
                                    let mut buffer2 = String::new();

                                    while let Some(chunk) = stream2.next().await {
                                        if let Ok(chunk) = chunk {
                                            buffer2.push_str(&String::from_utf8_lossy(&chunk));

                                            while let Some(line_end) = buffer2.find('\n') {
                                                let line = buffer2[..line_end].trim().to_string();
                                                buffer2 = buffer2[line_end + 1..].to_string();

                                                if line.is_empty() || line == "data: [DONE]" {
                                                    continue;
                                                }

                                                if let Some(data) = line.strip_prefix("data: ") {
                                                    if let Ok(chunk_resp) = serde_json::from_str::<crate::models::ai::ChatCompletionResponse>(data) {
                                                        if let Some(choice) = chunk_resp.choices.first() {
                                                            if let Some(delta) = &choice.delta {
                                                                if let Some(content) = &delta.content {
                                                                    let _ = tx.send(AIStreamEvent {
                                                                        event_type: "content".to_string(),
                                                                        content: Some(content.clone()),
                                                                        done: false,
                                                                        usage: None,
                                                                        tool_name: None,
                                                                    }).await;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let err_msg = search_resp.msg.or(search_resp.message).unwrap_or_default();
                    let _ = tx.send(AIStreamEvent {
                        event_type: "content".to_string(),
                        content: Some(format!("\n选股API返回错误：{}\n", err_msg)),
                        done: false,
                        usage: None,
                    tool_name: None,
                    }).await;
                }
            }
            Err(e) => {
                let _ = tx.send(AIStreamEvent {
                    event_type: "content".to_string(),
                    content: Some(format!("\n选股搜索失败：{}\n", e)),
                    done: false,
                    usage: None,
                tool_name: None,
                }).await;
            }
        }
    }

    // Done
    let _ = tx.send(AIStreamEvent {
        event_type: "done".to_string(),
        content: None,
        done: true,
        usage: None,
    tool_name: None,
    }).await;

    Ok(())
}

fn extract_search_query(text: &str) -> Option<String> {
    let start_tag = "<SEARCH_QUERY>";
    let end_tag = "</SEARCH_QUERY>";

    if let Some(start) = text.find(start_tag) {
        let after_start = start + start_tag.len();
        if let Some(end) = text[after_start..].find(end_tag) {
            let query = text[after_start..after_start + end].trim();
            if !query.is_empty() {
                return Some(query.to_string());
            }
        }
    }
    None
}

fn build_stocks_summary(result: &crate::services::smart_stock::SmartStockResult) -> String {
    // Build a compact text summary of the stocks for AI analysis
    let mut lines = Vec::new();

    // Find key column keys
    let code_key = "SECURITY_CODE";
    let name_key = "SECURITY_SHORT_NAME";

    for (i, stock) in result.data_list.iter().enumerate().take(30) {
        let code = stock.get(code_key)
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let name = stock.get(name_key)
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        // Collect all visible numeric fields
        let mut fields = Vec::new();
        for col in &result.columns {
            if col.hidden_need || col.key == code_key || col.key == name_key
                || col.key == "SERIAL" || col.key == "MARKET_SHORT_NAME" {
                continue;
            }

            if let Some(children) = &col.children {
                for child in children {
                    if let Some(val) = stock.get(&child.key) {
                        let title = child.date_msg.as_deref().unwrap_or(&child.title);
                        fields.push(format!("{}:{}", title, format_value(val)));
                    }
                }
            } else {
                if let Some(val) = stock.get(&col.key) {
                    let title = &col.title;
                    let unit = col.unit.as_deref().unwrap_or("");
                    let unit_str = if unit.is_empty() { String::new() } else { format!("({})", unit) };
                    fields.push(format!("{}{}:{}", title, unit_str, format_value(val)));
                }
            }
        }

        lines.push(format!("{}. {}({}) {}", i + 1, name, code, fields.join(" ")));
    }

    lines.join("\n")
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f == f.floor() && f.abs() < 1e10 {
                    format!("{}", f as i64)
                } else {
                    format!("{:.2}", f)
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Null => "-".to_string(),
        _ => v.to_string(),
    }
}
