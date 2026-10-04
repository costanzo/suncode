# CLI Foundation

**Status:** Implemented and focused-tested

The native Rust CLI package lives under `apps/cli` and builds the `suncode` executable. It is an independent package outside the agent workspace that depends only on `sdks/rust` plus argument, terminal, secret-input, async, and serialization libraries. It embeds `suncode_sdk::AsyncAgentSdk` directly on Tokio with Browser and Computer host capabilities disabled; this is a host ceiling, not a change to persisted settings. It does not call the C ABI, open SQLite, contact providers, invoke operation crates, or implement policy. Clap validates syntax only. The normative contract is `contracts/cli.md`.

The CLI shares the ordinary data directory and its single-instance lock: it cannot run while the desktop or another CLI owns the same directory (exit 5); a different `SUNCODE_DATA_DIRECTORY` gives independent local state.

Administrative commands are `doctor`, `models`, `auth list`, `auth set`, `auth remove`, and `config list`. `auth set` requires an interactive terminal and reads the credential without echo; credentials remain SQLite-owned through SDK methods. API-key arguments and environment variables are unsupported. The one-shot conversational command is documented separately in `features/cli-run/`.

Global CLI options currently resolve explicit values over `SUNCODE_OUTPUT`, `SUNCODE_COLOR`, and `SUNCODE_USER_ID`. The default logical user is `os:` plus the local account name; it is an ownership label, not authentication. `--color` is validated but output is not colored yet. SDK bootstrap continues using `SUNCODE_DATA_DIRECTORY`, `SUNCODE_DATABASE_PATH`, and `SUNCODE_NON_INTERACTIVE`. Invalid values fail closed.

Text reports use stdout and errors use stderr. JSONL uses the versioned envelope in `contracts/cli.md`; SDK diagnostics remain on stderr. The CLI opens one embedded SDK, executes one command, consumes explicit shutdown, and only then emits a success report, so a successful result never precedes failed cleanup; a shutdown failure replaces the success report. JSON errors carry only code and message. Human column layout is not a machine contract. Stable exit mappings cover invalid input, unavailable providers/models/credentials, policy outcomes, data-directory contention, and ordinary failure.

`chat`, interactive approvals/questions, TUI, PTY, Browser/Computer UX, daemon/attach, distribution, and general CI policy profiles remain follow-up work. Session list/archive and one-shot resume are documented in dedicated features.
