use anyhow::Result;
use serde_json::Value;

use crate::services::history_kline::HistoryKlineService;
use crate::services::market_scanner::MarketScanner;
use crate::services::technical_indicators;
use crate::services::news_service;
use crate::services::smart_stock::SmartStockService;
use crate::models::watchlist::KlineItem;
use crate::utils::http;

/// AI 可调用的工具定义（OpenAI function calling 格式）— 诊股专用
pub fn get_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_stock_quote",
                "description": "获取股票实时行情快照，包括最新价、涨跌幅、PE/PB/ROE、市值、换手率、量比、主力净流入、5日/20日涨幅等多维度数据",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "股票代码，如 sh600519、sz000001"
                        }
                    },
                    "required": ["code"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_kline_data",
                "description": "获取股票历史K线数据（前复权），包括日期、开高低收、成交量、涨跌幅。可选日K或周K，可指定获取最近N根K线",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "股票代码，如 sh600519、sz000001"
                        },
                        "period": {
                            "type": "string",
                            "enum": ["day", "week"],
                            "description": "K线周期，day=日K线，week=周K线，默认day"
                        },
                        "count": {
                            "type": "integer",
                            "description": "获取最近N根K线，默认60，最多120"
                        }
                    },
                    "required": ["code"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_technical_indicators",
                "description": "获取股票技术分析指标，包括：MA均线(5/10/20/60)、MACD(DIF/DEA/柱)、KDJ、RSI(6/12/24)、布林带(上/中/下轨)，以及技术信号（金叉/死叉/超买超卖/背离等）、均线排列状态、量价关系",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "股票代码，如 sh600519、sz000001"
                        },
                        "period": {
                            "type": "string",
                            "enum": ["day", "week"],
                            "description": "K线周期，默认day"
                        }
                    },
                    "required": ["code"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_fund_flow",
                "description": "获取股票资金流向数据，包括主力净流入金额和主力净占比",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "股票代码，如 sh600519、sz000001"
                        }
                    },
                    "required": ["code"]
                }
            }
        }),
    ]
}

/// 执行工具调用，返回 JSON 字符串结果
pub async fn execute_tool(name: &str, arguments: &str) -> Result<String> {
    let args: Value = serde_json::from_str(arguments).unwrap_or(Value::Object(Default::default()));

    match name {
        "get_stock_quote" => {
            let code = args["code"].as_str().unwrap_or("").to_string();
            get_stock_quote(&code).await
        }
        "get_kline_data" => {
            let code = args["code"].as_str().unwrap_or("").to_string();
            let period = args["period"].as_str().unwrap_or("day").to_string();
            let count = args["count"].as_u64().unwrap_or(60) as usize;
            get_kline_data(&code, &period, count).await
        }
        "get_technical_indicators" => {
            let code = args["code"].as_str().unwrap_or("").to_string();
            let period = args["period"].as_str().unwrap_or("day").to_string();
            get_technical_indicators(&code, &period).await
        }
        "get_fund_flow" => {
            let code = args["code"].as_str().unwrap_or("").to_string();
            get_fund_flow(&code).await
        }
        _ => Ok(format!("未知工具: {}", name)),
    }
}

/// 获取实时行情快照
async fn get_stock_quote(code: &str) -> Result<String> {
    let scanner = MarketScanner::new()?;
    let codes = vec![code.to_string()];
    let snapshots = scanner.fetch_stocks_by_codes(&codes).await?;

    if let Some(s) = snapshots.first() {
        let result = serde_json::json!({
            "code": s.code,
            "name": s.name,
            "price": s.price,
            "change_pct": format!("{:.2}%", s.change_pct),
            "change_amount": s.change_amount,
            "open": s.open,
            "high": s.high,
            "low": s.low,
            "pre_close": s.pre_close,
            "volume": format!("{:.0}手", s.volume),
            "amount": format_amount(s.amount),
            "amplitude": format!("{:.2}%", s.amplitude),
            "turnover_rate": format!("{:.2}%", s.turnover_rate),
            "pe_ttm": if s.pe_ttm > 0.0 { format!("{:.2}", s.pe_ttm) } else { "N/A".to_string() },
            "pb": if s.pb > 0.0 { format!("{:.2}", s.pb) } else { "N/A".to_string() },
            "roe": if s.roe != 0.0 { format!("{:.2}%", s.roe) } else { "N/A".to_string() },
            "total_market_cap": format_amount(s.total_market_cap),
            "float_market_cap": format_amount(s.float_market_cap),
            "volume_ratio": format!("{:.2}", s.volume_ratio),
            "main_net_inflow": format_amount(s.main_net_inflow),
            "pct_5d": format!("{:.2}%", s.pct_5d),
            "pct_20d": format!("{:.2}%", s.pct_20d),
            "revenue_yoy": if s.revenue_yoy != 0.0 { format!("{:.2}%", s.revenue_yoy) } else { "N/A".to_string() },
        });
        Ok(serde_json::to_string_pretty(&result)?)
    } else {
        Ok(format!("未找到股票 {} 的行情数据", code))
    }
}

/// 获取K线数据
async fn get_kline_data(code: &str, period: &str, count: usize) -> Result<String> {
    let count = count.min(120);
    let kline_service = HistoryKlineService::new()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start = "2023-01-01";

    let all_items = kline_service.fetch_kline(code, period, start, &today, 640).await?;

    // 取最近 count 根
    let items: Vec<&KlineItem> = if all_items.len() > count {
        all_items[all_items.len() - count..].iter().collect()
    } else {
        all_items.iter().collect()
    };

    let klines: Vec<Value> = items.iter().map(|k| {
        serde_json::json!({
            "date": k.date,
            "open": format!("{:.2}", k.open),
            "close": format!("{:.2}", k.close),
            "high": format!("{:.2}", k.high),
            "low": format!("{:.2}", k.low),
            "volume": format!("{:.0}", k.volume),
            "change_pct": format!("{:.2}%", k.change_pct),
        })
    }).collect();

    let result = serde_json::json!({
        "code": code,
        "period": period,
        "count": klines.len(),
        "klines": klines,
    });

    Ok(serde_json::to_string(&result)?)
}

/// 获取技术指标
async fn get_technical_indicators(code: &str, period: &str) -> Result<String> {
    let kline_service = HistoryKlineService::new()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start = "2023-01-01";

    let klines = kline_service.fetch_kline(code, period, start, &today, 640).await?;

    if klines.is_empty() {
        return Ok(format!("未找到股票 {} 的K线数据，无法计算技术指标", code));
    }

    let indicators = technical_indicators::compute_indicators(&klines);
    let signals = technical_indicators::detect_signals(&klines, &indicators);
    let ma_alignment = technical_indicators::determine_ma_alignment(&indicators);
    let volume_price = technical_indicators::determine_volume_price_relation(&klines);
    let summary = technical_indicators::generate_summary(&ma_alignment, &volume_price, &signals);

    let n = klines.len();

    // 只返回最近20根K线的指标数值，避免数据量过大
    let recent_count = 20.min(n);
    let start_idx = n - recent_count;

    let mut recent_indicators = Vec::new();
    for i in start_idx..n {
        let mut ind = serde_json::Map::new();
        ind.insert("date".to_string(), Value::String(klines[i].date.clone()));
        ind.insert("close".to_string(), serde_json::json!(format!("{:.2}", klines[i].close)));
        ind.insert("volume".to_string(), serde_json::json!(format!("{:.0}", klines[i].volume)));

        if let Some(v) = indicators.ma5[i] { ind.insert("MA5".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.ma10[i] { ind.insert("MA10".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.ma20[i] { ind.insert("MA20".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.ma60[i] { ind.insert("MA60".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.macd_dif[i] { ind.insert("MACD_DIF".to_string(), serde_json::json!(format!("{:.4}", v))); }
        if let Some(v) = indicators.macd_dea[i] { ind.insert("MACD_DEA".to_string(), serde_json::json!(format!("{:.4}", v))); }
        if let Some(v) = indicators.macd_hist[i] { ind.insert("MACD_HIST".to_string(), serde_json::json!(format!("{:.4}", v))); }
        if let Some(v) = indicators.kdj_k[i] { ind.insert("KDJ_K".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.kdj_d[i] { ind.insert("KDJ_D".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.kdj_j[i] { ind.insert("KDJ_J".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.rsi6[i] { ind.insert("RSI6".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.rsi12[i] { ind.insert("RSI12".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.boll_upper[i] { ind.insert("BOLL_UPPER".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.boll_middle[i] { ind.insert("BOLL_MIDDLE".to_string(), serde_json::json!(format!("{:.2}", v))); }
        if let Some(v) = indicators.boll_lower[i] { ind.insert("BOLL_LOWER".to_string(), serde_json::json!(format!("{:.2}", v))); }

        recent_indicators.push(Value::Object(ind));
    }

    let signals_json: Vec<Value> = signals.iter().map(|s| {
        serde_json::json!({
            "type": s.signal_type,
            "direction": s.direction,
            "description": s.description,
            "strength": s.strength,
            "date": s.date,
        })
    }).collect();

    let result = serde_json::json!({
        "code": code,
        "period": period,
        "total_klines": n,
        "ma_alignment": format!("{:?}", ma_alignment),
        "volume_price_relation": format!("{:?}", volume_price),
        "summary": summary,
        "signals": signals_json,
        "recent_indicators": recent_indicators,
    });

    Ok(serde_json::to_string(&result)?)
}

/// 获取资金流向
async fn get_fund_flow(code: &str) -> Result<String> {
    let scanner = MarketScanner::new()?;
    let codes = vec![code.to_string()];
    let flows = scanner.fetch_fund_flow(&codes).await?;

    if let Some((c, net_inflow, net_pct)) = flows.first() {
        let result = serde_json::json!({
            "code": c,
            "main_net_inflow": format_amount(*net_inflow),
            "main_net_inflow_raw": net_inflow,
            "main_net_pct": format!("{:.2}%", net_pct),
        });
        Ok(serde_json::to_string_pretty(&result)?)
    } else {
        Ok(format!("未找到股票 {} 的资金流向数据", code))
    }
}

/// AI 选股专用工具定义（理性分析模式）
pub fn get_pick_tool_definitions() -> Vec<Value> {
    vec![
        // ===== 信息采集层 =====
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_market_news",
                "description": "获取最新市场财经新闻摘要，聚合财联社电报、东方财富要闻、新浪滚动新闻，返回近期重要新闻标题和摘要",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "count": { "type": "integer", "description": "每个源获取条数，默认30，最多60" }
                    },
                    "required": []
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_economic_data",
                "description": "获取宏观经济数据，包括GDP(国内生产总值)、CPI(居民消费价格指数)、PPI(工业品出厂价格指数)、PMI(采购经理人指数)。返回最近4期数据，帮助判断经济周期和政策方向",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "indicator": {
                            "type": "string",
                            "enum": ["all", "gdp", "cpi", "ppi", "pmi"],
                            "description": "指标类型：all=全部(默认)，gdp/cpi/ppi/pmi=单个指标"
                        }
                    },
                    "required": []
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_global_indexes",
                "description": "获取全球主要股票指数行情，包括道琼斯、纳斯达克、标普500、恒生指数、日经225、欧洲主要指数等，帮助判断外盘环境对A股的影响",
                "parameters": { "type": "object", "properties": {}, "required": [] }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_financial_calendar",
                "description": "获取近期财经日历/重要事件，包括经济数据发布日、重要会议、政策窗口期等，帮助判断事件驱动的投资机会",
                "parameters": { "type": "object", "properties": {}, "required": [] }
            }
        }),
        // ===== 大盘/个股分析层 =====
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_kline_data",
                "description": "获取股票或指数的历史K线数据（前复权），包括日期、开高低收、成交量、涨跌幅。支持指数代码：sh000001(上证指数)、sz399001(深证成指)、sz399006(创业板指)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "股票/指数代码，如sh600519、sh000001(上证指数)" },
                        "period": { "type": "string", "enum": ["day", "week"], "description": "K线周期，默认day" },
                        "count": { "type": "integer", "description": "最近N根K线，默认30，最多120" }
                    },
                    "required": ["code"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_technical_indicators",
                "description": "获取股票或指数的技术分析指标：MA均线、MACD、KDJ、RSI、布林带，以及技术信号（金叉/死叉/背离等）。支持指数代码：sh000001(上证)、sz399001(深证)、sz399006(创业板)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "股票/指数代码" },
                        "period": { "type": "string", "enum": ["day", "week"], "description": "K线周期，默认day" }
                    },
                    "required": ["code"]
                }
            }
        }),
        // ===== 选股层 =====
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_stocks_by_condition",
                "description": "通过自然语言条件选股(东方财富NLP选股器)。支持概念/板块、技术指标、涨跌幅、换手率、估值、市值、资金、财务等条件自由组合",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "keyword": { "type": "string", "description": "自然语言选股条件，如\"人工智能，涨幅大于1%，MACD金叉，换手率大于3%\"" },
                        "page_size": { "type": "integer", "description": "返回数量，默认20，最多50" }
                    },
                    "required": ["keyword"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_concept_boards",
                "description": "通过自然语言搜索概念板块/行业板块数据，查询板块涨幅排行或特定概念板块情况",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "keyword": { "type": "string", "description": "板块查询条件，如\"今日涨幅前15的概念板块\"" },
                        "page_size": { "type": "integer", "description": "返回数量，默认20，最多50" }
                    },
                    "required": ["keyword"]
                }
            }
        }),
        // ===== 验证层 =====
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "batch_get_stock_quotes",
                "description": "批量获取多只股票详细行情（最新价、涨跌幅、PE/PB/ROE、市值、换手率、量比、主力净流入、5日/20日涨幅等），一次最多20只",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "codes": { "type": "array", "items": { "type": "string" }, "description": "股票代码数组，如[\"sh600519\",\"sz000001\"]" }
                    },
                    "required": ["codes"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_stock_quote",
                "description": "获取单只股票实时行情快照",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "股票代码，如sh600519" }
                    },
                    "required": ["code"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_fund_flow",
                "description": "获取股票资金流向数据，包括主力净流入金额和占比",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "股票代码" }
                    },
                    "required": ["code"]
                }
            }
        }),
        // ===== 深入分析层 =====
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_stock_news",
                "description": "按关键词搜索个股相关新闻/资讯，验证候选股是否有真实催化剂或利空消息。只对最终候选的3-5只股票使用",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "keyword": { "type": "string", "description": "搜索关键词，通常为股票名称，如\"贵州茅台\"" }
                    },
                    "required": ["keyword"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_stock_notices",
                "description": "获取上市公司最新公告(业绩预告/重大合同/定增/减持等)，比新闻更权威。只对最终候选股使用",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "股票代码，如sh600519" }
                    },
                    "required": ["code"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_industry_report",
                "description": "获取行业或个股研报摘要列表(标题/机构/评级/日期)，帮助了解机构观点。不传code返回最新行业研报，传code返回个股研报",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "可选，股票代码。不传则返回最新行业研报" }
                    },
                    "required": []
                }
            }
        }),
    ]
}

/// 执行选股工具调用
/// qgqp_b_id: 东财用户标识，用于 NLP 选股 API 的 fingerprint 字段
pub async fn execute_pick_tool(name: &str, arguments: &str, qgqp_b_id: &str) -> Result<String> {
    let args: Value = serde_json::from_str(arguments).unwrap_or(Value::Object(Default::default()));

    match name {
        "get_market_news" => {
            let count = args["count"].as_u64().unwrap_or(30).min(60) as u32;
            get_market_news(count).await
        }
        "get_economic_data" => {
            let indicator = args["indicator"].as_str().unwrap_or("all").to_string();
            get_economic_data(&indicator).await
        }
        "get_global_indexes" => {
            get_global_indexes().await
        }
        "get_financial_calendar" => {
            get_financial_calendar().await
        }
        "search_stocks_by_condition" => {
            let keyword = args["keyword"].as_str().unwrap_or("").to_string();
            let page_size = args["page_size"].as_u64().unwrap_or(20).min(50) as u32;
            search_stocks_by_condition(&keyword, page_size, qgqp_b_id).await
        }
        "search_concept_boards" => {
            let keyword = args["keyword"].as_str().unwrap_or("").to_string();
            let page_size = args["page_size"].as_u64().unwrap_or(20).min(50) as u32;
            search_concept_boards(&keyword, page_size, qgqp_b_id).await
        }
        "batch_get_stock_quotes" => {
            let codes: Vec<String> = args["codes"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            batch_get_stock_quotes(&codes).await
        }
        "search_stock_news" => {
            let keyword = args["keyword"].as_str().unwrap_or("").to_string();
            search_stock_news_tool(&keyword).await
        }
        "get_stock_notices" => {
            let code = args["code"].as_str().unwrap_or("").to_string();
            get_stock_notices(&code).await
        }
        "get_industry_report" => {
            let code = args["code"].as_str().map(|s| s.to_string());
            get_industry_report(code.as_deref()).await
        }
        // 诊股工具也支持
        "get_stock_quote" | "get_kline_data" | "get_technical_indicators" | "get_fund_flow" => {
            execute_tool(name, arguments).await
        }
        _ => Ok(format!("未知工具: {}", name)),
    }
}

/// 获取市场最新新闻摘要（每个源独立超时8秒，任一失败不影响其他）
async fn get_market_news(count: u32) -> Result<String> {
    use tokio::time::{timeout, Duration};

    let cls_fut = timeout(Duration::from_secs(8), news_service::fetch_cls_telegraph(count));
    let em_fut = timeout(Duration::from_secs(8), news_service::fetch_eastmoney_news(1, count));
    let sina_fut = timeout(Duration::from_secs(8), news_service::fetch_sina_roll_news(1, count));

    let (cls, em, sina) = tokio::join!(cls_fut, em_fut, sina_fut);

    let mut items = Vec::new();

    if let Ok(Ok(news)) = cls {
        for n in news.into_iter().take(count as usize) {
            let importance_tag = if n.importance >= 2 { "【重要】" } else if n.importance >= 1 { "【关注】" } else { "" };
            let stocks_str = if !n.related_stocks.is_empty() {
                format!(" [关联股票: {}]", n.related_stocks.join(","))
            } else {
                String::new()
            };
            let summary = truncate_str(&n.summary, 200);
            items.push(serde_json::json!({
                "source": "财联社",
                "time": n.publish_time,
                "title": format!("{}{}", importance_tag, n.title),
                "summary": summary,
                "related_stocks": stocks_str,
            }));
        }
    }

    if let Ok(Ok(news)) = em {
        for n in news.into_iter().take((count / 2) as usize) {
            let summary = truncate_str(&n.summary, 150);
            items.push(serde_json::json!({
                "source": "东方财富",
                "time": n.publish_time,
                "title": n.title,
                "summary": summary,
            }));
        }
    }

    if let Ok(Ok(news)) = sina {
        for n in news.into_iter().take((count / 3) as usize) {
            items.push(serde_json::json!({
                "source": "新浪财经",
                "time": n.publish_time,
                "title": n.title,
            }));
        }
    }

    if items.is_empty() {
        return Ok(r#"{"total":0,"news":[],"note":"所有新闻源暂时不可用，请AI基于自身知识进行分析"}"#.to_string());
    }

    let result = serde_json::json!({
        "total": items.len(),
        "news": items,
    });
    Ok(serde_json::to_string(&result)?)
}

/// NLP 智能选股：复用 SmartStockService::search_stock（已验证可用）
async fn search_stocks_by_condition(keyword: &str, page_size: u32, qgqp_b_id: &str) -> Result<String> {
    if keyword.is_empty() {
        return Ok(r#"{"error":"请提供选股条件关键词"}"#.to_string());
    }

    if qgqp_b_id.is_empty() {
        return Ok(r#"{"error":"未配置东财用户标识(qgqp_b_id)，请在设置中配置后重试","keyword":""}"#.to_string());
    }

    match SmartStockService::search_stock(keyword, page_size as usize, qgqp_b_id).await {
        Ok(resp) => {
            if resp.code != 100 {
                let msg = resp.msg.or(resp.message).unwrap_or_default();
                return Ok(serde_json::json!({
                    "error": format!("选股条件解析失败(code={}): {}", resp.code, msg),
                    "keyword": keyword,
                }).to_string());
            }

            let mut stocks = Vec::new();
            if let Some(data) = &resp.data {
                for item in data.result.data_list.iter().take(page_size as usize) {
                    let sec_code = item["SECURITY_CODE"].as_str().unwrap_or("");
                    let sec_name = item["SECURITY_SHORT_NAME"].as_str().unwrap_or("");
                    let values = &item["values"];
                    let mut stock_info = serde_json::json!({
                        "code": sec_code,
                        "name": sec_name,
                    });
                    if let Some(obj) = values.as_object() {
                        for (key, val) in obj {
                            if let Some(v) = val.as_str() {
                                if !v.is_empty() {
                                    stock_info[key] = Value::String(v.to_string());
                                }
                            } else if let Some(v) = val.as_f64() {
                                stock_info[key] = serde_json::json!(v);
                            }
                        }
                    }
                    stocks.push(stock_info);
                }
            }

            let total_count = resp.data.as_ref()
                .and_then(|d| d.result.data_list.first())
                .map(|_| stocks.len())
                .unwrap_or(0);

            let output = serde_json::json!({
                "keyword": keyword,
                "total_count": total_count,
                "returned": stocks.len(),
                "stocks": stocks,
            });
            Ok(serde_json::to_string(&output)?)
        }
        Err(e) => {
            Ok(serde_json::json!({
                "error": format!("选股API调用失败: {}", e),
                "keyword": keyword,
            }).to_string())
        }
    }
}

/// NLP 板块搜索：复用 SmartStockService::search_board（使用正确的Host header）
async fn search_concept_boards(keyword: &str, page_size: u32, qgqp_b_id: &str) -> Result<String> {
    if keyword.is_empty() {
        return Ok(r#"{"error":"请提供板块查询条件"}"#.to_string());
    }

    if qgqp_b_id.is_empty() {
        return Ok(r#"{"error":"未配置东财用户标识(qgqp_b_id)，请在设置中配置后重试","keyword":""}"#.to_string());
    }

    match SmartStockService::search_board(keyword, page_size as usize, qgqp_b_id).await {
        Ok(resp) => {
            if resp.code != 100 {
                let msg = resp.msg.or(resp.message).unwrap_or_default();
                return Ok(serde_json::json!({
                    "error": format!("板块查询解析失败(code={}): {}", resp.code, msg),
                    "keyword": keyword,
                }).to_string());
            }

            let mut boards = Vec::new();
            if let Some(data) = &resp.data {
                for item in data.result.data_list.iter().take(page_size as usize) {
                    let sec_code = item["SECURITY_CODE"].as_str().unwrap_or("");
                    let sec_name = item["SECURITY_SHORT_NAME"].as_str().unwrap_or("");
                    let values = &item["values"];
                    let mut board_info = serde_json::json!({
                        "code": sec_code,
                        "name": sec_name,
                    });
                    if let Some(obj) = values.as_object() {
                        for (key, val) in obj {
                            if let Some(v) = val.as_str() {
                                if !v.is_empty() {
                                    board_info[key] = Value::String(v.to_string());
                                }
                            } else if let Some(v) = val.as_f64() {
                                board_info[key] = serde_json::json!(v);
                            }
                        }
                    }
                    boards.push(board_info);
                }
            }

            let total_count = boards.len();

            let output = serde_json::json!({
                "keyword": keyword,
                "total_count": total_count,
                "returned": boards.len(),
                "boards": boards,
            });
            Ok(serde_json::to_string(&output)?)
        }
        Err(e) => {
            Ok(serde_json::json!({
                "error": format!("板块查询API调用失败: {}", e),
                "keyword": keyword,
            }).to_string())
        }
    }
}

/// 批量获取股票行情
async fn batch_get_stock_quotes(codes: &[String]) -> Result<String> {
    if codes.is_empty() {
        return Ok(r#"{"error":"未提供股票代码"}"#.to_string());
    }
    let codes: Vec<String> = codes.iter().take(20).cloned().collect();
    let scanner = MarketScanner::new()?;
    let snapshots = scanner.fetch_stocks_by_codes(&codes).await?;

    let stocks: Vec<Value> = snapshots.iter().map(|s| {
        serde_json::json!({
            "code": s.code,
            "name": s.name,
            "price": s.price,
            "change_pct": format!("{:.2}%", s.change_pct),
            "pe_ttm": if s.pe_ttm > 0.0 { format!("{:.2}", s.pe_ttm) } else { "N/A".to_string() },
            "pb": if s.pb > 0.0 { format!("{:.2}", s.pb) } else { "N/A".to_string() },
            "roe": if s.roe != 0.0 { format!("{:.2}%", s.roe) } else { "N/A".to_string() },
            "total_market_cap": format_amount(s.total_market_cap),
            "turnover_rate": format!("{:.2}%", s.turnover_rate),
            "volume_ratio": format!("{:.2}", s.volume_ratio),
            "main_net_inflow": format_amount(s.main_net_inflow),
            "pct_5d": format!("{:.2}%", s.pct_5d),
            "pct_20d": format!("{:.2}%", s.pct_20d),
            "revenue_yoy": if s.revenue_yoy != 0.0 { format!("{:.2}%", s.revenue_yoy) } else { "N/A".to_string() },
            "amount": format_amount(s.amount),
        })
    }).collect();

    let result = serde_json::json!({
        "total": stocks.len(),
        "stocks": stocks,
    });
    Ok(serde_json::to_string(&result)?)
}

// ============================================================
// 新增工具实现：宏观经济 / 全球指数 / 财经日历 / 个股新闻 / 公告 / 研报
// ============================================================

/// 获取宏观经济数据（GDP/CPI/PPI/PMI）
async fn get_economic_data(indicator: &str) -> Result<String> {
    let client = http::build_datacenter_client()?;
    let ts = chrono::Utc::now().timestamp_millis();
    let mut result = serde_json::Map::new();

    let indicators: Vec<&str> = match indicator {
        "gdp" => vec!["gdp"],
        "cpi" => vec!["cpi"],
        "ppi" => vec!["ppi"],
        "pmi" => vec!["pmi"],
        _ => vec!["gdp", "cpi", "ppi", "pmi"],
    };

    for ind in &indicators {
        let (report_name, columns) = match *ind {
            "gdp" => ("RPT_ECONOMY_GDP", "REPORT_DATE,TIME,DOMESTICL_PRODUCT_BASE,SUM_SAME,FIRST_SAME,SECOND_SAME,THIRD_SAME"),
            "cpi" => ("RPT_ECONOMY_CPI", "REPORT_DATE,TIME,NATIONAL_SAME,NATIONAL_BASE,NATIONAL_SEQUENTIAL,NATIONAL_ACCUMULATE"),
            "ppi" => ("RPT_ECONOMY_PPI", "REPORT_DATE,TIME,BASE,BASE_SAME,BASE_ACCUMULATE"),
            "pmi" => ("RPT_ECONOMY_PMI", "REPORT_DATE,TIME,MAKE_INDEX,MAKE_SAME,NMAKE_INDEX,NMAKE_SAME"),
            _ => continue,
        };

        let url = format!(
            "https://datacenter-web.eastmoney.com/api/data/v1/get?callback=datatable&reportName={}&columns={}&pageNumber=1&pageSize=4&sortColumns=REPORT_DATE&sortTypes=-1&source=WEB&client=WEB&_={}",
            report_name, columns, ts
        );

        match client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    // JSONP 解包: datatable...({...})
                    if let Some(start) = text.find('(') {
                        let end = text.rfind(')').unwrap_or(text.len());
                        if start < end {
                            if let Ok(json) = serde_json::from_str::<Value>(&text[start + 1..end]) {
                                if let Some(data) = json["result"]["data"].as_array() {
                                    let items: Vec<Value> = data.iter().take(4).map(|item| {
                                        let mut entry = serde_json::Map::new();
                                        if let Some(time) = item["TIME"].as_str() {
                                            entry.insert("time".to_string(), Value::String(time.to_string()));
                                        }
                                        // 复制所有数值字段
                                        if let Some(obj) = item.as_object() {
                                            for (k, v) in obj {
                                                if k != "REPORT_DATE" && k != "TIME" {
                                                    if let Some(num) = v.as_f64() {
                                                        entry.insert(k.to_lowercase(), serde_json::json!(format!("{:.2}", num)));
                                                    } else if let Some(s) = v.as_str() {
                                                        entry.insert(k.to_lowercase(), Value::String(s.to_string()));
                                                    }
                                                }
                                            }
                                        }
                                        Value::Object(entry)
                                    }).collect();
                                    result.insert(ind.to_string(), Value::Array(items));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                result.insert(ind.to_string(), serde_json::json!({"error": format!("获取{}数据超时", ind.to_uppercase())}));
            }
        }
    }

    if result.is_empty() {
        return Ok(r#"{"error":"获取宏观经济数据失败"}"#.to_string());
    }

    Ok(serde_json::to_string(&Value::Object(result))?)
}

/// 获取全球主要股票指数
async fn get_global_indexes() -> Result<String> {
    let client = http::build_qq_finance_client()?;
    let url = "https://proxy.finance.qq.com/ifzqgtimg/appstock/app/rank/indexRankDetail2";

    let resp = client.get(url).send().await?;
    let json: Value = resp.json().await?;

    let data = &json["data"];
    let mut indexes = Vec::new();

    // API 返回结构: data.common/america/europe/asia 各为对象数组
    // 每个对象: { name, code, zxj(最新价), zdf(涨跌幅), location, state }
    let sections = ["common", "america", "europe", "asia"];
    for section in &sections {
        if let Some(arr) = data[*section].as_array() {
            let mut count = 0;
            for item in arr {
                let name = item["name"].as_str().unwrap_or("");
                let code = item["code"].as_str().unwrap_or("");
                let price = item["zxj"].as_str().unwrap_or("0");
                let change_pct = item["zdf"].as_str().unwrap_or("0");
                let location = item["location"].as_str().unwrap_or("");

                if !name.is_empty() {
                    indexes.push(serde_json::json!({
                        "name": name,
                        "code": code,
                        "price": price,
                        "change_pct": format!("{}%", change_pct),
                        "location": location,
                        "region": section,
                    }));
                    count += 1;
                }

                if count >= 4 { break; }
            }
        }
    }

    let result = serde_json::json!({
        "total": indexes.len(),
        "indexes": indexes,
    });
    Ok(serde_json::to_string(&result)?)
}

/// 获取财经日历
async fn get_financial_calendar() -> Result<String> {
    let client = http::build_cls_client()?;
    let url = "https://www.cls.cn/api/calendar/web/list?app=CailianpressWeb&flag=0&os=web&sv=8.4.6&type=0&sign=4b839750dc2f6b803d1c8ca00d2b40be";

    let resp = client.get(url).send().await?;
    let json: Value = resp.json().await?;

    let mut events = Vec::new();
    // API 返回结构: data[] 是按天分组的数组，每天有 calendar_day + items[]
    // 每个 item: { title, calendar_time, event: {title, country, star}, economic: {...}, ... }
    if let Some(days) = json["data"].as_array() {
        for day in days {
            let calendar_day = day["calendar_day"].as_str().unwrap_or("");
            if let Some(items) = day["items"].as_array() {
                for item in items {
                    if events.len() >= 15 { break; }

                    let title = item["title"].as_str().unwrap_or("");
                    let time = item["calendar_time"].as_str().unwrap_or(calendar_day);
                    // 重要性: 优先从 event.star / economic.star 取
                    let star = item["event"]["star"].as_i64()
                        .or_else(|| item["economic"]["star"].as_i64())
                        .unwrap_or(0);
                    // 国家信息
                    let country = item["event"]["country"].as_str()
                        .or_else(|| item["economic"]["country"].as_str())
                        .unwrap_or("");

                    if !title.is_empty() {
                        events.push(serde_json::json!({
                            "title": truncate_str(title, 100),
                            "date": time,
                            "importance": star,
                            "country": country,
                        }));
                    }
                }
            }
            if events.len() >= 15 { break; }
        }
    }

    let result = serde_json::json!({
        "total": events.len(),
        "events": events,
    });
    Ok(serde_json::to_string(&result)?)
}

/// 个股新闻搜索（复用 news_service::fetch_stock_news）
async fn search_stock_news_tool(keyword: &str) -> Result<String> {
    if keyword.is_empty() {
        return Ok(r#"{"error":"请提供搜索关键词"}"#.to_string());
    }

    match news_service::fetch_stock_news(keyword, 1, 8).await {
        Ok(items) => {
            let news: Vec<Value> = items.iter().take(8).map(|n| {
                serde_json::json!({
                    "title": n.title,
                    "summary": truncate_str(&n.summary, 100),
                    "source": n.source,
                    "time": n.publish_time,
                })
            }).collect();

            let result = serde_json::json!({
                "keyword": keyword,
                "total": news.len(),
                "news": news,
            });
            Ok(serde_json::to_string(&result)?)
        }
        Err(e) => {
            Ok(serde_json::json!({
                "keyword": keyword,
                "error": format!("搜索新闻失败: {}", e),
                "total": 0,
                "news": [],
            }).to_string())
        }
    }
}

/// 获取上市公司公告（复用 news_service::fetch_announcements）
async fn get_stock_notices(code: &str) -> Result<String> {
    if code.is_empty() {
        return Ok(r#"{"error":"请提供股票代码"}"#.to_string());
    }

    match news_service::fetch_announcements(Some(code), 1, 5).await {
        Ok(items) => {
            let notices: Vec<Value> = items.iter().take(5).map(|a| {
                serde_json::json!({
                    "title": a.title,
                    "date": a.notice_date,
                    "category": a.category,
                    "stock_name": a.stock_name,
                })
            }).collect();

            let result = serde_json::json!({
                "code": code,
                "total": notices.len(),
                "notices": notices,
            });
            Ok(serde_json::to_string(&result)?)
        }
        Err(e) => {
            Ok(serde_json::json!({
                "code": code,
                "error": format!("获取公告失败: {}", e),
                "total": 0,
                "notices": [],
            }).to_string())
        }
    }
}

/// 获取行业/个股研报摘要（复用 news_service::fetch_reports）
async fn get_industry_report(code: Option<&str>) -> Result<String> {
    match news_service::fetch_reports(code, 1, 8).await {
        Ok(items) => {
            let reports: Vec<Value> = items.iter().take(8).map(|r| {
                serde_json::json!({
                    "title": r.title,
                    "org": r.org_name,
                    "rating": r.rating,
                    "date": r.publish_date,
                    "industry": r.industry,
                    "stock": if r.stock_name.is_empty() { "-".to_string() } else { format!("{}({})", r.stock_name, r.stock_code) },
                })
            }).collect();

            let result = serde_json::json!({
                "total": reports.len(),
                "reports": reports,
            });
            Ok(serde_json::to_string(&result)?)
        }
        Err(e) => {
            Ok(serde_json::json!({
                "error": format!("获取研报失败: {}", e),
                "total": 0,
                "reports": [],
            }).to_string())
        }
    }
}

/// 选股工具名称中文映射
pub fn pick_tool_name_to_chinese(name: &str) -> &str {
    match name {
        "get_market_news" => "市场新闻",
        "get_economic_data" => "宏观经济",
        "get_global_indexes" => "全球指数",
        "get_financial_calendar" => "财经日历",
        "search_stocks_by_condition" => "NLP智能选股",
        "search_concept_boards" => "NLP板块搜索",
        "batch_get_stock_quotes" => "批量行情",
        "get_stock_quote" => "实时行情",
        "get_fund_flow" => "资金流向",
        "get_kline_data" => "K线数据",
        "get_technical_indicators" => "技术指标",
        "search_stock_news" => "个股新闻",
        "get_stock_notices" => "公司公告",
        "get_industry_report" => "研报摘要",
        _ => name,
    }
}

/// 对工具返回结果生成人类可读的摘要（用于前端展示）
pub fn summarize_tool_result(tool_name: &str, result: &str) -> String {
    let json: serde_json::Value = serde_json::from_str(result).unwrap_or(serde_json::Value::Null);

    match tool_name {
        "get_market_news" => {
            let total = json["total"].as_u64().unwrap_or(0);
            if total == 0 {
                return "未获取到新闻数据".to_string();
            }
            let mut lines = vec![format!("获取到 {} 条新闻", total)];
            if let Some(news) = json["news"].as_array() {
                for (i, n) in news.iter().take(8).enumerate() {
                    let source = n["source"].as_str().unwrap_or("");
                    let title = n["title"].as_str().unwrap_or("");
                    lines.push(format!("{}. [{}] {}", i + 1, source, title));
                }
                if news.len() > 8 {
                    lines.push(format!("... 等共 {} 条", news.len()));
                }
            }
            lines.join("\n")
        }
        "search_stocks_by_condition" => {
            let keyword = json["keyword"].as_str().unwrap_or("");
            let total = json["total_count"].as_u64().unwrap_or(0);
            let returned = json["returned"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("选股条件「{}」匹配到 {} 只股票（返回 {} 只）", keyword, total, returned)];
            if let Some(stocks) = json["stocks"].as_array() {
                for (i, s) in stocks.iter().take(10).enumerate() {
                    let name = s["name"].as_str().unwrap_or("");
                    let code = s["code"].as_str().unwrap_or("");
                    lines.push(format!("{}. {}({})", i + 1, name, code));
                }
                if stocks.len() > 10 {
                    lines.push(format!("... 等共 {} 只", stocks.len()));
                }
            }
            if let Some(err) = json["error"].as_str() {
                lines.push(format!("错误: {}", err));
            }
            lines.join("\n")
        }
        "search_concept_boards" => {
            let keyword = json["keyword"].as_str().unwrap_or("");
            let total = json["total_count"].as_u64().unwrap_or(0);
            let returned = json["returned"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("板块查询「{}」匹配到 {} 个板块（返回 {} 个）", keyword, total, returned)];
            if let Some(boards) = json["boards"].as_array() {
                for (i, b) in boards.iter().take(10).enumerate() {
                    let name = b["name"].as_str().unwrap_or("");
                    let code = b["code"].as_str().unwrap_or("");
                    lines.push(format!("{}. {}({})", i + 1, name, code));
                }
                if boards.len() > 10 {
                    lines.push(format!("... 等共 {} 个", boards.len()));
                }
            }
            if let Some(err) = json["error"].as_str() {
                lines.push(format!("错误: {}", err));
            }
            lines.join("\n")
        }
        "get_economic_data" => {
            let mut lines = Vec::new();
            for key in &["gdp", "cpi", "ppi", "pmi"] {
                if let Some(arr) = json[*key].as_array() {
                    if let Some(first) = arr.first() {
                        let time = first["time"].as_str().unwrap_or("");
                        lines.push(format!("{}: 最新 {} 期", key.to_uppercase(), time));
                    }
                }
            }
            if lines.is_empty() { "宏观经济数据".to_string() } else { lines.join(" | ") }
        }
        "get_global_indexes" => {
            let total = json["total"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("获取到 {} 个全球指数", total)];
            if let Some(indexes) = json["indexes"].as_array() {
                for idx in indexes.iter().take(6) {
                    let name = idx["name"].as_str().unwrap_or("");
                    let chg = idx["change_pct"].as_str().unwrap_or("0%");
                    lines.push(format!("· {} {}", name, chg));
                }
            }
            lines.join("\n")
        }
        "get_financial_calendar" => {
            let total = json["total"].as_u64().unwrap_or(0);
            format!("获取到 {} 条财经日历事件", total)
        }
        "search_stock_news" => {
            let keyword = json["keyword"].as_str().unwrap_or("");
            let total = json["total"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("「{}」相关新闻 {} 条", keyword, total)];
            if let Some(news) = json["news"].as_array() {
                for (i, n) in news.iter().take(5).enumerate() {
                    let title = n["title"].as_str().unwrap_or("");
                    lines.push(format!("{}. {}", i + 1, title));
                }
            }
            lines.join("\n")
        }
        "get_stock_notices" => {
            let code = json["code"].as_str().unwrap_or("");
            let total = json["total"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("{} 最新公告 {} 条", code, total)];
            if let Some(notices) = json["notices"].as_array() {
                for (i, n) in notices.iter().take(5).enumerate() {
                    let title = n["title"].as_str().unwrap_or("");
                    lines.push(format!("{}. {}", i + 1, title));
                }
            }
            lines.join("\n")
        }
        "get_industry_report" => {
            let total = json["total"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("获取到 {} 条研报", total)];
            if let Some(reports) = json["reports"].as_array() {
                for (i, r) in reports.iter().take(5).enumerate() {
                    let title = r["title"].as_str().unwrap_or("");
                    let org = r["org"].as_str().unwrap_or("");
                    lines.push(format!("{}. [{}] {}", i + 1, org, title));
                }
            }
            lines.join("\n")
        }
        "batch_get_stock_quotes" => {
            let total = json["total"].as_u64().unwrap_or(0);
            let mut lines = vec![format!("获取到 {} 只股票行情", total)];
            if let Some(stocks) = json["stocks"].as_array() {
                for s in stocks.iter().take(20) {
                    let name = s["name"].as_str().unwrap_or("");
                    let code = s["code"].as_str().unwrap_or("");
                    let pct = s["change_pct"].as_str().unwrap_or("0%");
                    let pe = s["pe_ttm"].as_str().unwrap_or("N/A");
                    let roe = s["roe"].as_str().unwrap_or("N/A");
                    let cap = s["total_market_cap"].as_str().unwrap_or("");
                    lines.push(format!("· {}({}) {} PE:{} ROE:{} 市值:{}", name, code, pct, pe, roe, cap));
                }
            }
            lines.join("\n")
        }
        "get_stock_quote" => {
            let name = json["name"].as_str().unwrap_or("");
            let code = json["code"].as_str().unwrap_or("");
            let price = json["price"].as_f64().map(|v| format!("{:.2}", v)).unwrap_or_default();
            let pct = json["change_pct"].as_str().unwrap_or("");
            let pe = json["pe_ttm"].as_str().unwrap_or("N/A");
            let roe = json["roe"].as_str().unwrap_or("N/A");
            let cap = json["total_market_cap"].as_str().unwrap_or("");
            let main = json["main_net_inflow"].as_str().unwrap_or("");
            format!("{}({}) ¥{} {} PE:{} ROE:{} 市值:{} 主力净流入:{}", name, code, price, pct, pe, roe, cap, main)
        }
        "get_fund_flow" => {
            let code = json["code"].as_str().unwrap_or("");
            let inflow = json["main_net_inflow"].as_str().unwrap_or("0");
            let pct = json["main_net_pct"].as_str().unwrap_or("0%");
            format!("{} 主力净流入:{} 占比:{}", code, inflow, pct)
        }
        "get_kline_data" => {
            let code = json["code"].as_str().unwrap_or("");
            let count = json["count"].as_u64().unwrap_or(0);
            let period = json["period"].as_str().unwrap_or("day");
            format!("{} {}K线 {} 根", code, if period == "week" { "周" } else { "日" }, count)
        }
        "get_technical_indicators" => {
            let code = json["code"].as_str().unwrap_or("");
            let ma = json["ma_alignment"].as_str().unwrap_or("");
            let vp = json["volume_price_relation"].as_str().unwrap_or("");
            let summary = json["summary"].as_str().unwrap_or("");
            format!("{} 均线:{} 量价:{}\n{}", code, ma, vp, summary)
        }
        _ => {
            format!("工具 {} 返回 {} 字节数据", tool_name, result.len())
        }
    }
}

/// 安全截断 UTF-8 字符串（按字符数而非字节数）
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

fn format_amount(v: f64) -> String {
    let abs = v.abs();
    if abs >= 1e8 {
        format!("{:.2}亿", v / 1e8)
    } else if abs >= 1e4 {
        format!("{:.0}万", v / 1e4)
    } else {
        format!("{:.0}", v)
    }
}
