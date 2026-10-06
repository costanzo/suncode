use super::*;
use std::path::Path;
use suncode_skills::{DiagnosticKind, DiscoveryOptions, SkillCatalog, SkillInfo, SkillSource};

impl AsyncAgentSdk {
    pub fn list_skills(&self, project_id: &str) -> SdkResult<SkillsResult> {
        let project = self.project_for_user(project_id)?;
        let catalog = SkillCatalog::discover(&DiscoveryOptions {
            project_root: project.canonical_root.into(),
            user_config_directory: None,
            user_home_directory: None,
            explicit_paths: Vec::new(),
        });
        Ok(SkillsResult {
            project_id: project_id.to_string(),
            skills: catalog.skills.iter().map(skill_dto).collect(),
            diagnostics: catalog.diagnostics.iter().map(diagnostic_dto).collect(),
        })
    }

    pub fn load_skill(&self, project_id: &str, name: &str) -> SdkResult<SkillDocumentResult> {
        let project = self.project_for_user(project_id)?;
        let catalog = SkillCatalog::discover(&DiscoveryOptions {
            project_root: project.canonical_root.into(),
            user_config_directory: None,
            user_home_directory: None,
            explicit_paths: Vec::new(),
        });
        let document = catalog.load(name).map_err(|diagnostic| {
            BusinessError::new("skill_load_failed", diagnostic.message.clone())
                .details(serde_json::to_value(diagnostic).unwrap_or_else(|_| serde_json::json!({})))
        })?;
        Ok(SkillDocumentResult {
            project_id: project_id.to_string(),
            skill: skill_dto(&document.info),
            content: document.content,
            resource_files: document.resource_files,
        })
    }
}

fn skill_dto(skill: &SkillInfo) -> SkillDto {
    SkillDto {
        name: skill.name.clone(),
        description: skill.description.clone(),
        location: skill.location.to_string_lossy().into_owned(),
        base_directory: skill.base_directory.to_string_lossy().into_owned(),
        source: match skill.source {
            SkillSource::Explicit => "explicit",
            SkillSource::ProjectSuncode => "project_suncode",
            SkillSource::ProjectAgents => "project_agents",
            SkillSource::UserSuncode => "user_suncode",
            SkillSource::UserAgents => "user_agents",
        }
        .into(),
        explicit_only: skill.explicit_only,
    }
}

fn diagnostic_dto(diagnostic: &suncode_skills::SkillDiagnostic) -> SkillDiagnosticDto {
    SkillDiagnosticDto {
        kind: match diagnostic.kind {
            DiagnosticKind::Warning => "warning",
            DiagnosticKind::Collision => "collision",
        }
        .into(),
        message: diagnostic.message.clone(),
        path: diagnostic
            .path
            .as_deref()
            .map(Path::to_string_lossy)
            .map(|v| v.into_owned()),
        skill_name: diagnostic.skill_name.clone(),
        winner: diagnostic
            .winner
            .as_deref()
            .map(Path::to_string_lossy)
            .map(|v| v.into_owned()),
    }
}
