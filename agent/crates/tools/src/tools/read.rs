use super::super::arguments::ReadArguments;
use super::super::{filesystem, CoreFailure};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    args: ReadArguments,
) -> Result<serde_json::Value, CoreFailure> {
    filesystem::read(root, &args)
}
