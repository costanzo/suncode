# Requirement

## Background

The executable desktop specimens and the production Avalonia client share the same visual foundation, but a full compliance review found behavioral, information-architecture, responsive, accessibility, token, Markdown, settings, and performance gaps. The most important gap is that the composer presents images as message attachments while the current text-only turn contract stores them as session-owned placeholder images.

## Goals

- Make production desktop behavior match `DESIGN.md` and the executable desktop specimens.
- Make image attachments belong to and travel with a submitted user message.
- Restore the required Review hierarchy: approval, turn changes, touched files, runtime health.
- Keep every supporting surface reachable at compact widths.
- Preserve native keyboard and accessibility semantics.
- Bring session state, Markdown, Settings, chrome, tokens, and performance into compliance.

## Non-goals

- Add a web runtime or make the React design system a production dependency.
- Add image understanding to providers that do not advertise image input support.
- Add Git mutation, hosted execution, or other deferred product scope.
- Replace the approved Quiet Control Desk visual identity.

## Requirements

- Extend the hand-written Rust SDK/C ABI/C# contract so a submitted turn can reference up to three persisted session images and the resulting user message exposes those attachments.
- Validate attachment ownership, count, existence, MIME type, and bounded size inside Rust before turn admission.
- Keep approval visually dominant; place changes, touched files, and runtime health below it in that order.
- Provide a persistent compact-width surface menu for Sessions, Explorer, Review, Source control, and Provider trace.
- Give icon-only actions accessible names, preserve visible focus, use native selection controls, manage dialog focus, and announce dynamic feedback.
- Expose meaningful per-session running, approval, question, failure, and idle states.
- Reconcile durable layout values with the executable specimens and production-responsive behavior.
- Complete Markdown presentation and verification for both themes.
- Align Settings content and navigation with its specimen, including image storage and provider overview.
- Avoid eagerly materializing unbounded conversation controls or full-resolution image decodes.

## Edge cases

- A selected image is removed, belongs to another session, is unreadable, oversized, unsupported, or disappears before submit.
- The selected provider/model does not support image input.
- A compact window starts with all panels closed and must still restore each surface.
- Long paths, long translated labels, empty changes, disconnected runtime, and no selected session remain understandable.
- Keyboard-only and screen-reader users can operate every overlay and review decision.

## Acceptance criteria

- The desktop no longer implies that session-only references are sent with a message.
- A supported image attachment is submitted through Rust and restored on its user message.
- Review and compact navigation match the required information architecture.
- Focus, automation names, selection semantics, and status announcements are present for affected controls.
- Design specimens and production stay aligned in dark/light and compact/expanded states.
- Focused Rust and .NET tests, design-system build, `git diff --check`, and relevant broader checks pass.

## Open questions

- None. Existing executable specimens resolve the previously conflicting title-bar, composer, and responsive values.
