# Requirement

## Background

The Mobile app began as a hard-coded UI prototype. The Remote Control HTTP and WebSocket contracts now need a usable multiplatform client foundation so Android and iOS can pair with a Desktop and follow primary Session state.

## Goals

- Add hand-written Kotlin Multiplatform DTOs aligned with the current remote-control contracts.
- Preserve additive WebSocket compatibility by retaining unknown event types and payload fields.
- Establish a typed HTTP/WebSocket boundary, secure credential storage, offline projection, reconnect recovery, and native QR pairing.

## Non-goals

- Changing the Remote Server implementation.
- Supporting hosted multi-user tenancy or arbitrary Desktop provisioning.

## Requirements

- DTOs must cover health, authentication, pairing, Host/Project, Session, pending approval/question, accepted commands, sync, and event envelopes.
- JSON names must match the written contracts.
- DTOs must remain in shared common code and be testable without platform UI code.
- Android and iOS must keep credentials in platform secure storage and use the shared repository for pairing and Session actions.
- Expired access tokens must refresh once and invalid refresh credentials must be cleared.
- Host and Project projections must survive offline startup and recover through `/v1/sync`.

## Edge cases

- WebSocket event types are additive; unknown strings must remain representable.
- Nullable response data and nullable optional fields must decode correctly.
- `POST /v1/sessions` has no success response body and therefore needs no fabricated Session response DTO.

## Acceptance criteria

- Shared code contains serializable DTOs for the listed contract surfaces.
- Focused tests decode representative HTTP and WebSocket payloads.
- Android and iOS bootstrap the live repository only when a Remote Server URL is configured; previews retain the Fake repository.
- Native camera scanners return an opaque pairing payload to the shared pairing flow without duplicating exchange logic.
