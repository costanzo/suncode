use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub use suncode_common::tool::ToolDefinition;
pub use suncode_types::{ContentPart, Message, ToolCall, Usage};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientToolsetDefinition {
    pub type_name: String,
    pub toolset_name: String,
    #[serde(default)]
    pub configuration: Value,
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
    pub computer_use: bool,
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
    pub default_api_base: String,
    pub capabilities: ModelCapabilities,
    pub reasoning_efforts: Vec<String>,
    pub limits: ModelLimits,
    pub availability: String,
}

pub struct CompletionRequest<'a> {
    pub messages: &'a [Message],
    pub wire_model: &'a str,
    pub tools: &'a [ToolDefinition],
    pub client_toolsets: &'a [ClientToolsetDefinition],
    pub reasoning_effort: Option<&'a str>,
    pub max_output_tokens: Option<u64>,
    pub transfer_progress: mpsc::UnboundedSender<TransferProgressDelta>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransferProgressDelta {
    pub uploaded_bytes: u64,
    pub downloaded_bytes: u64,
}

pub type CompletionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Completion, suncode_common::BusinessError>> + Send + 'a>>;

/// Resolves credentials without coupling providers to persistence.
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
