# Test Plan

- Verify the exact 13-table schema, indexes, triggers, integrity check, and foreign-key check.
- Verify global/project/session precedence and global fallback for project default models.
- Verify custom OpenAI-compatible providers round-trip while unknown adapters fail.
- Run the full Rust workspace, strict database/LLM Clippy, Avalonia build, formatting, and diff checks.
