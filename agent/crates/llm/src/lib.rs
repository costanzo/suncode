//! Provider-neutral LLM contracts, model routing, and built-in HTTP adapters.

mod anthropic;
mod errors;
mod http;
mod normalize;
mod openai_compatible;
mod registry;
mod stream;
mod types;

/// Upper bound for establishing a provider connection.
pub(crate) const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// Maximum silence between response bytes. Generous because non-streaming
/// responses arrive only after the model finishes generating.
pub(crate) const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

pub use anthropic::AnthropicProvider;
pub use openai_compatible::OpenAiCompatibleProvider;
pub use registry::{ModelProviderRegistry, ModelRoute};
pub use suncode_common::BusinessError;
pub use types::{
    ApiKeyResolver, ClientToolsetDefinition, Completion, CompletionFuture, CompletionRequest,
    ContentPart, LlmProvider, Message, ModelCapabilities, ModelDescriptor, ModelLimits, ToolCall,
    ToolDefinition, TransferProgressDelta, Usage,
};
