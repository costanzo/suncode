use super::super::{project_inspect, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn execute(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    project_inspect(root, params)
}
