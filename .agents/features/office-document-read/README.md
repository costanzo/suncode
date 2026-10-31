# Office Document Reading

The Rust document tools read Office and OpenDocument files through one normalized `suncode-documents` facade.

- Word `.doc` and `.docx` files use `office_oxide` and expose bounded block reads and searches for headings, paragraphs, list items, and tables.
- PowerPoint `.ppt` and `.pptx` files use `office_oxide` and expose bounded slide reads and searches, including hidden-slide state and speaker notes when present.
- Excel `.xls` and `.xlsx`, plus OpenDocument Spreadsheet `.ods`, use calamine and expose typed cell ranges, formulas where available, sheet metadata, and searches.
- PDF parsing uses `pdf-extract` for existing layout-aware extraction and a targeted `lopdf` page adapter for the named Chinese encodings `UniGB-UCS2-H` and `UniGB-UTF16-H`.

The parser adapters remain private to `suncode-documents`; tool, SDK, and client layers consume the existing normalized JSON shapes. Parser output changes advance the document fingerprint version so cached parses are not reused across parser migrations.

Focused verification covers generated DOCX, PPTX, XLSX, and ODS fixtures. Legacy DOC, PPT, and XLS files use the same production adapters but still need representative binary fixtures in broader release verification.

## Chinese PDF text

Pages using `UniGB-UCS2-H` or `UniGB-UTF16-H` use the Rust-owned `lopdf` text decoder. Valid explicit `ToUnicode` maps take precedence; invalid maps return `parse_failed` instead of silently substituting another encoding. Direct, indirect, and inherited font resources are resolved before choosing the page adapter. Other pages retain the existing layout-aware extraction path.

Unsupported named Type 0 encodings in page font resources return `parse_failed` with the encoding name. The targeted adapter is not full Adobe-GB1/Adobe-CNS1 CMap support. It rejects pages combining these Chinese encodings with Form XObjects, or other composite fonts lacking a usable `ToUnicode` map, rather than silently omitting their text. Image XObjects remain compatible. The alternate decoder follows content-stream order; equivalent visual reading order for complex layouts is not guaranteed.

Generated PDF regression fixtures cover both encodings, UTF-16 supplementary characters, explicit and invalid Unicode maps, page counts and empty-page warnings, character-bounded reading, search offsets and pagination, unsupported encodings, image/Form XObjects, damaged content references, and preservation of the existing extractor on other pages. Document fingerprints use parser version 3.
