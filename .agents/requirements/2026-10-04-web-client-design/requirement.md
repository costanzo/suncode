# Requirement

## Background

SunCode's Mobile remote-control client is implemented against `contracts/remote-control/mobile.openapi.yaml`. A browser client is now requested for the same Remote Server workflow, with a responsive desktop-first experience and browser-compatible end-to-end encryption. This delivery defines the reviewable design before any `apps/web` implementation begins.

## Goals

- Add a Web project surface to the design-system review browser.
- Show a calm, componentized browser shell for Hosts, Projects, Sessions, live Session conversation, approvals/questions, and Settings.
- Make the QR pairing flow and E2E encryption trust state explicit without exposing secrets.
- Cover connected, degraded/offline, unauthorized, pairing, approval, question, loading, empty, and constrained-width states.
- Reuse the Mobile remote-control vocabulary and API semantics without inventing Web-only endpoints.

## Non-goals

- No `apps/web` application, React/Vite production package, Zustand store, API client, cryptography implementation, or Remote Server integration.
- No changes to the remote-control OpenAPI contracts.
- No browser QR scanner implementation; the design represents file/camera input and manual fallback affordances only.

## Requirements

- The Web design-system route must be reachable from Projects and contain stable child routes for the shell, pairing, Sessions, Session detail, and security/settings.
- The shell uses a browser-oriented left navigation rail, session list, conversation canvas, and optional right authority/context panel. Supporting regions yield at narrow widths.
- Pairing displays server endpoint, Host identity, one-time code exchange, E2E enabled state, key fingerprint, and safe recovery copy. The AES key is never displayed.
- Session detail supports live SSE status, Last-Event-ID/reconnect indication, messages, streamed activity, approvals, questions, cancellation, retry, and composer states.
- The design uses existing semantic tokens and universal controls; no production client dependency is added.

## Edge cases

- Remote Server reachable while Desktop is degraded.
- Session stream reconnecting or cursor expired and requiring a fresh snapshot.
- E2E key missing/rotated, requiring re-pairing.
- Empty Host/Project/Session lists and a pending approval/question.
- Browser viewport below the compact 620px conversation minimum.

## Acceptance criteria

- A reviewer can navigate to Web pages from the design-system Projects module and inspect light/dark states.
- The main shell and focused flows are componentized in the review source with shared fixture data rather than duplicated markup.
- `npm run build` and `git diff --check` pass.
- The design clearly labels the surface as review-only and does not claim a working Web client.

## Open questions

- Final browser support matrix and WebCrypto implementation details remain for the `apps/web` delivery.
- Whether Web should support multiple paired Hosts in one profile is represented as a design choice, not an API change.
