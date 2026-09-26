//! Provider-neutral LLM contracts, model routing, and built-in HTTP adapters.

mod anthropic;
mod normalize;
mod openai_compatible;
mod registry;
mod stream;
mod types;

pub use anthropic::AnthropicProvider;
pub use openai_compatible::OpenAiCompatibleProvider;
pub use registry::{ModelProviderRegistry, ModelRoute};
pub use suncode_common::BusinessError;
pub use types::{
    ApiKeyResolver, ClientToolsetDefinition, Completion, CompletionFuture, CompletionRequest,
    ContentPart, LlmProvider, Message, ModelCapabilities, ModelDescriptor, ModelLimits, ToolCall,
    ToolDefinition, TransferProgressDelta, Usage,
};
