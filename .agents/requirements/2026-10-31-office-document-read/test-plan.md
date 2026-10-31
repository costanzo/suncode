# Test Plan

## Scope

Format detection, parser normalization, query output, and tool descriptions for PDF plus the seven Office formats.

## Unit tests

- Detect OOXML, ODS, and CFB Word/PowerPoint/Excel signatures.
- Verify normalized format names and workbook output format.
- Verify range and search behavior remains bounded.

## Integration and conformance tests

Use minimal generated or in-memory fixtures for DOCX/PPTX/XLSX/ODS and checked-in binary fixtures where legacy formats require them.

## Regression checks

Run `cargo test -p suncode-documents` and the relevant tools tests.

## Manual checks

Inspect representative `document_inspect` and `spreadsheet_read` JSON for every supported format.

## Commands and results

- `cargo test -p suncode-documents`: passed (2 focused tests plus doc tests).
- `cargo test -p suncode-tool`: passed (60 tests plus doc tests).
- `cargo check -p suncode-tool -p suncode-core`: passed.
- `cargo check --manifest-path ../sdks/c/Cargo.toml`: passed.
- `git diff --check`: passed.

## Residual risks

Parser-specific handling of malformed legacy Office files and exact legacy slide/block boundaries remain release verification items because the repository has no checked-in binary DOC/PPT/XLS fixtures.
