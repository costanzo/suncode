use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "suncode",
    version,
    about = "SunCode coding agent CLI",
    after_help = "EXAMPLES:\n  suncode doctor\n  suncode run ./project --prompt \"Explain the failing test\"\n  printf '%s\\n' \"Fix the test\" | suncode run ./project --stdin\n  suncode session list ./project\n  suncode session resume SESSION_ID --prompt \"Continue the fix\"\n\nENVIRONMENT:\n  SunCode-owned environment variables use the SUNCODE_ prefix. Use --help on a command for its detailed options."
)]
pub struct Cli {
    /// Output format. `text` is human-readable; `jsonl` is automation-friendly.
    #[arg(long, global = true, value_enum)]
    pub output: Option<OutputMode>,

    /// Color policy for human-readable output.
    #[arg(long, global = true, value_enum)]
    pub color: Option<ColorMode>,

    /// Logical local user ID used for SDK ownership and data-directory scope.
    #[arg(long, global = true, value_name = "USER_ID")]
    pub user_id: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inspect embedded agent, database, provider, and host capability health.
    Doctor,
    /// List configured models, capabilities, and availability.
    Models,
    /// Run one coding turn against a project.
    #[command(
        after_help = "EXAMPLES:\n  suncode run --prompt \"Review this project\"\n  printf '%s\\n' \"Fix the build\" | suncode run ./project --stdin\n  suncode --output jsonl run ./project --prompt \"Summarize the diff\"\n\nThe command creates a new primary session. Use `session resume` to add a turn to an existing session."
    )]
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
    /// Manage provider credentials stored by the Rust SDK.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Inspect effective non-secret configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// List, resume, and archive saved primary sessions.
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// List configured/not-configured status without revealing secrets.
    List,
    /// Read and store a provider credential without terminal echo.
    Set {
        /// Provider ID, for example `deepseek`, `openai`, or `claude`.
        provider: String,
    },
    /// Remove a provider credential.
    Remove {
        /// Provider ID whose stored credential should be removed.
        provider: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// List effective global non-secret settings and redacted status values.
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
    Archive {
        /// Stable session ID returned by `session list` or `run` JSONL events.
        session_id: String,
    },
    /// Submit one new turn to an existing primary session.
    #[command(
        after_help = "EXAMPLES:\n  suncode session resume SESSION_ID --prompt \"Continue the previous task\"\n  printf '%s\\n' \"Run the tests again\" | suncode session resume SESSION_ID --stdin\n  suncode --output jsonl session resume SESSION_ID --prompt \"Summarize the result\"\n\nResume reopens archived primary sessions and reuses durable conversation context. It returns status 4 instead of bypassing a pending approval or structured question."
    )]
    Resume {
        /// Stable primary session ID returned by `session list` or a previous turn.
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

    #[test]
    fn help_contains_root_and_resume_guidance() {
        let root_help = Cli::try_parse_from(["suncode", "--help"])
            .unwrap_err()
            .to_string();
        assert!(root_help.contains("suncode session resume SESSION_ID"));
        assert!(root_help.contains("SUNCODE_"));

        let resume_help = Cli::try_parse_from(["suncode", "session", "resume", "--help"])
            .unwrap_err()
            .to_string();
        assert!(resume_help.contains("Continue the previous task"));
        assert!(resume_help.contains("pending approval"));
    }
}
