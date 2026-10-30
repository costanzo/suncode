//! Operation dispatch is intentionally explicit: one audited entry per tool.

use crate::arguments::{
    CheckpointRestoreArguments, EditArguments, GlobArguments, GrepArguments, ProcessArguments,
    ReadArguments, WebfetchArguments, WriteArguments, DocumentInspectArguments, DocumentReadArguments,
    DocumentSearchArguments, SpreadsheetReadArguments,
};
use crate::context::ExecutionContext;
use crate::error::fatal;
use crate::{checkpoint, edit, process, read, search, webfetch, write, documents};
use serde::de::DeserializeOwned;
use serde_json::Value;
use suncode_common::BusinessError;

pub(crate) fn dispatch(
    method: &str,
    params: &Value,
    context: ExecutionContext<'_>,
) -> Result<Value, BusinessError> {
    let ExecutionContext {
        project_root,
        checkpoint_root,
        cancellation,
        output_callback,
        network,
        documents,
    } = context;
    match method {
        "tool/read" => {
            // Keep the established field-level error for malformed public read calls.
            if params.get("path").and_then(Value::as_str).is_none() {
                return Err(fatal("invalid_arguments", "path is required"));
            }
            run_typed(params, |args: ReadArguments| {
                read::read(project_root, checkpoint_root, &args)
            })
        }
        "tool/glob" => run_typed(params, |args: GlobArguments| {
            search::glob(project_root, &args)
        }),
        "tool/grep" => run_typed(params, |args: GrepArguments| {
            search::find(project_root, &args)
        }),
        "tool/document_inspect" => run_typed(params, |args: DocumentInspectArguments| {
            documents::inspect(project_root, documents, &args, cancellation)
        }),
        "tool/document_read" => run_typed(params, |args: DocumentReadArguments| {
            documents::read(project_root, documents, &args, cancellation)
        }),
        "tool/document_search" => run_typed(params, |args: DocumentSearchArguments| {
            documents::search(project_root, documents, &args, cancellation)
        }),
        "tool/spreadsheet_read" => run_typed(params, |args: SpreadsheetReadArguments| {
            documents::spreadsheet(project_root, documents, &args, cancellation)
        }),
        "tool/webfetch" => run_typed(params, |args: WebfetchArguments| {
            webfetch::execute(checkpoint_root, args, cancellation, &network)
        }),
        "tool/write" => run_typed(params, |args: WriteArguments| {
            write::write(project_root, checkpoint_root, &args)
        }),
        "tool/edit" => run_typed(params, |args: EditArguments| {
            edit::edit(project_root, checkpoint_root, &args)
        }),
        "tool/bash" => run_typed(params, |args: ProcessArguments| {
            process::run(
                project_root,
                checkpoint_root,
                &args,
                cancellation,
                output_callback,
            )
        }),
        "checkpoint/restore" => run_typed(params, |args: CheckpointRestoreArguments| {
            checkpoint::restore(project_root, checkpoint_root, &args)
        }),
        _ => Err(fatal("method_unavailable", "method unavailable")),
    }
}

fn run_typed<T, F>(params: &Value, execute: F) -> Result<Value, BusinessError>
where
    T: DeserializeOwned,
    F: FnOnce(T) -> Result<Value, BusinessError>,
{
    let arguments = serde_json::from_value(params.clone())
        .map_err(|_| fatal("invalid_arguments", "tool arguments have an invalid shape"))?;
    execute(arguments)
}
