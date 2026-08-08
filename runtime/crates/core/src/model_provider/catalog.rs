use serde::Serialize;

pub const DEEPSEEK_MODEL_ID: &str = "deepseek-v4-flash";

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
    pub id: &'static str,
    pub capabilities: ModelCapabilities,
    pub limits: ModelLimits,
    pub availability: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelLimits {
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
}

pub fn deepseek_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "deepseek",
        id: DEEPSEEK_MODEL_ID,
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
