# Requirement

## Background

Settings lists the built-in model providers, but its provider navigation cannot collapse and provider detail pages expose the persisted endpoint as read-only text. Users need to collapse the provider list and edit an OpenAI-compatible provider URL without accessing SQLite.

## Goals

- Make Model providers an expandable navigation group with a directional chevron.
- Allow each provider detail page to edit and save its URL.
- Persist URL changes through the Rust SDK and apply them to subsequent provider requests.
- Keep the Avalonia client and executable design specimen aligned.

## Non-goals

- Add, remove, enable, or reorder providers or models.
- Change provider adapter types.
- Expose credentials or allow clients to access SQLite directly.

## Requirements

- Collapsing Model providers hides its provider children; expanding restores them.
- Selecting the parent opens the provider overview without forcing the group open.
- Provider detail pages show the current URL in an editable field with an explicit save action and status.
- Rust accepts only absolute HTTP or HTTPS URLs with a host and without embedded credentials.
- A successful update preserves the provider identity, models, adapter, enabled state, order, and credential.
- A successful update changes the provider route used by later model calls; an already-running provider request keeps the route it started with.

## Edge cases

- Empty, relative, non-HTTP, hostless, credential-bearing, and malformed URLs are rejected without changing the stored endpoint.
- Trimming whitespace and trailing slashes produces one normalized endpoint.
- A failed save leaves the editable value available for correction.

## Acceptance criteria

- Navigation expansion and collapse work in the design specimen and Avalonia.
- Provider URLs persist through the named Rust SDK/C ABI and refresh the desktop model projection.
- Focused Rust and .NET tests, the design-system build, and repository diff checks pass.

## Open questions

- None.
