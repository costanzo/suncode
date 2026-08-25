use super::super::arguments::GlobArguments;
use super::super::{search, CoreFailure};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    args: GlobArguments,
) -> Result<serde_json::Value, CoreFailure> {
    search::glob(root, &args)
}
