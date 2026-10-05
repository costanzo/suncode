# Browser CDP Contract

The Avalonia client owns one CEF Chromium instance per application and configures `RemoteDebuggingPort` on localhost (default `9222`). Rust owns the Browser Use policy and connects to `http://127.0.0.1:<port>/json/list`, then uses the page target's `webSocketDebuggerUrl`.

The Rust `suncode-browser` crate uses `chromiumoxide` to manage the CDP WebSocket, target attachment, command correlation, and event loop. The agent applies request timeouts and converts library failures into stable business errors. Page content is untrusted and cannot authorize actions. The client owns CEF windows, resources, and persistent profiles; Rust does not launch Node.js, Playwright, Chromium, or arbitrary JavaScript workers.

Supported Browser Use operations are mapped to CDP `Page.navigate`, `Runtime.evaluate` for bounded semantic DOM actions, `Input.dispatchKeyEvent`, `Page.captureScreenshot`, and `Page.close`. Raw CSS/XPath, model supplied scripts, executable paths, and proxy credentials are not accepted as tool inputs.
