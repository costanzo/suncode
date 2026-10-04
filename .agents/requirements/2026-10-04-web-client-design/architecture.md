# Architecture

## Current state

Mobile remote control uses the Remote Server mobile contract, one Host per selected flow, authenticated HTTP, and one Session SSE stream. The design-system has Mobile specimens and a deferred Web platform placeholder.

## Proposed design

Add `design-system/src/projects/web/` as a review-only project surface. A shared `WebRemoteShell` composes navigation, Host/session lists, conversation, and authority panels. Focused pages reuse the same primitives and fixtures for pairing, Session detail, and security states.

## Boundaries and dependencies

The design-system owns only fixtures and visual rules. It imports existing `Icon`, `Button`, `PagePrimitives`, and semantic token styles. It does not import or call `apps/remote-server`, Rust SDK code, SQLite, WebCrypto, or any future `apps/web` module.

## Data and control flow

Fixture state models the mobile API concepts: Host connection state, Project/session projection, session SSE state, pending approval/question, and pairing E2E trust. Local specimen tabs switch presentation state only.

## Security and failure handling

The pairing specimen shows the QR payload's endpoint/Host identity and a shortened fingerprint only. It explicitly states that the AES key remains hidden and that rotation requires re-pairing. Unauthorized, degraded, cursor-expired, and offline states use existing warning/danger semantics and recovery actions.

## Compatibility and migration

No protocol or production architecture changes. The eventual Web client must consume the same `mobile.openapi.yaml` paths and event vocabulary, with browser-safe E2E crypto and the same routing headers.

## Risks and rollback

The change is isolated to the design-system and requirement package. Removing the Web route and its files cleanly restores the prior deferred placeholder without touching production clients.

## Open questions

- Browser QR acquisition and persistence strategy should be confirmed before production implementation.
