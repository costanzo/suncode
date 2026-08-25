use super::super::arguments::GrepArguments;
use super::super::{search, CoreFailure};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    args: GrepArguments,
) -> Result<serde_json::Value, CoreFailure> {
    search::find(root, &args)
}
