# Requirement

## Background

Computer Use today is purely visual. `suncode-computer` exposes 17 `computer_*` actions: screenshot, zoom, cursor position, wait, and synthetic mouse and keyboard input. The model has to infer every control, label, and state from pixels and then click by coordinate. The `ComputerBackend` trait has no way to read the operating system's accessibility tree, so the model gets none of the semantic data the platform already publishes (roles, names, values, enabled/focused/selected state, element bounds). Only Enigo uses the macOS Accessibility permission today, and only to inject input.

Reading the accessibility tree helps the model find targets more reliably in dense or low-contrast UI. It also lets the model read text without zooming, and confirm state changes (checkbox toggled, field value set) without another full screenshot round trip.

## Goals

- Add a read-only accessibility observation capability to Computer Use on macOS (AX API), Windows (UI Automation), and Linux X11 desktops (AT-SPI2) in the same delivery.
- Give each reported element a bounding box in the coordinate space of the latest screenshot, so the existing `computer_*` input tools can act on it unchanged.
- Preserve every existing Computer Use invariant: global enablement, host ceiling, control ownership, emergency stop, interactive-only execution, approval policy, frame lifetime, and the rule that observed content stays out of logs, traces, DTOs, and SQLite JSON.
- Report accessibility readiness and permission state truthfully in runtime info and Desktop Settings.

## Non-goals

- Semantic element actions (AX press, UIA Invoke/Value patterns, AT-SPI actions, focus or set-value by element reference). Input stays coordinate-based through existing tools.
- Durable element handles or element references that remain valid across calls.
- Multi-display trees, secure or locked desktops, and Wayland (Computer Use as a whole does not support Wayland yet).
- Reading SunCode's own Avalonia window as a privileged path. It appears in the tree like any other application.
- Treating the accessibility tree as authorization, ground truth for security decisions, or a replacement for screenshots.
- Advertising the capability to models that do not already satisfy the Computer Use contract.

## Requirements

### Tool surface

- Add one observation tool, `computer_accessibility_tree`, registered and gated exactly like the existing `computer_*` tools, with `Risk::ComputerObserve`.
- Inputs:
  - `scope`: `focused_window` (default), `frontmost_application`, or `point`.
  - `coordinate`: required when `scope` is `point`; a pixel coordinate in the latest screenshot. The result is the element at that point and its ancestor chain, plus a bounded subtree.
  - `max_depth`: optional, bounded by a hard ceiling.
  - `max_nodes`: optional, bounded by a hard ceiling.
  - `include_offscreen`: optional boolean, default `false`.
- Output for the model: a compact, line-oriented text outline. Each node has a snapshot-local index, a normalized role, optional name, value, and description, normalized state flags, and a `[x,y,w,h]` bounding box in screenshot pixels when mappable. A header carries the application name, the window title, the display generation, and truncation counters.
- The tool requires a current screenshot frame for coordinate mapping and returns `computer_frame_required` without one. The current frame is the same frame the coordinate input tools use.
- If the display generation changed since the latest screenshot, the call fails with the existing stale-frame behavior. It never reports boxes in a different coordinate space.

### Normalization

- Map platform roles into one small provider-neutral role vocabulary (for example `button`, `link`, `text_field`, `text`, `checkbox`, `radio`, `combo_box`, `list`, `list_item`, `menu`, `menu_item`, `tab`, `table`, `row`, `cell`, `image`, `group`, `window`, `scroll_area`, `slider`, `toolbar`, `unknown`). Write the vocabulary down in the contract.
- Map platform states into normalized flags: `focused`, `enabled`/`disabled`, `selected`, `checked`/`mixed`, `expanded`/`collapsed`, `editable`, `offscreen`, `password`.
- Skip structural nodes with no role, no name, and only one child, so platform wrapper noise does not reach the model.
- Bound each string field and the whole result. A truncated value is marked as truncated.

### Privacy and safety

- Never report values of password or secure-text elements. Emit the role and `password` flag with a redacted value marker.
- The text outline reaches the provider only as transient tool-result content, like screenshot image content. The durable tool-result value records only redacted metadata: scope, node count, truncation counters, display generation, and duration. It holds no names, values, or titles.
- Only the newest accessibility-tree results stay in provider context. Older ones become text omission markers, following the image retention policy.
- Logs, traces, events, SDK DTOs, and SQLite never contain names, values, titles, or descriptions from the tree.
- System guidance tells the model that accessibility content is untrusted application data and never authorization, exactly like on-screen pixels.

### Execution

- Tree collection runs on the serialized Computer Use executor path so it cannot interleave with input. It observes the same cancellation, control-ownership, and enablement checks as other actions.
- Collection has a hard wall-clock deadline and per-call node ceilings. Partial results are returned with a truncation reason rather than failing when the deadline hits after some nodes were collected.
- Platform calls that block or hang on an unresponsive target application must not block emergency stop or user takeover.

### Permissions and runtime state

- Extend runtime info with `accessibility_permission` (`allowed`, `denied`, `unknown`, `unsupported`) and an accessibility backend availability flag.
- On macOS, accessibility read uses the same trusted-process Accessibility permission as input. On Windows, UI Automation needs no extra permission, but elevated target windows are reported as inaccessible. On Linux, AT-SPI2 availability depends on the session accessibility bus being reachable and enabled.
- `computer_accessibility_tree` is advertised only when the accessibility backend is available on the host. Missing permission returns a typed `computer_accessibility_permission_required` error without prompting.
- Desktop Settings shows accessibility read status as its own row next to capture and input control, with platform-specific guidance.

## Edge cases

- The application has no accessibility support, or the tree is empty: return a header with zero nodes and a hint, not an error.
- An element has no frame, or its frame is outside the primary display: emit it without a box (or skip it when `include_offscreen` is false). Never clamp it to a misleading box.
- Retina, per-monitor DPI, and fractional scaling: boxes use the same point-to-pixel normalization as the screenshot frame.
- The frontmost application changes between the screenshot and the tree call: the result header names the actual application so the model can detect the mismatch.
- The target application hangs: the deadline returns a partial or empty result, and other Computer Use calls keep working.
- Very large trees (browsers, IDEs, spreadsheets): node and depth ceilings plus offscreen pruning keep results bounded.
- Elevated or protected processes (Windows UAC integrity levels, macOS secure input): report them as inaccessible rather than failing the call.
- User takeover or emergency stop during collection: the call cancels and returns the standard control-unavailable or disabled error.
- AT-SPI bus missing on a minimal X11 session: the capability is unavailable and the tool is not advertised.

## Acceptance criteria

- On macOS, Windows, and X11 Linux, a model can request `computer_accessibility_tree` after a screenshot and receive a bounded outline whose boxes line up with the screenshot on standard and high-DPI displays.
- Clicking the center of a reported box with existing `computer_left_click` hits the intended control in the conformance fixtures on each platform.
- Password field values never appear in the tool result, logs, events, DTOs, or SQLite.
- Durable session history contains only redacted tree metadata, and only the newest tree results stay in provider context.
- Runtime info and Desktop Settings report accessibility read state separately from capture and input permissions, matching the design-system specimen.
- The capability is not advertised when Computer Use is disabled, under user control, unsupported by the host, or unsupported by the model.
- Focused Rust, C#, and design-system checks pass.

## Open questions

- The exact default and ceiling values for `max_depth`, `max_nodes`, per-field length, and the collection deadline. Proposed: depth 12/25, nodes 300/1000, field 200 characters, deadline 2 seconds. Confirm with conformance measurements on large applications.
- How many accessibility-tree results stay in provider context. Proposed: two, matching screenshots.
- Whether a combined `computer_screenshot` option that attaches the tree in the same call is worth adding later to save one round trip. Deferred.
