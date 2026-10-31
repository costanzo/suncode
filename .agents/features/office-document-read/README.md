# Office Document Reading

The Rust document tools read Office and OpenDocument files through one normalized `suncode-documents` facade.

- Word `.doc` and `.docx` files use `office_oxide` and expose bounded block reads and searches for headings, paragraphs, list items, and tables.
- PowerPoint `.ppt` and `.pptx` files use `office_oxide` and expose bounded slide reads and searches, including hidden-slide state and speaker notes when present.
- Excel `.xls` and `.xlsx`, plus OpenDocument Spreadsheet `.ods`, use calamine and expose typed cell ranges, formulas where available, sheet metadata, and searches.
- PDF parsing remains on the existing PDF adapter.

The parser adapters remain private to `suncode-documents`; tool, SDK, and client layers consume the existing normalized JSON shapes. Parser output changes advance the document fingerprint version so cached parses are not reused across parser migrations.

Focused verification covers generated DOCX, PPTX, XLSX, and ODS fixtures. Legacy DOC, PPT, and XLS files use the same production adapters but still need representative binary fixtures in broader release verification.
