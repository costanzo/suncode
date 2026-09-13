# Agents Review Surface

The `agents/` directory owns agent-specific design documentation in the review browser. Each top-level agent gets an independently addressable page under `src/agents/`, with deeper behavior areas represented as stable child routes.

`General` is the first agent. Its `Prompt` page is the catalog for system behavior, repository instructions, runtime context, tool guidance, questions, and context-compaction prompts. Prompt entries should identify their scope and owner, distinguish implemented behavior from design drafts, and keep long prompt content in readable, bounded code surfaces.
