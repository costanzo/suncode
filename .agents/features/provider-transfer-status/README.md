# Provider Transfer Status

**Status:** Implemented and focused-tested

The Workspace footer always displays aggregate provider transfer rates while a project is open, using upward and downward arrows and showing `0 B/s` when there is no traffic. The Rust OpenAI-compatible and Anthropic HTTP adapters count request and response body bytes; core publishes coalesced `provider.exchange.progress` events through the existing session stream. Avalonia refreshes once per second and derives the rate from bytes received during each one-second sample interval. It identifies active provider/model exchanges with cumulative bytes in the tooltip.

Rates describe HTTP body bytes observed by `reqwest`, not TLS/protocol overhead or whole-device network-interface throughput. Samples are transient and are not stored in SQLite. The footer continues to show zero rates between exchanges.
