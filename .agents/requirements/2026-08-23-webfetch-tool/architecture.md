# Architecture

## Current state

Core assembles model schemas and authorizes calls, while `suncode-tool` owns the audited synchronous operation dispatcher executed from core's blocking task boundary. Only provider adapters currently perform outbound HTTP.

## Proposed design

Add a core schema and `NetworkAccess` risk for `webfetch`. Core validates approval-visible arguments, then maps the call to `tool/webfetch`. The operations crate owns the rustls HTTP client, redirect policy, bounded body collection, charset decoding, parser-backed HTML conversion, preview, and artifact retention.

## Boundaries and dependencies

Provider and desktop code do not perform the request. `suncode-tool` adds blocking `reqwest`, `html2md-rs`, and entity/charset decoding dependencies. Core adds `url` only for pre-approval validation.

## Data and control flow

`webfetch(url, format?, timeout?)` -> core preflight -> policy/approval -> audited dispatcher -> bounded HTTP response -> optional HTML conversion -> preview plus optional artifact -> normalized tool result.

## Security and failure handling

Only HTTP and HTTPS URLs without embedded credentials are accepted. Redirects remain on the approved host and endpoint, except standard port 80 HTTP may upgrade to port 443 HTTPS. Network, status, content-type, timeout, parsing, and size failures use stable operation errors without including response bodies or credentials. Invalid arguments are returned to the model as recoverable tool errors before approval.

## Compatibility and migration

No database, C ABI, or persisted DTO migration is required. The provider tool schema gains one tool. Existing Full Control semantics apply to the new known network risk.

## Risks and rollback

Remote servers may stall within the requested timeout, and DNS resolution remains controlled by the host OS. Removing the schema and dispatcher entry rolls back model access without affecting persisted history.

## Open questions

- None.
