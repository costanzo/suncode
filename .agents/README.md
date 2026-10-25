# Project Knowledge Base

This directory is the source of truth for project context used by contributors and coding agents.

## Reading order

1. `AGENTS.md` — rules for maintaining this knowledge base.
2. `PRODUCT.md` — product purpose, users, current scope, and explicit deferrals.
3. `ARCHITECTURE.md` — approved system architecture and boundaries.
4. `features/` — durable implemented behavior and its current verification status.
5. `specs/` — current technical contracts and implementation facts.
6. `DECISIONS.md` — durable architectural decisions and the constraints they explain.
7. `requirements/` — template and policy for active delivery work; completed packages are removed after closeout.

## Directory ownership

| Path | Purpose |
| --- | --- |
| `features/` | Durable implemented behavior and known verification gaps |
| `requirements/` | Template and policy for active delivery work; no completed delivery archive |
| `specs/` | Current API, protocol, storage, security, and operational contracts |
| `DECISIONS.md` | Project-level architectural decisions and their consequences |

## Naming

- Requirement directories: `requirements/YYYY-MM-DD-short-topic/`
- Feature directories: `features/short-topic/`
- Decision identifiers: `ADR-YYYYMMDD-short-topic`

Start a requirement from `requirements/_template/` only when substantial work needs a temporary durable delivery record. At closeout, promote stable behavior to `features/`, current contracts to `specs/` or `contracts/`, and consequential tradeoffs to `DECISIONS.md`; then remove the dated package. Keep transient tool state out of this directory. `.agents` is tracked project knowledge; `.codex`, `.claude`, logs, and local brainstorming sessions are not.
