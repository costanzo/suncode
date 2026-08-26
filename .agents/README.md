# Project Knowledge Base

This directory is the source of truth for project context used by contributors and coding agents.

## Reading order

1. `AGENTS.md` — rules for maintaining this knowledge base.
2. `PRODUCT.md` — product purpose, users, and scope.
3. `ARCHITECTURE.md` — approved system architecture and boundaries.
4. `features/` — stable, implemented product capabilities.
5. `requirements/` — lightweight template and policy for a new delivery; historical packages are consolidated into `features/`.
6. `specs/` — current technical contracts and implementation facts.
7. `DECISIONS.md` — accepted decisions and superseded choices.

## Directory ownership

| Path | Purpose |
| --- | --- |
| `features/` | Durable behavior after it is implemented |
| `requirements/` | Template and policy for a new delivery; completed behavior belongs in `features/` |
| `specs/` | Current API, protocol, storage, security, and operational contracts |
| `DECISIONS.md` | Project-level architectural decisions and their consequences |

## Naming

- Requirement directories: `requirements/YYYY-MM-DD-short-topic/`
- Feature directories: `features/short-topic/`
- Decision identifiers: `ADR-YYYYMMDD-short-topic`

Start a requirement from `requirements/_template/`. Keep transient tool state out of this directory. `.agents` is tracked project knowledge; `.codex`, `.claude`, logs, and local brainstorming sessions are not.
