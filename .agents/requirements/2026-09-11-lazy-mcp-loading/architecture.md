# Architecture

`open_project` and `select_project` only select and validate the project. The Workspace calls `start_mcp_project`, which schedules enabled project-scoped MCP reconciliation with the existing four-connection concurrency limit and returns an initial `McpLoadProgressResult`. `mcp_load_progress` reads in-memory state. The Avalonia ViewModel polls while loading and binds the result to a compact footer progress bar. `settled` counts connected and terminally failed servers; the indicator disappears when `settled == total`.
