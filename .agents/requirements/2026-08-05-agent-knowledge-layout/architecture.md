# Architecture

Use a tracked `.agents/` knowledge base modeled after the reference repository:

- Project context: `PRODUCT.md`, `ARCHITECTURE.md`, and `DECISIONS.md`
- Stable capabilities: `features/`
- Dated delivery history: `requirements/`
- Current technical facts: `specs/`

Root `AGENTS.md` remains the standard repository entry point and directs contributors into `.agents/`. Tool-specific runtime state remains outside the knowledge base and is ignored.
