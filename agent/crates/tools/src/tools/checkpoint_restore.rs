use super::super::arguments::CheckpointRestoreArguments;
use super::super::{checkpoint, BusinessError};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    args: CheckpointRestoreArguments,
) -> Result<serde_json::Value, BusinessError> {
    checkpoint::restore(root, checkpoint, &args)
}
