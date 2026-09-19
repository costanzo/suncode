# Browser Worker Protocol

**Status:** Initial implementation contract

The bundled Browser Use worker is a private first-party process launched and owned by the Rust agent. It is not a client-facing API, extension protocol, or compatibility promise for third-party workers.

## Transport

- stdin/stdout
- one unsigned 32-bit big-endian payload length followed by one UTF-8 JSON object
- maximum frame size: 8 MiB
- stdout contains protocol frames only
- stderr is diagnostic text and is bounded/redacted by Rust

The worker emits one `hello` event immediately after startup:

```json
{
  "id": null,
  "ok": true,
  "event": "hello",
  "result": {
    "protocolVersion": 1,
    "workerVersion": "0.1.0",
    "nodeVersion": "24.11.1",
    "playwrightVersion": "1.55.0",
    "chromiumVersion": "140.0.7339.16",
    "chromiumRevision": "1187",
    "target": "darwin-arm64"
  }
}
```

Rust validates every identity field against the outer-package-protected target-specific runtime manifest before making Browser Use available.

## Requests and responses

```json
{"id":1,"method":"probe","params":{}}
```

```json
{"id":1,"ok":true,"result":{}}
```

Errors use a bounded stable code and safe message. Page content, console output, paths outside the dedicated browser runtime, credentials, and stack traces must not enter protocol error messages.

## Initial methods

- `probe`
- `start`
- `state`
- `open`
- `snapshot`
- `navigate`
- `click`
- `fill`
- `press`
- `screenshot`
- `close_page`
- `take_control`
- `return_control`
- `stop`
- `shutdown`

Only Rust supplies profile, download-staging, proxy, timeout, and headless-test parameters. Model-facing calls cannot provide filesystem paths, launch arguments, executable names, proxy credentials, or arbitrary scripts.

## Element targets

Actions accept either a short-lived reference from `snapshot` or one structured semantic locator:

- role and accessible name
- label
- placeholder
- test ID
- visible text

Raw CSS, XPath, JavaScript, and Playwright source are not protocol inputs. A reference is bound to one page revision and fails with `browser_reference_stale` after navigation, action, user-control handoff, or mismatched element fingerprint.
