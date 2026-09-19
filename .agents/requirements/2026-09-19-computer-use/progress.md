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
- Computer Use model capability, configuration, core manager, Rust/C/C# runtime DTOs, and ABI version 13 are implemented.
- The design authority and design-system Settings specimen now define Computer Use as an independent first-level page between Network and Browser use.
- Avalonia Settings now exposes enablement, selected-model support, backend/display state, redacted permission state, input authority copy, and a working emergency stop through the Rust/C/C# SDK path.
- Emergency stop persists disabled state, makes active cooperative execution observe cancellation, and releases held input before returning.
- Agent SDK, persistence, SQLite, architecture, and decision documentation now include the current Computer Use contracts.
- Enigo revision `707ab1300c004336a58cbfa4c64ad14a38894b65` exposes cross-platform permission status, macOS Screen Recording/Accessibility requests, Windows GDI capture, and X11 RandR primary-display capture.
- Avalonia Settings now invokes permission prompts only from explicit user actions and refreshes the redacted runtime state afterward.
- Provider screenshots now preserve aspect ratio and input-coordinate mapping while limiting the image to a 1568-pixel edge, 1,000,000 pixels, and a 5 MiB PNG payload.
- Provider context retains image bytes for only the two newest Computer Use screenshots; older tool results remain correlated but carry an omission marker.
- User/agent desktop-control handoff is exclusive; user takeover cooperatively cancels actions and releases input, and either handoff retires the coordinate frame.
- Windows capture uses a per-monitor-DPI-aware GDI path and the same physical-pixel space as cursor location and absolute input.
- X11 capture resolves the RandR primary output/CRTC, captures that root-window region, decodes server byte order and visual masks, and retains the CRTC root offset as input bounds.

## In progress

- Real Windows and X11 capture/input conformance across DPI, scaling, and multi-display layouts.
- Complete batch halt behavior across durable approval continuation.
- Cross-platform capture backends and platform conformance.

## Blocked

- None.

## Log

### 2026-09-19

- Requirement initialized after user approval.
- Browser Use explicitly excluded from this delivery.
