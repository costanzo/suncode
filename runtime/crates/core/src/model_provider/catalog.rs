use crate::credentials::ProviderKind;
use serde::Serialize;

pub const DEEPSEEK_MODEL_ID: &str = "deepseek-v4-flash";
pub const DEEPSEEK_PRO_MODEL_ID: &str = "deepseek-v4-pro";
pub const ZHIPU_MODEL_ID: &str = "glm-5.2";
pub const ZHIPU_LATEST_MODEL_ID: &str = "glm-5.3";
pub const OPENAI_MODEL_ID: &str = "gpt-5.6-sol";
pub const OPENAI_FAST_MODEL_ID: &str = "gpt-5.5";
pub const KIMI_MODEL_ID: &str = "kimi-k2.7-code";
pub const KIMI_FLAGSHIP_MODEL_ID: &str = "kimi-k3";
pub const CLAUDE_MODEL_ID: &str = "claude-opus-5";
pub const CLAUDE_SONNET_MODEL_ID: &str = "claude-sonnet-5";
pub const GEMINI_MODEL_ID: &str = "gemini-3.6-flash";
pub const GEMINI_PRO_MODEL_ID: &str = "gemini-3.5";

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
    vec![
        deepseek_model(),
        deepseek_pro_model(),
        zhipu_model(),
        zhipu_latest_model(),
        openai_model(),
        openai_fast_model(),
        kimi_model(),
        kimi_flagship_model(),
        claude_model(),
        claude_sonnet_model(),
        gemini_model(),
        gemini_pro_model(),
    ]
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

pub fn deepseek_pro_model() -> ModelDescriptor {
    let mut model = deepseek_model();
    model.id = DEEPSEEK_PRO_MODEL_ID;
    model.wire_model = DEEPSEEK_PRO_MODEL_ID;
    model
}

pub fn zhipu_latest_model() -> ModelDescriptor {
    let mut model = zhipu_model();
    model.id = ZHIPU_LATEST_MODEL_ID;
    model.wire_model = ZHIPU_LATEST_MODEL_ID;
    model
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

pub fn openai_fast_model() -> ModelDescriptor {
    let mut model = openai_model();
    model.id = OPENAI_FAST_MODEL_ID;
    model.wire_model = OPENAI_FAST_MODEL_ID;
    model
}

pub fn kimi_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "kimi",
        provider_label: ProviderKind::Kimi.label(),
        id: KIMI_MODEL_ID,
        wire_model: "kimi-k2.7-code",
        api_base: "https://api.moonshot.ai/v1",
        capabilities: ModelCapabilities {
            streaming: true,
            tool_use: true,
            vision: false,
            structured_output: false,
            cancellation: true,
        },
        limits: ModelLimits {
            max_input_tokens: Some(262_144),
            max_output_tokens: None,
        },
        availability: "configured",
    }
}

pub fn kimi_flagship_model() -> ModelDescriptor {
    let mut model = kimi_model();
    model.id = KIMI_FLAGSHIP_MODEL_ID;
    model.wire_model = KIMI_FLAGSHIP_MODEL_ID;
    model
}

pub fn claude_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "claude",
        provider_label: ProviderKind::Claude.label(),
        id: CLAUDE_MODEL_ID,
        wire_model: "claude-opus-5",
        api_base: "https://api.anthropic.com/v1",
        capabilities: ModelCapabilities {
            streaming: true,
            tool_use: true,
            vision: false,
            structured_output: false,
            cancellation: true,
        },
        limits: ModelLimits {
            max_input_tokens: Some(1_000_000),
            max_output_tokens: None,
        },
        availability: "configured",
    }
}

pub fn claude_sonnet_model() -> ModelDescriptor {
    let mut model = claude_model();
    model.id = CLAUDE_SONNET_MODEL_ID;
    model.wire_model = CLAUDE_SONNET_MODEL_ID;
    model
}

pub fn gemini_model() -> ModelDescriptor {
    ModelDescriptor {
        provider: "gemini",
        provider_label: ProviderKind::Gemini.label(),
        id: GEMINI_MODEL_ID,
        wire_model: "gemini-3.6-flash",
        api_base: "https://generativelanguage.googleapis.com/v1beta/openai",
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

pub fn gemini_pro_model() -> ModelDescriptor {
    let mut model = gemini_model();
    model.id = GEMINI_PRO_MODEL_ID;
    model.wire_model = GEMINI_PRO_MODEL_ID;
    model
}
