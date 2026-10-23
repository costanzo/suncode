# Architecture

## Current state

`suncode-computer` owns provider-neutral `ComputerAction` parsing, the serialized `ComputerExecutor<Box<dyn ComputerBackend>>`, coordinate frames, PNG artifacts, and the `computer_*` tool catalog. `ComputerRuntime` holds the executor behind one `Mutex` and runs each action on a dedicated thread through `run_blocking`, polling cancellation, enablement, and control owner. `take_user_control`, `set_enabled(false)`, and emergency stop lock the same mutex to release held input and retire the frame.

`ComputerBackend` covers capture and synthetic input only. `EnigoBackend` (pinned Enigo fork) is the only implementation. Permission state comes from `EnigoBackend::permission_info()` (capture and input). Core `ComputerManager` (`agent/crates/core/src/agent/computer.rs`) forwards SDK management calls. Tool results with `model_text` or `image` keep that content in provider context only. Session history stores the redacted `value`, and `prune_tool_images` keeps the two newest images.

## Proposed design

### New backend boundary

Add a separate trait instead of growing `ComputerBackend`:

```rust
/// Read-only, OS-facing accessibility snapshot source.
pub trait AccessibilityBackend: Send + Sync {
    fn runtime_info(&self) -> AccessibilityRuntimeInfo;
    fn snapshot(
        &self,
        request: &AccessibilityRequest,
        cancelled: &dyn Fn() -> bool,
    ) -> ComputerResult<AccessibilitySnapshot>;
}
```

- `AccessibilityRequest` carries the scope (`FocusedWindow`, `FrontmostApplication`, `Point { input_x, input_y }`), the depth and node ceilings, the offscreen flag, and a deadline `Instant`.
- `AccessibilitySnapshot` is a flat, depth-annotated node list in input (logical) coordinates, plus application name, window title, and truncation reason. Platform code never sees screenshot pixels.
- `AccessibilityNode { depth, role: Role, name, value, description, states: StateFlags, bounds: Option<InputRect> }`. `Role` and `StateFlags` are the normalized provider-neutral vocabulary, defined in `suncode-computer`.

A separate trait keeps the existing input/capture contract, its test doubles, and the Enigo fork untouched. It also lets accessibility run outside the executor mutex (see Control flow). The trait is read-only by construction: no method mutates target state.

### Platform implementations

Each lives behind `cfg(target_os)` in a new `suncode-computer` module `accessibility/`:

| Platform | API | Crate (pinned, already in lockfile transitively where noted) | Notes |
| --- | --- | --- | --- |
| macOS | `AXUIElement` (ApplicationServices) | `objc2-application-services` + `objc2-core-foundation` 0.3.x (`objc2-core-foundation` already present) | Uses `AXIsProcessTrusted` for permission state. Sets `AXUIElementSetMessagingTimeout` per element to the remaining deadline. Frames come from `kAXPositionAttribute`/`kAXSizeAttribute` in global points, the same space Enigo input uses. |
| Windows | UI Automation (`IUIAutomation`, cached tree walk with `IUIAutomationCacheRequest`) | `windows` 0.62 (already present) with `Win32_UI_Accessibility` and `Win32_System_Com` | COM is initialized MTA on the collector thread. `BoundingRectangle` is in physical pixels, so it converts to the executor's input space with the same per-monitor DPI logic the GDI capture backend uses. Elevated windows return `E_ACCESSDENIED`, which maps to an inaccessible marker. |
| Linux X11 | AT-SPI2 over D-Bus | `atspi` (async, `zbus` 4.x already present) driven by a private current-thread runtime on the collector thread | Availability means `org.a11y.Bus` is reachable and `IsEnabled` is true. `GetExtents(ATSPI_COORD_TYPE_SCREEN)` returns screen coordinates aligned with X11 RandR. |

Exact crate versions are pinned during implementation. Any crate not already in the lockfile is flagged for review before it is added.

### Model-facing tool

- `ComputerAction::AccessibilityTree { scope, coordinate, max_depth, max_nodes, include_offscreen }` is parsed and validated like other actions. `is_known_member` and `computer_tool_definitions` gain `accessibility_tree`.
- `ComputerRuntime::risk` maps it to `Risk::ComputerObserve`, so approval behavior matches screenshot.
- `catalog` advertises it only when `advertises_tools(...)` holds **and** the accessibility backend reports available.
- `tool_result` gains an `ActionOutcome::Accessibility(AccessibilityReport)` arm. It returns `model_text` holding the outline and a redacted `value`: `{"type":"computer_accessibility","scope","nodeCount","truncated","truncationReason","displayGeneration","durationMs"}`.

### Outline format

```
app="TextEdit" window="Untitled" generation=7 nodes=42 truncated=none
[0] window "Untitled" [0,48,1440,852]
  [1] toolbar [0,48,1440,52]
    [2] button "Bold" enabled [212,60,28,28]
  [3] text_field "Body" focused editable value="Hello…" (truncated) [0,100,1440,800]
  [4] text_field password value=<redacted> [600,400,240,24]
```

Indices are local to one snapshot and are not addressable later. Boxes are `[x,y,width,height]` in pixels of the latest provider screenshot frame, so they line up with what the model sees and with coordinates accepted by `computer_left_click`.

## Boundaries and dependencies

- `suncode-computer` owns the trait, the normalized vocabulary, platform collectors, coordinate mapping, redaction, bounding, and outline rendering. It owns no policy, persistence, or provider protocol (unchanged).
- `agent/crates/core` owns catalog gating through the existing provider path, pruning of older tree results from provider context, and system guidance text.
- `sdks/rust`, `sdks/c`, and `sdks/csharp` gain only additive runtime-info fields (`accessibilityAvailable`, `accessibilityPermission`). There are no new methods: macOS shares the existing input-permission request path.
- `apps/desktop-avalonia` renders one extra Settings status row. `design-system/` specifies it first.
- Browser Use is unaffected. Its DOM/a11y data stays inside the CEF CDP path.

## Data and control flow

1. The model calls `computer_accessibility_tree`. Core validates it, applies observe policy, and dispatches to `ComputerRuntime::execute`.
2. The runtime checks host, enablement, and control owner, then takes the executor mutex **briefly** to copy the current provider frame's `DisplayGeometry`, source/provider dimensions, and frame generation. No frame gives `computer_frame_required`. The mutex is released before collection starts.
3. On a `run_blocking` collector thread the runtime calls `AccessibilityBackend::snapshot` with a deadline. The cancellation closure checks the per-call flag, enablement, control owner, and that the frame generation is unchanged. `Point` scope converts the pixel coordinate to input space with `ComputerFrame::input_point`.
4. The collector walks breadth-first under node, depth, and deadline ceilings, prunes structural wrappers, skips offscreen nodes unless requested, and redacts secure-text values at the source.
5. The runtime maps each input-space box to provider-frame pixels with the inverse of the existing axis mapping, then intersects it with the display. Boxes that do not intersect become `None`. It then renders the bounded outline.
6. Before returning, the runtime re-checks the frame generation. If the user took control, emergency stop ran, or a new screenshot replaced the frame, the result is discarded with the standard error.
7. Core records the result. The outline stays in provider context, the redacted metadata goes to history and events, and `prune_tool_texts` (or a generalized `prune_tool_media`) replaces older tree outlines with omission markers.

Keeping collection outside the executor mutex matters. AX, UIA, and AT-SPI calls into a hung application can block for seconds, and `take_user_control`/emergency stop must never wait behind them. Since the tree is read-only, running it concurrently with a later stop is safe. The stop path flips the atomics, and the collector's next cancellation check returns early. Tool calls stay serialized (`concurrent` is still `false`), so collection never overlaps model-issued input.

## Security and failure handling

- Tree content is untrusted third-party data. Guidance and the contract say it is never authorization, the same rule as screenshots.
- Secure-text redaction happens in the platform collector before strings leave the platform module. Detection uses macOS `AXSecureTextField` subrole, UIA `IsPassword`, and AT-SPI role `PASSWORD_TEXT`.
- Names, values, titles, and descriptions never enter `ToolResult.value`, `BusinessError` messages, tracing fields, events, or SQLite. Errors carry only codes and platform status categories.
- Per-field and total-size bounds prevent provider-context blow-up. The deadline and ceilings prevent unbounded walks.
- Typed errors: `computer_frame_required`, `computer_frame_stale`, `computer_accessibility_unavailable`, `computer_accessibility_permission_required`, `computer_accessibility_failed`, plus the existing disabled and control-unavailable errors. An inaccessible target (elevated process, secure input) is a successful result with an `inaccessible` header flag.
- Collector panics are caught at the thread boundary and become `computer_accessibility_failed`. A Windows COM or AT-SPI runtime failure never poisons the executor mutex because collection does not hold it.

## Compatibility and migration

- Additive tool. Existing models and sessions see one more `computer_*` tool only when all gates pass. Stored sessions need no migration.
- Additive runtime-info fields. Older C# clients ignore unknown JSON fields. C ABI symbols are unchanged, and the ABI version stays unless the runtime-info struct layout is native rather than JSON. Confirm during implementation.
- No SQLite schema change.
- Anthropic's native client toolset has no accessibility member. The tool is offered only as an ordinary function tool, even when the native toolset optimization is active.

## Risks and rollback

- Platform tree quality varies widely (Electron, Java, custom-drawn, games). The screenshot remains the primary source, and guidance says so.
- Coordinate-mapping errors under fractional scaling or mixed-DPI layouts would steer clicks wrong. Conformance fixtures must check box alignment, not just tree content.
- New native dependencies (macOS ApplicationServices bindings, Windows UIA features, `atspi`) widen the build and audit surface. Each is pinned and gated per platform.
- Rollback: drop `accessibility_tree` from `is_known_member` and the catalog. Runtime-info fields can stay as `unsupported`. Nothing persists that needs cleanup.

## Open questions

- Whether to generalize `prune_tool_images` into one media-retention pass for images and tree outlines, or add a sibling pass. Leaning toward generalizing.
- Whether Windows should prefer the UIA control view or the content view as the default walker. Leaning toward the control view.
- Whether X11 needs explicit opt-in guidance when `org.a11y.Status.IsEnabled` is false, or whether SunCode may set it. Default: never toggle it, only report it.
