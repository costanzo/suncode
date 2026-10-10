# Architecture

The adapter is benchmark-only Python code. Harbor owns sandbox lifecycle and
task verification; the adapter starts the preinstalled `suncode` binary with
the task instruction on stdin. `SUNCODE_*` environment variables are supplied
by Harbor's agent environment and remain controlled by `suncode-config`.

SunCode JSONL is parsed after the one-shot command completes and converted to
ATIF-v1.7 under Harbor's agent logs. The adapter does not open SQLite, call the
Rust SDK, or implement policy.

The adapter resolves the binary through Harbor's agent environment precedence.
It preserves stdout, stderr, and a trajectory before classifying nonzero CLI
exit codes. The converter supplies required ATIF messages, encodes structured
tool observations as JSON text, deduplicates assistant/requested tool events,
and maps cumulative SunCode usage to ATIF aggregate fields. Focused checks
validate representative trajectories against Harbor 0.24.0 when installed.

Task image architecture must match the installed CLI. On an ARM64 host,
AMD64-only prebuilt images require an AMD64 binary or Harbor's source-image
build option. Local Docker mounts do not install binaries in cloud environments.
