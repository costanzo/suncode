use std::io::{self, IsTerminal};

use serde_json::{json, to_value};
use suncode_sdk::AsyncAgentSdk;

use crate::{
    args::{AuthCommand, Command, ConfigCommand},
    error::CliError,
    output::CommandReport,
};

pub async fn execute(sdk: &AsyncAgentSdk, command: &Command) -> Result<CommandReport, CliError> {
    match command {
        Command::Doctor => doctor(sdk).await,
        Command::Models => models(sdk),
        Command::Auth { command } => auth(sdk, command),
        Command::Config { command } => config(sdk, command),
    }
}

async fn doctor(sdk: &AsyncAgentSdk) -> Result<CommandReport, CliError> {
    let diagnostics = sdk.diagnostics().map_err(CliError::from_business)?;
    let version = AsyncAgentSdk::version();
    let configured = diagnostics
        .credentials
        .iter()
        .filter(|credential| credential.configured)
        .count();
    let text = format!(
        "SunCode Agent SDK v{}\nAgent: {}\nDatabase: {}\nConfigured providers: {configured}/{}\nBrowser Use: unavailable in CLI host\nComputer Use: unavailable in CLI host",
        version.version,
        diagnostics.health.agent,
        if diagnostics.health.ok { "ready" } else { "unavailable" },
        diagnostics.credentials.len()
    );
    let data = json!({
        "sdk": version,
        "diagnostics": diagnostics,
        "host_capabilities": {
            "browser_use": false,
            "computer_use": false
        }
    });
    Ok(CommandReport {
        event_type: "doctor.result",
        data,
        text,
    })
}

fn models(sdk: &AsyncAgentSdk) -> Result<CommandReport, CliError> {
    let result = sdk.list_models().map_err(CliError::from_business)?;
    let text = if result.models.is_empty() {
        "No models configured.".into()
    } else {
        result
            .models
            .iter()
            .map(|model| {
                format!(
                    "{:<18} {:<22} {}",
                    model.provider, model.id, model.availability
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(CommandReport {
        event_type: "models.result",
        data: to_value(&result).map_err(|error| CliError::io(error.to_string()))?,
        text,
    })
}

fn auth(sdk: &AsyncAgentSdk, command: &AuthCommand) -> Result<CommandReport, CliError> {
    match command {
        AuthCommand::List => {
            let result = sdk.list_credentials().map_err(CliError::from_business)?;
            let text = result
                .credentials
                .iter()
                .map(|credential| {
                    format!(
                        "{:<18} {}",
                        credential.provider,
                        if credential.configured {
                            "configured"
                        } else {
                            "not configured"
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(CommandReport {
                event_type: "auth.list",
                data: to_value(&result).map_err(|error| CliError::io(error.to_string()))?,
                text,
            })
        }
        AuthCommand::Set { provider } => {
            let supported = sdk
                .list_credentials()
                .map_err(CliError::from_business)?
                .credentials
                .into_iter()
                .any(|credential| credential.provider == *provider);
            if !supported {
                return Err(CliError {
                    code: "provider_not_found".into(),
                    message: "provider is not supported".into(),
                    exit_code: 3,
                });
            }
            if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
                return Err(CliError::invalid(
                    "auth set requires an interactive terminal; API-key environment variables are unsupported",
                ));
            }
            let value = rpassword::prompt_password(format!("API key for {provider}: "))
                .map_err(CliError::from)?;
            if value.trim().is_empty() {
                return Err(CliError::invalid("API key must not be empty"));
            }
            let result = sdk
                .set_credential(provider, &value)
                .map_err(CliError::from_business)?;
            Ok(CommandReport {
                event_type: "auth.updated",
                data: to_value(&result).map_err(|error| CliError::io(error.to_string()))?,
                text: format!("Credential stored for {}.", result.provider),
            })
        }
        AuthCommand::Remove { provider } => {
            let result = sdk
                .remove_credential(provider)
                .map_err(CliError::from_business)?;
            Ok(CommandReport {
                event_type: "auth.updated",
                data: to_value(&result).map_err(|error| CliError::io(error.to_string()))?,
                text: format!("Credential removed for {}.", result.provider),
            })
        }
    }
}

fn config(sdk: &AsyncAgentSdk, command: &ConfigCommand) -> Result<CommandReport, CliError> {
    match command {
        ConfigCommand::List => {
            let result = sdk
                .list_settings(None, None)
                .map_err(CliError::from_business)?;
            let text = result
                .settings
                .iter()
                .map(|setting| format!("{:<32} {}", setting.key, compact_json(&setting.value)))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(CommandReport {
                event_type: "config.list",
                data: to_value(&result).map_err(|error| CliError::io(error.to_string()))?,
                text,
            })
        }
    }
}

fn compact_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".into())
}
