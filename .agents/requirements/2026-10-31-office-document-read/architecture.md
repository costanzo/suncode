# Architecture

## Current state

`suncode-documents` owns format detection, parser adapters, an in-memory cache, and normalized query output. The tools crate only calls this facade.

## Proposed design

Use `office_oxide::Document` and its format-agnostic IR for DOC/DOCX/PPT/PPTX. Use calamine's runtime `Sheets` reader for XLS/XLSX/ODS. Convert both libraries into the existing internal `ParsedDocument`, `ParsedWorkbook`, `Unit`, and `Sheet` types.

## Boundaries and dependencies

Only `suncode-documents` depends directly on parser libraries. The tools and agent layers retain their current dependency on the normalized facade.

## Data and control flow

Bytes are size-checked, detected, parsed once, cached by path and fingerprint, normalized, and then queried for inspection, bounded reads, searches, or spreadsheet ranges.

## Security and failure handling

Existing size and cancellation checks remain around parsing. Parser failures are mapped to `parse_failed`; unsupported or encrypted inputs retain explicit error codes. Parser-specific types never cross the crate boundary.

## Compatibility and migration

The tool names and request fields stay unchanged. Response `format` values expand to `doc`, `docx`, `ppt`, `pptx`, `xls`, `xlsx`, and `ods`.

## Risks and rollback

The main risk is changed block/slide boundaries from the new IR. The old parser files can be retained until focused fixtures establish parity, then removed with the dependency cleanup.

## Open questions

Legacy presentation notes and exact table flattening require fixture verification.
