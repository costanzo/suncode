use super::super::arguments::GlobArguments;
use super::super::{search, BusinessError};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    args: GlobArguments,
) -> Result<serde_json::Value, BusinessError> {
    search::glob(root, &args)
}
