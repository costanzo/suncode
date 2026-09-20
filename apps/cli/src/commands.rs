use futures_util::StreamExt;
use std::io::{self, IsTerminal, Read};
use uuid::Uuid;

use serde_json::{json, to_value};
use suncode_sdk::AsyncAgentSdk;

use crate::{
    args::{AuthCommand, Command, ConfigCommand, SessionCommand},
    error::CliError,
    output::CommandReport,
};

pub async fn execute(sdk: &AsyncAgentSdk, command: &Command) -> Result<CommandReport, CliError> {
    match command {
        Command::Doctor => doctor(sdk).await,
        Command::Models => models(sdk),
        Command::Auth { command } => auth(sdk, command),
        Command::Config { command } => config(sdk, command),
        Command::Session {
            command: SessionCommand::Resume { .. },
        } => unreachable!("session resume is dispatched with CLI configuration"),
        Command::Session { command } => session(sdk, command),
        Command::Run { .. } => unreachable!("run is dispatched with CLI configuration"),
    }
}

pub async fn run(
    sdk: &AsyncAgentSdk,
    path: Option<&str>,
    prompt: Option<&str>,
    stdin: bool,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    output: crate::args::OutputMode,
) -> Result<CommandReport, CliError> {
    let input = read_prompt(prompt, stdin)?;
    let path = path.unwrap_or(".");
    let project = sdk
        .open_project(path, None)
        .map_err(CliError::from_business)?;
    let session = sdk
        .create_session(&project.project_id, None, model)
        .map_err(CliError::from_business)?;
    let watch = sdk
        .watch_session(&session.session_id)
        .map_err(CliError::from_business)?;
    let mut events = watch.events;
    let session_id = session.session_id.clone();
    let response = execute_turn(
        sdk,
        &session_id,
        &input,
        model,
        reasoning_effort,
        output,
        &mut events,
    )
    .await?;
    turn_report("run.result", &session_id, response)
}

pub async fn resume(
    sdk: &AsyncAgentSdk,
    session_id: &str,
    prompt: Option<&str>,
    stdin: bool,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    output: crate::args::OutputMode,
) -> Result<CommandReport, CliError> {
    let input = read_prompt(prompt, stdin)?;
    let watch = sdk
        .watch_session(session_id)
        .map_err(CliError::from_business)?;
    if sdk
        .pending_approval(session_id)
        .map_err(CliError::from_business)?
        .is_some()
    {
        return Err(CliError {
            code: "approval_required".into(),
            message: "session has a pending approval".into(),
            exit_code: 4,
        });
    }
    if watch.snapshot.pending_question.is_some() {
        return Err(CliError {
            code: "question_required".into(),
            message: "session has a pending interactive question".into(),
            exit_code: 4,
        });
    }
    sdk.reopen_session(session_id)
        .map_err(CliError::from_business)?;
    let mut events = watch.events;
    let response = execute_turn(
        sdk,
        session_id,
        &input,
        model,
        reasoning_effort,
        output,
        &mut events,
    )
    .await?;
    turn_report("session.resume.result", session_id, response)
}

fn read_prompt(prompt: Option<&str>, stdin: bool) -> Result<String, CliError> {
    match (prompt, stdin) {
        (Some(prompt), false) if !prompt.is_empty() => Ok(prompt.to_string()),
        (None, true) => {
            if io::stdin().is_terminal() {
                return Err(CliError::invalid(
                    "--stdin requires piped input and cannot read an interactive terminal",
                ));
            }
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            if input.is_empty() {
                return Err(CliError::invalid("stdin prompt is empty"));
            }
            Ok(input)
        }
        (Some(_), true) => unreachable!("clap enforces prompt/stdin exclusivity"),
        (None, false) => Err(CliError::invalid("--prompt TEXT or --stdin is required")),
        (Some(_), false) => Err(CliError::invalid("prompt is empty")),
    }
}

async fn execute_turn(
    sdk: &AsyncAgentSdk,
    session_id: &str,
    input: &str,
    model: Option<&str>,
    reasoning_effort: Option<&str>,
    output: crate::args::OutputMode,
    events: &mut suncode_sdk::SessionEventStream,
) -> Result<suncode_sdk::TurnResponse, CliError> {
    let input_id = Uuid::new_v4().to_string();
    let submit = sdk.submit_turn(session_id, input, &input_id, model, reasoning_effort);
    tokio::pin!(submit);
    let mut active_turn_id = None;
    let mut interrupt_requested = false;
    let mut cancel_sent = false;
    let response = loop {
        tokio::select! {
            biased;

            signal = tokio::signal::ctrl_c() => {
                signal.map_err(|error| CliError::io(error.to_string()))?;
                if interrupt_requested {
                    return Err(CliError::interrupted());
                }
                interrupt_requested = true;
                eprintln!("interrupt requested; cancelling active turn");
                if let Some(turn_id) = active_turn_id.as_deref() {
                    request_cancel(sdk, session_id, turn_id)?;
                    cancel_sent = true;
                }
            }
            event = events.next() => {
                match event {
                    Some(Ok(event)) => {
                        if let suncode_sdk::AgentEventPayload::TurnState(payload) = &event.payload {
                            active_turn_id = Some(payload.turn_id.clone());
                            if interrupt_requested && !cancel_sent {
                                request_cancel(sdk, session_id, &payload.turn_id)?;
                                cancel_sent = true;
                            }
                        }
                        crate::output::write_event(output, &event)?;
                    }
                    Some(Err(suncode_sdk::SubscriptionError::Lagged { .. })) => {
                        let replacement = sdk.watch_session(session_id).map_err(CliError::from_business)?;
                        *events = replacement.events;
                    }
                    Some(Err(error)) => return Err(CliError::io(error.to_string())),
                    None => return Err(CliError::io("session event stream closed")),
                }
            }
            result = &mut submit => {
                match result {
                    Ok(response) => break response,
                    Err(error) if interrupt_requested && error.code == "cancelled" => {
                        return Err(CliError::interrupted());
                    }
                    Err(error) => return Err(CliError::from_business(error)),
                }
            }
        }
    };
    loop {
        match events.try_recv() {
            Ok(event) => crate::output::write_event(output, &event)?,
            Err(suncode_sdk::SubscriptionError::Empty) => break,
            Err(suncode_sdk::SubscriptionError::Lagged { .. }) => {
                let replacement = sdk
                    .watch_session(session_id)
                    .map_err(CliError::from_business)?;
                *events = replacement.events;
            }
            Err(suncode_sdk::SubscriptionError::Closed) => {
                return Err(CliError::io("session event stream closed"));
            }
        }
    }
    Ok(response)
}

fn turn_report(
    event_type: &'static str,
    session_id: &str,
    response: suncode_sdk::TurnResponse,
) -> Result<CommandReport, CliError> {
    let data = serde_json::to_value(&response).map_err(|error| CliError::io(error.to_string()))?;
    let text = match &response {
        suncode_sdk::TurnResponse::Completed { message, .. } => message.text_content(),
        suncode_sdk::TurnResponse::AwaitingApproval { .. } => {
            return Err(CliError {
                code: "approval_required".into(),
                message: "turn requires interactive approval".into(),
                exit_code: 4,
            });
        }
        suncode_sdk::TurnResponse::AwaitingQuestion { .. } => {
            return Err(CliError {
                code: "question_required".into(),
                message: "turn requires an interactive question answer".into(),
                exit_code: 4,
            });
        }
        suncode_sdk::TurnResponse::Queued { .. } => {
            return Err(CliError::io("turn was unexpectedly queued"));
        }
    };
    Ok(CommandReport {
        event_type,
        data: json!({"session_id": session_id, "response": data}),
        text,
    })
}

fn request_cancel(sdk: &AsyncAgentSdk, session_id: &str, turn_id: &str) -> Result<(), CliError> {
    match sdk.cancel_turn(session_id, turn_id) {
        Ok(_) => Ok(()),
        Err(error) if error.code == "conflict" => Ok(()),
        Err(error) => Err(CliError::from_business(error)),
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

fn session(sdk: &AsyncAgentSdk, command: &SessionCommand) -> Result<CommandReport, CliError> {
    match command {
        SessionCommand::List { path } => {
            let project = sdk
                .open_project(path.as_deref().unwrap_or("."), None)
                .map_err(CliError::from_business)?;
            let result = sdk
                .list_sessions(&project.project_id)
                .map_err(CliError::from_business)?;
            let text = if result.sessions.is_empty() {
                "No sessions found.".into()
            } else {
                result
                    .sessions
                    .iter()
                    .map(|session| {
                        let state = result
                            .session_states
                            .get(&session.session_id)
                            .map(String::as_str)
                            .unwrap_or(&session.status);
                        format!(
                            "{}\t{}\t{}\t{}",
                            session.session_id,
                            state,
                            session.title.as_deref().unwrap_or("(untitled)"),
                            session.model_id.as_deref().unwrap_or("(default)")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(CommandReport {
                event_type: "session.list",
                data: to_value(&result).map_err(|error| CliError::io(error.to_string()))?,
                text,
            })
        }
        SessionCommand::Archive { session_id } => {
            let session = sdk
                .archive_session(session_id)
                .map_err(CliError::from_business)?;
            Ok(CommandReport {
                event_type: "session.archived",
                data: to_value(&session).map_err(|error| CliError::io(error.to_string()))?,
                text: format!("Archived session {}.", session.session_id),
            })
        }
        SessionCommand::Resume { .. } => {
            unreachable!("session resume is dispatched with CLI configuration")
        }
    }
}

fn compact_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".into())
}
