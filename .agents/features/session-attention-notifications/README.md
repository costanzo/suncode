# Session Attention Notifications

**Status:** Implemented and focused-tested; installed-platform smoke tests pending

The Avalonia desktop monitors one Rust-owned, bounded global attention stream independently of the selected-session UI watch. When no SunCode top-level window is active, it sends a system notification for primary turn completion/failure, primary or child approval requests, and primary questions. Cancelled/interrupted turns and child completion/failure are excluded. Foreground-suppressed correlation IDs are recorded and are not replayed later.

A versioned, bounded local ledger deduplicates live and reconciled candidates without storing prompts, responses, paths, tool data, or provider errors. macOS uses User Notifications through a bundled Swift bridge, Windows uses desktop toast protocol activation, and Linux uses freedesktop D-Bus notifications with a default action when supported.

Notification clicks carry only validated navigation intent. A secondary process forwards that intent to the primary process through a current-user Named Pipe on Windows or a user-only Unix Domain Socket on macOS/Linux, receives an acknowledgement, and exits before opening the SDK. The primary process revalidates project/session ownership through SDK-backed state and navigates to the primary session or child approval surface.

Installed display and click behavior still requires release smoke testing on signed macOS, installed Windows, GNOME, and KDE packages.
