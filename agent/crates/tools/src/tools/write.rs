use super::super::arguments::WriteArguments;
use super::super::{write, BusinessError};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    args: WriteArguments,
) -> Result<serde_json::Value, BusinessError> {
    write::write(root, checkpoint, &args)
}
