# Built-in Subagents

## Catalog

Definitions are compiled into `agent/crates/core/src/agent/builtin_agents.rs` and expose stable ID, unique name, display name, description, version, instructions, exact allowlist, model policy, MCP policy, delegation policy, and tool-call limit.

- Architect, UI/UX, Product: `read`, `glob`, `grep`, `webfetch`, `todowrite`; limit 32.
- SWE, QA, SRE: `read`, `glob`, `grep`, `write`, `edit`, `bash`, `todowrite`, `webfetch`; limit 64.

All six inherit the parent model and reasoning effort, deny MCP, deny `question`, and deny `delegate_agent`.

## Invocation lifecycle

`delegate_agent` is advertised only to primary continuations. Core validates the agent name and task, creates a `kind=child` session, creates `subagent_invocation`, and starts an internal child submission. State progresses through `created`, `running`, optional `awaiting_approval`, and a terminal `completed`, `failed`, `cancelled`, or `interrupted` result. Startup marks unfinished created/running invocations interrupted without automatic replay.

Parent cancellation and an active child share one cancellation token. After a child suspends for approval the parent may finish with an awaiting result; normal approval resolution resumes the child and updates the invocation independently. Child checkpoints remain scoped to the child session.

## SDK and desktop

`list_agents` returns the fixed catalog. `list_child_sessions(parent_session_id)` returns child session records, current UI states, and invocation records. Public child submit, retry, rename, pin, archive, and reopen calls fail with `child_session_read_only`.

Avalonia keeps the primary session selected as the authority context. The Child sessions bay lists only that parent's children. Selecting one opens a read-only timeline with no composer. ContentSwitcher persists child-session recent items. Settings uses the same expandable navigation and overview/detail pattern as Model providers. Pending child approvals expose only `Allow once` and `Deny` in the child detail.
