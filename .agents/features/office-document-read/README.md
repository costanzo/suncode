# Office Document Reading

**Status:** Implemented with focused Office/PDF tests; legacy binary Office conformance remains open.

The Rust document tools read Office and OpenDocument files through one normalized `suncode-documents` facade.

- Word `.doc` and `.docx` files use `office_oxide` and expose bounded block reads and searches for headings, paragraphs, list items, and tables.
- PowerPoint `.ppt` and `.pptx` files use `office_oxide` and expose bounded slide reads and searches, including hidden-slide state and speaker notes when exposed by the parser.
- Excel `.xls` and `.xlsx`, plus OpenDocument Spreadsheet `.ods`, use calamine and expose typed cell ranges, formulas where available, sheet metadata, and searches.
- PDF parsing uses `pdf-extract` for existing layout-aware extraction and a targeted `lopdf` page adapter for the named Chinese encodings `UniGB-UCS2-H` and `UniGB-UTF16-H`.

The parser adapters remain private to `suncode-documents`; tool, SDK, and client layers consume the existing normalized JSON shapes. Parser output changes advance the document fingerprint version so cached parses are not reused across parser migrations.

## Scope and boundaries

The model-facing tools are `document_inspect`, `document_read`, `document_search`, and `spreadsheet_read`. They report the detected format (`pdf`, `doc`, `docx`, `ppt`, `pptx`, `xls`, `xlsx`, or `ods`) through the existing normalized responses. Detection uses file content, including compound-file stream names and package metadata, rather than trusting filename extensions. Workbook ranges use `spreadsheet_read`; `document_read` addresses pages, slides, or Word blocks.

The tools use the audited project/dependency read path. The document facade rejects inputs over 64 MiB, caches parses by path and content fingerprint, checks cancellation around parsing, and maps parser failures to structured errors. Cancellation is cooperative around the synchronous parser call, not an interruptible parser sandbox. Encrypted OOXML containers are explicitly rejected; encrypted or malformed legacy files still need format-specific verification. Missing XLS/ODS metadata uses normalized defaults.

This is text and cell extraction, not Office editing, document creation, visual page rendering, OCR, or formula recalculation. Formatting and complex visual reading order are not preserved as an editable document. No new client-facing document API is implied.

## Verification and remaining work

Focused verification covers generated DOCX, PPTX, XLSX, and ODS fixtures. Legacy DOC, PPT, and XLS adapters exist, but representative binary detection/parsing fixtures, malformed/encrypted samples, exact block/slide boundaries, table flattening, and legacy presentation notes remain unverified. The former delivery's all-format acceptance criterion was therefore only partially verified; its completed checklist is not evidence of full legacy-format conformance.

Evidence: `agent/crates/documents/src/{detect,office,xlsx,query,tests}.rs` and `agent/crates/tools/src/{definitions/documents,dispatch}.rs`. Run `cargo test --manifest-path agent/Cargo.toml -p suncode-documents` and `cargo test --manifest-path agent/Cargo.toml -p suncode-tool`. These source and test references replace the completed 2026-10-31 delivery package.

## Chinese PDF text

Pages using `UniGB-UCS2-H` or `UniGB-UTF16-H` use the Rust-owned `lopdf` text decoder. Valid explicit `ToUnicode` maps take precedence; invalid maps return `parse_failed` instead of silently substituting another encoding. Direct, indirect, and inherited font resources are resolved before choosing the page adapter. Other pages retain the existing layout-aware extraction path.

Unsupported named Type 0 encodings in page font resources return `parse_failed` with the encoding name. The targeted adapter is not full Adobe-GB1/Adobe-CNS1 CMap support. It rejects pages combining these Chinese encodings with Form XObjects, or other composite fonts lacking a usable `ToUnicode` map, rather than silently omitting their text. Image XObjects remain compatible. The alternate decoder follows content-stream order; equivalent visual reading order for complex layouts is not guaranteed.

Generated PDF regression fixtures cover both encodings, UTF-16 supplementary characters, explicit and invalid Unicode maps, page counts and empty-page warnings, character-bounded reading, search offsets and pagination, unsupported encodings, image/Form XObjects, damaged content references, and preservation of the existing extractor on other pages. Document fingerprints use parser version 3.
