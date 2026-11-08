// Explorer fixtures used by the desktop workspace specimens.

export const explorerNodes = [
  {
    id: "agents",
    parent: "project-root",
    name: ".agents",
    path: "/projects/suncode/.agents",
    kind: "folder",
    depth: 1,
  },
  {
    id: "apps",
    parent: "project-root",
    name: "apps",
    path: "/projects/suncode/apps",
    kind: "folder",
    depth: 1,
  },
  {
    id: "desktop",
    parent: "apps",
    name: "desktop-avalonia",
    path: "/projects/suncode/apps/desktop-avalonia",
    kind: "folder",
    depth: 2,
  },
  {
    id: "views",
    parent: "desktop",
    name: "Views",
    path: "/projects/suncode/apps/desktop-avalonia/Views",
    kind: "folder",
    depth: 3,
  },
  {
    id: "workspace-file",
    parent: "views",
    name: "ProjectWorkspace.axaml",
    path: "/projects/suncode/apps/desktop-avalonia/Views/Projects/ProjectWorkspace.axaml",
    kind: "file",
    depth: 4,
    selected: true,
  },
  {
    id: "agent",
    parent: "project-root",
    name: "agent",
    path: "/projects/suncode/agent",
    kind: "folder",
    depth: 1,
  },
  {
    id: "design-system",
    parent: "project-root",
    name: "design-system",
    path: "/projects/suncode/design-system",
    kind: "folder",
    depth: 1,
  },
];

export const dependencyNodes = [
  {
    id: "shared",
    parent: "dependencies",
    name: "shared-ui",
    path: "/projects/shared-ui",
    kind: "folder",
    depth: 1,
    isDependency: true,
  },
  {
    id: "shared-readme",
    parent: "shared",
    name: "README.md",
    path: "/projects/shared-ui/README.md",
    kind: "file",
    depth: 2,
    isDependency: true,
  },
  {
    id: "other-dependency",
    parent: "dependencies",
    name: "other-dependency-folder",
    path: "/projects/other-dependency-folder",
    kind: "folder",
    depth: 1,
    isDependency: true,
  },
];

export const constrainedExplorerNodes = [
  {
    id: "stress-agents",
    parent: "stress-project-root",
    name: ".agents",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents",
    kind: "folder",
    depth: 1,
  },
  {
    id: "stress-requirements",
    parent: "stress-agents",
    name: "requirements",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents/requirements",
    kind: "folder",
    depth: 2,
  },
  {
    id: "stress-package",
    parent: "stress-requirements",
    name: "2026-08-29-workspace-explorer",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer",
    kind: "folder",
    depth: 3,
  },
  {
    id: "stress-frontend",
    parent: "stress-package",
    name: "frontend",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend",
    kind: "folder",
    depth: 4,
  },
  {
    id: "stress-components",
    parent: "stress-frontend",
    name: "components",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend/components",
    kind: "folder",
    depth: 5,
  },
  {
    id: "stress-selection",
    parent: "stress-components",
    name: "selection",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend/components/selection",
    kind: "folder",
    depth: 6,
  },
  {
    id: "stress-dropdown",
    parent: "stress-selection",
    name: "ModelProviderDropdown.jsx",
    path: "/projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend/components/selection/ModelProviderDropdown.jsx",
    kind: "file",
    depth: 7,
    selected: true,
  },
  {
    id: "stress-apps",
    parent: "stress-project-root",
    name: "apps",
    path: "/projects/client-work/organization/platform/desktop/suncode/apps",
    kind: "folder",
    depth: 1,
  },
  {
    id: "stress-dependency",
    parent: "stress-dependencies",
    name: "shared-ui-foundation",
    path: "/projects/dependencies/design-system/shared-ui-foundation",
    kind: "folder",
    depth: 1,
    isDependency: true,
  },
];

export const projectRoot = {
  id: "project-root",
  name: "suncode",
  path: "/projects/suncode",
  kind: "project-root",
  depth: 0,
};

export const dependencyRoot = {
  id: "dependencies",
  name: "Dependencies",
  path: "",
  kind: "dependencies",
  depth: 0,
  isDependency: true,
};

export const constrainedProjectRoot = {
  id: "stress-project-root",
  name: "suncode",
  path: "/projects/client-work/organization/platform/desktop/suncode",
  kind: "project-root",
  depth: 0,
};

export const constrainedDependencyRoot = {
  id: "stress-dependencies",
  name: "Dependencies",
  path: "",
  kind: "dependencies",
  depth: 0,
  isDependency: true,
};
