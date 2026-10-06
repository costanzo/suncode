# Computer Use

**Status:** Partially implemented and focused-tested. Platform conformance is incomplete.

Computer Use is a first-party Rust capability that observes and operates the user's primary display. It is separate from Browser Use and MCP. `suncode-computer` owns 17 provider-neutral desktop actions exposed as stable `computer_*` function tools: validation, ordered execution, coordinate transforms, zoom cropping, PNG encoding, cancellation checks, and held-input cleanup. It runs on a pinned revision of the maintained Enigo fork. Core `ComputerManager` owns enablement, policy, approvals, and runtime state.

## What users can rely on

- `computer_use_enabled` is global and defaults to `false`. The provider-neutral function tools are offered only when the capability is enabled, the host capability ceiling allows it, the user does not hold control, and the selected model advertises the Computer Use contract. The contract requires function calling and vision support; the seeded DeepSeek V4.1 Flash, OpenAI, Kimi, Claude, and Gemini vision routes advertise it. Anthropic's native client toolset remains an optional provider optimization, not a prerequisite.
- Batches run in response order and stop at the first failure. Every member still gets a result, and members after the failure get the provider's halt text. Observation actions follow the interactive default policy. Any batch that produces input needs approval even under Full Control. Non-interactive use is denied.
- Screenshots and zoom images reach the provider as transient image content in ordinary function-tool results. They are capped at a 1568-pixel edge, 1,000,000 pixels, and a 5 MiB PNG, with aspect ratio and coordinate mapping preserved. Only the two newest Computer Use images stay in provider context, and older ones become omission markers. Screenshot bytes and typed text never reach logs, DTOs, traces, or SQLite JSON.
- Control is exclusive. User takeover removes the toolset, cancels cooperative execution, releases held input, and drops the coordinate frame. After control returns, a fresh screenshot is required before the next coordinate action.
- Recovery never replays an input action whose completion is unknown after restart.
- Emergency stop persists the disabled state, cancels active work, and releases every held key and mouse button before it returns. Re-enabling is explicit.
- `computer_runtime_info` returns only redacted facts: backend availability, display and screenshot sizes, capture and input permission state, control owner, and a safe error. Desktop Settings has a Computer Use page with enablement, model support, runtime and permission state, explicit permission requests, and emergency stop.
- Enabling Computer Use checks input permission before creating the backend and requests the operating-system permission when supported. If permission remains denied, the SDK returns `computer_input_permission_required` and does not persist enablement.

Capture backends exist for macOS (with Retina point-to-pixel normalization), Windows (per-monitor-DPI GDI), and X11 (RandR primary output).

Not implemented: complete serial-batch halt semantics across durable approval continuation; verified macOS, Windows, and X11 conformance across DPI, scaling, and multi-display layouts; Wayland (needs a unified portal ScreenCast and RemoteDesktop session); multi-display control; a Computer Use approval UI; secure or locked desktops; provider routes that do not support function calling, vision, or multimodal function results; and an interactive CLI Computer Use surface.

Contract: [`contracts/agent-sdk/README.md`](../../../contracts/agent-sdk/README.md). Decision: `ADR-20260919-first-party-computer-use`.
