// Review and source control fixtures used by the desktop workspace specimens.

export const changes = [
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/projects/desktop/workspace/WorkspacePrimitives.jsx",
    additions: 45,
    deletions: 8,
    staged: false,
    unstaged: true,
  },
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/styles/review.css",
    additions: 62,
    deletions: 24,
    staged: false,
    unstaged: true,
  },
];

export const diffLines = [
  { kind: "hunk", oldLine: "", newLine: "", text: "@@ -56,7 +56,7 @@" },
  { kind: "context", oldLine: "56", newLine: "56", text: "--workspace-content-height: 788px;" },
  { kind: "context", oldLine: "57", newLine: "57", text: "--workspace-composer-height: 126px;" },
  { kind: "context", oldLine: "58", newLine: "58", text: "--workspace-drawer-width: 1337px;" },
  { kind: "deletion", oldLine: "59", newLine: "", text: "--workspace-git-height: 360px;" },
  { kind: "addition", oldLine: "", newLine: "59", text: "--workspace-git-height: 340px;" },
  { kind: "context", oldLine: "60", newLine: "60", text: "--workspace-git-min-height: 240px;" },
  { kind: "context", oldLine: "61", newLine: "61", text: "--workspace-trace-height: 360px;" },
  { kind: "context", oldLine: "62", newLine: "62", text: "--workspace-trace-min-height: 300px;" },
];

export const completedTurnChanges = { added: 3, deleted: 1, edited: 5 };

export const completedTurnChangeSet = [
  {
    status: "A",
    kind: "added",
    path: "design-system/src/projects/desktop/workspace/source-control/index.jsx",
    additions: 38,
    deletions: 0,
    staged: false,
    unstaged: true,
  },
  {
    status: "A",
    kind: "added",
    path: "design-system/src/projects/desktop/workspace/provider-trace/index.jsx",
    additions: 32,
    deletions: 0,
    staged: false,
    unstaged: true,
  },
  {
    status: "A",
    kind: "added",
    path: "design-system/src/components/universal/radio/index.js",
    additions: 12,
    deletions: 0,
    staged: false,
    unstaged: true,
  },
  {
    status: "D",
    kind: "deleted",
    path: "design-system/src/projects/desktop/workspace/legacy-drawer.jsx",
    additions: 0,
    deletions: 74,
    staged: false,
    unstaged: true,
  },
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/projects/desktop/workspace/WorkspacePrimitives.jsx",
    additions: 45,
    deletions: 8,
    staged: false,
    unstaged: true,
  },
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/styles/review.css",
    additions: 62,
    deletions: 24,
    staged: false,
    unstaged: true,
  },
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/app/navigation.js",
    additions: 18,
    deletions: 6,
    staged: false,
    unstaged: true,
  },
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/shared/Icon.jsx",
    additions: 9,
    deletions: 2,
    staged: false,
    unstaged: true,
  },
  {
    status: "M",
    kind: "modified",
    path: "design-system/src/projects/desktop/workspace/conversation/index.jsx",
    additions: 7,
    deletions: 3,
    staged: false,
    unstaged: true,
  },
];

export const currentTurnTodos = [
  { content: "Inspect Avalonia workspace", status: "completed", icon: "check" },
  { content: "Build focused modules", status: "in-progress", icon: "activity" },
  { content: "Verify responsive routes", status: "pending", icon: "more" },
  { content: "Remove stale checkpoint", status: "cancelled", icon: "close" },
];
