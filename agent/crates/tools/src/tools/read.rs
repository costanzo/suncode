use super::super::arguments::ReadArguments;
use super::super::{filesystem, BusinessError};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    args: ReadArguments,
) -> Result<serde_json::Value, BusinessError> {
    filesystem::read(root, &args)
}
