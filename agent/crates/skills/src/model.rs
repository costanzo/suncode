use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SKILL_FILE_NAME: &str = "SKILL.md";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillSource {
    Explicit,
    ProjectSuncode,
    ProjectAgents,
    UserSuncode,
    UserAgents,
}

impl SkillSource {
    pub(crate) fn is_project_scoped(&self) -> bool {
        matches!(self, Self::ProjectSuncode | Self::ProjectAgents)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub location: PathBuf,
    pub base_directory: PathBuf,
    pub source: SkillSource,
    pub explicit_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillDocument {
    pub info: SkillInfo,
    pub content: String,
    pub resource_files: Vec<String>,
}
