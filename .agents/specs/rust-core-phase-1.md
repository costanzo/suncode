# Rust Core Phase 1

The Rust operations module is provided by the `suncode-tool` package, linked into the runtime, and called in-process.

It enforces canonical project paths, bounded reads and search, preconditioned file mutations, checkpoint capture and restore, and bounded process execution with filtered environment handling.

Content search is implemented with embedded ripgrep crates (`ignore`, `globset`, `grep-regex`, and `grep-searcher`). `search/find` uses Rust regular expressions, ripgrep-style standard ignore/hidden-file traversal, project-relative include globs, and a 2 MiB per-file bound. It returns relative paths, one-based byte columns, line previews, and bounded `truncated` results without requiring an external `rg` executable.
