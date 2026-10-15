//! Discovery precedence, project boundaries, collisions, and lookup.

use crate::{
    diagnostic::SkillDiagnostic,
    manifest,
    model::{SkillDocument, SkillInfo, SkillSource, SKILL_FILE_NAME},
    scan,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const MAX_RESOURCE_FILES: usize = 10;

#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub project_root: PathBuf,
    pub user_config_directory: Option<PathBuf>,
    pub user_home_directory: Option<PathBuf>,
    pub explicit_paths: Vec<PathBuf>,
}

impl DiscoveryOptions {
    /// Standard discovery for a project and an optional user home directory.
    pub fn for_user(project_root: impl Into<PathBuf>, user_directory: Option<&Path>) -> Self {
        Self {
            project_root: project_root.into(),
            user_config_directory: user_directory.map(user_config_directory),
            user_home_directory: user_directory.map(Path::to_path_buf),
            explicit_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillCatalog {
    pub skills: Vec<SkillInfo>,
    pub diagnostics: Vec<SkillDiagnostic>,
}

impl SkillCatalog {
    /// Earlier sources win name collisions: explicit, project, then user.
    pub fn discover(options: &DiscoveryOptions) -> Self {
        let mut catalog = Self::default();
        let project = canonical_directory(&options.project_root);
        let home = || std::env::var_os("HOME").map(PathBuf::from);
        let user_config = options
            .user_config_directory
            .clone()
            .or_else(|| home().map(|home| user_config_directory(&home)));
        let user_home = options.user_home_directory.clone().or_else(home);

        for path in &options.explicit_paths {
            catalog.add_explicit(path, &project);
        }
        for (path, source) in [
            (project.join(".suncode/skills"), SkillSource::ProjectSuncode),
            (project.join(".agents/skills"), SkillSource::ProjectAgents),
        ] {
            catalog.add_directory(&path, source, &project);
        }
        if let Some(user) = user_config {
            catalog.add_directory(&user.join("skills"), SkillSource::UserSuncode, &project);
        }
        if let Some(user) = user_home {
            catalog.add_directory(
                &user.join(".agents/skills"),
                SkillSource::UserAgents,
                &project,
            );
        }
        catalog
    }

    pub fn visible(&self) -> impl Iterator<Item = &SkillInfo> {
        self.skills.iter().filter(|skill| !skill.explicit_only)
    }

    pub fn find(&self, name: &str) -> Option<&SkillInfo> {
        self.skills.iter().find(|skill| skill.name == name)
    }

    pub fn load(&self, name: &str) -> Result<SkillDocument, SkillDiagnostic> {
        load_document(self.lookup(name)?)
    }

    /// Like `load`, but rejects skills restricted to explicit invocation.
    pub fn load_for_model(&self, name: &str) -> Result<SkillDocument, SkillDiagnostic> {
        let info = self.lookup(name)?;
        if info.explicit_only {
            return Err(SkillDiagnostic::warning(
                format!("skill `{name}` is restricted to explicit invocation"),
                Some(info.location.clone()),
            ));
        }
        load_document(info)
    }

    fn lookup(&self, name: &str) -> Result<&SkillInfo, SkillDiagnostic> {
        self.find(name)
            .ok_or_else(|| SkillDiagnostic::warning(format!("skill `{name}` was not found"), None))
    }

    fn add_directory(&mut self, directory: &Path, source: SkillSource, project: &Path) {
        for path in scan::skill_files(directory) {
            self.add_file(&path, source.clone(), project);
        }
    }

    fn add_explicit(&mut self, path: &Path, project: &Path) {
        let resolved = if path.is_dir() {
            path.join(SKILL_FILE_NAME)
        } else {
            path.to_path_buf()
        };
        self.add_file(&resolved, SkillSource::Explicit, project);
    }

    fn add_file(&mut self, path: &Path, source: SkillSource, project: &Path) {
        let Ok(canonical) = fs::canonicalize(path) else {
            return;
        };
        if source.is_project_scoped() && !canonical.starts_with(project) {
            self.diagnostics.push(SkillDiagnostic::warning(
                "project skill escapes project root",
                Some(path.to_path_buf()),
            ));
            return;
        }
        match manifest::read_info(&canonical, source) {
            Ok(info) => match self.find(&info.name) {
                Some(existing) => {
                    let winner = existing.location.clone();
                    self.diagnostics.push(SkillDiagnostic::collision(
                        info.name,
                        info.location,
                        winner,
                    ));
                }
                None => self.skills.push(info),
            },
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
    }
}

fn load_document(info: &SkillInfo) -> Result<SkillDocument, SkillDiagnostic> {
    let content = manifest::read_body(&info.location)
        .map_err(|message| SkillDiagnostic::warning(message, Some(info.location.clone())))?;
    let mut resource_files = scan::resource_files(&info.base_directory);
    resource_files.sort();
    resource_files.truncate(MAX_RESOURCE_FILES);
    Ok(SkillDocument {
        info: info.clone(),
        content,
        resource_files,
    })
}

fn user_config_directory(home: &Path) -> PathBuf {
    home.join(".config/suncode")
}

fn canonical_directory(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::DiagnosticKind;
    use std::fs;
    use tempfile::tempdir;

    fn write_skill(root: &Path, directory: &str, body: &str) {
        let dir = root.join(directory);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(SKILL_FILE_NAME), body).unwrap();
    }

    #[test]
    fn discovers_precedence_and_reports_collision() {
        let project = tempdir().unwrap();
        write_skill(
            project.path(),
            ".suncode/skills/release",
            "---\nname: release\ndescription: project\n---\nbody",
        );
        write_skill(
            project.path(),
            ".agents/skills/release",
            "---\nname: release\ndescription: other\n---\nbody",
        );
        let catalog = SkillCatalog::discover(&DiscoveryOptions {
            project_root: project.path().into(),
            user_config_directory: Some(project.path().join("missing-user-config")),
            user_home_directory: Some(project.path().join("missing-user-home")),
            explicit_paths: vec![],
        });
        assert_eq!(catalog.skills.len(), 1);
        assert_eq!(catalog.skills[0].description, "project");
        assert!(catalog
            .diagnostics
            .iter()
            .any(|item| item.kind == DiagnosticKind::Collision));
    }

    #[test]
    fn loads_body_and_bounded_resources() {
        let project = tempdir().unwrap();
        write_skill(
            project.path(),
            ".suncode/skills/release",
            "---\nname: release\ndescription: release files\n---\n\nDo the release.",
        );
        fs::write(
            project.path().join(".suncode/skills/release/reference.md"),
            "reference",
        )
        .unwrap();
        let catalog = SkillCatalog::discover(&DiscoveryOptions {
            project_root: project.path().into(),
            user_config_directory: Some(project.path().join("missing-user-config")),
            user_home_directory: Some(project.path().join("missing-user-home")),
            explicit_paths: vec![],
        });
        let document = catalog.load("release").unwrap();
        assert_eq!(document.content, "Do the release.");
        assert_eq!(document.resource_files, vec!["reference.md"]);
    }

    #[test]
    fn user_skill_directory_is_not_rejected_by_project_boundary() {
        let project = tempdir().unwrap();
        let user = tempdir().unwrap();
        write_skill(
            user.path(),
            ".agents/skills/release",
            "---\nname: release\ndescription: user\n---\nbody",
        );
        let catalog = SkillCatalog::discover(&DiscoveryOptions {
            project_root: project.path().into(),
            user_config_directory: Some(project.path().join("missing-user-config")),
            user_home_directory: Some(user.path().into()),
            explicit_paths: vec![],
        });
        assert_eq!(catalog.skills[0].description, "user");
    }
}
