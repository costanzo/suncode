//! CLI host launch selection. The embedded agent only consumes these launch facts.
use crate::error::CliError;
use std::path::PathBuf;
use suncode_sdk::{
    AsyncAgentSdk, ResolvedShell, ShellConfiguration, ShellMode, ShellPreferencesRequest,
};

pub(crate) fn configure(sdk: &AsyncAgentSdk) -> Result<(), CliError> {
    let settings = sdk
        .list_settings(None, None)
        .map_err(CliError::from_business)?;
    let mut configuration: ShellConfiguration = settings
        .settings
        .into_iter()
        .find(|setting| setting.key == "agent_shell")
        .map(|setting| serde_json::from_value(setting.value))
        .transpose()
        .map_err(|_| CliError::invalid("invalid agent_shell preference"))?
        .unwrap_or_default();
    if configuration.resolved.is_some() {
        return Ok(());
    }
    let path =
        match configuration.mode {
            ShellMode::System => {
                if cfg!(windows) {
                    PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| {
                        CliError::invalid("Windows system directory is unavailable")
                    })?)
                    .join("System32/WindowsPowerShell/v1.0/powershell.exe")
                } else {
                    PathBuf::from("/bin/sh")
                }
            }
            ShellMode::Custom => PathBuf::from(configuration.path.trim()),
            ShellMode::Managed => {
                return Err(CliError::invalid(
                    "Open Desktop Settings to validate the managed PowerShell installation",
                ))
            }
        };
    if !path.is_absolute() {
        return Err(CliError::invalid("Shell executable path must be absolute"));
    }
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    let family = match name.as_str() {
        "powershell.exe" | "pwsh.exe" if cfg!(windows) => "powershell",
        "pwsh" if cfg!(unix) => "powershell",
        "sh" | "bash" | "dash" | "zsh" if cfg!(unix) => name.as_str(),
        _ => return Err(CliError::invalid("Unsupported shell executable")),
    };
    configuration.resolved = Some(ResolvedShell {
        path: path.to_string_lossy().into_owned(),
        family: family.into(),
        version: "Host-selected shell".into(),
        interactive_args: vec![],
    });
    sdk.save_shell_preferences(ShellPreferencesRequest {
        agent: Some(configuration),
        terminal: None,
    })
    .map_err(CliError::from_business)?;
    Ok(())
}
