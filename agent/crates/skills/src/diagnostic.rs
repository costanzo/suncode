use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    Warning,
    Collision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillDiagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub path: Option<PathBuf>,
    pub skill_name: Option<String>,
    pub winner: Option<PathBuf>,
}

impl SkillDiagnostic {
    pub(crate) fn warning(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            kind: DiagnosticKind::Warning,
            message: message.into(),
            path,
            skill_name: None,
            winner: None,
        }
    }

    pub(crate) fn collision(name: String, path: PathBuf, winner: PathBuf) -> Self {
        Self {
            kind: DiagnosticKind::Collision,
            message: format!("skill name `{name}` collides with an earlier skill"),
            path: Some(path),
            skill_name: Some(name),
            winner: Some(winner),
        }
    }
}
