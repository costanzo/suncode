use super::super::arguments::ProcessArguments;
use super::super::{process as runtime_process, BusinessError};
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub(super) fn run(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    args: ProcessArguments,
    cancellation: Option<&AtomicBool>,
) -> Result<serde_json::Value, BusinessError> {
    runtime_process::run(root, checkpoint, &args, cancellation)
}
