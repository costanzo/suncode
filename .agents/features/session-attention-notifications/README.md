# Session Attention Notifications

**Status:** Implemented and focused-tested; installed-platform smoke tests pending

The Avalonia desktop monitors one Rust-owned, bounded global attention stream independently of the selected-session UI watch. When no SunCode top-level window is active, it sends a system notification for primary turn completion/failure, primary or child approval requests, and primary questions. Cancelled/interrupted turns and child completion/failure are excluded. Foreground-suppressed correlation IDs are recorded and are not replayed later.

Notifications name SunCode, the project display name, and the session title. They use separate wording for completion, failure, approval, and question, and never include paths, prompts, responses, tool data, provider errors, or credentials. Platform notification IDs are stable per correlation key, so retries do not duplicate a notification. On stream lag or monitor restart, the desktop reconciles against a bounded, Rust-owned candidate query built from normalized turns, pending approvals, and pending questions. This query is not a durable event journal. Notification transport or permission failures never change turn, approval, or question state, and they never surface as agent failures. If a request is already resolved, activation still opens its session. If a project or session no longer validates, activation only brings forward an existing SunCode window and records a bounded diagnostic.

A versioned, bounded local ledger deduplicates live and reconciled candidates without storing prompts, responses, paths, tool data, or provider errors. macOS uses User Notifications through a bundled Swift bridge, Windows uses desktop toast protocol activation, and Linux uses freedesktop D-Bus notifications with a default action when supported.

Notification clicks carry only validated navigation intent. A secondary process forwards that intent to the primary process through a current-user Named Pipe on Windows or a user-only Unix Domain Socket on macOS/Linux, receives an acknowledgement, and exits before opening the SDK. The primary process revalidates project/session ownership through SDK-backed state and navigates to the primary session or child approval surface.

Installed display and click behavior still requires release smoke testing on signed macOS, installed Windows, GNOME, and KDE packages.
