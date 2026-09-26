// Session fixtures used by the desktop workspace specimens.

export const primarySessions = [
  {
    id: "workspace-information-architecture",
    title: "Workspace information architecture",
    time: "2 min ago",
    pinned: true,
    status: "running",
  },
  {
    id: "provider-migration-review",
    title: "Provider migration review",
    time: "Yesterday",
    status: "approval",
  },
  {
    id: "desktop-navigation-polish",
    title: "Desktop navigation polish",
    time: "Aug 26",
    status: "idle",
  },
];

export const archivedSessions = [
  {
    id: "legacy-cli-session",
    title: "Legacy CLI session",
    time: "Aug 18",
    archived: true,
  },
  {
    id: "release-notes-draft",
    title: "Release notes draft",
    time: "Aug 12",
    archived: true,
  },
];

export const childSessions = [
  {
    id: "child-ui-ux-review",
    parentSessionId: "workspace-information-architecture",
    agentId: "builtin.ui-ux.v1",
    agentName: "ui-ux-agent",
    agentDisplayName: "UI/UX Agent",
    title: "Review child-session workspace UX",
    task: "Review the child-session panel, read-only detail view, and recent-content behavior against the desktop design system.",
    state: "running",
    time: "Now",
    duration: "38s",
    toolCalls: 3,
    model: "gpt-5.6-sol",
  },
  {
    id: "child-swe-contracts",
    parentSessionId: "workspace-information-architecture",
    agentId: "builtin.swe.v1",
    agentName: "swe-agent",
    agentDisplayName: "Software Engineering Agent",
    title: "Map session contract changes",
    task: "Identify the SDK and persistence contracts required for parent and child session relationships without implementing them.",
    state: "completed",
    time: "4 min ago",
    duration: "1m 12s",
    toolCalls: 6,
    model: "gpt-5.6-sol",
  },
  {
    id: "child-swe-approval",
    parentSessionId: "workspace-information-architecture",
    agentId: "builtin.swe.v1",
    agentName: "swe-agent",
    agentDisplayName: "Software Engineering Agent",
    title: "Validate focused design build",
    task: "Run the focused design-system verification after the parent session finishes the UI specimen.",
    state: "approval",
    time: "7 min ago",
    duration: "24s",
    toolCalls: 2,
    model: "gpt-5.6-sol",
  },
  {
    id: "child-ui-ux-failed",
    parentSessionId: "provider-migration-review",
    agentId: "builtin.ui-ux.v1",
    agentName: "ui-ux-agent",
    agentDisplayName: "UI/UX Agent",
    title: "Inspect compact-width layout",
    task: "Check the child-session detail at the compact workspace breakpoint.",
    state: "failed",
    time: "12 min ago",
    duration: "16s",
    toolCalls: 1,
    model: "gpt-5.6-sol",
  },
];

export const childSessionStateLabels = {
  running: "Running",
  completed: "Completed",
  approval: "Waiting for approval",
  failed: "Failed",
};
const sessions = primarySessions;

export const sessionStatusLabels = {
  running: "Agent running",
  approval: "Waiting for approval",
  question: "Waiting for answer",
  failed: "Turn failed",
  idle: "Agent idle",
};
