mod args;
mod commands;
mod config;
mod error;
mod output;

use std::process::ExitCode;

use clap::Parser;
use suncode_sdk::{AsyncAgentSdk, SdkHostCapabilities, SdkOpenOptions};

use crate::{
    args::{Cli, OutputMode},
    config::CliConfig,
    error::CliError,
    output::{write_error, write_report},
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

    let command_result = commands::execute(&sdk, &cli.command).await;
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
