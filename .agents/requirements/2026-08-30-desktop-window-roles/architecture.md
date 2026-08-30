# Architecture

`App` owns one `ProjectHubWindow`, zero or more project-keyed `WorkspaceWindow` instances, and singleton Settings/About dialog instances. `ProjectHubWindow` owns hub actions and startup initialization. `WorkspaceWindow` owns project-window menus, custom chrome, resizing, shortcuts, and workspace scrolling. Settings/About remain dedicated windows and rely on native operating-system decorations.
