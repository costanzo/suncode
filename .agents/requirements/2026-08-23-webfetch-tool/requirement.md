# Requirement

## Background

SunCode has no dedicated model tool for retrieving web reference content. OpenCode provides a bounded `webfetch` tool with URL, output format, and timeout parameters.

## Goals

- Add an OpenCode-aligned `webfetch` model tool.
- Keep outbound network access approval-gated and audited.
- Bound response size and model context impact.
- Present the requested URL clearly in desktop approvals and tool activity.

## Non-goals

- General web search, browser automation, authenticated sessions, form submission, or arbitrary HTTP methods.
- Image or binary tool-result attachments.
- Persisting a web cache.

## Requirements

1. Advertise `webfetch` with required `url`, optional `format` (`text`, `markdown`, or `html`, default Markdown), and optional seconds-based `timeout` (default 30, maximum 120).
2. Reject malformed/non-HTTP URLs, embedded credentials, unsupported formats, and invalid timeouts before approval and at the operations boundary.
3. Treat WebFetch as network access requiring approval, or an existing session Full Control grant.
4. Limit redirects to the approved origin or a standard HTTP-to-HTTPS upgrade.
5. Accept only textual responses and bound the raw body to 5 MiB.
6. Convert HTML using a structured parser, decode BOM or declared charsets, and return the requested representation.
7. Bound model output to a 64 KiB UTF-8 preview and retain larger converted content as a managed artifact.
8. Show the URL as the primary approval detail and a readable WebFetch label in tool activity.

## Edge cases

- Responses without a content type are treated as text but remain size-bounded.
- Cross-origin and non-HTTP redirects fail closed.
- Cloudflare challenge responses may retry once with an honest SunCode user agent without expanding the approved origin.
- Cancellation is checked before the request and while reading chunks; a blocked transport remains bounded by the request timeout.

## Acceptance criteria

- Registry, policy, argument validation, HTTP fetch, conversion, response bounds, artifact, and desktop projection tests pass.
- Runtime workspace, Avalonia, formatting, clippy, and diff checks pass or any unrelated existing failure is recorded.

## Open questions

- Binary and image retrieval depends on a future provider-neutral attachment contract.
