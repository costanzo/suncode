# SunCode Harbor Adapter

This directory is benchmark tooling, not a production SunCode dependency. The
adapter runs the already-installed `suncode` binary inside Harbor's task
sandbox and writes `agent/trajectory.json` in ATIF-v1.7 format.

Build and expose the CLI binary in the Harbor task image, then run a smoke job
from the repository root:

```bash
export SUNCODE_AGENT_BINARY=/usr/local/bin/suncode
harbor run \
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
