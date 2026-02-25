//! 东方财富 NLP 选股 API 集成测试
//!
//! 运行方式（需要提供有效的 qgqp_b_id）：
//!   QGQP_B_ID="你的东财cookie值" cargo test --test test_nlp_tools -- --nocapture
//!
//! 如果没有设置 QGQP_B_ID 环境变量，NLP选股/板块搜索测试会跳过。

use app_lib::services::smart_stock::SmartStockService;
use app_lib::services::stock_tools;

fn get_qgqp_b_id() -> Option<String> {
    std::env::var("QGQP_B_ID").ok().filter(|s| !s.is_empty())
}

// ==================== 工具定义验证测试 ====================

#[test]
fn test_pick_tool_definitions_valid_json() {
    let defs = stock_tools::get_pick_tool_definitions();
    assert!(!defs.is_empty(), "工具定义不应为空");
    for def in &defs {
        assert!(def["type"].as_str() == Some("function"), "每个工具应为function类型");
        let func = &def["function"];
        assert!(func["name"].as_str().is_some(), "每个工具应有name");
        assert!(func["description"].as_str().is_some(), "每个工具应有description");
        assert!(func["parameters"]["type"].as_str() == Some("object"), "参数应为object类型");
    }
    println!("共 {} 个工具定义，全部合法", defs.len());
}

#[test]
fn test_pick_tool_definitions_no_hot_strategies() {
    let defs = stock_tools::get_pick_tool_definitions();
    for def in &defs {
        let name = def["function"]["name"].as_str().unwrap_or("");
        assert_ne!(name, "get_hot_strategies", "get_hot_strategies 应已从选股工具中移除");
    }
}

#[test]
fn test_pick_tool_definitions_has_new_tools() {
    let defs = stock_tools::get_pick_tool_definitions();
    let names: Vec<&str> = defs.iter()
        .filter_map(|d| d["function"]["name"].as_str())
        .collect();

    let required_tools = [
        "get_market_news",
        "get_economic_data",
        "get_global_indexes",
        "get_financial_calendar",
        "get_kline_data",
        "get_technical_indicators",
        "search_stocks_by_condition",
        "search_concept_boards",
        "batch_get_stock_quotes",
        "get_stock_quote",
        "get_fund_flow",
        "search_stock_news",
        "get_stock_notices",
        "get_industry_report",
    ];

    for tool in &required_tools {
        assert!(names.contains(tool), "工具 {} 应在定义中", tool);
    }
    println!("全部 {} 个必需工具都已存在", required_tools.len());
}

#[test]
fn test_tool_name_chinese_mapping_complete() {
    let defs = stock_tools::get_pick_tool_definitions();
    for def in &defs {
        let name = def["function"]["name"].as_str().unwrap_or("");
        let chinese = stock_tools::pick_tool_name_to_chinese(name);
        assert_ne!(chinese, name, "工具 {} 应有中文映射，但返回了原名", name);
        println!("  {} -> {}", name, chinese);
    }
}

// ==================== summarize_tool_result 测试 ====================

#[test]
fn test_summarize_all_tools_have_handler() {
    let tool_names = [
        "get_market_news", "get_economic_data", "get_global_indexes",
        "get_financial_calendar", "search_stocks_by_condition", "search_concept_boards",
        "batch_get_stock_quotes", "get_stock_quote", "get_fund_flow",
        "get_kline_data", "get_technical_indicators", "search_stock_news",
        "get_stock_notices", "get_industry_report",
    ];

    for name in &tool_names {
        let result = stock_tools::summarize_tool_result(name, "{}");
        // 不应该走到 default 分支返回 "工具 xxx 返回 2 字节数据"
        assert!(
            !result.contains("字节数据"),
            "工具 {} 应有专门的摘要处理逻辑，但走了default分支: {}",
            name, result
        );
    }
}

#[test]
fn test_summarize_economic_data() {
    let data = serde_json::json!({
        "gdp": [{"time": "2025年前三季度", "sum_same": "5.20"}],
        "cpi": [{"time": "2025年09月", "national_same": "0.40"}],
    });
    let summary = stock_tools::summarize_tool_result("get_economic_data", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("GDP"));
    assert!(summary.contains("CPI"));
}

#[test]
fn test_summarize_global_indexes() {
    let data = serde_json::json!({
        "total": 2,
        "indexes": [
            { "name": "道琼斯", "change_pct": "0.52%" },
            { "name": "纳斯达克", "change_pct": "-0.31%" },
        ]
    });
    let summary = stock_tools::summarize_tool_result("get_global_indexes", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("2 个全球指数"));
    assert!(summary.contains("道琼斯"));
}

#[test]
fn test_summarize_stock_news() {
    let data = serde_json::json!({
        "keyword": "贵州茅台",
        "total": 3,
        "news": [
            { "title": "茅台三季报超预期" },
            { "title": "茅台推出新品系列" },
        ]
    });
    let summary = stock_tools::summarize_tool_result("search_stock_news", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("贵州茅台"));
    assert!(summary.contains("3 条"));
}

#[test]
fn test_summarize_stock_notices() {
    let data = serde_json::json!({
        "code": "sh600519",
        "total": 2,
        "notices": [
            { "title": "2025年第三季度报告" },
            { "title": "关于回购公司股份进展" },
        ]
    });
    let summary = stock_tools::summarize_tool_result("get_stock_notices", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("sh600519"));
    assert!(summary.contains("2 条"));
}

#[test]
fn test_summarize_industry_report() {
    let data = serde_json::json!({
        "total": 2,
        "reports": [
            { "title": "AI行业深度报告", "org": "中信证券" },
            { "title": "半导体投资展望", "org": "华泰证券" },
        ]
    });
    let summary = stock_tools::summarize_tool_result("get_industry_report", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("2 条研报"));
    assert!(summary.contains("中信证券"));
}

#[test]
fn test_summarize_search_stocks() {
    let data = serde_json::json!({
        "keyword": "人工智能",
        "total_count": 50,
        "returned": 2,
        "stocks": [
            { "code": "300001", "name": "特锐德" },
            { "code": "300002", "name": "神州泰岳" },
        ]
    });
    let summary = stock_tools::summarize_tool_result("search_stocks_by_condition", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("人工智能"));
    assert!(summary.contains("50 只"));
    assert!(summary.contains("特锐德"));
}

#[test]
fn test_summarize_search_boards() {
    let data = serde_json::json!({
        "keyword": "AI概念",
        "total_count": 5,
        "returned": 2,
        "boards": [
            { "code": "BK0800", "name": "AI芯片" },
            { "code": "BK0801", "name": "大模型" },
        ]
    });
    let summary = stock_tools::summarize_tool_result("search_concept_boards", &data.to_string());
    println!("summary: {}", summary);
    assert!(summary.contains("AI概念"));
    assert!(summary.contains("AI芯片"));
}

#[test]
fn test_summarize_error_case() {
    let data = serde_json::json!({
        "error": "选股条件解析失败",
        "keyword": "无效条件"
    });
    let summary = stock_tools::summarize_tool_result("search_stocks_by_condition", &data.to_string());
    println!("error summary: {}", summary);
    assert!(summary.contains("错误"));
}

// ==================== execute_pick_tool 路由测试 ====================

#[tokio::test]
async fn test_execute_pick_tool_unknown_returns_msg() {
    let result = stock_tools::execute_pick_tool("nonexistent_tool", "{}", "").await
        .expect("未知工具应返回Ok");
    assert!(result.contains("未知工具"), "应返回未知工具消息: {}", result);
}

#[tokio::test]
async fn test_execute_pick_tool_hot_strategies_is_unknown() {
    // get_hot_strategies 已从 pick tools 移除，调用应返回"未知工具"
    let result = stock_tools::execute_pick_tool("get_hot_strategies", r#"{"count":10}"#, "").await
        .expect("应返回Ok");
    assert!(result.contains("未知工具"), "get_hot_strategies 应已从pick tools移除: {}", result);
}

// ==================== 全球指数 / 财经日历 集成测试 ====================

#[tokio::test]
async fn test_get_global_indexes_real_api() {
    let result = stock_tools::execute_pick_tool("get_global_indexes", "{}", "").await
        .expect("全球指数API不应返回Err");
    let json: serde_json::Value = serde_json::from_str(&result).expect("应返回有效JSON");
    let total = json["total"].as_u64().unwrap_or(0);
    println!("全球指数: total={}, result={}", total, &result[..result.len().min(500)]);
    assert!(total > 0, "应获取到至少1个全球指数，实际total={}", total);
    // 验证字段完整性
    if let Some(indexes) = json["indexes"].as_array() {
        let first = &indexes[0];
        assert!(first["name"].as_str().is_some(), "应有name字段");
        assert!(first["price"].as_str().is_some(), "应有price字段");
        assert!(first["change_pct"].as_str().is_some(), "应有change_pct字段");
    }
}

#[tokio::test]
async fn test_get_financial_calendar_real_api() {
    let result = stock_tools::execute_pick_tool("get_financial_calendar", "{}", "").await
        .expect("财经日历API不应返回Err");
    let json: serde_json::Value = serde_json::from_str(&result).expect("应返回有效JSON");
    let total = json["total"].as_u64().unwrap_or(0);
    println!("财经日历: total={}, result={}", total, result.chars().take(500).collect::<String>());
    assert!(total > 0, "应获取到至少1条财经日历事件，实际total={}", total);
    // 验证字段完整性
    if let Some(events) = json["events"].as_array() {
        let first = &events[0];
        assert!(first["title"].as_str().is_some(), "应有title字段");
        assert!(first["date"].as_str().is_some(), "应有date字段");
    }
}

// ==================== NLP 选股测试（需要环境变量） ====================

#[tokio::test]
async fn test_search_stock_basic() {
    let qgqp_b_id = match get_qgqp_b_id() {
        Some(id) => id,
        None => {
            println!("SKIP: 未设置 QGQP_B_ID 环境变量，跳过 NLP 选股测试");
            return;
        }
    };

    let keyword = "人工智能";
    let result = SmartStockService::search_stock(keyword, 10, &qgqp_b_id).await;
    match result {
        Ok(resp) => {
            println!("=== NLP选股结果 ===");
            println!("code: {}", resp.code);
            assert_eq!(resp.code, 100, "选股API应返回code=100, 实际={}", resp.code);
            if let Some(data) = &resp.data {
                println!("匹配到 {} 只股票", data.result.data_list.len());
                assert!(!data.result.data_list.is_empty(), "关键词「{}」应该有匹配的股票", keyword);
            } else {
                panic!("选股API返回code=100但data为空");
            }
        }
        Err(e) => panic!("NLP选股API调用失败: {}", e),
    }
}

#[tokio::test]
async fn test_search_stock_empty_qgqp_b_id_returns_error() {
    let result = SmartStockService::search_stock("人工智能", 10, "").await;
    assert!(result.is_err(), "空的 qgqp_b_id 应该返回错误");
}

#[tokio::test]
async fn test_search_board_basic() {
    let qgqp_b_id = match get_qgqp_b_id() {
        Some(id) => id,
        None => {
            println!("SKIP: 未设置 QGQP_B_ID 环境变量，跳过板块搜索测试");
            return;
        }
    };

    let keyword = "人工智能";
    let result = SmartStockService::search_board(keyword, 10, &qgqp_b_id).await;
    match result {
        Ok(resp) => {
            assert_eq!(resp.code, 100, "板块搜索API应返回code=100");
            if let Some(data) = &resp.data {
                assert!(!data.result.data_list.is_empty(), "关键词「{}」应该有匹配的板块", keyword);
            }
        }
        Err(e) => panic!("板块搜索API调用失败: {}", e),
    }
}

#[tokio::test]
async fn test_execute_pick_tool_search_stocks() {
    let qgqp_b_id = match get_qgqp_b_id() {
        Some(id) => id,
        None => { println!("SKIP"); return; }
    };

    let result = stock_tools::execute_pick_tool(
        "search_stocks_by_condition",
        r#"{"keyword": "人工智能", "page_size": 5}"#,
        &qgqp_b_id,
    ).await.expect("不应返回Err");

    let json: serde_json::Value = serde_json::from_str(&result).expect("应返回有效JSON");
    if let Some(err) = json["error"].as_str() { panic!("NLP选股错误: {}", err); }
    let returned = json["returned"].as_u64().unwrap_or(0);
    assert!(returned > 0, "应匹配到股票");
}

#[tokio::test]
async fn test_execute_pick_tool_empty_qgqp_returns_error_json() {
    let result = stock_tools::execute_pick_tool(
        "search_stocks_by_condition",
        r#"{"keyword": "人工智能"}"#,
        "",
    ).await.expect("应返回Ok(错误JSON)");

    let json: serde_json::Value = serde_json::from_str(&result).expect("应返回有效JSON");
    assert!(json["error"].as_str().is_some(), "应包含error字段");
}
