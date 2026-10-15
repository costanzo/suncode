//! Idempotency journal for mutating operations.
//!
//! A record is written as `pending` before the mutation and finished as
//! `succeeded` or `failed` afterwards, so a crash leaves an explicit unknown
//! completion instead of a silent retry.

use crate::error::{fatal, retryable};
use crate::support::{now_string, sha256_hex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use suncode_common::BusinessError;

#[derive(Serialize, Deserialize)]
pub(crate) struct JournalRecord {
    operation_id: String,
    operation: String,
    path: Option<String>,
    pre_image_sha256: Option<String>,
    post_image_sha256: Option<String>,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) result: Option<Value>,
    created_at: String,
    updated_at: String,
}

fn journal_directory(checkpoint_root: &Path) -> PathBuf {
    checkpoint_root
        .parent()
        .unwrap_or(checkpoint_root)
        .join("journal")
}

pub(crate) fn journal_id(
    idempotency_key: Option<&str>,
    operation_id: Option<&str>,
) -> Option<String> {
    idempotency_key
        .or(operation_id)
        .filter(|value| !value.is_empty())
        .map(|value| sha256_hex(value.as_bytes()))
}

fn journal_path(checkpoint_root: &Path, id: &str) -> PathBuf {
    journal_directory(checkpoint_root).join(format!("{}.json", id))
}

pub(crate) fn load_journal(checkpoint_root: &Path, id: &str) -> Option<JournalRecord> {
    serde_json::from_slice(&fs::read(journal_path(checkpoint_root, id)).ok()?).ok()
}

fn save_journal(checkpoint_root: &Path, record: &JournalRecord) -> Result<(), BusinessError> {
    fs::create_dir_all(journal_directory(checkpoint_root))
        .map_err(|_| retryable("journal_failed", "operation journal could not be created"))?;
    fs::write(
        journal_path(checkpoint_root, &record.operation_id),
        serde_json::to_vec(record)
            .map_err(|_| fatal("journal_failed", "operation journal could not be encoded"))?,
    )
    .map_err(|_| retryable("journal_failed", "operation journal could not be written"))
}

pub(crate) fn journal_intent(
    checkpoint_root: &Path,
    idempotency_key: Option<&str>,
    operation_id: Option<&str>,
    operation: &str,
    path: Option<&str>,
    pre: Option<&[u8]>,
    post: Option<&[u8]>,
) -> Result<Option<String>, BusinessError> {
    let Some(id) = journal_id(idempotency_key, operation_id) else {
        return Ok(None);
    };
    if let Some(existing) = load_journal(checkpoint_root, &id) {
        if existing.status == "succeeded" {
            return Err(fatal(
                "operation_already_completed",
                "operation already completed",
            ));
        }
        if existing.status == "pending" {
            return Err(fatal(
                "unknown_completion",
                "operation completion is unknown and must be reconciled",
            ));
        }
    }
    let now = now_string();
    save_journal(
        checkpoint_root,
        &JournalRecord {
            operation_id: id.clone(),
            operation: operation.to_string(),
            path: path.map(str::to_string),
            pre_image_sha256: pre.map(sha256_hex),
            post_image_sha256: post.map(sha256_hex),
            status: "pending".to_string(),
            result: None,
            created_at: now.clone(),
            updated_at: now,
        },
    )?;
    Ok(Some(id))
}

pub(crate) fn journal_finish(
    checkpoint_root: &Path,
    id: Option<&str>,
    status: &str,
    result: Option<Value>,
) {
    let Some(id) = id else {
        return;
    };
    let Some(mut record) = load_journal(checkpoint_root, id) else {
        return;
    };
    record.status = status.to_string();
    record.result = result;
    record.updated_at = now_string();
    let _ = save_journal(checkpoint_root, &record);
}
