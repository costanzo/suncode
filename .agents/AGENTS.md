# .agents Working Rules

This directory contains durable project knowledge, not tool runtime state.

## Before work

1. Read `.agents/README.md`.
2. Read `PRODUCT.md` and `ARCHITECTURE.md` for system-wide work.
3. Read the relevant feature, requirement, specification, and decision records.
4. If documentation conflicts with code, record the discrepancy instead of guessing.

## New requirements

Create `.agents/requirements/YYYY-MM-DD-short-topic/` from `_template/`. Keep `progress.md` and `todo.md` current while implementing. When work is complete, promote stable behavior into `features/`, current technical facts into `specs/`, and important tradeoffs into `DECISIONS.md`.

## Boundaries

- Do not recreate legacy tool-specific documentation or brainstorming directories.
- Do not store process IDs, logs, caches, credentials, or personal machine paths here.
- Do not treat proposed architecture as implemented behavior.
- Keep generated protocol source and user-facing product documentation in the repository locations defined by `.agents/ARCHITECTURE.md`, not inside this knowledge base.
- Prefer focused updates; do not copy the same fact into multiple files unless one location is an index.