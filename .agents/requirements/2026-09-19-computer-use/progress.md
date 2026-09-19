# Progress

- Status: In progress
- Last updated: 2026-09-19

## Completed

- Product direction confirmed as first-party Computer Use, independent of MCP and Browser Use.
- Enigo selected as the low-level desktop backend.
- Claude `computer_toolset_20260801` selected as the first provider protocol.
- Initial scope confirmed: primary display, interactive mode, approval for every input batch, turn-scoped screenshots, and stable macOS/Windows/X11 before Wayland is declared stable.
- Current Enigo source and SunCode provider/tool boundaries assessed.
- Enigo now has a public display/capture/frame contract, macOS capture implementation, RGBA cropping, and explicit screenshot-pixel to input-coordinate mapping.
- macOS Enigo input coordinates now normalize Retina CGEvent points to physical pixels.
- `suncode-computer` now owns the 17 Claude member actions, validation, ordered execution, coordinate transforms, zoom cropping, PNG encoding, cancellation checks, and held-input cleanup.
- Provider-neutral client toolsets and toolset identity are represented in `suncode-llm`.
- An Anthropic Messages adapter and native Claude provider route are implemented.
- Computer Use model capability, configuration, core manager, Rust/C/C# runtime DTOs, and ABI version 11 are implemented.
- The design authority and design-system Settings specimen now define Computer Use as an independent first-level page between Network and Browser use.
- Avalonia Settings now exposes enablement, selected-model support, backend/display state, redacted permission state, input authority copy, and a working emergency stop through the Rust/C/C# SDK path.
- Emergency stop persists disabled state, makes active cooperative execution observe cancellation, and releases held input before returning.
- Agent SDK, persistence, SQLite, architecture, and decision documentation now include the current Computer Use contracts.

## In progress

- Remaining platform capture implementations and explicit operating-system permission probes.
- Complete batch halt behavior across durable approval continuation.
- Screenshot resize/context-pruning limits and full control-handoff state.

## Blocked

- None.

## Log

### 2026-09-19

- Requirement initialized after user approval.
- Browser Use explicitly excluded from this delivery.
