//! SQLite resources and database-file setup.

use std::fs::OpenOptions;
use std::io;
use std::path::Path;

pub const TABLE_NAMES: &[&str] = &[
    "approval_request",
    "audit_record",
    "checkpoint",
    "checkpoint_manifest",
    "configuration",
    "llm_model",
    "llm_model_provider",
    "project",
    "project_dependency",
    "session",
    "session_call",
    "session_image",
    "session_message",
    "session_tool_use",
    "session_turn",
    "session_turn_todo",
];

const SCHEMA_SCRIPTS: &[&str] = &[
    include_str!("schema/audit_record.sql"),
    include_str!("schema/project.sql"),
    include_str!("schema/project_dependency.sql"),
    include_str!("schema/session.sql"),
    include_str!("schema/configuration.sql"),
    include_str!("schema/session_turn.sql"),
    include_str!("schema/session_turn_todo.sql"),
    include_str!("schema/session_call.sql"),
    include_str!("schema/session_tool_use.sql"),
    include_str!("schema/approval_request.sql"),
    include_str!("schema/checkpoint_manifest.sql"),
    include_str!("schema/checkpoint.sql"),
    include_str!("schema/session_message.sql"),
    include_str!("schema/session_image.sql"),
    include_str!("schema/llm_model_provider.sql"),
    include_str!("schema/llm_model.sql"),
];

const DATA_SCRIPTS: &[&str] = &[
    include_str!("data/llm_model_provider.sql"),
    include_str!("data/llm_model.sql"),
];

pub fn schema_scripts() -> &'static [&'static str] {
    SCHEMA_SCRIPTS
}
pub fn data_scripts() -> &'static [&'static str] {
    DATA_SCRIPTS
}

/// Ensures the SQLite file and its parent directory exist without opening it.
/// Diesel remains exclusively responsible for opening and using the connection.
pub fn ensure_database(path: &Path) -> io::Result<bool> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let existed = path.exists();
    OpenOptions::new().create(true).append(true).open(path)?;
    Ok(existed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_reopens_database_file_idempotently() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nested").join("agent.sqlite3");
        assert!(!ensure_database(&path).unwrap());
        assert!(path.is_file());
        assert!(ensure_database(&path).unwrap());
        assert_eq!(schema_scripts().len(), TABLE_NAMES.len());
        assert_eq!(data_scripts().len(), 2);
    }
}
