use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub(crate) struct ReadArguments {
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) offset: Option<u64>,
    #[serde(default)]
    pub(crate) limit: Option<u64>,
    #[serde(default)]
    pub(crate) max_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GlobArguments {
    pub(crate) pattern: String,
    #[serde(default)]
    pub(crate) max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GrepArguments {
    pub(crate) query: String,
    #[serde(default = "default_grep_pattern")]
    pub(crate) pattern: String,
    #[serde(default)]
    pub(crate) max_results: Option<usize>,
}

fn default_grep_pattern() -> String {
    "**/*".into()
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReplacementArguments {
    pub(crate) old: String,
    pub(crate) new: String,
    #[serde(default)]
    pub(crate) replace_all: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EditArguments {
    pub(crate) path: String,
    pub(crate) expected_base64: String,
    pub(crate) replacements: Vec<ReplacementArguments>,
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
    #[serde(default)]
    pub(crate) operation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WriteArguments {
    pub(crate) path: String,
    pub(crate) content_base64: String,
    #[serde(default)]
    pub(crate) expected_base64: Option<String>,
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
    #[serde(default)]
    pub(crate) operation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProcessArguments {
    pub(crate) program: String,
    #[serde(default)]
    pub(crate) args: Vec<String>,
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(default)]
    pub(crate) env: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) timeout_ms: Option<u64>,
    #[serde(default)]
    pub(crate) sandbox_profile: Option<String>,
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
    #[serde(default)]
    pub(crate) operation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WebfetchArguments {
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) format: Option<String>,
    #[serde(default)]
    pub(crate) timeout: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckpointRestoreArguments {
    pub(crate) checkpoint_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitDiffArguments {
    pub(crate) path: String,
    #[serde(default = "default_git_scope")]
    pub(crate) scope: String,
}

fn default_git_scope() -> String {
    "all".into()
}
