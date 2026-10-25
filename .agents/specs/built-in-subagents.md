# Built-in Subagents

## Catalog

Definitions are compiled into the `suncode-agents` crate (`agent/crates/agents`). Each `AgentDefinition` has a stable ID, unique name, display name, description, version, instructions, a tool scope (the full catalog or an exact allowlist), a model policy, explicit capabilities (delegate, ask the user, use MCP/Browser/Computer tools, use Skills), and an optional tool-call limit. The primary agent is an explicit definition (`builtin.primary.v1`); sessions without an agent id run it. The core harness reads only these definitions to build prompts, advertise tools, and enforce capabilities; definitions can narrow but never widen core policy, approval, or authority rules. The SDK `list_agents` DTO derives its model, MCP, and delegation policy text from these typed fields.

- Architect, UI/UX, Product: `read`, `glob`, `grep`, `webfetch`, `todowrite`; limit 32.
- SWE, QA, SRE: `read`, `glob`, `grep`, `write`, `edit`, `bash`, `todowrite`, `webfetch`; limit 64.

All six inherit the parent model and reasoning effort, deny MCP, deny `question`, and deny `delegate_agent`. The shared coding workflow and the host-environment tool sentences keep a rule only when the snapshotted allowlist offers the tool that rule names. Read-only specialists therefore do not receive bash, file-mutation, verification, destructive-command, or question guidance. Implementation specialists keep the bash rules, including not reverting unrelated work and not running broad destructive commands, but they are told to explain an unclear destructive scope instead of asking the user. An empty allowlist is the primary catalog and keeps the full workflow.

## Invocation lifecycle

`delegate_agent` is advertised only to primary continuations. Core validates the agent name and task, creates a `kind=child` session, creates `session_subagent_invocation`, and starts an internal child submission. State progresses through `created`, `running`, optional `awaiting_approval`, and a terminal `completed`, `failed`, `cancelled`, or `interrupted` result. Startup marks unfinished created/running invocations interrupted without automatic replay.

Parent cancellation and an active child share one cancellation token. After a child suspends for approval the parent may finish with an awaiting result; normal approval resolution resumes the child and updates the invocation independently. Child checkpoints remain scoped to the child session.

## SDK and desktop

`list_agents` returns the fixed catalog. `list_child_sessions(parent_session_id)` returns child session records, current UI states, and invocation records. Public child submit, retry, rename, pin, archive, and reopen calls fail with `child_session_read_only`.

Avalonia keeps the primary session selected as the authority context. The Child sessions bay lists only that parent's children. Selecting one opens a read-only timeline with no composer. ContentSwitcher persists child-session recent items. Settings uses the same expandable navigation and overview/detail pattern as Model providers. Pending child approvals expose only `Allow once` and `Deny` in the child detail.
