use crate::credentials::ProviderKind;
use serde::Serialize;

pub const DEEPSEEK_MODEL_ID: &str = "deepseek-v4-flash";
pub const ZHIPU_MODEL_ID: &str = "glm-5.2";
pub const OPENAI_MODEL_ID: &str = "gpt-5.6-sol";

#[derive(Debug, Clone, Serialize)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub tool_use: bool,
    pub vision: bool,
    pub structured_output: bool,
    pub cancellation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDescriptor {
    pub provider: &'static str,
    pub provider_label: &'static str,
    pub id: &'static str,
    pub wire_model: &'static str,
    pub api_base: &'static str,
    pub capabilities: ModelCapabilities,
    pub limits: ModelLimits,
    pub availability: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ModelLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

pub fn all_models() -> Vec<ModelDescriptor> {
    vec![deepseek_model(), zhipu_model(), openai_model()]
}

pub fn deepseek_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "deepseek",
        provider_label: ProviderKind::DeepSeek.label(),
        id: DEEPSEEK_MODEL_ID,
        wire_model: "deepseek-v4-flash",
        api_base: "https://api.deepseek.com",
        capabilities: ModelCapabilities {
            streaming: true,
            tool_use: true,
            vision: false,
            structured_output: false,
            cancellation: true,
        },
        limits: ModelLimits {
            max_input_tokens: None,
            max_output_tokens: None,
        },
        availability: "configured",
    }
}

pub fn zhipu_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "zhipu",
        provider_label: ProviderKind::Zhipu.label(),
        id: ZHIPU_MODEL_ID,
        wire_model: "glm-5.2",
        api_base: "https://open.bigmodel.cn/api/paas/v4",
        capabilities: ModelCapabilities {
            streaming: true,
            tool_use: true,
            vision: false,
            structured_output: false,
            cancellation: true,
        },
        limits: ModelLimits {
            max_input_tokens: Some(1_000_000),
            max_output_tokens: Some(128_000),
        },
        availability: "configured",
    }
}

pub fn openai_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "openai",
        provider_label: ProviderKind::OpenAI.label(),
        id: OPENAI_MODEL_ID,
        wire_model: "gpt-5.6-sol",
        api_base: "https://api.openai.com/v1",
        capabilities: ModelCapabilities {
            streaming: true,
            tool_use: true,
            vision: false,
            structured_output: false,
            cancellation: true,
        },
        limits: ModelLimits {
            max_input_tokens: Some(1_048_576),
            max_output_tokens: Some(128_000),
        },
        availability: "configured",
    }
}
