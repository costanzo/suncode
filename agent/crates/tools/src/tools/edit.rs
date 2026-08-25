use super::super::{mutations, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    mutations::edit(root, checkpoint, params)
}
