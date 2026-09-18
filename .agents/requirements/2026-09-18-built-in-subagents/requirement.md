# Requirement

## Background

SunCode currently has one general agent per session. The product needs a fixed catalog of specialist agents that the main agent may invoke while preserving one user-facing conversation, explicit authority, durable observability, and the existing Rust-owned execution boundary.

## Goals

- Add six fixed Rust-defined agents: Architect, UI/UX Agent, Product Agent, SWE Agent, QA Agent, and SRE Agent.
- Let only the main agent delegate a bounded task to a specialist.
- Persist every delegation as a child session linked to the main session and originating turn/tool call.
- Restrict the model-visible and executable tool catalog for each specialist.
- Expose the fixed catalog and child sessions through the SDK.
- Show the catalog in read-only Settings UI and child sessions in the Workspace UI approved by the design system.

## Non-goals

- User-created, edited, enabled, disabled, or deleted agents.
- Direct user messaging to child sessions.
- Nested delegation or arbitrary delegation graphs.
- MCP tools inside child sessions in the first release.
- Hosted agents, tenancy, remote identity, or executable extensions.

## Requirements

1. Agent definitions are compiled into Rust and have immutable stable ID, unique name, display name, description, version, instructions, tool allowlist, model policy, and limits.
2. The catalog contains:
   - `architect-agent` / Architect
   - `ui-ux-agent` / UI/UX Agent
   - `product-agent` / Product Agent
   - `swe-agent` / Software Engineering Agent
   - `qa-agent` / QA Agent
   - `sre-agent` / SRE Agent
3. The main agent receives one `delegate_agent` conversation tool whose agent argument is restricted to the built-in names.
4. Each invocation creates a new child session. Child sessions never appear in the primary Sessions list and cannot accept public `submit_turn`, rename, pin, archive, reopen, or retry calls.
5. Child sessions inherit project, model, and reasoning effort from the parent invocation and have a maximum delegation depth of one.
6. Child model requests receive only the allowed built-in tools. MCP and `delegate_agent` are denied. Tool execution revalidates the allowlist.
7. Machine-affecting tools retain normal validation, policy, approval, audit, and checkpoint behavior.
8. Parent cancellation requests cancellation of an active child invocation. Interrupted invocations remain visible and are not automatically replayed.
9. SDK consumers can list built-in agents, list child sessions for one parent, and read an existing child with normal snapshot/usage/trace methods.
10. Avalonia exposes the approved right-side list, read-only central detail, ContentSwitcher child-session item, and Settings agent catalog.

## Edge cases

- Unknown agent names fail without creating a session.
- A child cannot invoke `delegate_agent`, even if a provider returns the call despite the advertised catalog.
- Child results that fail, cancel, require approval, or are interrupted remain correlated to the parent tool use.
- Restoring a child from recent content also restores its parent as the primary authority context.
- Archived parent sessions hide their children from active project UI.

## Acceptance criteria

- Focused Rust tests prove catalog uniqueness, tool filtering, delegation, parent linkage, public child-session write denial, and recovery-safe terminal states.
- Rust/C/C# contract tests cover new DTOs and methods.
- Avalonia focused tests cover child content selection and Settings catalog projection.
- The design-system and production UI agree for supported states.
- `git diff --check` and applicable builds/tests pass.

## Open questions

- Cross-session unified undo for child mutations remains a required implementation concern; the first release must not claim parent-turn undo covers child changes unless the operation grouping is implemented and tested.
