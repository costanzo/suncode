//! Canonical model-facing types and the provider streaming contract.

use crate::domain::{Message, ToolCall, Usage};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ProviderError {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: String,
    pub usage: Option<Usage>,
}

pub type CompletionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Completion, ProviderError>> + Send + 'a>>;

/// Provider-neutral completion interface consumed by the agent loop.
pub trait LlmProvider: Send + Sync {
    fn complete<'a>(
        &'a self,
        messages: &'a [Message],
        wire_model: &'a str,
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
        messages: &'a [Message],
        wire_model: &'a str,
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        (**self).complete(messages, wire_model, cancellation, deltas)
    }
}
