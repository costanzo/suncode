# Architecture

Avalonia owns a narrow read-only Git service and structured process runner. View models consume desktop-owned DTOs using the SDK-provided canonical project root. Processes have fixed Git arguments, literal pathspecs, disabled pager/external diff/textconv/fsmonitor, optional locks off, output/time bounds, and cancellation. Discovery verifies candidates with --version. UI availability is independent of repository state. This is an explicit client ownership exception; agent operations remain Rust audited.
