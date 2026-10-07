# Architecture

The adapter is benchmark-only Python code. Harbor owns sandbox lifecycle and
task verification; the adapter starts the preinstalled `suncode` binary with
the task instruction on stdin. `SUNCODE_*` environment variables are supplied
by Harbor's agent environment and remain controlled by `suncode-config`.

SunCode JSONL is parsed after the one-shot command completes and converted to
ATIF-v1.7 under Harbor's agent logs. The adapter does not open SQLite, call the
Rust SDK, or implement policy.
