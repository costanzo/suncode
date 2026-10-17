# Requirement

## Background

SunCode currently provides project-scoped Java, JavaScript/TypeScript, and Python semantic analysis through fixed jdtls, vtsls, and basedpyright catalog entries. Rust projects need the same five bounded read-only semantic tools through rust-analyzer.

## Goals

- Add rust-analyzer as the proposed fourth built-in language-server type.
- Design a focused project configuration surface before production implementation.
- Detect the rust-analyzer executable and report the available Rust/Cargo toolchain.
- Expose stable Cargo analysis and diagnostic choices without recreating rust-analyzer's full configuration surface.

## Non-goals

- Bundling or installing Rust, Cargo, rustup, rust-analyzer, targets, or project dependencies.
- Exposing arbitrary launch arguments, raw initialization JSON, completion, rename, formatting, code actions, or editor commands.
- Bundling rust-analyzer or changing the shared five-tool semantic surface.

## Requirements

- The catalog identifies rust-analyzer as Rust semantic analysis with independent installation detection.
- Project configuration includes enablement, executable, root markers, Cargo feature policy, all-target analysis, build scripts, procedural macros, check-on-save behavior, check command, optional compilation target, write-only environment changes, and timeouts.
- The detail page shows detected Rust and Cargo toolchain versions as read-only runtime facts.
- SunCode owns stdio launch behavior, the Rust language id, launch construction, settings mapping, validation, and defaults.
- Save and Reset remain disabled until the normalized draft differs from the loaded project configuration.
- Runtime failure retains configuration and exposes an actionable Retry.

## Edge cases

- rust-analyzer is supplied by rustup rather than installed as a standalone executable.
- A monorepo contains several Cargo workspaces or a `rust-project.json` root.
- Build scripts or procedural macros fail, hang, or require unavailable host dependencies.
- All-feature or all-target analysis is expensive for a large workspace.
- A custom compilation target is not installed for the selected toolchain.

## Acceptance criteria

- The design-system catalog shows jdtls, vtsls, basedpyright, and rust-analyzer.
- A direct route opens the rust-analyzer detail page.
- The page covers default, custom feature, custom target, dirty, disabled, indexing, ready, failed, and retry states.
- Light, dark, and constrained-width layouts remain usable.
- Production Rust, SDK bindings, persistence, and Avalonia behavior match the approved design.

## Resolved decisions

- rust-analyzer records use configuration version 5 with a nested typed settings object.
- ABI 22 adds the typed `configure_rust_analyzer_language_server` operation.
- Detection requires a successful `rust-analyzer --version`, so a rustup shim without the component remains not detected.
