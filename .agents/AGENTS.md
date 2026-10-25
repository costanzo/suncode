# .agents Working Rules

This directory contains durable project knowledge, not tool runtime state.

## Before work

1. Read `.agents/README.md`.
2. Read `PRODUCT.md` and `ARCHITECTURE.md` for system-wide work.
3. Read the relevant feature, specification, and decision records. Use `requirements/` only for a new delivery that needs a durable work record.
4. If documentation conflicts with code, record the discrepancy instead of guessing.

## New requirements

For a new substantial delivery, create `.agents/requirements/YYYY-MM-DD-short-topic/` from `_template/` only when a temporary work record is useful. Keep `progress.md` and `todo.md` current while implementing. At closeout, promote stable behavior into `features/`, current technical facts into `specs/` or `contracts/`, and important tradeoffs into `DECISIONS.md`; then delete the dated delivery package. Do not keep completed requirement packages as a history archive. Preserve important decision history in `DECISIONS.md` instead.

## Boundaries

- Do not recreate legacy tool-specific documentation or brainstorming directories.
- Do not store process IDs, logs, caches, credentials, or personal machine paths here.
- Do not treat proposed architecture as implemented behavior.
- Keep protocol contracts and user-facing product documentation in the repository locations defined by `.agents/ARCHITECTURE.md`, not inside this knowledge base.
- Prefer focused updates; do not copy the same fact into multiple files unless one location is an index.

## Amendments

When a requirement changes after review, edit `requirement.md` and `architecture.md` directly. Only create a separate amendment file when the change is long and self-contained, and in that case state in its header whether it is normative and have the parent document point to it.

An amendment that has been folded into its parent must be marked consolidated and non-normative, so readers are never left guessing which document describes current intent.

## Closeout and decision history

Features describe current implemented behavior, including known verification gaps. Decision records preserve consequential tradeoffs and superseded architectural conclusions. Dated requirement packages are temporary delivery records and are removed after closeout; do not copy their plans, logs, or checklists into feature documentation.
