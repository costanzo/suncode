//! Local, file-backed Agent Skills discovery and loading.
//!
//! This crate owns all Skill filesystem access. The agent core only consumes
//! validated metadata and bounded documents returned by this API.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const MAX_SKILL_BYTES: u64 = 64 * 1024;
pub const MAX_DESCRIPTION_CHARS: usize = 1024;
pub const MAX_NAME_CHARS: usize = 64;
pub const MAX_RESOURCE_FILES: usize = 10;
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillCatalog {
    pub skills: Vec<SkillInfo>,
    pub diagnostics: Vec<SkillDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub project_root: PathBuf,
    pub user_config_directory: Option<PathBuf>,
    pub user_home_directory: Option<PathBuf>,
    pub explicit_paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "disable-model-invocation")]
    disable_model_invocation: Option<bool>,
}

impl SkillCatalog {
    pub fn discover(options: &DiscoveryOptions) -> Self {
        let mut catalog = Self::default();
        let project = canonical_directory(&options.project_root);
        let user_config = options
            .user_config_directory
            .clone()
            .or_else(default_user_config_directory);
        let user_home = options
            .user_home_directory
            .clone()
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from));

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
        let info = self
            .find(name)
            .ok_or_else(|| warning(format!("skill `{name}` was not found"), None))?;
        load_document(info).map_err(|message| warning(message, Some(info.location.clone())))
    }

    pub fn load_for_model(&self, name: &str) -> Result<SkillDocument, SkillDiagnostic> {
        let info = self
            .find(name)
            .ok_or_else(|| warning(format!("skill `{name}` was not found"), None))?;
        if info.explicit_only {
            return Err(warning(
                format!("skill `{name}` is restricted to explicit invocation"),
                Some(info.location.clone()),
            ));
        }
        load_document(info).map_err(|message| warning(message, Some(info.location.clone())))
    }

    fn add_directory(&mut self, directory: &Path, source: SkillSource, project: &Path) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                self.add_directory(&path, source.clone(), project);
            } else if path.file_name().and_then(|name| name.to_str()) == Some(SKILL_FILE_NAME) {
                self.add_file(&path, source.clone(), project, false);
            }
        }
    }

    fn add_explicit(&mut self, path: &Path, project: &Path) {
        let resolved = if path.is_dir() {
            path.join(SKILL_FILE_NAME)
        } else {
            path.to_path_buf()
        };
        self.add_file(&resolved, SkillSource::Explicit, project, true);
    }

    fn add_file(&mut self, path: &Path, source: SkillSource, project: &Path, explicit: bool) {
        let canonical = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(_) => return,
        };
        let project_scoped = matches!(
            source,
            SkillSource::ProjectSuncode | SkillSource::ProjectAgents
        );
        if project_scoped && !explicit && !canonical.starts_with(project) {
            self.diagnostics.push(warning(
                "project skill escapes project root".into(),
                Some(path.to_path_buf()),
            ));
            return;
        }
        match parse_info(&canonical, source) {
            Ok(info) => {
                if let Some(existing) = self.skills.iter().find(|skill| skill.name == info.name) {
                    self.diagnostics.push(SkillDiagnostic {
                        kind: DiagnosticKind::Collision,
                        message: format!(
                            "skill name `{}` collides with an earlier skill",
                            info.name
                        ),
                        path: Some(info.location.clone()),
                        skill_name: Some(info.name),
                        winner: Some(existing.location.clone()),
                    });
                } else {
                    self.skills.push(info);
                }
            }
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
    }
}

fn parse_info(path: &Path, source: SkillSource) -> Result<SkillInfo, SkillDiagnostic> {
    let metadata =
        fs::metadata(path).map_err(|error| warning(error.to_string(), Some(path.to_path_buf())))?;
    if metadata.len() > MAX_SKILL_BYTES {
        return Err(warning(
            format!("skill exceeds {} bytes", MAX_SKILL_BYTES),
            Some(path.to_path_buf()),
        ));
    }
    let raw = fs::read_to_string(path)
        .map_err(|error| warning(error.to_string(), Some(path.to_path_buf())))?;
    let (frontmatter, _) =
        split_frontmatter(&raw).map_err(|message| warning(message, Some(path.to_path_buf())))?;
    let parsed: Frontmatter = serde_yaml::from_str(frontmatter)
        .map_err(|error| warning(error.to_string(), Some(path.to_path_buf())))?;
    let name = parsed.name.unwrap_or_else(|| {
        path.parent()
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string()
    });
    validate_name(&name).map_err(|message| warning(message, Some(path.to_path_buf())))?;
    let description = parsed.description.unwrap_or_default().trim().to_string();
    if description.is_empty() || description.chars().count() > MAX_DESCRIPTION_CHARS {
        return Err(warning(
            format!("skill description must contain 1 through {MAX_DESCRIPTION_CHARS} characters"),
            Some(path.to_path_buf()),
        ));
    }
    let base_directory = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok(SkillInfo {
        name,
        description,
        location: path.to_path_buf(),
        base_directory,
        source,
        explicit_only: parsed.disable_model_invocation.unwrap_or(false),
    })
}

fn load_document(info: &SkillInfo) -> Result<SkillDocument, String> {
    let raw = fs::read_to_string(&info.location).map_err(|error| error.to_string())?;
    let (_, content) = split_frontmatter(&raw)?;
    let mut resource_files = Vec::new();
    collect_resources(
        &info.base_directory,
        &info.base_directory,
        &mut resource_files,
    );
    resource_files.sort();
    resource_files.truncate(MAX_RESOURCE_FILES);
    Ok(SkillDocument {
        info: info.clone(),
        content: content.trim().to_string(),
        resource_files,
    })
}

fn collect_resources(root: &Path, directory: &Path, result: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if path.file_name().and_then(|name| name.to_str()) == Some(SKILL_FILE_NAME) {
            continue;
        }
        if path.is_dir() {
            collect_resources(root, &path, result);
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(root) {
                result.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let mut lines = raw.split_inclusive('\n');
    if lines.next().map(str::trim) != Some("---") {
        return Err("SKILL.md must start with YAML frontmatter".into());
    }
    let start = raw.find('\n').ok_or("frontmatter is incomplete")? + 1;
    let rest = &raw[start..];
    let end_offset = rest
        .find("\n---")
        .ok_or("frontmatter closing delimiter is missing")?;
    let end = start + end_offset;
    let content_start = end + "\n---".len();
    let content_start = if raw.as_bytes().get(content_start) == Some(&b'\n') {
        content_start + 1
    } else {
        content_start
    };
    Ok((&raw[start..end], &raw[content_start..]))
}

fn validate_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.chars().count() <= MAX_NAME_CHARS
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");
    valid.then_some(()).ok_or_else(|| {
        "skill name must match ^[a-z0-9]+(-[a-z0-9]+)*$ and be at most 64 characters".into()
    })
}

fn canonical_directory(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn default_user_config_directory() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/suncode"))
}

fn warning(message: String, path: Option<PathBuf>) -> SkillDiagnostic {
    SkillDiagnostic {
        kind: DiagnosticKind::Warning,
        message,
        path,
        skill_name: None,
        winner: None,
    }
}

pub fn render_available_skills<I>(skills: I) -> String
where
    I: IntoIterator<Item = SkillInfo>,
{
    let skills = skills.into_iter().collect::<Vec<_>>();
    if skills.is_empty() {
        return String::new();
    }
    let mut output = String::from("Skills provide specialized instructions and workflows for specific tasks. Use the skill tool to load a matching skill.\n\n<available_skills>\n");
    for skill in skills {
        output.push_str(&format!("  <skill>\n    <name>{}</name>\n    <description>{}</description>\n    <location>{}</location>\n  </skill>\n", escape_xml(&skill.name), escape_xml(&skill.description), escape_xml(&skill.location.to_string_lossy())));
    }
    output.push_str("</available_skills>");
    output
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
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
            user_config_directory: None,
            user_home_directory: None,
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
            user_config_directory: None,
            user_home_directory: None,
            explicit_paths: vec![],
        });
        let document = catalog.load("release").unwrap();
        assert_eq!(document.content, "Do the release.");
        assert_eq!(document.resource_files, vec!["reference.md"]);
    }

    #[test]
    fn renders_escaped_available_skill_metadata() {
        let info = SkillInfo {
            name: "a-b".into(),
            description: "a < b".into(),
            location: PathBuf::from("/tmp/a"),
            base_directory: PathBuf::from("/tmp"),
            source: SkillSource::Explicit,
            explicit_only: false,
        };
        let output = render_available_skills([info]);
        assert!(output.contains("a &lt; b"));
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
            user_config_directory: None,
            user_home_directory: Some(user.path().into()),
            explicit_paths: vec![],
        });
        assert_eq!(catalog.skills[0].description, "user");
    }
}
