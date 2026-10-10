# SunCode Harbor Adapter

This directory is benchmark tooling, not a production SunCode dependency. The
adapter runs the native `suncode` CLI inside Harbor's task container and writes
`agent/suncode.jsonl`, `agent/suncode.stderr.txt`, and an ATIF-v1.7
`agent/trajectory.json`. Failed CLI commands retain their output before Harbor
classifies the failure.

## Prerequisites

Run the examples from the repository root. Docker must be running. Install the
Harbor version used to validate this adapter:

```bash
uv tool install --python 3.12 'harbor==0.24.0'
docker info
```

The adapter checks for an existing binary; it does not install one. A macOS
binary cannot run in the Linux task container. The build below uses Linux ARM64
for Docker on Apple Silicon, a separate Cargo output directory, and Debian
Bookworm to avoid requiring a newer glibc than the Ubuntu 24.04 smoke image.
For an AMD64 task container, build with `--platform linux/amd64` and copy the
library from `/usr/lib/x86_64-linux-gnu/` instead.

```bash
mkdir -p .codex/terminal-bench/lib

docker run --rm --platform linux/arm64 \
  -v "$PWD:/src" -w /src \
  rust:bookworm bash -c '
    set -e
    apt-get update
    apt-get install -y pkg-config libxkbcommon-dev
    CARGO_TARGET_DIR=/src/.codex/terminal-bench-build \
      cargo build --release --locked --manifest-path apps/cli/Cargo.toml
    cp /src/.codex/terminal-bench-build/release/suncode /src/.codex/terminal-bench/
    cp -L /usr/lib/aarch64-linux-gnu/libxkbcommon.so.0 \
      /src/.codex/terminal-bench/lib/
  '

docker run --rm --platform linux/arm64 \
  -v "$PWD/.codex/terminal-bench:/opt/suncode:ro" \
  -e LD_LIBRARY_PATH=/opt/suncode/lib \
  ubuntu:24.04 /opt/suncode/suncode --help
```

The SDK still compiles its Computer Use dependency, even though the CLI host
disables that capability. Task images must match the binary's architecture and
supply its dynamic runtime dependencies. Check the build with `ldd` if another
task image cannot load it.

## Local smoke

Supply a provider key in the host environment. The example uses DeepSeek Flash;
other providers require a matching SunCode model ID and credential variable.
Use Harbor's literal `${VARIABLE}` reference so the actual credential does not
appear in command arguments or saved job configuration. Harbor can load an
ignored `.env` file with `--env-file .env`.

```bash
PYTHONPATH="$PWD" harbor run \
  --env-file .env \
  -d hello-world@1.0 \
  -e docker \
  -a benchmarks.harbor.suncode_agent:SunCodeAgent \
  -m deepseek/deepseek-v4-flash \
  --mounts "[{\"type\":\"bind\",\"source\":\"$PWD/.codex/terminal-bench\",\"target\":\"/opt/suncode\",\"read_only\":true}]" \
  --ae SUNCODE_AGENT_BINARY=/opt/suncode/suncode \
  --ae LD_LIBRARY_PATH=/opt/suncode/lib \
  --ae 'SUNCODE_DEEPSEEK_API_KEY=${SUNCODE_DEEPSEEK_API_KEY}' \
  --ae SUNCODE_DATA_DIRECTORY=/tmp/suncode-data \
  --ae SUNCODE_NON_INTERACTIVE=true \
  --ae SUNCODE_FULL_CONTROL=true \
  --ae SUNCODE_TOOL_ALLOWLIST=read,glob,grep,write,edit,bash,todowrite,skill \
  --ae SUNCODE_TURN_TIMEOUT_MS=1800000 \
  --ae SUNCODE_BASH_TIMEOUT_MS=600000 \
  --ae SUNCODE_TOOL_CALL_LIMIT=256 \
  --ae SUNCODE_LOG_OUTPUT=none \
  --jobs-dir jobs/terminal-bench \
  --job-name suncode-hello-smoke \
  -l 1 -k 1 -n 1
```

`SUNCODE_FULL_CONTROL` preauthorizes policy-mediated operations inside the task
container; permanent hard-deny rules still apply. The CLI cannot answer
interactive approval or question prompts. Each trial has its own container and
SunCode data directory. This run does not inherit desktop settings or stored
credentials.

## Terminal-Bench

After smoke succeeds, use the same command with `-d terminal-bench@2.0`, a new
job name, and optionally `--include-task-name cancel-async-tasks` for a specific
case. Keep `-l 1` for an initial real task. Remove `-l 1` and the task filter to
run the full dataset; `-k` controls attempts per task and `-n` concurrency.

Some Terminal-Bench prebuilt images are AMD64-only. On Apple Silicon, add
`--force-build` to build the task's Dockerfile for the local ARM64 daemon, or
provide an AMD64 CLI for the prebuilt container. An ARM64 CLI inside an AMD64
image can fail with `cannot execute: required file not found` before contacting
the provider. Check architecture compatibility for every task; source builds
may themselves contain architecture-specific dependencies.

Cloud environments require separately installing or uploading the matching
Linux binary and runtime dependencies. A local Docker bind mount does not
provision a Modal container.

Inspect `result.json` and the trial's verifier output to determine success.
Setup, authentication, provider, and timeout errors must be distinguished from
a completed task receiving reward zero. A passing hello-world smoke does not
establish Terminal-Bench performance.

SunCode's file tools require project-relative paths, even when task instructions
name absolute destinations. An absolute `write` path can produce `scope_denied`
and terminate the CLI turn. The adapter records this as an agent error; it does
not rewrite tool calls or bypass the Rust project boundary.

## Focused verification

```bash
python3 -m unittest benchmarks.harbor.test_trajectory benchmarks.harbor.test_agent
```

Use Harbor's Python environment to additionally validate trajectories against
its installed ATIF models and exercise the installed-agent wrapper. Those
checks run when Harbor is importable; the pure converter tests also run
without Harbor.
