# Architecture

## Current state

Rust selects Agent shell using platform-specific code. Avalonia interactive Terminal uses RoyalTerminal with an unspecified command. Settings use Rust-owned configuration and hand-written SDK bindings. Windows publishing bundles PowerShell 7 through a Node packaging script.

## Proposed design

Two independently persisted global shell preferences select known shell families and validated executable paths. Agent owns non-interactive launch semantics and captures its effective shell for the lifetime of each turn, including approval/recovery. Avalonia owns interactive PTY startup using its selected shell. Configuration persistence remains SDK-owned.

The proposed installer is Rust-owned, with official versioned x64/ARM64 ZIP metadata, expected digest verification, bounded safe extraction, staged installation, executable probe, progress/cancellation, and cleanup. A separate optional step applies both settings after successful installation. It uses shared network configuration and does not modify system PATH.

## Design-system stage

A Terminal section extends existing Settings navigation and shared controls. Fixtures simulate all validation and installation; local React state persists between panel navigation and resets only when review platform/architecture/scenario changes. Reviewer-only controls live outside the product window. DESIGN.md is the normative visual/interaction reference.

## Compatibility and risks

Default Windows PowerShell 5.1 differs from PowerShell 7 in syntax and encoding. Shell identity in Agent context must match execution. Pending approvals must not change interpreter on resume. Invalid paths cannot silently fall back. No SQLite table change is anticipated for key/value preferences. Interactive Terminal remains desktop-owned.
