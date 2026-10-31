//! Pure shell configuration and runtime facts shared by the agent and native hosts.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellConfiguration {
    #[serde(default)]
    pub mode: ShellMode,
    #[serde(default)]
    pub path: String,
    /// Launch facts supplied by the host; the agent never discovers or probes shells.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<ResolvedShell>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShellMode {
    #[default]
    System,
    Custom,
    Managed,
}

impl ShellConfiguration {
    pub fn validate(&self) -> bool {
        if self
            .resolved
            .as_ref()
            .is_some_and(|shell| !shell.validate())
        {
            return false;
        }
        match self.mode {
            ShellMode::System | ShellMode::Managed => self.path.is_empty(),
            ShellMode::Custom => {
                !self.path.trim().is_empty()
                    && self.path.len() <= 4096
                    && !self.path.chars().any(char::is_control)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedShell {
    pub path: String,
    pub family: String,
    pub version: String,
    pub interactive_args: Vec<String>,
}

impl ResolvedShell {
    /// Structural validation only. Does not access files or execute the interpreter.
    pub fn validate(&self) -> bool {
        std::path::Path::new(&self.path).is_absolute()
            && self.path.len() <= 4096
            && !self.path.chars().any(char::is_control)
            && matches!(
                self.family.as_str(),
                "powershell" | "sh" | "bash" | "zsh" | "dash"
            )
            && self.version.len() <= 128
            && !self.version.chars().any(char::is_control)
            && self.interactive_args.len() <= 8
            && self
                .interactive_args
                .iter()
                .all(|arg| arg.len() <= 256 && !arg.chars().any(char::is_control))
    }
}
