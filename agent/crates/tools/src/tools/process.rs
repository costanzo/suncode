use super::super::arguments::ProcessArguments;
use super::super::{process as runtime_process, BusinessError, ProcessOutputCallback};
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub(super) fn run(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    args: ProcessArguments,
    cancellation: Option<&AtomicBool>,
    output_callback: Option<ProcessOutputCallback>,
) -> Result<serde_json::Value, BusinessError> {
    runtime_process::run(root, checkpoint, &args, cancellation, output_callback)
}
