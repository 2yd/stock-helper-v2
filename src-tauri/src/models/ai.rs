use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model_name: String,
    pub max_tokens: u32,
    /// 分析/默认 temperature（精确场景，建议 0.1~0.3）
    pub temperature: f64,
    /// 选股 temperature（发散场景，建议 0.5~0.8）
    #[serde(default = "default_pick_temperature")]
    pub pick_temperature: f64,
    pub timeout_secs: u64,
    pub enabled: bool,
}

fn default_pick_temperature() -> f64 {
    0.7
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "默认模型".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model_name: "gpt-4o-mini".to_string(),
            max_tokens: 2048,
            temperature: 0.3,
            pick_temperature: 0.7,
            timeout_secs: 300,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInstruction {
    pub action: InstructionAction,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstructionAction {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "watch")]
    Watch,
    #[serde(rename = "eliminate")]
    Eliminate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    pub id: String,
    pub code: String,
    pub name: String,
    pub model_name: String,
    pub question: String,
    pub content: String,
    pub created_at: String,
}

// ========== Chat Completion 数据结构（支持 Function Calling）==========

/// Chat message with optional tool_calls and tool_call_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn assistant_text(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            name: None,
        }
    }

    /// 构建同时携带思考内容和工具调用的 assistant 消息
    /// 用于 Agent 模式下透传 AI 的思考过程
    pub fn assistant_tool_calls_with_content(content: Option<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            name: None,
        }
    }

    pub fn tool_result(tool_call_id: &str, name: &str, content: &str) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
            name: Some(name.to_string()),
        }
    }
}

/// Tool call from assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Chat completion request with tools support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: Option<ChatChoiceMessage>,
    pub delta: Option<ChatDelta>,
    pub finish_reason: Option<String>,
}

/// Message in non-streaming response (may contain tool_calls)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoiceMessage {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

/// Streaming tool call delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaToolCall {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<DeltaFunctionCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstructionRequest {
    pub stocks: Vec<StockSummaryForAI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSummaryForAI {
    pub code: String,
    pub name: String,
    pub open_pct: f64,
    pub current_pct: f64,
    pub score: u32,
    pub bid_amount: f64,
    pub streak_days: u32,
    pub turnover: f64,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstructionResponse {
    pub instructions: Vec<StockInstructionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInstructionResult {
    pub code: String,
    pub action: String,
    pub label: String,
    pub reason: String,
}

/// 前端流式事件（支持工具调用状态通知）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStreamEvent {
    pub event_type: String,  // "content" | "tool_call" | "tool_result" | "done" | "error" | "thinking"
    pub content: Option<String>,
    pub done: bool,
    pub usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_tool_calls_with_content() {
        let tc = ToolCall {
            id: "call_1".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "get_stock_quote".to_string(),
                arguments: r#"{"code":"sh600519"}"#.to_string(),
            },
        };
        let msg = ChatMessage::assistant_tool_calls_with_content(
            Some("我来查看茅台行情".to_string()),
            vec![tc.clone()],
        );
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, Some("我来查看茅台行情".to_string()));
        assert!(msg.tool_calls.is_some());
        assert_eq!(msg.tool_calls.as_ref().unwrap().len(), 1);

        // 验证序列化包含 content 和 tool_calls
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"content\""));
        assert!(json.contains("我来查看茅台行情"));
        assert!(json.contains("get_stock_quote"));
    }

    #[test]
    fn test_assistant_tool_calls_with_content_none() {
        let tc = ToolCall {
            id: "call_2".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "get_market_news".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let msg = ChatMessage::assistant_tool_calls_with_content(None, vec![tc]);
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, None);
        assert!(msg.tool_calls.is_some());

        // content 为 None 时 JSON 中不应包含 content 字段（skip_serializing_if）
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("\"content\""));
    }

    #[test]
    fn test_assistant_tool_calls_backward_compat() {
        let tc = ToolCall {
            id: "call_3".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test".to_string(),
                arguments: "{}".to_string(),
            },
        };
        // 旧方法保持 content = None
        let msg = ChatMessage::assistant_tool_calls(vec![tc]);
        assert_eq!(msg.content, None);
        assert!(msg.tool_calls.is_some());
    }
}
