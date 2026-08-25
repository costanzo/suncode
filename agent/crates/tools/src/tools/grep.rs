use super::super::arguments::GrepArguments;
use super::super::{search, BusinessError};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    args: GrepArguments,
) -> Result<serde_json::Value, BusinessError> {
    search::find(root, &args)
}
