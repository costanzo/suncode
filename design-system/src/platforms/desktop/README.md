# Desktop Platform

This is the Phase 1 platform adaptation layer for the Avalonia desktop client. Universal component semantics are consumed here and desktop-only anatomy is documented under `components/` and `pages/` as it becomes implemented.

Windows distribution uses Inno Setup's standard native wizard and uninstaller. This platform-owned surface keeps Inno's native controls, sizing, typography, focus, DPI and theme behavior instead of adopting Avalonia tokens or a React imitation. Installation is current-user; desktop shortcut and post-install launch are initially unchecked. Silent setup never launches the application. Uninstall preserves user data. The executable definition is `apps/desktop-avalonia/packaging/windows/SunCode.iss`; its packaging guide owns lifecycle verification and outstanding installed-platform checks. Application-owned update UI remains specified by Settings → Updates.
