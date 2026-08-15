# Requirement

## Background

The Rust `search/find` operation currently walks every project file and performs an in-memory literal substring search. The public grep tool promises text or regular-expression search but does not implement regular expressions or ripgrep-compatible ignore traversal.

## Goals

- Embed ripgrep's reusable Rust crates in the operations crate.
- Implement regular-expression content search without requiring an installed `rg` executable.
- Respect ripgrep-style hidden-file and ignore-file traversal defaults.
- Preserve the existing project boundary, bounded result count, file-size bound, and JSON response shape.

## Non-goals

- Reproduce the `rg` command-line interface or every ripgrep flag.
- Add PCRE2 support.
- Change the public tool name or operation method.
- Change glob-only search in this delivery.

## Requirements

- Searches remain confined to the canonical opened project.
- Invalid regular expressions return `invalid_arguments` without starting a search.
- The include pattern filters project-relative paths.
- Search skips hidden, ignored, binary, non-UTF-8, and files larger than 2 MiB.
- Results include relative path, one-based line, one-based byte column, and a bounded preview.
- `max_results` remains clamped to 1-500 and reports truncation when the limit stops search.

## Edge cases

- Multiple matches on one line count as separate results.
- A match at the result boundary marks the response truncated only when more matching data exists.
- Ignore files and hidden paths follow the embedded traversal configuration.

## Acceptance criteria

- Focused operations tests cover regular expressions, ignore behavior, include filtering, multiple matches, and result truncation.
- The operations and runtime crates compile and test successfully.
