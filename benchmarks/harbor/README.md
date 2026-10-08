# SunCode Harbor Adapter

This directory is benchmark tooling, not a production SunCode dependency. The
adapter runs the already-installed `suncode` binary inside Harbor's task
sandbox and writes `agent/trajectory.json` in ATIF-v1.7 format.

## Local Docker smoke

Run these commands from the repository root. Docker Desktop must be running,
and the host must have an OpenAI key available. Harbor's task container needs
the Linux ARM64 CLI binary, so build it inside a matching Docker image first:

```bash
export OPENAI_API_KEY='your-openai-key'

docker run --rm --platform linux/arm64 \
  -v "$PWD:/src" -w /src \
  rust:latest \
  cargo build --release --manifest-path apps/cli/Cargo.toml
```

Then run one deterministic Terminal-Bench smoke task locally:

```bash
PYTHONPATH="$PWD" harbor run \
  -t hello-world/hello-world \
  -e docker \
  -a benchmarks.harbor.suncode_agent:SunCodeAgent \
  -m openai/gpt-5.6-sol \
  --mounts "[{\"type\":\"bind\",\"source\":\"$PWD/apps/cli/target/release/suncode\",\"target\":\"/usr/local/bin/suncode\",\"read_only\":true}]" \
  --ae SUNCODE_AGENT_BINARY=/usr/local/bin/suncode \
  --ae SUNCODE_OPENAI_API_KEY="$OPENAI_API_KEY" \
  --ae SUNCODE_NON_INTERACTIVE=true \
  --ae SUNCODE_FULL_CONTROL=true \
  --ae SUNCODE_TOOL_ALLOWLIST=read,glob,grep,write,edit,bash,todowrite,skill \
  --ae SUNCODE_TURN_TIMEOUT_MS=1800000 \
  --ae SUNCODE_BASH_TIMEOUT_MS=600000 \
  --ae SUNCODE_TOOL_CALL_LIMIT=256 \
  --jobs-dir jobs/terminal-bench-smoke \
  --job-name suncode-hello-smoke \
  -l 1 -k 1 -n 1
```

The result is written below `jobs/terminal-bench-smoke/`. A `0.0` reward with
`provider_unconfigured` means the key was not passed into the container; a
task failure after the agent starts is a real benchmark result.

## Full Terminal-Bench run

After the local smoke succeeds, run the full benchmark through the selected
Harbor environment:

```bash
PYTHONPATH="$PWD" harbor run \
  -t terminal-bench/terminal-bench@4.0.0 \
  -e modal \
  -a benchmarks.harbor.suncode_agent:SunCodeAgent \
  -m openai/gpt-5.6-sol \
  --ae SUNCODE_OPENAI_API_KEY="$OPENAI_API_KEY" \
  --ae SUNCODE_NON_INTERACTIVE=true \
  --ae SUNCODE_FULL_CONTROL=true \
  --ae SUNCODE_TOOL_ALLOWLIST=read,glob,grep,write,edit,bash,todowrite,skill \
  --ae SUNCODE_TURN_TIMEOUT_MS=1800000 \
  --ae SUNCODE_BASH_TIMEOUT_MS=600000 \
  --ae SUNCODE_TOOL_CALL_LIMIT=256 \
  -l 1 -k 1 -n 1
```

The adapter expects `SUNCODE_*` values to be present in the task environment.
Pass secrets with Harbor's agent environment mechanism; do not commit them or
write them to a task directory. Each Harbor trial gets its own environment and
therefore its own SunCode data directory when `SUNCODE_DATA_DIRECTORY` is set
by the task image or runner.
