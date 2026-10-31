# Requirement

## Background

The document tools currently read DOCX with `docx-rs`, PPTX with a local XML parser, and XLSX with calamine. Legacy Word/PowerPoint/Excel files and ODS workbooks are not supported.

## Goals

- Read DOC and DOCX with `office_oxide`.
- Read PPT and PPTX with `office_oxide`.
- Read XLS, XLSX, and ODS with calamine.
- Preserve the existing document tool response shape and bounded read/search behavior.
- Keep parser dependencies behind `suncode-documents`.

## Non-goals

- Writing or editing Office files.
- Changing the document tool names or request schemas.
- Adding new SDK or client-facing document APIs.

## Requirements

- Format detection must distinguish DOC, PPT, and XLS compound files and ODS packages.
- Word and presentation content must map to the existing block/slide model.
- Workbook responses must identify XLS, XLSX, or ODS rather than always reporting XLSX.
- Existing size, cancellation, path, and parse error boundaries remain in force.

## Edge cases

- Encrypted or password-protected Office files must fail closed.
- Corrupt, truncated, mislabeled, and unsupported Office packages must return parse or unsupported-format errors.
- Hidden slides and speaker notes remain represented for presentations where the parser exposes them.
- ODS and XLS sheets may not provide all XLSX metadata; missing metadata must use the existing normalized defaults.

## Acceptance criteria

- Focused tests cover detection and representative parsing for DOC, DOCX, PPT, PPTX, XLS, XLSX, and ODS.
- `document_inspect`, `document_read`, `document_search`, and `spreadsheet_read` report the actual format.
- `cargo test -p suncode-documents` passes.

## Open questions

- Exact block boundaries for legacy DOC and PPT are determined by the `office_oxide` IR and verified by fixtures.
