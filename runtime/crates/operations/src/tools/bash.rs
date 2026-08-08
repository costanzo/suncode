use super::super::{process as runtime_process, CoreFailure};
use serde_json::Value;
use std::path::Path;

pub(super) fn run(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    runtime_process::run(root, checkpoint, params)
}
pub(super) fn start(root: Option<&Path>, params: &Value) -> Result<Value, CoreFailure> {
    runtime_process::start(root, params)
}
pub(super) fn status(params: &Value) -> Result<Value, CoreFailure> {
    runtime_process::status(params)
}
pub(super) fn cancel(params: &Value) -> Result<Value, CoreFailure> {
    runtime_process::cancel(params)
}
pub(super) fn operation_status(
    checkpoint: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    runtime_process::operation_status(checkpoint, params)
}
pub(super) fn reconcile(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    params: &Value,
) -> Result<Value, CoreFailure> {
    runtime_process::reconcile(root, checkpoint, params)
}
pub(super) fn recovery(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
) -> Result<Value, CoreFailure> {
    runtime_process::recovery(root, checkpoint)
}
