use super::super::{checkpoint, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    checkpoint::restore(root, checkpoint, params)
}
