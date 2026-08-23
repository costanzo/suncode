# Test Plan

- `cargo test -p suncode-tool`
- `cargo test -p suncode-runtime`
- Verify read range output and out-of-range errors.
- Verify glob excludes ignored and hidden files.
- Verify edit preserves BOM/CRLF and rejects overlapping ranges.
- Verify a non-zero bash exit produces `status=failed` and failed tool state.
- Run `git diff --check`.
