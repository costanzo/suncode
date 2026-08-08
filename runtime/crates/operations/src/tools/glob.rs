use super::super::{search, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn execute(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    search::glob(root, params)
}
