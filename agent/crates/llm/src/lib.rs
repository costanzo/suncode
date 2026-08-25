//! Provider-neutral LLM contracts, model routing, and built-in HTTP adapters.

mod normalize;
mod openai_compatible;
mod registry;
mod stream;
mod types;

pub use openai_compatible::OpenAiCompatibleProvider;
pub use registry::{ModelProviderRegistry, ModelRoute};
pub use suncode_common::BusinessError;
pub use types::{
    ApiKeyResolver, Completion, CompletionFuture, CompletionRequest, ContentPart, LlmProvider,
    Message, ModelCapabilities, ModelDescriptor, ModelLimits, ToolCall, ToolDefinition, Usage,
};
