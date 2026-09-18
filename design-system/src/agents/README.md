# Agents Review Surface

The `agents/` directory owns agent-specific design documentation in the review browser. Each top-level agent gets an independently addressable page under `src/agents/`, with deeper behavior areas represented as stable child routes.

`General` is the primary agent. Its `Prompt` page is the catalog for system behavior, repository instructions, runtime context, tool guidance, questions, and context-compaction prompts. Prompt entries should identify their scope and owner, distinguish implemented behavior from design drafts, and keep long prompt content in readable, bounded code surfaces.

The navigation also exposes the six immutable built-in specialists: Architect, UI/UX Agent, Product Agent, Software Engineering Agent, QA Agent, and SRE Agent. Their pages are generated from one shared review catalog so the top-level Agents navigation, module index, and Desktop Settings specimen use the same names, IDs, descriptions, allowlists, and boundaries. Each specialist also has a `Prompt` child route that follows the General agent's prompt catalog pattern and documents its role instruction alongside the shared child-session guardrails.
