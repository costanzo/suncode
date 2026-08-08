use super::super::{artifacts, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn read(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    artifacts::read(root, params)
}
pub(super) fn sweep(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    artifacts::sweep(root, params)
}
