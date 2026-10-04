mod args;
mod commands;
mod config;
mod error;
mod output;

use std::{panic::AssertUnwindSafe, process::ExitCode};

use futures_util::FutureExt;

use clap::Parser;
use suncode_sdk::{AsyncAgentSdk, SdkHostCapabilities, SdkOpenOptions};

use crate::{
    args::{Cli, OutputMode},
    config::CliConfig,
    error::CliError,
    output::{write_error, write_report, CommandReport},
};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = error.exit_code();
            let _ = error.print();
            return ExitCode::from(u8::try_from(exit_code).unwrap_or(2));
        }
    };
    let config = match CliConfig::resolve(&cli) {
        Ok(config) => config,
        Err(error) => {
            write_error(cli.output.unwrap_or(OutputMode::Text), &error);
            return ExitCode::from(error.exit_code);
        }
    };
    let _color_mode = config.color;

    let sdk = match AsyncAgentSdk::open_with_options(
        &config.user_id,
        SdkOpenOptions {
            host_capabilities: SdkHostCapabilities {
                browser_use: false,
                computer_use: false,
            },
        },
    )
    .await
    {
        Ok(sdk) => sdk,
        Err(error) => {
            let error = CliError::from_business(error);
            write_error(config.output, &error);
            return ExitCode::from(error.exit_code);
        }
    };

    let command_result =
        contain(dispatch(&cli, &config, &sdk), is_turn_command(&cli.command)).await;
    let shutdown_result = sdk.shutdown().await.map_err(CliError::from_business);
    let result = match (command_result, shutdown_result) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(report), Ok(())) => write_report(config.output, &report).map(|_| report),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            write_error(config.output, &error);
            ExitCode::from(error.exit_code)
        }
    }
}

/// Turn commands own interrupt handling inside the turn driver, which cancels the
/// active turn first. Every other command is interrupted directly.
fn is_turn_command(command: &args::Command) -> bool {
    matches!(
        command,
        args::Command::Run { .. }
            | args::Command::Session {
                command: args::SessionCommand::Resume { .. }
            }
    )
}

/// Contains panics and, for non-turn commands, user interrupts so `main` still
/// performs consuming SDK shutdown before exiting.
async fn contain(
    command: impl std::future::Future<Output = Result<CommandReport, CliError>>,
    turn_command: bool,
) -> Result<CommandReport, CliError> {
    let command = AssertUnwindSafe(command).catch_unwind();
    let outcome = if turn_command {
        command.await
    } else {
        tokio::select! {
            outcome = command => outcome,
            _ = tokio::signal::ctrl_c() => return Err(CliError::interrupted()),
        }
    };
    outcome.unwrap_or_else(|_| {
        Err(CliError::internal(
            "command panicked; SDK shutdown attempted",
        ))
    })
}

async fn dispatch(
    cli: &Cli,
    config: &CliConfig,
    sdk: &AsyncAgentSdk,
) -> Result<CommandReport, CliError> {
    match &cli.command {
        args::Command::Run {
            path,
            prompt,
            stdin,
            model,
            reasoning_effort,
        } => {
            commands::run(
                sdk,
                path.as_deref(),
                prompt.as_deref(),
                *stdin,
                model.as_deref(),
                reasoning_effort.as_deref(),
                config.output,
            )
            .await
        }
        args::Command::Session {
            command:
                args::SessionCommand::Resume {
                    session_id,
                    prompt,
                    stdin,
                    model,
                    reasoning_effort,
                },
        } => {
            commands::resume(
                sdk,
                session_id,
                prompt.as_deref(),
                *stdin,
                model.as_deref(),
                reasoning_effort.as_deref(),
                config.output,
            )
            .await
        }
        command => commands::execute(sdk, command).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn contain_converts_a_panic_into_an_internal_error() {
        let result = contain(async { panic!("boom") }, false).await;
        let error = result.expect_err("panic must become an error");
        assert_eq!(error.code, "internal_error");
        assert_eq!(error.exit_code, 1);
    }

    #[tokio::test]
    async fn contain_passes_turn_command_results_through() {
        let report = CommandReport {
            event_type: "test.result",
            data: serde_json::Value::Null,
            text: String::new(),
        };
        let result = contain(async { Ok(report) }, true).await;
        assert_eq!(result.expect("result").event_type, "test.result");
    }
}
