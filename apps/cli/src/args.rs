use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "suncode", version, about = "SunCode coding agent CLI")]
pub struct Cli {
    #[arg(long, global = true, value_enum)]
    pub output: Option<OutputMode>,

    #[arg(long, global = true, value_enum)]
    pub color: Option<ColorMode>,

    #[arg(long, global = true)]
    pub user_id: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inspect the embedded agent and local configuration health.
    Doctor,
    /// List configured models and availability.
    Models,
    /// Manage provider credentials.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Inspect effective non-secret configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// List provider credential status.
    List,
    /// Read and store a provider credential without terminal echo.
    Set { provider: String },
    /// Remove a provider credential.
    Remove { provider: String },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// List effective global non-secret settings.
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputMode {
    Text,
    Jsonl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_foundation_commands_and_global_options() {
        let cli = Cli::try_parse_from([
            "suncode", "--output", "jsonl", "--color", "never", "auth", "remove", "claude",
        ])
        .unwrap();
        assert_eq!(cli.output, Some(OutputMode::Jsonl));
        assert_eq!(cli.color, Some(ColorMode::Never));
        assert!(matches!(
            cli.command,
            Command::Auth {
                command: AuthCommand::Remove { provider }
            } if provider == "claude"
        ));
    }

    #[test]
    fn rejects_unimplemented_commands() {
        assert!(Cli::try_parse_from(["suncode", "run"]).is_err());
        assert!(Cli::try_parse_from(["suncode", "chat"]).is_err());
    }
}
