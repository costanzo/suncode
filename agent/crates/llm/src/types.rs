use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentPart>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: vec![ContentPart {
                kind: "text".into(),
                text: text.into(),
            }],
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter(|part| part.kind == "text")
            .map(|part| part.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cache_read_tokens: Option<u64>,
    pub cache_miss_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: String,
    pub usage: Option<Usage>,
    pub provider_request_id: Option<String>,
    pub provider_response_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub tool_use: bool,
    pub vision: bool,
    pub structured_output: bool,
    pub cancellation: bool,
    pub reasoning_effort: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ModelLimits {
    pub max_input_tokens: Option<u64>,
    pub auto_compact_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModelDescriptor {
    pub provider: String,
    pub provider_label: String,
    pub id: String,
    pub wire_model: String,
    pub api_base: String,
    pub capabilities: ModelCapabilities,
    pub limits: ModelLimits,
    pub availability: String,
}

pub struct CompletionRequest<'a> {
    pub messages: &'a [Message],
    pub wire_model: &'a str,
    pub tools: &'a [ToolDefinition],
    pub reasoning_effort: Option<&'a str>,
}

pub type CompletionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Completion, suncode_common::BusinessError>> + Send + 'a>>;

/// Resolves credentials without coupling providers to persistence or environment handling.
pub trait ApiKeyResolver: Send + Sync {
    fn api_key(&self, provider_id: &str) -> Option<String>;
}

/// Provider-neutral completion interface consumed by an agent loop.
pub trait LlmProvider: Send + Sync {
    fn complete<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a>;
}

impl<T> LlmProvider for Arc<T>
where
    T: LlmProvider + ?Sized,
{
    fn complete<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        (**self).complete(request, cancellation, deltas)
    }
}
