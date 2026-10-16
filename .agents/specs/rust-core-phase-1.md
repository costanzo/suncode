# Rust Core Phase 1

## Build output

The `agent/` workspace declares every external and internal dependency once in `[workspace.dependencies]`, with package `version` and `edition` in `[workspace.package]`. Member crates use `name.workspace = true` and may only add `features`. Workspace entries carry the version, source, `default-features = false` where needed, and only the features every member requires. Each dependency therefore has a single version across the workspace. All outbound HTTP clients use the shared `suncode-http` crate on that single reqwest version.


The agent, Rust facade, C binding, and CLI keep separate manifests and lockfiles but share the repository-root `target/` through `.cargo/config.toml`. Cargo must run from the repository root or a descendant to discover this configuration; callers outside the repository can pass its path with `--config`. Dev/test profiles retain line-table debug information (`debug = 1`) and Cargo's default incremental compilation. Release profiles remain unchanged. Rust CI disables incremental compilation and caches the shared output; CLI release packaging reads `target/<triple>/release`.

The managed SDK invokes locked Cargo builds with the repository root as the working directory and an explicit shared target directory. Its native-library copy source uses the same directory. `SunCodeCargoTargetDirectory` overrides `CARGO_TARGET_DIR`, which overrides the root `target/` default; relative overrides resolve from the repository root. This changes build placement only, with no SDK or runtime contract change.

## Source Control ownership

The Rust operations package has no Git implementation or libgit2 dependency. Former `git/status` and `git/diff-file` names fail with `method_unavailable`. Avalonia owns optional read-only system Git inspection; agent process policy, repository ignore rules, checkpoints, and undo remain in Rust.

## Operations

The Rust operations module is provided by the `suncode-tool` package, linked into the runtime, and called in-process.

It enforces canonical project paths, bounded reads and search, preconditioned file mutations, checkpoint capture and restore, bounded process execution with filtered environment handling, and bounded approval-gated HTTP(S) text retrieval with same-origin redirect enforcement.

Content search is implemented with embedded ripgrep crates (`ignore`, `globset`, `grep-regex`, and `grep-searcher`). `search/find` uses Rust regular expressions, ripgrep-style standard ignore/hidden-file traversal, project-relative include globs, and a 2 MiB per-file bound. It returns relative paths, one-based byte columns, line previews, and bounded `truncated` results without requiring an external `rg` executable.

`tool/webfetch` accepts textual HTTP(S) responses up to 5 MiB, decodes BOM or declared charsets, and converts HTML to Markdown or plain text through a structured parser. Its model preview is 64 KiB; larger converted output is retained in managed artifact storage. It rejects embedded URL credentials, cross-origin redirects, binary content, and image content.

The model-facing tool registry also includes `question` and `todowrite`. Core validates one to eight prompts with labeled options, pauses the current continuation in the turn recovery snapshot, and emits `question.asked`. The embedded SDK exposes ordered answer and reject calls; answers are validated against option labels, `multiple`, and `custom` before the continuation resumes. `todowrite` validates a complete replacement list of up to 100 prioritized items with at most one `in_progress` item, stores it in the current turn continuation, emits `todo.updated`, and transactionally replaces the authoritative `session_turn_todo` rows. A session snapshot returns `pendingQuestion` while the request is waiting and includes current todos in normalized conversation turns; the tool result in `session_tool_use` is retained as call history only. Restart recovery selects the question continuation path when a resolved question is found in `resuming` state.

Browser Use is implemented by the Rust-owned `suncode-browser` client using `chromiumoxide` against the Avalonia-owned CEF remote debugging endpoint. Rust owns target discovery, request timeouts, policy, lifecycle, artifacts, cancellation, and control ownership; CEF owns Chromium and UI.

Before every provider request, core reloads `<project>/AGENTS.md` as bounded system context. After a successful project file read, it canonicalizes the target, discovers unseen nested `AGENTS.md` files nearest-first up to the project root, and adds them to `repository_instructions` in the normalized tool result. Root instructions are not duplicated in read results; direct instruction-file reads, dependencies, path escapes, non-UTF-8 files, empty files, and files over 32 KiB do not trigger automatic attachment. One read is capped at 16 nested files and 64 KiB of nested instruction content.

Model capability metadata includes `supports_reasoning_effort` and the persisted per-model `reasoning_efforts` catalog, projected as `capabilities.reasoning_effort` and `reasoning_efforts`. `submit_turn` accepts only an effort advertised by the selected model, stores it in the continuation snapshot, and the OpenAI Responses-compatible adapter maps it to the `reasoning.effort` request field.
