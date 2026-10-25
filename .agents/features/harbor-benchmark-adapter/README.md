# Harbor Benchmark Adapter

**Status:** Implemented; focused converter and adapter tests available

`benchmarks/harbor/` provides a Harbor `BaseInstalledAgent` adapter for running the native Rust CLI in Terminal-Bench tasks. It passes task instructions to `suncode run` through stdin, preserves JSONL and stderr artifacts, and converts the resulting trace into ATIF v1.7 at Harbor's expected agent log path.

This is isolated benchmark tooling. It is not a production dependency, does not install or provision the CLI binary, and does not bypass the CLI's Rust policy or tool boundary. The container must provide a matching Linux binary and runtime libraries. Use the focused checks in `benchmarks/harbor/README.md`; Harbor-specific validation runs when Harbor is installed.
