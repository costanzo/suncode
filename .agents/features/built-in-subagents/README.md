# Built-in Specialist Agents

**Status:** Implemented and focused-tested

SunCode ships six immutable Rust-defined specialists: Architect, UI/UX Agent, Product Agent, Software Engineering Agent, QA Agent, and SRE Agent. Users cannot create, edit, enable, disable, reorder, delete, or directly message them.

Only a primary agent may delegate. Each invocation creates a child session linked to the primary session and originating turn/tool call, inherits the model and reasoning effort, and runs through the normal Rust policy, approval, audit, checkpoint, cancellation, persistence, and recovery paths. Child requests advertise only their role's built-in tool allowlist and cannot use MCP, ask the user questions, or delegate again.

An unknown agent name fails with `unknown_agent` and creates no session. Ordinary delegation completes synchronously inside the parent tool call. A child that suspends for approval returns an awaiting result, so the parent may finish while the child resumes independently once the approval is resolved. Parent cancellation reaches the active child. Startup marks unfinished invocations interrupted and does not replay them. Child sessions never appear in the primary Sessions list. Browser Use, Computer Use, and LSP tools are not in any specialist allowlist.

The SDK lists the catalog and child sessions. Avalonia shows Agents in Settings, child sessions in a mutually exclusive right bay, recent child content in ContentSwitcher, and a read-only central timeline. Pending child machine-operation approvals can be allowed once or denied inline. Child mutations keep child-session checkpoint manifests; parent-turn undo does not claim to include them.
