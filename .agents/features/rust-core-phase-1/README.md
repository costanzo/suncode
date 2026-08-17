# Rust Core Phase 1

The audited Rust operations module in the `suncode-tool` package handles canonical path checks, bounded reads and search, read-only Git status and file diffs, preconditioned writes and edits, checkpoint capture and restore, artifact handling, and bounded process execution.

It is called in-process by the runtime and reports operation results through typed runtime DTOs.

Its content search uses embedded ripgrep libraries with bounded Rust-regex matching and ripgrep-compatible standard ignore filters; the desktop runtime does not depend on a system-installed `rg` binary.

Its Git inspection uses `git2` with vendored libgit2. It discovers repositories containing the opened project, filters all paths back to that project, and returns bounded structured status, hunks, lines, and patch text without requiring an installed Git executable. Git mutations, remotes, and credentials are not part of this read-only slice.
