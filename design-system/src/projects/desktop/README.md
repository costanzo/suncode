# Desktop Project

This project boundary reviews the four Avalonia desktop windows: ProjectHub, Workspace, Settings, and About. Workspace alone owns a custom 36px title bar; the other three specimens begin at the client area because their title bars and window controls are supplied by the operating system. `project-hub/` mirrors the actual `ProjectHub.axaml` content using component-owned React specimens for buttons, project rows, and empty state. `settings/` mirrors `SettingsWindow.axaml` as a dedicated, independently routable settings surface with General and Model provider pages.
