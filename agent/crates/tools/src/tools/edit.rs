use super::super::arguments::EditArguments;
use super::super::{mutations, BusinessError};
use std::path::Path;

pub(super) fn execute(
    root: Option<&Path>,
    checkpoint: Option<&Path>,
    args: EditArguments,
) -> Result<serde_json::Value, BusinessError> {
    mutations::edit(root, checkpoint, &args)
}
