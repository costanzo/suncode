use super::super::{filesystem, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn execute(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    filesystem::metadata(root, params)
}
