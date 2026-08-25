use super::super::arguments::WriteArguments;
use super::super::{write, CoreFailure};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    args: WriteArguments,
) -> Result<serde_json::Value, CoreFailure> {
    write::write(root, checkpoint, &args)
}
