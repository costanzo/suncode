# Requirements

Completed delivery packages are consolidated into [`../features/`](../features/) and current specifications under [`../specs/`](../specs/) or `contracts/`. Dated requirement packages are temporary and are removed at closeout; this directory is not a delivery archive.

There are no active dated delivery packages after the 2026-11-04 consolidation. Only this lifecycle policy and `_template/` remain for future work. Outstanding verification is recorded in the owning feature, including legacy Office fixtures in [`office-document-read`](../features/office-document-read/README.md).

Use the template for a new substantial delivery only when the work needs a temporary durable record. Once implemented, promote stable behavior to `features/` and current technical facts to `specs/` or `contracts/`, record important tradeoffs in `DECISIONS.md`, then remove the dated package. Any incomplete follow-up stays documented in the owning feature with its current status.
