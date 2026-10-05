# Browser CDP Contract

The Avalonia client owns one CEF Chromium instance per application and configures `RemoteDebuggingPort` on localhost (default `9222`). Rust owns the Browser Use policy and connects to `http://127.0.0.1:<port>/json/list`, then uses the page target's `webSocketDebuggerUrl`.

Rust sends bounded JSON CDP commands over WebSocket. Responses are limited to 8 MiB, matched by numeric command id, and converted to stable business errors. Page content is untrusted and cannot authorize actions. The client owns CEF windows, resources, and persistent profiles; Rust does not launch Node.js, Playwright, Chromium, or arbitrary JavaScript workers.

Supported Browser Use operations are mapped to CDP `Page.navigate`, `Runtime.evaluate` for bounded semantic DOM actions, `Input.dispatchKeyEvent`, `Page.captureScreenshot`, and `Page.close`. Raw CSS/XPath, model supplied scripts, executable paths, and proxy credentials are not accepted as tool inputs.
