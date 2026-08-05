# Test Plan

## Structural checks

- List all repository files excluding `.git`.
- Confirm every expected `.agents` index and requirement file exists.
- Confirm no legacy tool path or directive remains.

## Repository checks

- Run `git diff --check`.
- Inspect `git status --short` and the diff summary.
- Recheck ignored and trackable representative paths.

## Result

See the final task handoff for executed command results.