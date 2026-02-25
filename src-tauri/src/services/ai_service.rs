use anyhow::{Result, anyhow};
use futures::StreamExt;
use crate::models::ai::*;
use crate::services::stock_tools;
use crate::utils::http::build_ai_client;

const MAX_TOOL_ROUNDS: usize = 8;
const MAX_PICK_TOOL_ROUNDS: usize = 15;

pub struct AIService;

impl AIService {
    /// Batch generate instructions for all stocks in a strategy zone.
    /// (保持原有功能不变)
    pub async fn batch_generate_instructions(
        config: &AIConfig,
        stocks: &[StockSummaryForAI],
    ) -> Result<Vec<StockInstructionResult>> {
        if stocks.is_empty() {
            return Ok(vec![]);
        }

        let client = build_ai_client(config.timeout_secs)?;

        let stocks_text = stocks.iter().map(|s| {
            format!(
                "{}({}) 今开{:.1}% 最新{:.1}% 得分{} 竞价{:.0}万 {}板 换手{:.1}% 标签:{}",
                s.name, s.code, s.open_pct, s.current_pct, s.score,
                s.bid_amount / 10000.0, s.streak_days, s.turnover,
                s.labels.join(",")
            )
        }).collect::<Vec<_>>().join("\n");

        let prompt = format!(
            "你是一个专业的A股短线交易员助手。请根据以下竞价选股数据，为每只股票给出操作指令。\n\
            指令类型：buy(买入)、watch(观察)、eliminate(淘汰)\n\
            \n\
            判断标准：\n\
            - 得分>=80且竞价抢筹的：buy，标签如\"龙头抢筹\"\"竞价达标\"\n\
            - 得分60-80或有被卡位风险的：watch，标签如\"梯队PK被卡位\"\n\
            - 得分<60或深水低开的：eliminate，标签如\"淘汰:深水核按钮\"\n\
            \n\
            股票数据：\n{}\n\
            \n\
            请严格以JSON数组格式输出，每个元素包含code、action、label、reason字段，不要输出其他内容：",
            stocks_text
        );

        let req = ChatCompletionRequest {
            model: config.model_name.clone(),
            messages: vec![ChatMessage::user(&prompt)],
            max_tokens: Some(config.max_tokens),
            temperature: Some(config.temperature),
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(anyhow!("AI API error ({}): {}", status, body));
        }

        let response: ChatCompletionResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow!("AI response parse error: {} body: {}", e, &body[..200.min(body.len())]))?;

        let content = response.choices.first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.clone())
            .unwrap_or_default();

        let json_str = extract_json_array(&content)?;
        let instructions: Vec<StockInstructionResult> = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("Instruction parse error: {} content: {}", e, &json_str[..200.min(json_str.len())]))?;

        Ok(instructions)
    }

    /// Agent 模式：带工具调用的 AI 股票分析
    /// AI 可以主动调用工具获取实时行情、K线、技术指标等数据
    pub async fn diagnose_stock_with_tools(
        config: &AIConfig,
        code: &str,
        name: &str,
        sender: tokio::sync::mpsc::Sender<AIStreamEvent>,
    ) -> Result<(String, Option<TokenUsage>)> {
        let client = build_ai_client(config.timeout_secs)?;
        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let tools = stock_tools::get_tool_definitions();

        let system_prompt = format!(
            "你是一位拥有20年实战经验的顶级A股技术分析师。你可以通过工具获取股票的真实数据。\n\
            \n\
            当前分析标的：{}({})\n\
            \n\
            **工作流程**：\n\
            1. 先调用 get_stock_quote 获取实时行情（价格、PE/PB/ROE、市值、换手率、量比、主力净流入等）\n\
            2. 调用 get_kline_data 获取最近60根日K线数据\n\
            3. 调用 get_technical_indicators 获取技术指标（MA/MACD/KDJ/RSI/BOLL/信号等）\n\
            4. 如需要，调用 get_fund_flow 获取详细资金流向\n\
            5. 综合所有数据给出专业分析\n\
            \n\
            **分析要求**：\n\
            基于真实数据进行分析，给出：\n\
            1. **行情概览**：当前价格、涨跌、市值、估值水平（PE/PB/ROE）\n\
            2. **技术面分析**：K线形态、均线系统、MACD/KDJ/RSI/BOLL 等指标研判、支撑压力位\n\
            3. **资金面分析**：主力资金动向、换手率、量比分析\n\
            4. **操作建议**：明确给出买入/持有/减仓/清仓建议，附具体参考价位（止盈/止损位）\n\
            5. **风险提示**：当前主要风险因素\n\
            \n\
            请用简洁专业的语言，引用具体数据支撑你的观点。",
            name, code
        );

        let mut messages: Vec<ChatMessage> = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(&format!("请对 {}({}) 进行全面的技术分析和诊断。", name, code)),
        ];

        let mut full_content = String::new();
        let mut total_usage: Option<TokenUsage> = None;

        // Phase 1: Tool calling loop (non-streaming, easy to parse tool_calls)
        for _round in 0..MAX_TOOL_ROUNDS {
            let req = ChatCompletionRequest {
                model: config.model_name.clone(),
                messages: messages.clone(),
                max_tokens: Some(config.max_tokens),
                temperature: Some(config.temperature),
                stream: Some(false),
                tools: Some(tools.clone()),
                tool_choice: None,
            };

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .header("Content-Type", "application/json")
                .json(&req)
                .send()
                .await?;

            if !resp.status().is_success() {
                let body = resp.text().await?;
                return Err(anyhow!("AI API error: {}", body));
            }

            let body = resp.text().await?;
            let response: ChatCompletionResponse = serde_json::from_str(&body)
                .map_err(|e| anyhow!("AI response parse error: {} body: {}", e, &body[..500.min(body.len())]))?;

            // Accumulate usage
            if let Some(usage) = &response.usage {
                total_usage = Some(match total_usage {
                    Some(mut u) => {
                        u.prompt_tokens += usage.prompt_tokens;
                        u.completion_tokens += usage.completion_tokens;
                        u.total_tokens += usage.total_tokens;
                        u
                    }
                    None => usage.clone(),
                });
            }

            let choice = response.choices.first()
                .ok_or_else(|| anyhow!("AI 返回空 choices"))?;

            let finish_reason = choice.finish_reason.as_deref().unwrap_or("");
            let msg = choice.message.as_ref().ok_or_else(|| anyhow!("AI 返回空 message"))?;

            // Check if the model wants to call tools
            if (finish_reason == "tool_calls" || msg.tool_calls.is_some())
                && msg.tool_calls.as_ref().map_or(false, |tc| !tc.is_empty())
            {
                let tool_calls = msg.tool_calls.as_ref().unwrap();
                // Add assistant message with tool_calls to history
                messages.push(ChatMessage::assistant_tool_calls(tool_calls.clone()));

                // Execute each tool call
                for tc in tool_calls {
                    let tool_name = &tc.function.name;
                    let tool_args = &tc.function.arguments;

                    // Notify frontend about tool call
                    let _ = sender.send(AIStreamEvent {
                        event_type: "tool_call".to_string(),
                        content: Some(format!("正在获取数据: {}", tool_name_to_chinese(tool_name))),
                        done: false,
                        usage: None,
                        tool_name: Some(tool_name.clone()),
                    }).await;

                    // Execute the tool
                    let result = match stock_tools::execute_tool(tool_name, tool_args).await {
                        Ok(r) => r,
                        Err(e) => format!("工具调用失败: {}", e),
                    };

                    // Notify frontend about result
                    let _ = sender.send(AIStreamEvent {
                        event_type: "tool_result".to_string(),
                        content: Some(format!("已获取: {}", tool_name_to_chinese(tool_name))),
                        done: false,
                        usage: None,
                        tool_name: Some(tool_name.clone()),
                    }).await;

                    // Add tool result to messages
                    messages.push(ChatMessage::tool_result(&tc.id, tool_name, &result));
                }

                // Continue to next round
                continue;
            }

            // No more tool calls — break out and do streaming final response
            // If the model already returned some text, add it as assistant context
            if let Some(content) = &msg.content {
                if !content.is_empty() {
                    // Model already gave an answer in non-streaming mode; just add to history
                    // We'll re-request in streaming mode below for good UX
                    messages.push(ChatMessage::assistant_text(content));
                }
            }
            break;
        }

        // Phase 2: Stream the final analysis
        // Ask AI to output final analysis in streaming mode (no tools, force text)
        // If messages already has the assistant's non-stream answer, ask it to re-format
        // Otherwise just let it continue
        let last_is_assistant = messages.last().map_or(false, |m| m.role == "assistant" && m.content.is_some());
        if last_is_assistant {
            // Remove the non-streaming answer and re-request as streaming
            messages.pop();
        }

        let req = ChatCompletionRequest {
            model: config.model_name.clone(),
            messages: messages.clone(),
            max_tokens: Some(config.max_tokens),
            temperature: Some(config.temperature),
            stream: Some(true),
            tools: None,
            tool_choice: None,
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("AI API error: {}", body));
        }

        // Stream final response
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    if line == "data: [DONE]" {
                        let _ = sender.send(AIStreamEvent {
                            event_type: "done".to_string(),
                            content: None,
                            done: true,
                            usage: total_usage.clone(),
                            tool_name: None,
                        }).await;
                    }
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(chunk_resp) = serde_json::from_str::<ChatCompletionResponse>(data) {
                        if let Some(choice) = chunk_resp.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    full_content.push_str(content);
                                    let _ = sender.send(AIStreamEvent {
                                        event_type: "content".to_string(),
                                        content: Some(content.clone()),
                                        done: false,
                                        usage: None,
                                        tool_name: None,
                                    }).await;
                                }
                            }
                        }
                        // Accumulate streaming usage if present
                        if let Some(usage) = &chunk_resp.usage {
                            total_usage = Some(match total_usage {
                                Some(mut u) => {
                                    u.prompt_tokens += usage.prompt_tokens;
                                    u.completion_tokens += usage.completion_tokens;
                                    u.total_tokens += usage.total_tokens;
                                    u
                                }
                                None => usage.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok((full_content, total_usage))
    }

    /// 原版流式分析（保留给其他场景使用）
    pub async fn analyze_stock_stream(
        config: &AIConfig,
        code: &str,
        name: &str,
        context_data: &str,
        sender: tokio::sync::mpsc::Sender<AIStreamEvent>,
    ) -> Result<(String, Option<TokenUsage>)> {
        let client = build_ai_client(config.timeout_secs)?;

        let prompt = format!(
            "你是一个专业的A股分析师。请对股票 {}({}) 进行深度分析。\n\
            \n\
            当前数据：\n{}\n\
            \n\
            请从以下维度分析并给出建议：\n\
            1. **技术面分析**：近期K线形态、支撑压力位、成交量变化\n\
            2. **资金面分析**：主力资金动向、竞价表现、换手率\n\
            3. **题材面分析**：所属板块热度、概念催化剂\n\
            4. **操作建议**：具体的买入/卖出建议、止盈止损位\n\
            5. **风险提示**：需要注意的风险因素\n\
            \n\
            请用简洁专业的语言，重点突出操作建议。",
            name, code, context_data
        );

        let req = ChatCompletionRequest {
            model: config.model_name.clone(),
            messages: vec![ChatMessage::user(&prompt)],
            max_tokens: Some(config.max_tokens),
            temperature: Some(config.temperature),
            stream: Some(true),
            tools: None,
            tool_choice: None,
        };

        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("AI API error: {}", body));
        }

        let mut full_content = String::new();
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    if line == "data: [DONE]" {
                        let _ = sender.send(AIStreamEvent {
                            event_type: "done".to_string(),
                            content: None,
                            done: true,
                            usage: None,
                            tool_name: None,
                        }).await;
                    }
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(chunk_resp) = serde_json::from_str::<ChatCompletionResponse>(data) {
                        if let Some(choice) = chunk_resp.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    full_content.push_str(content);
                                    let _ = sender.send(AIStreamEvent {
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

        Ok((full_content, None))
    }

    /// AI 自主选股：Agent 模式，让 AI 自主获取新闻/板块/行情，独立做出选股决策
    pub async fn ai_pick_stocks_with_tools(
        config: &AIConfig,
        qgqp_b_id: &str,
        sender: tokio::sync::mpsc::Sender<AIStreamEvent>,
    ) -> Result<(String, Option<TokenUsage>)> {
        let client = build_ai_client(config.timeout_secs)?;
        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let tools = stock_tools::get_pick_tool_definitions();

        let today = chrono::Local::now().format("%Y年%m月%d日 %H:%M").to_string();

        let system_prompt = format!(
            "# 角色\n\
            你是一位拥有20年实战经验的独立投研分析师（A股方向）。你的核心能力是**自主决策**——根据数据和逻辑独立判断下一步该做什么，而不是机械执行固定流程。\n\
            \n\
            当前时间：{}\n\
            \n\
            # 核心目标\n\
            自主分析当前市场环境，推荐3-8只值得关注的A股股票。你需要自行决定获取哪些数据、如何解读、如何形成投资逻辑。\n\
            \n\
            # 可用工具（按类分组，你自主决定调用顺序和组合）\n\
            \n\
            **宏观/市场类**（帮你建立全局认知）：\n\
            - get_market_news：最新财经新闻政策\n\
            - get_economic_data：GDP/CPI/PPI/PMI 宏观数据\n\
            - get_global_indexes：全球主要指数行情\n\
            - get_financial_calendar：近期财经事件日历\n\
            \n\
            **大盘/板块类**（帮你判断方向）：\n\
            - get_kline_data：K线数据（可用于指数或个股）\n\
            - get_technical_indicators：技术指标（MA/MACD/KDJ/RSI/BOLL）\n\
            - search_concept_boards：按关键词搜索概念板块\n\
            \n\
            **选股类**（帮你筛选标的）：\n\
            - search_stocks_by_condition：自然语言条件选股（如\"新能源,涨幅大于0%,涨幅小于5%,市盈率小于30\"）\n\
            - batch_get_stock_quotes：批量获取个股行情快照\n\
            - get_stock_quote：单只个股详细行情\n\
            \n\
            **个股深度类**（仅对 Top 3-5 候选使用，不要逐一遍历）：\n\
            - search_stock_news：个股/关键词新闻\n\
            - get_stock_notices：上市公司公告\n\
            - get_industry_report：机构研报\n\
            - get_fund_flow：资金流向\n\
            \n\
            # 决策原则\n\
            \n\
            1. **先全局后局部**：优先建立宏观认知（新闻/经济/外盘/大盘），再聚焦行业方向和个股。但如果你已掌握足够宏观信息，可以直接进入下一步。\n\
            2. **每轮工具调用 ≤3 个**：避免一次消耗过多上下文，每轮最多并行调用3个工具。\n\
            3. **每轮调用前说明意图**：用1-2句话说明\"接下来要做什么、为什么\"，让用户理解你的思考过程。\n\
            4. **个股深入分析限 Top 3-5 只**：只对最终候选股做新闻/公告/研报查询，避免浪费。\n\
            5. **独立思考**：不要因为某概念是热门就推荐，要有你自己的分析逻辑链条。\n\
            \n\
            # 反思机制\n\
            \n\
            - 当选股工具返回0结果时，**先诊断原因**：① 条件组合是否矛盾 ② 关键词是否过于具体 ③ 行业限制是否过窄。\n\
            - 诊断后**调整条件重试**，不要简单重复相同参数。例如：放宽涨幅区间、换用更通用的行业关键词、拆分复合条件。\n\
            - 如果某个投资方向反复无法选出标的，**果断切换到其他方向**，不要死磕。\n\
            - 当工具报错时，分析是参数问题还是服务异常——参数问题则修正重试，服务异常则跳过该工具继续分析。\n\
            \n\
            # 选股硬约束\n\
            \n\
            - 严禁推荐当日涨停（涨幅>=9.5%）或连板股票\n\
            - 优先选择涨幅在-2%~5%之间、尚处于低位但有逻辑支撑的个股\n\
            - 避免推荐近5日涨幅超过15%的标的，寻找同板块补涨机会\n\
            - 关注基本面质量（ROE、营收增速）和合理估值\n\
            \n\
            # 输出格式\n\
            \n\
            最终报告用 Markdown 格式输出，包含：\n\
            1. **宏观环境判断**（经济周期 + 政策方向 + 外盘影响 + 大盘走势）\n\
            2. **投资逻辑**（你的独立思考过程和看好方向）\n\
            3. **推荐股票列表**——用以下格式包裹在 <PICKS> 标签中：\n\
            \n\
            <PICKS>\n\
            [\n\
              {{\n\
                \"code\": \"sh600519\",\n\
                \"name\": \"贵州茅台\",\n\
                \"reason\": \"推荐理由（需包含分析逻辑）\",\n\
                \"rating\": \"buy\",\n\
                \"sector\": \"所属概念板块\",\n\
                \"highlights\": [\"ROE 30%\", \"营收增速 15%\"]\n\
              }}\n\
            ]\n\
            </PICKS>\n\
            \n\
            rating 取值：strong_buy（强烈推荐）、buy（推荐买入）、watch（建议关注）\n\
            \n\
            4. **风险提示**\n\
            \n\
            **重要**：只能通过提供的工具获取数据，禁止编造。",
            today
        );

        let mut messages: Vec<ChatMessage> = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user("请开始分析当前A股市场，自主获取数据并给出你的选股推荐。"),
        ];

        let mut full_content = String::new();
        let mut total_usage: Option<TokenUsage> = None;
        let mut empty_search_count: u32 = 0; // 连续空结果计数
        let mut reflection_injected = false;  // 反思提示是否已注入

        // Phase 1: Tool calling loop
        for _round in 0..MAX_PICK_TOOL_ROUNDS {
            let req = ChatCompletionRequest {
                model: config.model_name.clone(),
                messages: messages.clone(),
                max_tokens: Some(config.max_tokens),
                temperature: Some(config.temperature),
                stream: Some(false),
                tools: Some(tools.clone()),
                tool_choice: None,
            };

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .header("Content-Type", "application/json")
                .json(&req)
                .send()
                .await?;

            if !resp.status().is_success() {
                let body = resp.text().await?;
                return Err(anyhow!("AI API error: {}", body));
            }

            let body = resp.text().await?;
            let response: ChatCompletionResponse = serde_json::from_str(&body)
                .map_err(|e| anyhow!("AI response parse error: {} body: {}", e, &body[..500.min(body.len())]))?;

            if let Some(usage) = &response.usage {
                total_usage = Some(match total_usage {
                    Some(mut u) => {
                        u.prompt_tokens += usage.prompt_tokens;
                        u.completion_tokens += usage.completion_tokens;
                        u.total_tokens += usage.total_tokens;
                        u
                    }
                    None => usage.clone(),
                });
            }

            let choice = response.choices.first()
                .ok_or_else(|| anyhow!("AI 返回空 choices"))?;

            let finish_reason = choice.finish_reason.as_deref().unwrap_or("");
            let msg = choice.message.as_ref().ok_or_else(|| anyhow!("AI 返回空 message"))?;

            if (finish_reason == "tool_calls" || msg.tool_calls.is_some())
                && msg.tool_calls.as_ref().map_or(false, |tc| !tc.is_empty())
            {
                let tool_calls = msg.tool_calls.as_ref().unwrap();

                // 透传 AI 思考内容（如果模型在 tool_calls 同时返回了 content）
                let thinking_content = msg.content.as_ref().filter(|c| !c.is_empty()).cloned();
                if let Some(ref thinking) = thinking_content {
                    let _ = sender.send(AIStreamEvent {
                        event_type: "thinking".to_string(),
                        content: Some(thinking.clone()),
                        done: false,
                        usage: None,
                        tool_name: None,
                    }).await;
                }

                // 使用新方法，同时携带 content 和 tool_calls
                messages.push(ChatMessage::assistant_tool_calls_with_content(
                    thinking_content,
                    tool_calls.clone(),
                ));

                for tc in tool_calls {
                    let tool_name = &tc.function.name;
                    let tool_args = &tc.function.arguments;

                    let _ = sender.send(AIStreamEvent {
                        event_type: "tool_call".to_string(),
                        content: Some(format!("正在获取数据: {}", stock_tools::pick_tool_name_to_chinese(tool_name))),
                        done: false,
                        usage: None,
                        tool_name: Some(tool_name.clone()),
                    }).await;

                    let result = match stock_tools::execute_pick_tool(tool_name, tool_args, qgqp_b_id).await {
                        Ok(r) => r,
                        Err(e) => format!("工具调用失败: {}", e),
                    };

                    // 空结果反思兜底：仅针对 search_stocks_by_condition，连续2次空结果注入提示
                    if tool_name == "search_stocks_by_condition" {
                        let is_empty = result.contains("\"total\":0") || result.contains("\"total\": 0") || result.contains("未找到");
                        if is_empty {
                            empty_search_count += 1;
                        } else {
                            empty_search_count = 0;
                        }

                        if empty_search_count >= 2 && !reflection_injected {
                            reflection_injected = true;
                            messages.push(ChatMessage::tool_result(&tc.id, tool_name, &result));
                            // 注入反思提示
                            messages.push(ChatMessage::user(
                                "注意：选股条件已连续2次未匹配到结果。请分析原因：条件是否过于严格？关键词是否过于具体？行业限制是否过窄？请调整条件后重试，或者果断切换到其他投资方向。"
                            ));
                            let summary = stock_tools::summarize_tool_result(tool_name, &result);
                            let _ = sender.send(AIStreamEvent {
                                event_type: "tool_result".to_string(),
                                content: Some(summary),
                                done: false,
                                usage: None,
                                tool_name: Some(tool_name.clone()),
                            }).await;
                            continue; // 跳过下面的正常 tool_result 处理
                        }
                    }

                    // 生成摘要发给前端展示
                    let summary = stock_tools::summarize_tool_result(tool_name, &result);

                    let _ = sender.send(AIStreamEvent {
                        event_type: "tool_result".to_string(),
                        content: Some(summary),
                        done: false,
                        usage: None,
                        tool_name: Some(tool_name.clone()),
                    }).await;

                    messages.push(ChatMessage::tool_result(&tc.id, tool_name, &result));
                }
                continue;
            }

            if let Some(content) = &msg.content {
                if !content.is_empty() {
                    messages.push(ChatMessage::assistant_text(content));
                }
            }
            break;
        }

        // Phase 2: Stream the final analysis
        let last_is_assistant = messages.last().map_or(false, |m| m.role == "assistant" && m.content.is_some());
        if last_is_assistant {
            messages.pop();
        }

        // 选股分析报告需要较长输出（宏观分析 + 投资逻辑 + PICKS JSON + 风险提示），
        // 确保 max_tokens 不低于 4096，避免输出被截断导致 <PICKS> 标签不完整
        let pick_max_tokens = config.max_tokens.max(4096);

        let req = ChatCompletionRequest {
            model: config.model_name.clone(),
            messages: messages.clone(),
            max_tokens: Some(pick_max_tokens),
            temperature: Some(config.temperature),
            stream: Some(true),
            tools: None,
            tool_choice: None,
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("AI API error: {}", body));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut dsml_detected = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    // done 事件由 ai_pick_cmd.rs 统一发送，此处不再发送，避免重复
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(chunk_resp) = serde_json::from_str::<ChatCompletionResponse>(data) {
                        if let Some(choice) = chunk_resp.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    if content.contains("<\u{ff5c}") || content.contains("DSML") || content.contains("<｜") {
                                        dsml_detected = true;
                                    }
                                    full_content.push_str(content);
                                    if !dsml_detected {
                                        let _ = sender.send(AIStreamEvent {
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
                        if let Some(usage) = &chunk_resp.usage {
                            total_usage = Some(match total_usage {
                                Some(mut u) => {
                                    u.prompt_tokens += usage.prompt_tokens;
                                    u.completion_tokens += usage.completion_tokens;
                                    u.total_tokens += usage.total_tokens;
                                    u
                                }
                                None => usage.clone(),
                            });
                        }
                    }
                }
            }
        }

        // 返回清理后的内容（done 事件由调用方统一发送）
        let clean_content = clean_dsml_artifacts(&full_content);
        Ok((clean_content, total_usage))
    }

    /// AI 找相似股：基于给定股票，从同板块中找出低位补涨机会
    pub async fn find_similar_stocks_with_tools(
        config: &AIConfig,
        code: &str,
        name: &str,
        sector: &str,
        qgqp_b_id: &str,
        sender: tokio::sync::mpsc::Sender<AIStreamEvent>,
    ) -> Result<(String, Option<TokenUsage>)> {
        let client = build_ai_client(config.timeout_secs)?;
        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let tools = stock_tools::get_pick_tool_definitions();

        let today = chrono::Local::now().format("%Y年%m月%d日 %H:%M").to_string();

        let system_prompt = format!(
            "# 角色\n\
            你是一位资深A股投研分析师，擅长挖掘板块内补涨机会。\n\
            \n\
            当前时间：{}\n\
            \n\
            # 任务\n\
            用户正在关注 {}（{}），所属概念板块：{}。\n\
            该股可能已涨幅较大或涨停，追高风险大。请帮用户在同概念或相关板块中，找出3-6只尚未大涨、有补涨潜力的个股。\n\
            \n\
            # 决策原则\n\
            1. 先了解目标股特征（行情、涨幅、估值），再搜索同板块低位标的\n\
            2. 每轮调用前说明意图（1-2句话）\n\
            3. 每轮工具调用 ≤3 个\n\
            4. 选股结果为空时，分析是条件太严还是板块本身偏弱，然后调整重试\n\
            \n\
            # 选股标准\n\
            - 与目标股同概念或相近板块\n\
            - 今日涨幅远低于目标股（优先<5%）\n\
            - **严禁推荐涨停股（涨幅>=9.5%）**\n\
            - 基本面不低于目标股\n\
            - 市值级别相近\n\
            \n\
            # 输出格式\n\
            先说明目标股特征和选股逻辑，然后用 <PICKS> 标签给出推荐：\n\
            \n\
            <PICKS>\n\
            [\n\
              {{\n\
                \"code\": \"sh600519\",\n\
                \"name\": \"示例股票\",\n\
                \"reason\": \"与目标股同属XX概念，但今日仅涨1.5%...\",\n\
                \"rating\": \"buy\",\n\
                \"sector\": \"所属板块\",\n\
                \"highlights\": [\"补涨空间大\", \"ROE 20%\"]\n\
              }}\n\
            ]\n\
            </PICKS>\n\
            \n\
            rating 取值：strong_buy、buy、watch\n\
            \n\
            **重要**：禁止编造数据。用 Markdown 输出分析。",
            today, name, code, sector
        );

        let mut messages: Vec<ChatMessage> = vec![
            ChatMessage::system(&system_prompt),
            ChatMessage::user(&format!(
                "请帮我找出与 {}({}) 相似但尚未大涨的补涨机会股票。该股所属板块：{}",
                name, code, sector
            )),
        ];

        let mut full_content = String::new();
        let mut total_usage: Option<TokenUsage> = None;

        // Phase 1: Tool calling loop (reuse same pattern as ai_pick)
        for _round in 0..MAX_PICK_TOOL_ROUNDS {
            let req = ChatCompletionRequest {
                model: config.model_name.clone(),
                messages: messages.clone(),
                max_tokens: Some(config.max_tokens),
                temperature: Some(config.temperature),
                stream: Some(false),
                tools: Some(tools.clone()),
                tool_choice: None,
            };

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", config.api_key))
                .header("Content-Type", "application/json")
                .json(&req)
                .send()
                .await?;

            if !resp.status().is_success() {
                let body = resp.text().await?;
                return Err(anyhow!("AI API error: {}", body));
            }

            let body = resp.text().await?;
            let response: ChatCompletionResponse = serde_json::from_str(&body)
                .map_err(|e| anyhow!("AI response parse error: {} body: {}", e, &body[..500.min(body.len())]))?;

            if let Some(usage) = &response.usage {
                total_usage = Some(match total_usage {
                    Some(mut u) => {
                        u.prompt_tokens += usage.prompt_tokens;
                        u.completion_tokens += usage.completion_tokens;
                        u.total_tokens += usage.total_tokens;
                        u
                    }
                    None => usage.clone(),
                });
            }

            let choice = response.choices.first()
                .ok_or_else(|| anyhow!("AI 返回空 choices"))?;

            let finish_reason = choice.finish_reason.as_deref().unwrap_or("");
            let msg = choice.message.as_ref().ok_or_else(|| anyhow!("AI 返回空 message"))?;

            if (finish_reason == "tool_calls" || msg.tool_calls.is_some())
                && msg.tool_calls.as_ref().map_or(false, |tc| !tc.is_empty())
            {
                let tool_calls = msg.tool_calls.as_ref().unwrap();

                // 透传 AI 思考内容
                let thinking_content = msg.content.as_ref().filter(|c| !c.is_empty()).cloned();
                if let Some(ref thinking) = thinking_content {
                    let _ = sender.send(AIStreamEvent {
                        event_type: "thinking".to_string(),
                        content: Some(thinking.clone()),
                        done: false,
                        usage: None,
                        tool_name: None,
                    }).await;
                }

                messages.push(ChatMessage::assistant_tool_calls_with_content(
                    thinking_content,
                    tool_calls.clone(),
                ));

                for tc in tool_calls {
                    let tool_name = &tc.function.name;
                    let tool_args = &tc.function.arguments;

                    let _ = sender.send(AIStreamEvent {
                        event_type: "tool_call".to_string(),
                        content: Some(format!("正在获取: {}", stock_tools::pick_tool_name_to_chinese(tool_name))),
                        done: false,
                        usage: None,
                        tool_name: Some(tool_name.clone()),
                    }).await;

                    let result = match stock_tools::execute_pick_tool(tool_name, tool_args, qgqp_b_id).await {
                        Ok(r) => r,
                        Err(e) => format!("工具调用失败: {}", e),
                    };

                    let summary = stock_tools::summarize_tool_result(tool_name, &result);

                    let _ = sender.send(AIStreamEvent {
                        event_type: "tool_result".to_string(),
                        content: Some(summary),
                        done: false,
                        usage: None,
                        tool_name: Some(tool_name.clone()),
                    }).await;

                    messages.push(ChatMessage::tool_result(&tc.id, tool_name, &result));
                }
                continue;
            }

            if let Some(content) = &msg.content {
                if !content.is_empty() {
                    messages.push(ChatMessage::assistant_text(content));
                }
            }
            break;
        }

        // Phase 2: Stream the final analysis
        let last_is_assistant = messages.last().map_or(false, |m| m.role == "assistant" && m.content.is_some());
        if last_is_assistant {
            messages.pop();
        }

        // 相似股分析也需要较长输出，确保 max_tokens 不低于 4096
        let similar_max_tokens = config.max_tokens.max(4096);

        let req = ChatCompletionRequest {
            model: config.model_name.clone(),
            messages: messages.clone(),
            max_tokens: Some(similar_max_tokens),
            temperature: Some(config.temperature),
            stream: Some(true),
            tools: None,
            tool_choice: None,
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("AI API error: {}", body));
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut dsml_detected = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    // done 事件由 ai_pick_cmd.rs 统一发送，此处不再发送
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(chunk_resp) = serde_json::from_str::<ChatCompletionResponse>(data) {
                        if let Some(choice) = chunk_resp.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    if content.contains("<\u{ff5c}") || content.contains("DSML") || content.contains("<｜") {
                                        dsml_detected = true;
                                    }
                                    full_content.push_str(content);
                                    if !dsml_detected {
                                        let _ = sender.send(AIStreamEvent {
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
                        if let Some(usage) = &chunk_resp.usage {
                            total_usage = Some(match total_usage {
                                Some(mut u) => {
                                    u.prompt_tokens += usage.prompt_tokens;
                                    u.completion_tokens += usage.completion_tokens;
                                    u.total_tokens += usage.total_tokens;
                                    u
                                }
                                None => usage.clone(),
                            });
                        }
                    }
                }
            }
        }

        let clean_content = clean_dsml_artifacts(&full_content);
        Ok((clean_content, total_usage))
    }
}

fn extract_json_array(text: &str) -> Result<String> {
    let text = text.trim();
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return Ok(text[start..=end].to_string());
        }
    }
    let stripped = text
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Some(start) = stripped.find('[') {
        if let Some(end) = stripped.rfind(']') {
            return Ok(stripped[start..=end].to_string());
        }
    }
    Err(anyhow!("Cannot find JSON array in AI response"))
}

fn tool_name_to_chinese(name: &str) -> &str {
    match name {
        "get_stock_quote" => "实时行情",
        "get_kline_data" => "K线数据",
        "get_technical_indicators" => "技术指标",
        "get_fund_flow" => "资金流向",
        _ => name,
    }
}

/// 清理 DeepSeek DSML 标记等模型内部格式
fn clean_dsml_artifacts(content: &str) -> String {
    // 匹配 <｜...｜> 相关的 DSML 块（包括 function_calls 等）
    let re = regex::Regex::new(r"<[｜\u{ff5c}][^>]*>[\s\S]*").unwrap_or_else(|_| regex::Regex::new(".^").unwrap());
    let cleaned = re.replace(content, "").to_string();
    // 清理未闭合的 <PICKS> 标签（AI 输出截断时可能残留）
    let re_picks = regex::Regex::new(r"<PICKS>[\s\S]*").unwrap_or_else(|_| regex::Regex::new(".^").unwrap());
    let cleaned = if cleaned.contains("<PICKS>") && !cleaned.contains("</PICKS>") {
        re_picks.replace(&cleaned, "").to_string()
    } else {
        cleaned
    };
    cleaned.trim_end().to_string()
}
