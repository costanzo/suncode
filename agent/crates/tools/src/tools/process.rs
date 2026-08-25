use super::super::{process as runtime_process, CoreFailure};
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub(super) fn run(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    params: &Value,
    cancellation: Option<&AtomicBool>,
) -> Result<Value, CoreFailure> {
    runtime_process::run(root, checkpoint, params, cancellation)
}
