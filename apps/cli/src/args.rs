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
    /// Run one coding turn against a project.
    Run {
        /// Project directory, defaulting to the current directory.
        path: Option<String>,
        /// Prompt text. Exactly one of `--prompt` and `--stdin` is required.
        #[arg(long, conflicts_with = "stdin", required_unless_present = "stdin")]
        prompt: Option<String>,
        /// Read the prompt from stdin.
        #[arg(long, conflicts_with = "prompt", required_unless_present = "prompt")]
        stdin: bool,
        /// Select a model for this turn.
        #[arg(long, env = "SUNCODE_MODEL")]
        model: Option<String>,
        /// Select a reasoning effort for this turn.
        #[arg(long, env = "SUNCODE_REASONING_EFFORT")]
        reasoning_effort: Option<String>,
    },
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
    /// Inspect and manage saved sessions.
    Session {
        #[command(subcommand)]
        command: SessionCommand,
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

#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    /// List primary sessions for a project.
    List {
        /// Project directory, defaulting to the current directory.
        path: Option<String>,
    },
    /// Archive a primary session.
    Archive { session_id: String },
    /// Submit one new turn to an existing primary session.
    Resume {
        session_id: String,
        /// Prompt text. Exactly one of `--prompt` and `--stdin` is required.
        #[arg(long, conflicts_with = "stdin", required_unless_present = "stdin")]
        prompt: Option<String>,
        /// Read the prompt from stdin.
        #[arg(long, conflicts_with = "prompt", required_unless_present = "prompt")]
        stdin: bool,
        /// Override the model for this turn.
        #[arg(long, env = "SUNCODE_MODEL")]
        model: Option<String>,
        /// Override the reasoning effort for this turn.
        #[arg(long, env = "SUNCODE_REASONING_EFFORT")]
        reasoning_effort: Option<String>,
    },
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
        assert!(Cli::try_parse_from(["suncode", "chat"]).is_err());
    }

    #[test]
    fn run_requires_exactly_one_prompt_source() {
        assert!(Cli::try_parse_from(["suncode", "run"]).is_err());
        assert!(Cli::try_parse_from(["suncode", "run", "--prompt", "hello", "--stdin"]).is_err());
        assert!(Cli::try_parse_from(["suncode", "run", "--prompt", "hello"]).is_ok());
    }

    #[test]
    fn parses_session_commands() {
        assert!(matches!(
            Cli::try_parse_from(["suncode", "session", "list", "/tmp/project"])
                .unwrap()
                .command,
            Command::Session {
                command: SessionCommand::List { path: Some(path) }
            } if path == "/tmp/project"
        ));
        assert!(matches!(
            Cli::try_parse_from(["suncode", "session", "archive", "session-1"])
                .unwrap()
                .command,
            Command::Session {
                command: SessionCommand::Archive { session_id }
            } if session_id == "session-1"
        ));
        assert!(matches!(
            Cli::try_parse_from([
                "suncode",
                "session",
                "resume",
                "session-1",
                "--prompt",
                "continue"
            ])
            .unwrap()
            .command,
            Command::Session {
                command: SessionCommand::Resume {
                    session_id,
                    prompt: Some(prompt),
                    stdin: false,
                    ..
                }
            } if session_id == "session-1" && prompt == "continue"
        ));
        assert!(Cli::try_parse_from(["suncode", "session", "resume", "session-1"]).is_err());
    }
}
