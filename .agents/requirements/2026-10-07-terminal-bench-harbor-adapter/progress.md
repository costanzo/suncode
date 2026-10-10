# Progress

- Status: Complete
- Last updated: 2026-10-10

## Completed

- Added Harbor installed-agent adapter.
- Added pure JSONL to ATIF-v1.7 conversion and fixture test.
- Documented binary and sandbox prerequisites.
- Installed Harbor 0.24.0 and built the CLI for Linux ARM64.
- Corrected ATIF conversion and verified five focused adapter/converter tests.
- Preserved failed-command logs and honored Harbor agent-environment binary overrides.
- Verified the SunCode hello-world smoke with reward 1.0 using DeepSeek Flash.
- Identified AMD64 prebuilt task images as incompatible with an ARM64 CLI; documented source-image builds and matching binary alternatives.
- Ran Terminal-Bench 2.0 `cancel-async-tasks` with a source-built ARM64 image. Provider/tool execution started, but an absolute `write` path caused `scope_denied` and a nonzero CLI exit; Harbor recorded reward 0. This is not a full-dataset score.
- Added prompt guidance for translating task absolute file paths to project-relative SunCode file-tool paths, rebuilt the ARM64 binary, and passed `cancel-async-tasks` with reward 1.0.
- Passed Terminal-Bench 2.0 `regex-log` with reward 1.0 using DeepSeek Flash and a source-built ARM64 image; its date-matching verifier test passed without agent exceptions. These individual results are not a full-dataset score.
- Passed Terminal-Bench 2.0 `openssl-selfsigned-cert` with reward 1.0 using the same provider and ARM64 setup; all six verifier tests passed without agent exceptions.
