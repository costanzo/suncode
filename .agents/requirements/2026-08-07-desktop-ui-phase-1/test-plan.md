# Test Plan

## Scope

The current delivery validates the completeness and internal consistency of the requirements only. The later implementation must verify functional behavior, client-runtime contracts, accessibility, rendering, performance, recovery, and platform integration.

## Requirements review

- Trace every primary user job to functional requirements and acceptance criteria.
- Confirm each surface defines loading, empty, active, disabled, error, offline, and recovery states where applicable.
- Confirm every machine-affecting action crosses the TypeScript runtime and Rust enforcement boundary.
- Confirm proposed behavior is not described as implemented.
- Review wording and wireframes with product, Qt, runtime, accessibility, and security owners.

## Unit tests for later implementation

- Deterministic reduction of snapshots and ordered session events.
- Duplicate, missing, and out-of-order event handling.
- Composer submission, acknowledgement, cancellation, draft preservation, and IME behavior.
- Approval scope rendering and duplicate-decision prevention.
- Undo affordance state: available, partially expired, expired, unavailable, and in-conflict.
- Undo confirmation lists the affected files and states that external side effects are not reversed.
- Authority review renders profile-sourced authorization distinctly from a person's approval.
- Multiple tool calls from one assistant message render as sibling activity entries with independent states, rather than as a single sequence.
- Capability-gated controls and settings validation.
- Safe Markdown, links, file references, and output truncation.
- Responsive layout-state selection and preference migration.

## Integration and conformance tests for later implementation

- The hand-written client adapter agrees with the runtime implementation on every shared test vector, including malformed and unknown-field cases.
- Authenticated startup, version negotiation, and capability negotiation.
- Project open and session lifecycle against a fixture runtime.
- Streaming messages and activity with backpressure.
- Approval request, decision, expiry, and resolution on another client.
- Undo of a turn's changes, including the conflict path where a file changed outside the agent.
- Disconnect during a turn, event replay, and snapshot fallback.
- Runtime restart, core unavailable, incompatible version, and typed diagnostic failure.
- Version skew: the client detects an incompatible resident runtime before opening a session and offers the drain-and-restart path.
- Large history, output, attachment, and diff limits.

## Visual and accessibility checks for later implementation

- Screenshot comparison at 720 x 560, 1024 x 768, 1280 x 800, 1440 x 900, and one high-DPI viewport.
- Light, dark, follow-system, high-contrast where available, and reduced-motion settings.
- 100%, 150%, and 200% UI scaling with long translated labels and paths.
- Keyboard-only completion of the primary workflow.
- Screen-reader verification on every supported platform.
- Focus order, focus restoration after dialogs or drawers, and non-color status cues.
- No clipping, overlap, unintended horizontal page scroll, or layout shifts during streaming.

## Performance checks for later implementation

- Startup responsiveness separated from runtime-ready latency.
- Smooth scrolling and selection in long conversations and diffs.
- Bounded memory under long event streams and command output.
- Stable interaction latency while multiple independent sessions run.
- Repeated pane resize, theme switch, and session switch during streaming.

## Security checks for later implementation

- Approval scope cannot be broadened in the client.
- Markdown, links, file references, attachments, and clipboard content are handled as untrusted input.
- Secrets do not appear in settings readback, logs, notifications, diagnostics, or crash reports.
- Project content rendered in the UI cannot trigger filesystem access or navigation outside declared handlers.
- Stale or cross-project events and approvals cannot mutate the active session.

## Regression checks

- Run `git diff --check` for this documentation delivery.
- Run repository documentation validation when such a command exists.
- For implementation, run the repository's full verification command on every supported platform.

## Commands and results

- Pending until the documentation changes are complete.

## Residual risks

- The desktop OS matrix, Qt presentation technology, and UI-to-runtime contract are not yet decided.
- No executable prototype currently validates rich text, diff rendering, or accessibility assumptions.
- Concrete performance budgets cannot be accepted until reference hardware and data-size limits are defined.

