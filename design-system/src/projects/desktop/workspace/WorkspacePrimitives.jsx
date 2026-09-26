import { useEffect, useRef, useState } from "react";
import { Button } from "../../../components/universal/button/index.js";
import { ModelDropdown, SingleDropdown } from "../../../components/universal/dropdown/index.js";
import { ConfirmationDialog, Modal } from "../../../components/universal/modal/index.js";
import { Radio } from "../../../components/universal/radio/index.js";
import { Icon } from "../../../shared/Icon.jsx";
import { TrafficLights } from "../../../shared/TrafficLights.jsx";
import { DialogWindowConfirmation } from "../dialog-window/index.jsx";

export { TrafficLights } from "../../../shared/TrafficLights.jsx";

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
const sessions = primarySessions;

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

const workspaceRecentProjects = [
  { id: "suncode", name: "suncode", path: "~/Projects/suncode", current: true },
  {
    id: "avalonia-desktop",
    name: "Avalonia desktop",
    path: "~/Projects/suncode/apps/desktop-avalonia",
  },
  { id: "shared-ui", name: "shared-ui", path: "~/Projects/shared-ui" },
];

const sessionStatusLabels = {
  running: "Agent running",
  approval: "Waiting for approval",
  question: "Waiting for answer",
  failed: "Turn failed",
  idle: "Agent idle",
};

const explorerNodes = [
  {
    id: "agents",
    parent: "project-root",
    name: ".agents",
    path: "/Users/shuyi/Projects/suncode/.agents",
    kind: "folder",
    depth: 1,
  },
  {
    id: "apps",
    parent: "project-root",
    name: "apps",
    path: "/Users/shuyi/Projects/suncode/apps",
    kind: "folder",
    depth: 1,
  },
  {
    id: "desktop",
    parent: "apps",
    name: "desktop-avalonia",
    path: "/Users/shuyi/Projects/suncode/apps/desktop-avalonia",
    kind: "folder",
    depth: 2,
  },
  {
    id: "views",
    parent: "desktop",
    name: "Views",
    path: "/Users/shuyi/Projects/suncode/apps/desktop-avalonia/Views",
    kind: "folder",
    depth: 3,
  },
  {
    id: "workspace-file",
    parent: "views",
    name: "ProjectWorkspace.axaml",
    path: "/Users/shuyi/Projects/suncode/apps/desktop-avalonia/Views/Projects/ProjectWorkspace.axaml",
    kind: "file",
    depth: 4,
    selected: true,
  },
  {
    id: "agent",
    parent: "project-root",
    name: "agent",
    path: "/Users/shuyi/Projects/suncode/agent",
    kind: "folder",
    depth: 1,
  },
  {
    id: "design-system",
    parent: "project-root",
    name: "design-system",
    path: "/Users/shuyi/Projects/suncode/design-system",
    kind: "folder",
    depth: 1,
  },
];
const dependencyNodes = [
  {
    id: "shared",
    parent: "dependencies",
    name: "shared-ui",
    path: "/Users/shuyi/Projects/shared-ui",
    kind: "folder",
    depth: 1,
    isDependency: true,
  },
  {
    id: "shared-readme",
    parent: "shared",
    name: "README.md",
    path: "/Users/shuyi/Projects/shared-ui/README.md",
    kind: "file",
    depth: 2,
    isDependency: true,
  },
  {
    id: "other-dependency",
    parent: "dependencies",
    name: "other-dependency-folder",
    path: "/Users/shuyi/Projects/other-dependency-folder",
    kind: "folder",
    depth: 1,
    isDependency: true,
  },
];

const constrainedExplorerNodes = [
  {
    id: "stress-agents",
    parent: "stress-project-root",
    name: ".agents",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents",
    kind: "folder",
    depth: 1,
  },
  {
    id: "stress-requirements",
    parent: "stress-agents",
    name: "requirements",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents/requirements",
    kind: "folder",
    depth: 2,
  },
  {
    id: "stress-package",
    parent: "stress-requirements",
    name: "2026-08-29-workspace-explorer",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer",
    kind: "folder",
    depth: 3,
  },
  {
    id: "stress-frontend",
    parent: "stress-package",
    name: "frontend",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend",
    kind: "folder",
    depth: 4,
  },
  {
    id: "stress-components",
    parent: "stress-frontend",
    name: "components",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend/components",
    kind: "folder",
    depth: 5,
  },
  {
    id: "stress-selection",
    parent: "stress-components",
    name: "selection",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend/components/selection",
    kind: "folder",
    depth: 6,
  },
  {
    id: "stress-dropdown",
    parent: "stress-selection",
    name: "ModelProviderDropdown.jsx",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/.agents/requirements/2026-08-29-workspace-explorer/frontend/components/selection/ModelProviderDropdown.jsx",
    kind: "file",
    depth: 7,
    selected: true,
  },
  {
    id: "stress-apps",
    parent: "stress-project-root",
    name: "apps",
    path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode/apps",
    kind: "folder",
    depth: 1,
  },
  {
    id: "stress-dependency",
    parent: "stress-dependencies",
    name: "shared-ui-foundation",
    path: "/Users/shuyi/Projects/dependencies/design-system/shared-ui-foundation",
    kind: "folder",
    depth: 1,
    isDependency: true,
  },
];

const projectRoot = {
  id: "project-root",
  name: "suncode",
  path: "/Users/shuyi/Projects/suncode",
  kind: "project-root",
  depth: 0,
};
const dependencyRoot = {
  id: "dependencies",
  name: "Dependencies",
  path: "",
  kind: "dependencies",
  depth: 0,
  isDependency: true,
};
const constrainedProjectRoot = {
  id: "stress-project-root",
  name: "suncode",
  path: "/Users/shuyi/Projects/client-work/organization/platform/desktop/suncode",
  kind: "project-root",
  depth: 0,
};
const constrainedDependencyRoot = {
  id: "stress-dependencies",
  name: "Dependencies",
  path: "",
  kind: "dependencies",
  depth: 0,
  isDependency: true,
};

const changes = [
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

const diffLines = [
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

const workspaceModelGroups = [
  { id: "openai", label: "OpenAI", models: ["gpt-5.6-sol", "gpt-5.5"] },
  { id: "claude", label: "Claude", models: ["claude-sonnet-5", "claude-opus-5"] },
  { id: "deepseek", label: "DeepSeek", models: ["deepseek-v4-flash", "deepseek-v4-pro"] },
];

const runningConversationToolCalls = [
  {
    icon: "terminal",
    title: "Run mvn compile for the workspace shell specimen",
    state: "Running",
    tone: "running",
    request: "mvn -pl design-system compile",
    result: "Command still running",
    liveLabel: "Command output",
    liveOutput: [
      "[INFO] Scanning for projects...",
      "[INFO] ------------------------------------------------------------------------",
      "[INFO] Building design-system 0.0.0-review",
      "[INFO] --- frontend-maven-plugin:1.15.0:npm (npm install) @ design-system ---",
      "[INFO] added 214 packages in 4s",
      "[INFO] --- frontend-maven-plugin:1.15.0:npm (npm run build) @ design-system ---",
      "> design-system@0.0.0 build",
      "> vite build",
      "vite v7.1.3 building for production...",
      "transforming modules...",
      "rendering chunks...",
    ],
    error: "",
  },
  {
    icon: "files",
    title: "Updated workspace routes and modules",
    state: "Succeeded",
    tone: "success",
    request: "workspace route modules",
    result: "8 modules updated",
    error: "",
  },
];

const completedTurnChanges = { added: 3, deleted: 1, edited: 5 };
const completedTurnChangeSet = [
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
const currentTurnTodos = [
  { content: "Inspect Avalonia workspace", status: "completed", icon: "check" },
  { content: "Build focused modules", status: "in-progress", icon: "activity" },
  { content: "Verify responsive routes", status: "pending", icon: "more" },
  { content: "Remove stale checkpoint", status: "cancelled", icon: "close" },
];

function IconButton({ icon, label, active = false, onClick, disabled = false }) {
  return (
    <button
      type="button"
      className={`workspace-icon-button ${active ? "is-active" : ""}`}
      aria-label={label}
      aria-pressed={active}
      onClick={onClick}
      disabled={disabled}
    >
      <Icon name={icon} size={15} />
    </button>
  );
}

function TurnChangeSummary({ added, deleted, edited, onViewChanges }) {
  const stats = [
    ["added", added, "is-added"],
    ["deleted", deleted, "is-deleted"],
    ["edited", edited, "is-edited"],
  ];
  const content = (
    <>
      <span className="workspace-turn-summary-heading">
        <Icon name="check" size={12} />
        <strong>Changes</strong>
      </span>
      <span className="workspace-turn-summary-stats">
        {stats.map(([label, count, tone]) => (
          <span key={label} className={`workspace-turn-summary-stat ${tone}`}>
            <b>{count}</b>
            <small>{label}</small>
          </span>
        ))}
      </span>
    </>
  );
  if (!onViewChanges)
    return (
      <div
        className="workspace-turn-summary"
        role="status"
        aria-label={`Turn complete: ${added} files added, ${deleted} files deleted, ${edited} files edited`}
      >
        {content}
      </div>
    );
  return (
    <button
      type="button"
      className="workspace-turn-summary is-actionable"
      aria-label={`View changes from this turn: ${added} files added, ${deleted} files deleted, ${edited} files edited`}
      title="View turn changes"
      onClick={onViewChanges}
    >
      {content}
    </button>
  );
}

export function SessionPanel({
  compact = false,
  standalone = false,
  initialSessions = sessions,
  initialArchivedSessions = archivedSessions,
  selectedSessionId,
  onArchiveRequest,
  onSelectSession,
}) {
  const [selected, setSelected] = useState(0);
  const [items, setItems] = useState(initialSessions);
  const [archivedItems, setArchivedItems] = useState(initialArchivedSessions);
  const [menu, setMenu] = useState(null);
  const [archivedMenu, setArchivedMenu] = useState(null);
  const [archiveDrawerOpen, setArchiveDrawerOpen] = useState(false);
  const [confirmation, setConfirmation] = useState(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [renameOpen, setRenameOpen] = useState(false);
  const [renameIndex, setRenameIndex] = useState(null);
  const [renameTitle, setRenameTitle] = useState("");
  const createSession = () => {
    setNewTitle("");
    setCreateOpen(true);
  };
  const confirmCreate = () => {
    const title = newTitle.trim();
    if (!title) return;
    setItems((current) => [{ title, time: "Just now" }, ...current]);
    setSelected(0);
    setCreateOpen(false);
  };
  const openRename = (index) => {
    setRenameIndex(index);
    setRenameTitle(items[index]?.title ?? "");
    setMenu(null);
    setRenameOpen(true);
  };
  const confirmRename = () => {
    const title = renameTitle.trim();
    if (!title || renameIndex === null) return;
    setItems((current) =>
      current.map((item, itemIndex) => (itemIndex === renameIndex ? { ...item, title } : item)),
    );
    setRenameOpen(false);
    setRenameIndex(null);
  };
  const togglePin = (index) => {
    setItems((current) =>
      current.map((item, itemIndex) =>
        itemIndex === index ? { ...item, pinned: !item.pinned } : item,
      ),
    );
    setMenu(null);
  };
  const openArchiveConfirmation = (index) => {
    setMenu(null);
    onArchiveRequest?.({
      session: items[index],
      confirm: () => {
        const session = items[index];
        setItems((current) => current.filter((_, itemIndex) => itemIndex !== index));
        setArchivedItems((current) => [{ ...session, archived: true }, ...current]);
        setSelected(0);
      },
    });
  };
  const openRestoreConfirmation = (index) => {
    setArchivedMenu(null);
    setConfirmation({ kind: "restore", session: archivedItems[index], index });
  };
  const openDeleteConfirmation = (index) => {
    setArchivedMenu(null);
    setConfirmation({ kind: "delete", session: archivedItems[index], index });
  };
  const confirmArchivedAction = () => {
    if (!confirmation) return;
    const { index, session, kind } = confirmation;
    if (kind === "restore") {
      const restored = { ...session, archived: false };
      setArchivedItems((current) => current.filter((_, itemIndex) => itemIndex !== index));
      setItems((current) => [restored, ...current]);
      setArchiveDrawerOpen(false);
      setSelected(0);
      onSelectSession?.(restored);
    } else {
      setArchivedItems((current) => current.filter((_, itemIndex) => itemIndex !== index));
    }
    setConfirmation(null);
  };
  return (
    <aside
      className={`workspace-panel workspace-sessions ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}
    >
      <header className="workspace-panel-header">
        <span>SESSIONS</span>
        <IconButton icon="plus" label="New session" onClick={createSession} />
      </header>
      <div className="workspace-session-list">
        {items.map((session, index) => (
          <div className="workspace-session-wrap" key={`${session.title}-${index}`}>
            <button
              type="button"
              className={`workspace-session ${
                selectedSessionId
                  ? (session.id ?? session.title) === selectedSessionId
                    ? "is-selected"
                    : ""
                  : selected === index
                    ? "is-selected"
                    : ""
              }`}
              onClick={() => {
                setSelected(index);
                onSelectSession?.(session);
              }}
            >
              <span className="workspace-session-pin">
                {session.pinned && <Icon name="pin" size={12} />}
              </span>
              <span>
                <strong>{session.title}</strong>
                <small>{session.time}</small>
              </span>
            </button>
            <span
              className={`workspace-session-status is-${session.status ?? "idle"}`}
              role="img"
              aria-label={sessionStatusLabels[session.status] ?? sessionStatusLabels.idle}
              title={sessionStatusLabels[session.status] ?? sessionStatusLabels.idle}
              aria-hidden={session.status === "idle" || !session.status}
            />
            <button
              type="button"
              className="workspace-session-more"
              aria-label={`Actions for ${session.title}`}
              aria-expanded={menu === index}
              onClick={() => setMenu(menu === index ? null : index)}
            >
              <Icon name="more" size={14} />
            </button>
            {menu === index && (
              <div className="workspace-session-menu">
                <button type="button" onClick={() => openRename(index)}>
                  Rename
                </button>
                <button type="button" onClick={() => togglePin(index)}>
                  {session.pinned ? "Unpin" : "Pin"}
                </button>
                <button type="button" onClick={() => openArchiveConfirmation(index)}>
                  Archive
                </button>
              </div>
            )}
          </div>
        ))}
        {!items.length && (
          <div className="workspace-session-empty">
            <Icon name="components" size={22} />
            <strong>No sessions yet</strong>
            <span>Use + to create one.</span>
          </div>
        )}
      </div>
      <button
        type="button"
        className={`workspace-archive-drawer-toggle ${archiveDrawerOpen ? "is-open" : ""}`}
        aria-expanded={archiveDrawerOpen}
        onClick={() => {
          setArchiveDrawerOpen((open) => !open);
          setArchivedMenu(null);
        }}
      >
        <Icon name="arrow-up" size={13} />
        <span>Archived sessions</span>
        <small>{archivedItems.length}</small>
      </button>
      {archiveDrawerOpen && (
        <div className="workspace-archived-drawer" aria-label="Archived sessions">
          <div className="workspace-archived-drawer-header">
            <span>ARCHIVED</span>
            <button
              type="button"
              className="workspace-archive-close"
              onClick={() => setArchiveDrawerOpen(false)}
              aria-label="Close archived sessions"
            >
              <Icon name="close" size={13} />
            </button>
          </div>
          <div className="workspace-session-list workspace-archived-list">
            {archivedItems.map((session, index) => (
              <div className="workspace-session-wrap" key={session.id ?? session.title}>
                <button
                  type="button"
                  className="workspace-session"
                  onClick={() => onSelectSession?.(session)}
                >
                  <span className="workspace-session-pin" />
                  <span>
                    <strong>{session.title}</strong>
                    <small>{session.time}</small>
                  </span>
                </button>
                <span className="workspace-session-archived-label">Archived</span>
                <button
                  type="button"
                  className="workspace-session-more"
                  aria-label={`Actions for ${session.title}`}
                  aria-expanded={archivedMenu === index}
                  onClick={() => setArchivedMenu(archivedMenu === index ? null : index)}
                >
                  <Icon name="more" size={14} />
                </button>
                {archivedMenu === index && (
                  <div className="workspace-session-menu workspace-archived-menu">
                    <button type="button" onClick={() => openRestoreConfirmation(index)}>
                      Restore
                    </button>
                    <button
                      type="button"
                      className="is-danger"
                      onClick={() => openDeleteConfirmation(index)}
                    >
                      Delete permanently
                    </button>
                  </div>
                )}
              </div>
            ))}
            {!archivedItems.length && (
              <div className="workspace-session-empty">
                <strong>No archived sessions</strong>
                <span>Archived sessions will appear here.</span>
              </div>
            )}
          </div>
        </div>
      )}
      <ConfirmationDialog
        open={confirmation?.kind === "restore"}
        title="Restore this session?"
        description="It will return to the active session list and become editable again."
        confirmLabel="Restore session"
        confirmVariant="primary"
        onCancel={() => setConfirmation(null)}
        onConfirm={confirmArchivedAction}
      >
        <div className="confirmation-dialog-target">
          <span>SESSION</span>
          <strong>{confirmation?.session?.title}</strong>
        </div>
      </ConfirmationDialog>
      <ConfirmationDialog
        open={confirmation?.kind === "delete"}
        title="Delete this session permanently?"
        description="This permanently removes the conversation, child sessions, delegated work, and managed images from SunCode. It cannot be undone."
        confirmLabel="Delete permanently"
        onCancel={() => setConfirmation(null)}
        onConfirm={confirmArchivedAction}
      >
        <div className="confirmation-dialog-target">
          <span>SESSION</span>
          <strong>{confirmation?.session?.title}</strong>
        </div>
      </ConfirmationDialog>
      <Modal
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        title="New session"
        description="Give this conversation a name before you begin."
        className="workspace-session-modal"
        actions={
          <>
            <button className="btn" onClick={() => setCreateOpen(false)}>
              Cancel
            </button>
            <button className="btn btn-primary" onClick={confirmCreate} disabled={!newTitle.trim()}>
              Create session
            </button>
          </>
        }
      >
        <input
          id="new-session-name"
          className="field"
          aria-label="Session name"
          value={newTitle}
          onChange={(event) => setNewTitle(event.target.value)}
          placeholder="e.g. Provider migration review"
          onKeyDown={(event) => {
            if (event.key === "Enter") confirmCreate();
          }}
        />
      </Modal>
      <Modal
        open={renameOpen}
        onClose={() => {
          setRenameOpen(false);
          setRenameIndex(null);
        }}
        title="Rename session"
        description="Choose a new name for this conversation."
        className="workspace-session-modal"
        actions={
          <>
            <button
              className="btn"
              onClick={() => {
                setRenameOpen(false);
                setRenameIndex(null);
              }}
            >
              Cancel
            </button>
            <button
              className="btn btn-primary"
              onClick={confirmRename}
              disabled={!renameTitle.trim()}
            >
              Save name
            </button>
          </>
        }
      >
        <input
          id="rename-session-name"
          className="field"
          aria-label="Session name"
          value={renameTitle}
          onChange={(event) => setRenameTitle(event.target.value)}
          placeholder="e.g. Provider migration review"
          onKeyDown={(event) => {
            if (event.key === "Enter") confirmRename();
          }}
        />
      </Modal>
    </aside>
  );
}

export function ExplorerPanel({
  compact = false,
  standalone = false,
  hasDependency = true,
  constrained = false,
  onFileSelect,
  selectedFileId,
}) {
  const roots = constrained
    ? [constrainedProjectRoot, constrainedDependencyRoot]
    : [projectRoot, dependencyRoot];
  const treeNodes = constrained ? constrainedExplorerNodes : explorerNodes;
  const dependencyTreeNodes = constrained
    ? constrainedExplorerNodes.filter((node) => node.isDependency)
    : dependencyNodes;
  const nodes = [
    ...roots.slice(0, 1),
    ...treeNodes.filter((node) => !node.isDependency),
    roots[1],
    ...(hasDependency ? dependencyTreeNodes : []),
  ];
  const [expanded, setExpanded] = useState(
    () =>
      new Set(
        constrained
          ? [
              "stress-project-root",
              "stress-agents",
              "stress-requirements",
              "stress-package",
              "stress-frontend",
              "stress-components",
              "stress-selection",
              "stress-dependencies",
            ]
          : ["project-root", "apps", "desktop", "views", "dependencies", "shared"],
      ),
  );
  const [localSelectedFileId, setLocalSelectedFileId] = useState(
    () => nodes.find((node) => node.kind === "file" && node.selected)?.id ?? null,
  );
  const activeFileId = selectedFileId ?? localSelectedFileId;
  const visibleNodes = nodes.filter((node) => {
    let parentId = node.parent;
    while (parentId) {
      if (!expanded.has(parentId)) return false;
      parentId = nodes.find((candidate) => candidate.id === parentId)?.parent;
    }
    return true;
  });
  const toggleNode = (node) => {
    if (node.kind === "file") return;
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(node.id)) next.delete(node.id);
      else next.add(node.id);
      return next;
    });
  };
  return (
    <aside
      className={`workspace-panel workspace-explorer ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} ${constrained ? "is-constrained" : ""}`}
    >
      <header className="workspace-panel-header">
        <span>EXPLORER</span>
        <span className="workspace-panel-actions">
          <IconButton icon="refresh" label="Refresh explorer" disabled />
          <IconButton icon="plus" label="Add dependency folder" disabled />
        </span>
      </header>
      <div className="workspace-tree" role="tree" aria-label="Project files">
        {visibleNodes.map((node) => {
          const isContainer = node.kind !== "file";
          const dependencyParent = constrained ? "stress-dependencies" : "dependencies";
          const isDependencyNode = node.isDependency || node.parent === dependencyParent;
          const extension = node.name.includes(".") ? node.name.split(".").pop().toLowerCase() : "";
          const fileIcon =
            extension === "md"
              ? "file-markdown"
              : ["jsx", "js", "ts", "tsx", "rs", "go", "java", "py"].includes(extension)
                ? "file-code"
                : ["json", "yaml", "yml", "toml", "xml", "axaml"].includes(extension)
                  ? "file-config"
                  : extension
                    ? "file-text"
                    : "file";
          const showPath = node.parent === dependencyParent;
          const iconName =
            node.kind === "project-root"
              ? "project"
              : node.kind === "dependencies"
                ? "dependencies"
                : isContainer
                  ? "folder"
                  : fileIcon;
          return (
            <button
              key={node.id}
              type="button"
              role="treeitem"
              aria-level={node.depth + 1}
              aria-expanded={isContainer ? expanded.has(node.id) : undefined}
              aria-selected={activeFileId === node.id || undefined}
              className={`workspace-tree-row ${activeFileId === node.id ? "is-selected" : ""} ${isDependencyNode ? "is-dependency" : ""} ${node.kind === "dependencies" ? "is-dependency-root" : ""} ${node.kind === "workspace" ? "is-workspace-root" : ""}`}
              style={{ "--tree-depth": node.depth }}
              onClick={() => {
                if (isContainer) {
                  toggleNode(node);
                  return;
                }
                setLocalSelectedFileId(node.id);
                onFileSelect?.(node);
              }}
            >
              {isContainer ? (
                <Icon
                  name="chevron-right"
                  className={expanded.has(node.id) ? "is-open" : ""}
                  size={8}
                />
              ) : (
                <span />
              )}
              <Icon name={iconName} size={14} />
              <span className="workspace-tree-copy" title={node.path || node.name}>
                <strong>{node.name}</strong>
                {showPath && node.path && <small>{node.path}</small>}
              </span>
            </button>
          );
        })}
      </div>
    </aside>
  );
}

const editorDocuments = {
  "workspace-file": {
    name: "ProjectWorkspace.axaml",
    path: "apps/desktop-avalonia/Views/Projects/ProjectWorkspace.axaml",
    language: "AXAML",
  },
  "shared-readme": {
    name: "README.md",
    path: "../shared-ui/README.md",
    language: "Markdown",
  },
  "stress-dropdown": {
    name: "ModelProviderDropdown.jsx",
    path: "frontend/components/selection/ModelProviderDropdown.jsx",
    language: "JavaScript JSX",
  },
};

const editorCodeLines = [
  [
    ["punctuation", "<"],
    ["keyword", "UserControl"],
    ["punctuation", " "],
    ["type", "xmlns"],
    ["punctuation", '="'],
    ["string", "https://github.com/avaloniaui"],
    ["punctuation", '"'],
    ["punctuation", ">"],
  ],
  [
    ["punctuation", "  <"],
    ["keyword", "Grid"],
    ["punctuation", " "],
    ["type", "RowDefinitions"],
    ["punctuation", '="'],
    ["string", "36,*,20"],
    ["punctuation", '" />'],
  ],
  [["comment", "  <!-- Conversation and editor share this content slot. -->"]],
  [
    ["punctuation", "  <"],
    ["keyword", "Grid"],
    ["punctuation", " "],
    ["type", "Grid.Row"],
    ["punctuation", '="'],
    ["number", "1"],
    ["punctuation", '">'],
  ],
  [
    ["punctuation", "    <"],
    ["keyword", "chat:ChatArea"],
    ["punctuation", " "],
    ["type", "IsVisible"],
    ["punctuation", '="'],
    ["string", "{Binding IsConversationVisible}"],
    ["punctuation", " />"],
  ],
  [
    ["punctuation", "    <"],
    ["keyword", "editor:ReadOnlyEditor"],
    ["punctuation", " "],
    ["type", "IsVisible"],
    ["punctuation", '="'],
    ["string", "{Binding IsEditorVisible}"],
    ["punctuation", " />"],
  ],
  [
    ["punctuation", "  </"],
    ["keyword", "Grid"],
    ["punctuation", ">"],
  ],
  [
    ["punctuation", "  <"],
    ["keyword", "TextBlock"],
    ["punctuation", " "],
    ["type", "Text"],
    ["punctuation", '="'],
    ["string", "READ ONLY"],
    ["punctuation", '" />'],
  ],
  [
    ["punctuation", "</"],
    ["keyword", "UserControl"],
    ["punctuation", ">"],
  ],
];

const markdownEditorLines = [
  [["keyword", "# Shared UI"]],
  [],
  [
    ["punctuation", "This package contains the "],
    ["type", "desktop component foundations"],
    ["punctuation", "."],
  ],
  [],
  [["keyword", "## Usage"]],
  [],
  [
    ["punctuation", "- "],
    ["string", "Import semantic tokens from the shared theme."],
  ],
  [
    ["punctuation", "- "],
    ["string", "Keep project files read-only when opened as dependencies."],
  ],
];

const jsxEditorLines = [
  [
    ["keyword", "export function"],
    ["punctuation", " "],
    ["type", "ModelProviderDropdown"],
    ["punctuation", "({ models }) {"],
  ],
  [
    ["keyword", "  const"],
    ["punctuation", " enabled = models.filter((model) => model.enabled);"],
  ],
  [
    ["keyword", "  return"],
    ["punctuation", " ("],
  ],
  [
    ["punctuation", "    <"],
    ["type", "SingleDropdown"],
    ["punctuation", " options={enabled} ariaLabel="],
    ["string", '"Model provider"'],
    ["punctuation", " />"],
  ],
  [["punctuation", "  );"]],
  [["punctuation", "}"]],
];

export function EditorPanel({
  file,
  state = "ready",
  compact = false,
  standalone = false,
  constrained = false,
}) {
  const document = editorDocuments[file?.id] ?? editorDocuments["workspace-file"];
  const documentLines =
    document.language === "Markdown"
      ? markdownEditorLines
      : document.language === "JavaScript JSX"
        ? jsxEditorLines
        : editorCodeLines;
  const stateLabel =
    state === "loading"
      ? "Loading file"
      : state === "error"
        ? "Unable to read file"
        : state === "empty"
          ? "Empty document"
          : "Read-only document";
  return (
    <section
      className={`workspace-panel workspace-editor ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} ${constrained ? "is-constrained" : ""} is-${state}`}
      aria-label="Read-only file editor"
    >
      <header className="workspace-editor-header">
        <div className="workspace-editor-file">
          <Icon
            name={
              state === "error"
                ? "file-text"
                : document.language === "AXAML"
                  ? "file-config"
                  : document.language === "Markdown"
                    ? "file-markdown"
                    : "file-code"
            }
            size={16}
          />
          <span>
            <strong>{document.name}</strong>
            <small title={document.path}>{document.path}</small>
          </span>
        </div>
        <div className="workspace-editor-meta">
          <span>{document.language}</span>
          <b>READ ONLY</b>
        </div>
      </header>
      {state === "ready" && (
        <pre
          className="workspace-editor-document"
          aria-label={`${document.name} source`}
          aria-readonly="true"
          tabIndex="0"
        >
          <code>
            {documentLines.map((line, index) => (
              <span className="workspace-editor-line" key={`${document.name}-${index + 1}`}>
                <span className="workspace-editor-line-number">{index + 1}</span>
                <span className="workspace-editor-line-code">
                  {line.map(([tone, text], tokenIndex) => (
                    <span
                      className={`workspace-editor-token is-${tone}`}
                      key={`${index}-${tokenIndex}`}
                    >
                      {text}
                    </span>
                  ))}
                </span>
              </span>
            ))}
          </code>
        </pre>
      )}
      {state !== "ready" && (
        <div
          className={`workspace-editor-state is-${state}`}
          role={state === "error" ? "alert" : "status"}
        >
          <Icon
            name={state === "error" ? "close" : state === "empty" ? "file-text" : "activity"}
            size={24}
          />
          <strong>{stateLabel}</strong>
          <span>
            {state === "loading"
              ? "Reading the selected project file..."
              : state === "error"
                ? "The file could not be read within the project boundary."
                : "This file contains no text to display."}
          </span>
        </div>
      )}
    </section>
  );
}

const createSampleAttachment = (name, title, detail) => {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200" viewBox="0 0 320 200"><rect width="320" height="200" fill="#f2f3f5"/><rect x="18" y="18" width="284" height="164" rx="8" fill="#ffffff" stroke="#d6d9de"/><rect x="34" y="38" width="68" height="10" rx="3" fill="#23262b"/><rect x="34" y="62" width="112" height="8" rx="3" fill="#d6d9de"/><rect x="34" y="82" width="210" height="8" rx="3" fill="#e7e9ed"/><rect x="34" y="112" width="${title === "Workspace" ? 152 : 184}" height="28" rx="5" fill="#${title === "Workspace" ? "dfe3e8" : "eceef1"}"/><circle cx="270" cy="48" r="8" fill="#${detail}"/><text x="34" y="164" fill="#7d848e" font-family="Arial,sans-serif" font-size="10">${title}</text></svg>`;
  return {
    id: `sample-${name}`,
    name,
    type: "image/svg+xml",
    url: `data:image/svg+xml,${encodeURIComponent(svg)}`,
  };
};

export const sampleConversationAttachments = [
  createSampleAttachment("workspace-layout.svg", "Workspace", "626a73"),
  createSampleAttachment("settings-reference.svg", "Settings", "8a919b"),
];

export function ConversationPanel({
  compact = false,
  standalone = false,
  state = "content-waiting",
  initialAttachments = [],
  initialSentAttachments = [],
  imageInputEnabled = false,
  onViewChanges,
  onOpenToolActivity,
}) {
  const [message, setMessage] = useState(
    state === "immersive-composer"
      ? "Please refactor the conversation layout into focused review states, preserve the existing attachment behavior, and keep the visual language aligned with Quiet Control Desk. I want the resulting specimen to stay calm even when the prompt is several paragraphs long.\n\nAlso add a clearer tool-inspection state so long-running commands can be observed without leaving the conversation surface."
      : "",
  );
  const [attachments, setAttachments] = useState(initialAttachments);
  const [sentAttachments, setSentAttachments] = useState(initialSentAttachments);
  const [previewAttachment, setPreviewAttachment] = useState(null);
  const [composerExpanded, setComposerExpanded] = useState(false);
  const [copiedResponse, setCopiedResponse] = useState(false);
  const [showScrollToBottom, setShowScrollToBottom] = useState(state === "scrolled-up");
  const conversationScrollRef = useRef(null);
  const attachmentInputRef = useRef(null);
  const localAttachmentUrls = useRef(new Set());
  const copyResetTimerRef = useRef(null);
  const hasSession = state !== "no-session";
  const hasContent = state !== "new-session" && hasSession;
  const archived = state === "archived";
  const updating = state === "content-updating" || state === "live-tool-stream";
  const thinking = state === "content-thinking";
  const modelUnavailable = state === "model-unavailable";
  const turnActive = updating || thinking;
  const scrollToBottom = () => {
    conversationScrollRef.current?.scrollTo({
      top: conversationScrollRef.current.scrollHeight,
      behavior: "smooth",
    });
    setShowScrollToBottom(false);
  };
  const activeTool = updating
    ? runningConversationToolCalls.find((tool) => tool.tone === "running")
    : null;
  const messageCharacters = Array.from(message).length;
  const handleAttachmentChange = (event) => {
    if (!imageInputEnabled) return;
    const selectedImages = Array.from(event.target.files ?? []).filter((file) =>
      file.type.startsWith("image/"),
    );
    if (selectedImages.length)
      setAttachments((current) => {
        const newAttachments = selectedImages
          .slice(0, Math.max(0, 3 - current.length))
          .map((file, index) => ({
            id: `${file.name}-${file.lastModified}-${index}`,
            name: file.name,
            type: file.type,
            url: URL.createObjectURL(file),
            local: true,
          }));
        newAttachments.forEach((attachment) => localAttachmentUrls.current.add(attachment.url));
        return [...current, ...newAttachments];
      });
    event.target.value = "";
  };
  const removeAttachment = (id) => {
    setAttachments((current) => {
      const removed = current.find((attachment) => attachment.id === id);
      if (removed?.local) {
        URL.revokeObjectURL(removed.url);
        localAttachmentUrls.current.delete(removed.url);
      }
      return current.filter((attachment) => attachment.id !== id);
    });
  };
  useEffect(
    () => () => {
      localAttachmentUrls.current.forEach((url) => URL.revokeObjectURL(url));
      if (copyResetTimerRef.current) window.clearTimeout(copyResetTimerRef.current);
    },
    [],
  );
  const sendMessage = () => {
    if (!message.trim() && !attachments.length) return;
    if (attachments.length) setSentAttachments((current) => [...current, ...attachments]);
    setAttachments([]);
    setMessage("");
    setComposerExpanded(false);
  };
  const copyResponse = async () => {
    await navigator.clipboard?.writeText(
      "I split Workspace into a complete composition and focused pages for sessions, explorer, conversation, review, source control, and provider trace.",
    );
    setCopiedResponse(true);
    if (copyResetTimerRef.current) window.clearTimeout(copyResetTimerRef.current);
    copyResetTimerRef.current = window.setTimeout(() => setCopiedResponse(false), 1400);
  };
  return (
    <section
      className={`workspace-conversation ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} workspace-conversation-${state}`}
    >
      {!hasSession && (
        <div className="workspace-conversation-empty">
          <Icon name="workspace" size={24} />
          <strong>No session selected</strong>
          <span>Create or select a session to start a conversation.</span>
        </div>
      )}
      {hasSession && !hasContent && (
        <div className="workspace-conversation-empty">
          <Icon name="plus" size={24} />
          <strong>New session</strong>
          <span>Send a message to start this conversation.</span>
        </div>
      )}
      {hasContent && (
        <>
          <div
            ref={conversationScrollRef}
            className="workspace-conversation-scroll"
            onScroll={(event) => {
              const element = event.currentTarget;
              setShowScrollToBottom(
                element.scrollHeight - element.clientHeight - element.scrollTop > 32,
              );
            }}
          >
            <div className="workspace-message workspace-message-user">
              {sentAttachments.length > 0 && (
                <div
                  className="workspace-message-attachments"
                  aria-label="Images sent with this message"
                >
                  {sentAttachments.map((attachment) => (
                    <button
                      type="button"
                      className="workspace-message-attachment"
                      key={attachment.id}
                      onClick={() => setPreviewAttachment(attachment)}
                      aria-label={`View ${attachment.name}`}
                      title="View image"
                    >
                      <img src={attachment.url} alt={attachment.name} />
                    </button>
                  ))}
                </div>
              )}
              <p>Review the conversation layout and keep the existing attachment behavior.</p>
            </div>
            {state === "intermediate-assistant" && (
              <div className="workspace-message workspace-message-assistant workspace-message-assistant-intermediate">
                <p>
                  Let me confirm the module types and dependency relationships before I summarize
                  the project structure.
                </p>
              </div>
            )}
            {activeTool && <div className="workspace-assistant-duration">Working for 18s</div>}
            {activeTool && (
              <button
                type="button"
                className="workspace-active-tool"
                onClick={() =>
                  onOpenToolActivity
                    ? onOpenToolActivity("0198e82c", 1)
                    : (window.location.hash = "/projects/desktop/workspace/tool-activity")
                }
              >
                <Icon name={activeTool.icon} size={14} />
                <span>{activeTool.title}</span>
                <small>Running · View in Tool activity</small>
                <Icon name="arrow" size={12} />
              </button>
            )}
            {!turnActive ? (
              <div className="workspace-message workspace-message-assistant">
                <div className="workspace-assistant-duration">Worked for 42s</div>
                <p>
                  I split Workspace into a complete composition and focused pages for sessions,
                  explorer, conversation, review, source control, and provider trace.
                </p>
                <div className="workspace-message-footer">
                  <div className="workspace-message-actions">
                    <button
                      type="button"
                      className={`workspace-copy ${copiedResponse ? "is-copied" : ""}`}
                      aria-label={copiedResponse ? "Copied response" : "Copy response"}
                      title={copiedResponse ? "Copied" : "Copy response"}
                      onClick={copyResponse}
                    >
                      <Icon name={copiedResponse ? "check" : "copy"} size={13} />
                    </button>
                    <time
                      className="workspace-message-completed-time"
                      dateTime="2026-09-12T14:32:18+08:00"
                      title="Completed at 14:32"
                    >
                      14:32
                    </time>
                  </div>
                  <TurnChangeSummary {...completedTurnChanges} onViewChanges={onViewChanges} />
                </div>
              </div>
            ) : (
              <div className="workspace-message workspace-message-assistant workspace-message-assistant-status">
                <p>
                  Inspecting the workspace shell and keeping the long-running build visible in the
                  conversation timeline.
                </p>
              </div>
            )}
            {thinking && (
              <div
                className="workspace-thinking-indicator"
                role="status"
                aria-label="Assistant is thinking"
              >
                <span>Thinking</span>
              </div>
            )}
            {updating && !thinking && (
              <div
                className="workspace-running-indicator"
                role="status"
                aria-label="Agent is working"
              >
                <i />
                <i />
                <i />
              </div>
            )}
          </div>
          {showScrollToBottom && (
            <button
              type="button"
              className="workspace-scroll-to-bottom"
              onClick={scrollToBottom}
              aria-label="Scroll to bottom"
              title="Scroll to bottom"
            >
              <Icon name="arrow-down" size={14} />
            </button>
          )}
        </>
      )}
      {hasSession && !archived && (
        <div className={`workspace-composer ${attachments.length ? "has-attachments" : ""}`}>
          {attachments.length > 0 && (
            <div className="workspace-attachment-strip" aria-label="Attached images">
              {attachments.map((attachment) => (
                <div className="workspace-attachment" key={attachment.id}>
                  <button
                    type="button"
                    className="workspace-attachment-preview"
                    onClick={() => setPreviewAttachment(attachment)}
                    aria-label={`View ${attachment.name}`}
                    title="View image"
                  >
                    <img src={attachment.url} alt={attachment.name} />
                  </button>
                  <button
                    type="button"
                    className="workspace-attachment-remove"
                    aria-label={`Remove ${attachment.name}`}
                    title="Remove image"
                    onClick={() => removeAttachment(attachment.id)}
                  >
                    <Icon name="close" size={10} />
                  </button>
                </div>
              ))}
            </div>
          )}
          {modelUnavailable && (
            <div className="workspace-composer-status" role="status">
              <span>Configure an API key in Settings to send messages.</span>
            </div>
          )}
          <textarea
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            placeholder={modelUnavailable ? "" : "Ask SunCode to work on this project"}
            aria-label="Message SunCode"
            disabled={modelUnavailable}
          />
          <div className="workspace-composer-footer">
            <div className="workspace-composer-actions">
              <button
                type="button"
                className="workspace-attach"
                aria-label="Add attachment"
                title={
                  !imageInputEnabled
                    ? "Selected model does not support image input"
                    : attachments.length >= 3
                      ? "Maximum 3 images"
                      : "Add image"
                }
                disabled={!imageInputEnabled || attachments.length >= 3}
                onClick={() => attachmentInputRef.current?.click()}
              >
                <Icon name="plus" size={14} />
              </button>
              <button
                type="button"
                className="workspace-expand-composer"
                aria-label="Open expanded composer"
                title="Open expanded composer"
                onClick={() => setComposerExpanded(true)}
                disabled={modelUnavailable}
              >
                <Icon name="expand" size={13} />
              </button>
            </div>
            <input
              ref={attachmentInputRef}
              className="workspace-attachment-input"
              type="file"
              accept="image/*"
              multiple
              tabIndex={-1}
              onChange={handleAttachmentChange}
            />
            <div className="workspace-composer-options">
              <ModelDropdown
                groups={
                  imageInputEnabled
                    ? [{ id: "specimen", label: "Specimen", models: ["vision-input specimen"] }]
                    : modelUnavailable
                      ? [{ id: "openai", label: "OpenAI", models: ["gpt-5.6-sol"] }]
                      : workspaceModelGroups
                }
                initialValue={
                  imageInputEnabled
                    ? "vision-input specimen"
                    : modelUnavailable
                      ? "gpt-5.6-sol"
                      : "gpt-5.6-sol"
                }
                className="workspace-model-dropdown"
              />
              <SingleDropdown
                options={["Medium", "High"]}
                initialValue="High"
                ariaLabel="Reasoning effort"
                className="workspace-reasoning-dropdown"
              />
              <Button
                variant="primary"
                className="workspace-send"
                icon="arrow-up"
                aria-label="Send message"
                disabled={modelUnavailable || (!message.trim() && !attachments.length)}
                onClick={sendMessage}
              />
            </div>
          </div>
        </div>
      )}
      {archived && (
        <div className="workspace-archived-readonly" role="status">
          <Icon name="lock" size={14} />
          <span>This session is archived and read-only.</span>
          <small>Restore it from Archived sessions to continue the conversation.</small>
        </div>
      )}
      <Modal
        open={Boolean(previewAttachment)}
        title={previewAttachment?.name ?? "Image preview"}
        onClose={() => setPreviewAttachment(null)}
        className="workspace-image-modal"
      >
        <div className="workspace-image-preview">
          {previewAttachment && <img src={previewAttachment.url} alt={previewAttachment.name} />}
        </div>
      </Modal>
      <Modal
        open={composerExpanded}
        onClose={() => setComposerExpanded(false)}
        className="workspace-composer-modal"
        hideTitle
        ariaLabel="Expanded composer"
        hideClose
        actions={
          <>
            <button type="button" className="btn" onClick={() => setComposerExpanded(false)}>
              Close
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={sendMessage}
              disabled={!message.trim() && !attachments.length}
            >
              Send message
            </button>
          </>
        }
      >
        <div className="workspace-composer-modal-content">
          <textarea
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            placeholder="Ask SunCode to work on this project"
            aria-label="Expanded message composer"
          />
          <div className="workspace-composer-modal-footer">
            <strong aria-live="polite">{messageCharacters} characters</strong>
          </div>
        </div>
      </Modal>
    </section>
  );
}

const childSessionStateLabels = {
  running: "Running",
  completed: "Completed",
  approval: "Waiting for approval",
  failed: "Failed",
};

export function ChildSessionsPanel({
  compact = false,
  standalone = false,
  sessions: items = childSessions,
  selectedChildId = items[0]?.id ?? "",
  parentTitle = "Workspace information architecture",
  onSelectChild,
}) {
  return (
    <aside
      className={`workspace-panel workspace-child-sessions ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}
    >
      <div className="workspace-child-sessions-header">
        <div>
          <h3>Child sessions</h3>
          <span>{parentTitle}</span>
        </div>
        <code>{items.length}</code>
      </div>
      {items.length ? (
        <div className="workspace-child-session-list" role="group" aria-label="Child sessions">
          {items.map((child) => (
            <button
              type="button"
              key={child.id}
              className={`workspace-child-session ${selectedChildId === child.id ? "is-selected" : ""}`}
              aria-current={selectedChildId === child.id ? "true" : undefined}
              onClick={() => onSelectChild?.(child)}
            >
              <span className="workspace-child-session-icon">
                <Icon name="agent" size={14} />
              </span>
              <span className="workspace-child-session-copy">
                <span>
                  <strong>{child.title}</strong>
                  <i
                    className={`workspace-child-session-status is-${child.state}`}
                    aria-label={childSessionStateLabels[child.state]}
                    title={childSessionStateLabels[child.state]}
                  />
                </span>
                <small>{child.agentDisplayName}</small>
                <span className="workspace-child-session-meta">
                  <b>{childSessionStateLabels[child.state]}</b>
                  <span>·</span>
                  <span>{child.time}</span>
                </span>
              </span>
              <Icon name="chevron-right" size={12} className="workspace-child-session-chevron" />
            </button>
          ))}
        </div>
      ) : (
        <div className="workspace-child-session-empty">
          <Icon name="agent" size={22} />
          <strong>No child sessions yet</strong>
          <span>Child sessions appear when the main agent delegates work.</span>
        </div>
      )}
    </aside>
  );
}

export function ChildSessionDetail({ child = childSessions[0], standalone = false, onBack }) {
  if (!child)
    return (
      <section className={`workspace-child-detail ${standalone ? "is-standalone" : ""}`}>
        <div className="workspace-child-detail-empty">
          <Icon name="agent" size={24} />
          <strong>Select a child session</strong>
          <span>Choose a delegated session from the right panel to inspect its activity.</span>
        </div>
      </section>
    );

  const isRunning = child.state === "running";
  const isCompleted = child.state === "completed";
  const isApproval = child.state === "approval";
  const isFailed = child.state === "failed";

  return (
    <section className={`workspace-child-detail ${standalone ? "is-standalone" : ""}`}>
      <header className="workspace-child-detail-header">
        {onBack ? (
          <button type="button" onClick={onBack} aria-label="Back to main session">
            <Icon name="chevron-right" size={13} />
          </button>
        ) : (
          <span className="workspace-child-detail-back-placeholder" aria-hidden="true" />
        )}
        <span className="workspace-child-detail-agent-icon">
          <Icon name="agent" size={15} />
        </span>
        <div>
          <strong>{child.title}</strong>
          <span>{child.agentDisplayName}</span>
        </div>
        <span className={`workspace-child-detail-state is-${child.state}`}>
          <i />
          {childSessionStateLabels[child.state]}
        </span>
      </header>
      <div className="workspace-child-detail-scroll">
        <dl className="workspace-child-detail-meta">
          <div>
            <dt>Agent</dt>
            <dd>
              <code>{child.agentName}</code>
            </dd>
          </div>
          <div>
            <dt>Model</dt>
            <dd>
              <code>{child.model}</code>
            </dd>
          </div>
          <div>
            <dt>Duration</dt>
            <dd>{child.duration}</dd>
          </div>
          <div>
            <dt>Tool calls</dt>
            <dd>{child.toolCalls}</dd>
          </div>
        </dl>
        <div className="workspace-child-timeline">
          <article className="workspace-child-event is-delegation">
            <span className="workspace-child-event-mark">
              <Icon name="message" size={13} />
            </span>
            <div>
              <header>
                <strong>Task from Main Agent</strong>
                <time>{child.time}</time>
              </header>
              <p>{child.task}</p>
            </div>
          </article>

          <article className="workspace-child-event is-tool">
            <span className="workspace-child-event-mark">
              <Icon name="tool" size={13} />
            </span>
            <div>
              <header>
                <strong>{isApproval ? "bash" : "read · glob · grep"}</strong>
                <time>{isApproval ? "Awaiting approval" : `${child.toolCalls} calls`}</time>
              </header>
              <code>
                {isApproval
                  ? "npm run build --prefix design-system"
                  : "Inspected the relevant workspace, settings, and design-system specifications."}
              </code>
            </div>
          </article>

          {isApproval && (
            <div className="workspace-child-approval">
              <Icon name="lock" size={15} />
              <div>
                <strong>Run a shell command</strong>
                <span>Shell command · npm run build --prefix design-system</span>
                <div>
                  <button className="btn primary small" type="button">Allow once</button>
                  <button className="btn small" type="button">Deny</button>
                </div>
              </div>
            </div>
          )}

          {isFailed && (
            <div className="workspace-child-failure-note">
              <Icon name="activity" size={15} />
              <div>
                <strong>Child session stopped</strong>
                <span>
                  The compact-width inspection could not load the requested view. The main agent can
                  decide whether to delegate it again.
                </span>
              </div>
            </div>
          )}

          {(isRunning || isCompleted) && (
            <article
              className={`workspace-child-event is-response ${isRunning ? "is-running" : ""}`}
            >
              <span className="workspace-child-event-mark">
                <Icon name="agent" size={13} />
              </span>
              <div>
                <header>
                  <strong>{child.agentDisplayName}</strong>
                  <time>{isRunning ? "Working" : "Completed"}</time>
                </header>
                {isRunning ? (
                  <>
                    <p>
                      I am checking how the new right panel yields space before the central reading
                      surface becomes too narrow.
                    </p>
                    <span className="workspace-child-working">Reviewing responsive states</span>
                  </>
                ) : (
                  <>
                    <p>
                      The contract should keep the primary session as the authority root while child
                      sessions remain independently inspectable.
                    </p>
                    <ul>
                      <li>Exclude child sessions from the primary Sessions list.</li>
                      <li>Expose child snapshots through read-only detail surfaces.</li>
                      <li>Keep delegated checkpoints scoped to the child session.</li>
                    </ul>
                  </>
                )}
              </div>
            </article>
          )}
        </div>
      </div>
    </section>
  );
}

export function ReviewPanel({ compact = false, standalone = false, state = "approval" }) {
  const running = state === "running" || state === "running-no-changes";
  const compacting = state === "compacting";
  const waiting = state === "approval" || state === "question";
  const idle = state === "idle";
  const failed = state === "failed";
  const noChanges = state === "running-no-changes";
  const inactive = idle;
  const statusTone = idle
    ? "idle"
    : failed
      ? "failed"
      : compacting
        ? "compacting"
        : running
          ? "running"
          : state === "approval"
            ? "approval"
            : "question";
  const statusLabel = idle
    ? "Agent idle"
    : failed
      ? "Turn failed"
      : compacting
        ? "Compacting conversation context"
        : running
          ? noChanges
            ? "Agent running, no file changes"
            : "Agent running"
          : state === "approval"
            ? "Waiting for approval"
            : "Waiting for answer";
  const [questionOption, setQuestionOption] = useState(null);
  const [customAnswer, setCustomAnswer] = useState("");
  const questionOptions = [
    {
      id: "a",
      label: "A · Stack list above detail",
      description: "Keep the focused trace easy to scan on narrow screens.",
    },
    {
      id: "b",
      label: "B · Keep a narrow split view",
      description: "Preserve side-by-side context when the viewport allows it.",
    },
    {
      id: "c",
      label: "C · Custom answer",
      description: "Provide a different behavior in your own words.",
    },
  ];
  const showTurnChanges = !inactive && !noChanges && !failed && !compacting;
  const turnChangeRows = completedTurnChangeSet;
  const [turnChangesOpen, setTurnChangesOpen] = useState(false);
  return (
    <aside
      className={`workspace-panel workspace-review ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}
    >
      <div className={`workspace-review-heading is-${statusTone}`}>
        <h3>
          {idle
            ? "No active process"
            : failed
              ? "Turn stopped"
              : compacting
                ? "Compacting context"
                : running
                  ? "1 active process"
                  : "Awaiting input"}
        </h3>
        <i role="status" aria-label={statusLabel} title={statusLabel} />
      </div>
      <ContextUsage />
      {inactive && (
        <div className="workspace-review-empty">
          <Icon name="activity" size={22} />
          <strong>Agent is idle</strong>
          <span>Start a turn from the conversation composer.</span>
        </div>
      )}
      {failed && (
        <div className="workspace-failure-card">
          <div>
            <span>TURN STOPPED</span>
            <b>FAILED</b>
          </div>
          <strong>Provider request failed</strong>
          <p>The turn ended before completion. No further tool calls will run.</p>
          <dl>
            <div>
              <dt>Reason</dt>
              <dd>Network unavailable</dd>
            </div>
            <div>
              <dt>Turn</dt>
              <dd>
                <code>turn_01JY7F3K9M</code>
              </dd>
            </div>
          </dl>
          <Button variant="primary" size="sm">
            Retry turn
          </Button>
        </div>
      )}
      {compacting && (
        <div className="workspace-process-card workspace-compaction-card">
          <div>
            <i />
            <strong>Context compaction</strong>
            <small>Compacting</small>
          </div>
          <code>Turn turn_01JY7F3K9M</code>
          <span>Context&nbsp; Summarizing earlier turns</span>
          <span>Next&nbsp; Resume model call</span>
        </div>
      )}
      {running && (
        <>
          <div className="workspace-process-card">
            <div>
              <i />
              <strong>Agent loop</strong>
              <small>Running</small>
            </div>
            <code>Turn turn_01JY7F3K9M</code>
            <span>Model&nbsp; gpt-5.6-sol</span>
            <span>
              Latest&nbsp;{" "}
              {noChanges ? "Inspecting project, no file changes yet" : "Editing workspace modules"}
            </span>
          </div>
          <div className="workspace-todo-card">
            <div>
              <span>TODO</span>
              <small>{currentTurnTodos.length} items</small>
            </div>
            {currentTurnTodos.map((todo) => (
              <p key={todo.content} className={`workspace-todo-item is-${todo.status}`}>
                <span className="workspace-todo-marker">
                  <Icon name={todo.icon} size={11} />
                </span>
                <span>{todo.content}</span>
              </p>
            ))}
          </div>
        </>
      )}
      {showTurnChanges && (
        <div className="workspace-turn-changes">
          <button
            type="button"
            className="workspace-turn-changes-summary"
            aria-expanded={turnChangesOpen}
            onClick={() => setTurnChangesOpen((open) => !open)}
          >
            <span className="workspace-turn-changes-summary-title">
              <strong>CHANGES</strong>
              <small>{turnChangeRows.length} files</small>
            </span>
            <span className="workspace-turn-changes-summary-meta">
              <small className="is-added">{completedTurnChanges.added} added</small>
              <small className="is-deleted">{completedTurnChanges.deleted} deleted</small>
              <small className="is-edited">{completedTurnChanges.edited} edited</small>
              {running && <small className="is-live">LIVE</small>}
            </span>
          </button>
          {turnChangesOpen && (
            <div className="workspace-turn-changes-list">
              {turnChangeRows.map((change) => (
                <div className="workspace-turn-change" key={change.path}>
                  <b className={`workspace-change-status is-${change.kind}`}>{change.status}</b>
                  <code title={change.path}>{change.path}</code>
                  <small>
                    +{change.additions} &nbsp;−{change.deletions}
                  </small>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
      {!compacting && <div className="workspace-review-divider" />}
      {waiting && (
        <>
          <span className="workspace-label">REVIEW QUEUE</span>
          {state === "approval" ? (
            <div className="workspace-approval-card">
              <div>
                <span>Approval required</span>
                <b>REVIEW</b>
              </div>
              <strong>Run the production design build</strong>
              <code>vite build</code>
              <div className="workspace-approval-actions">
                <Button variant="primary" size="sm">
                  Allow once
                </Button>
                <Button variant="danger" size="sm">
                  Deny
                </Button>
              </div>
              <Button size="sm">Allow for session</Button>
            </div>
          ) : (
            <div className="workspace-question-card">
              <div>
                <span>Clarification needed</span>
                <b>ANSWER</b>
              </div>
              <div className="workspace-question-prompt">
                <span>Scope</span>
                <strong>Which responsive behavior should the focused trace use?</strong>
              </div>
              <div className="workspace-question-options">
                {questionOptions.map((option) => (
                  <label
                    key={option.id}
                    className={`workspace-question-option ${questionOption === option.id ? "is-selected" : ""}`}
                  >
                    <Radio
                      className="workspace-question-radio"
                      name="trace-layout"
                      value={option.id}
                      checked={questionOption === option.id}
                      onChange={() => setQuestionOption(option.id)}
                    />
                    <span>
                      <strong>{option.label}</strong>
                      <small>{option.description}</small>
                    </span>
                  </label>
                ))}
              </div>
              <input
                className="workspace-question-custom"
                value={customAnswer}
                onChange={(event) => setCustomAnswer(event.target.value)}
                placeholder="Add a custom answer"
                aria-label="Custom answer"
              />
              <div className="workspace-question-actions">
                <Button
                  variant="primary"
                  size="sm"
                  disabled={!questionOption && !customAnswer.trim()}
                >
                  Submit answers
                </Button>
                <Button variant="danger" size="sm">
                  Skip
                </Button>
              </div>
            </div>
          )}
        </>
      )}
      {!compact && running && !noChanges && (
        <div className="workspace-checkpoint-card">
          <div>
            <span>CHECKPOINT</span>
            <small>3 files</small>
          </div>
          <strong>Workspace route implementation</strong>
          <code>
            navigation.js{"\n"}WorkspacePrimitives.jsx{"\n"}review.css
          </code>
          <Button size="sm">Undo</Button>
        </div>
      )}
    </aside>
  );
}

function formatContextTokens(value) {
  if (value >= 1000) return `${(value / 1000).toFixed(value >= 10000 ? 1 : 1)}k`;
  return `${value}`;
}

/**
 * The current request footprint, separate from cumulative session usage.
 * The production client should replace these specimen values with the provider's
 * current prompt token count and model context-window limit.
 */
export function ContextUsage({ used = 18420, limit = 32768, input = 15240, output = 3180, cached = 0 }) {
  const [expanded, setExpanded] = useState(false);
  const percentage = limit > 0 ? Math.min(100, Math.max(0, (used / limit) * 100)) : null;
  const tone = percentage === null ? "unknown" : percentage >= 90 ? "danger" : percentage >= 75 ? "warning" : "normal";
  const valueLabel = percentage === null
    ? "Unavailable"
    : `${formatContextTokens(used)} / ${formatContextTokens(limit)} tokens`;

  return (
    <section className={`workspace-context-usage is-${tone}`} aria-label="Context window usage">
      <button
        type="button"
        className="workspace-context-usage-toggle"
        aria-expanded={expanded}
        onClick={() => setExpanded((open) => !open)}
      >
        <span>
          <small>CONTEXT WINDOW</small>
          <strong>{valueLabel}</strong>
        </span>
        <b>{percentage === null ? "--" : `${Math.round(percentage)}%`}</b>
        <Icon name="chevron-right" size={12} />
      </button>
      <div
        className="workspace-context-usage-bar"
        role="progressbar"
        aria-label="Context window used"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={percentage === null ? undefined : Math.round(percentage)}
      >
        <span style={{ width: `${percentage ?? 0}%` }} />
      </div>
      {expanded && percentage !== null && (
        <dl className="workspace-context-usage-details">
          <div><dt>Input</dt><dd>{formatContextTokens(input)}</dd></div>
          <div><dt>Output</dt><dd>{formatContextTokens(output)}</dd></div>
          <div><dt>Cached</dt><dd>{formatContextTokens(cached)}</dd></div>
        </dl>
      )}
    </section>
  );
}

export function SourceControlPanel({
  onClose,
  standalone = false,
  clean = false,
  changeSet = changes,
}) {
  const [scope, setScope] = useState("all");
  const [selectedPath, setSelectedPath] = useState(
    () => changeSet[1]?.path ?? changeSet[0]?.path ?? "",
  );
  const [filter, setFilter] = useState("");
  const filtered = (clean ? [] : changeSet).filter((change) => {
    const matchesScope = scope === "all" || (scope === "staged" ? change.staged : change.unstaged);
    return matchesScope && change.path.toLowerCase().includes(filter.toLowerCase());
  });
  const selected = filtered.find((change) => change.path === selectedPath) ?? filtered[0] ?? null;
  return (
    <section className={`workspace-drawer workspace-git ${standalone ? "is-standalone" : ""}`}>
      <header>
        <Icon className="workspace-git-icon" name="git" size={16} />
        <strong>main</strong>
        <span className="workspace-git-divider" aria-hidden="true" />
        <div className="workspace-scope">
          {[
            ["all", "All"],
            ["staged", "Staged"],
            ["unstaged", "Unstaged"],
          ].map(([value, label]) => (
            <button
              key={value}
              type="button"
              className={scope === value ? "is-selected" : ""}
              onClick={() => setScope(value)}
            >
              {label}
            </button>
          ))}
        </div>
        <input
          value={filter}
          onChange={(event) => setFilter(event.target.value)}
          placeholder="Filter changed files"
          aria-label="Filter changed files"
        />
        <IconButton icon="refresh" label="Refresh Git status" onClick={() => setFilter("")} />
        <IconButton
          icon="copy"
          label="Copy patch"
          onClick={() =>
            navigator.clipboard?.writeText(
              diffLines
                .map(
                  (line) =>
                    `${line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : " "}${line.text}`,
                )
                .join("\n"),
            )
          }
        />
        <IconButton
          icon="close"
          label="Close source control"
          onClick={onClose}
          disabled={!onClose}
        />
      </header>
      <div className="workspace-git-body">
        <div className="workspace-change-list">
          <div className="workspace-drawer-label">
            {filtered.length} {filtered.length === 1 ? "file" : "files"}
          </div>
          {filtered.map((change) => (
            <button
              key={change.path}
              type="button"
              className={selected?.path === change.path ? "is-selected" : ""}
              onClick={() => setSelectedPath(change.path)}
            >
              <b className={`workspace-change-status is-${change.kind}`}>{change.status}</b>
              <span>
                <code>{change.path}</code>
                {change.oldPath && <small>from {change.oldPath}</small>}
              </span>
              <small>
                +{change.additions} -{change.deletions}
              </small>
            </button>
          ))}
          {!filtered.length && !clean && (
            <div className="workspace-change-empty">No changed files match this filter.</div>
          )}
        </div>
        <div className="workspace-diff">
          {selected ? (
            <>
              <div className="workspace-diff-heading">
                <code>{selected.path}</code>
                <span>
                  <b>+{selected.additions}</b> <i>-{selected.deletions}</i>
                </span>
              </div>
              <pre aria-label={`Diff for ${selected.path}`}>
                {diffLines.map((line, index) => (
                  <span
                    key={`${line.kind}-${index}`}
                    className={`workspace-diff-line diff-${line.kind}`}
                  >
                    <span>{line.oldLine}</span>
                    <span>{line.newLine}</span>
                    <i aria-hidden="true" />
                    <code>
                      {line.kind === "addition"
                        ? "+"
                        : line.kind === "deletion"
                          ? "-"
                          : line.kind === "hunk"
                            ? ""
                            : " "}
                      {line.text}
                    </code>
                  </span>
                ))}
              </pre>
            </>
          ) : (
            !clean && (
              <div className="workspace-diff-empty">No changed files match this filter.</div>
            )
          )}
        </div>
      </div>
    </section>
  );
}

export function ProviderTracePanel({ onClose, standalone = false, state = "expanded" }) {
  const [selected, setSelected] = useState(state === "context-compaction" ? 1 : 0);
  const [expandedTurn, setExpandedTurn] = useState(state !== "turn-collapsed");
  const traces = [
    {
      title: "Response · gpt-5.6-sol",
      time: "14:32:18",
      status: "Completed",
      tokens: "18,420 → 1,284",
      contents: [
        {
          kind: "user",
          label: "USER",
          title: "User message",
          summary: "Add the Workspace surface to the design system.",
          time: "14:32:18",
        },
        {
          kind: "assistant",
          label: "ASSISTANT",
          title: "Assistant message",
          summary: "I’ll inspect the current Avalonia composition first.",
          time: "14:32:19",
        },
        {
          kind: "tool",
          label: "TOOL CALL",
          title: "Read ProjectWorkspace.axaml",
          summary: "Read 218 lines from the project workspace view.",
          time: "14:32:24",
        },
      ],
    },
    {
      title: "Context compaction",
      kind: "compaction",
      time: "14:32:07",
      status: "Completed",
      tokens: "18,420 → 12,160",
      contents: [
        {
          kind: "system",
          label: "CONTEXT",
          title: "Context summary",
          summary:
            "Reduced the earlier conversation to 12,160 retained tokens and dropped 6 messages.",
          time: "14:32:07",
        },
      ],
    },
    {
      title: "Tool continuation",
      time: "14:31:46",
      status: "Completed",
      tokens: "12,918 → 826",
      contents: [
        {
          kind: "user",
          label: "USER",
          title: "Tool result",
          summary: "The workspace view and its review drawer are available.",
          time: "14:31:46",
        },
        {
          kind: "assistant",
          label: "ASSISTANT",
          title: "Assistant message",
          summary: "I’ll split each workspace area into a focused route.",
          time: "14:31:48",
        },
        {
          kind: "tool",
          label: "TOOL CALL",
          title: "Update workspace routes",
          summary: "Updated 8 route modules and navigation entries.",
          time: "14:31:55",
        },
      ],
    },
    {
      title: "Initial request",
      time: "14:30:09",
      status: "Completed",
      tokens: "8,204 → 612",
      contents: [
        {
          kind: "user",
          label: "USER",
          title: "User message",
          summary: "Keep the major areas independently reachable from the sidebar.",
          time: "14:30:09",
        },
        {
          kind: "assistant",
          label: "ASSISTANT",
          title: "Assistant message",
          summary: "I’ll preserve the desktop composition and add stable child routes.",
          time: "14:30:12",
        },
        {
          kind: "tool",
          label: "TOOL CALL",
          title: "List design-system files",
          summary: "Found the desktop workspace modules and shared primitives.",
          time: "14:30:18",
        },
      ],
    },
  ];
  const noTurns = state === "no-turns";
  const activeTrace = traces[selected] ?? traces[0];
  return (
    <section className={`workspace-drawer workspace-trace ${standalone ? "is-standalone" : ""}`}>
      <header>
        <Icon name="activity" size={16} />
        <strong>Provider trace</strong>
        <span>{noTurns ? "0 turns" : `1 turn · ${traces.length} calls`}</span>
        <div />
        <IconButton icon="refresh" label="Refresh provider trace" disabled />
        <IconButton
          icon="copy"
          label="Copy trace"
          onClick={() => navigator.clipboard?.writeText("Provider trace preview")}
        />
        <IconButton
          icon="close"
          label="Close provider trace"
          onClick={onClose}
          disabled={!onClose}
        />
      </header>
      {noTurns ? (
        <div className="workspace-trace-empty">
          <Icon name="activity" size={24} />
          <strong>No turns yet</strong>
          <span>Provider requests will appear here after the agent starts a turn.</span>
        </div>
      ) : (
        <div className="workspace-trace-body">
          <div className="workspace-trace-list">
            <div className="workspace-drawer-label">CURRENT SESSION</div>
            <button
              type="button"
              className={`workspace-trace-turn ${expandedTurn ? "is-expanded" : ""}`}
              onClick={() => setExpandedTurn(!expandedTurn)}
            >
              <Icon name="chevron-right" className={expandedTurn ? "is-open" : ""} size={11} />
              <span>
                <strong>Turn 0198e82c</strong>
                <small>Completed · 1.84 s</small>
              </span>
              <b>{traces.length} calls</b>
            </button>
            {expandedTurn && (
              <div className="workspace-trace-children">
                {traces.map((trace, index) => (
                  <div key={trace.title}>
                    <button
                      type="button"
                      className={`workspace-trace-call ${trace.kind ? `is-${trace.kind}` : ""} ${selected === index ? "is-selected" : ""}`}
                      onClick={() => setSelected(index)}
                    >
                      <span>
                        <strong>{trace.title}</strong>
                        <small>{trace.time}</small>
                      </span>
                      <span>
                        <b>{trace.status}</b>
                        <small>{trace.tokens}</small>
                      </span>
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
          <div className="workspace-trace-detail">
            <div className="workspace-trace-title">
              <code>{activeTrace.title}</code>
              <span>
                {activeTrace.kind === "compaction" ? "context build" : "1.84 s  gpt-5.6-sol"}
              </span>
            </div>
            <div className="workspace-trace-metrics">
              {[
                ["INPUT", activeTrace.kind === "compaction" ? "18,420" : "18,420"],
                ["OUTPUT", activeTrace.kind === "compaction" ? "12,160" : "1,284"],
                ["CACHE READ", activeTrace.kind === "compaction" ? "0" : "12,160"],
                ["CACHE WRITE", "0"],
                ["CACHE HIT", activeTrace.kind === "compaction" ? "—" : "66%"],
                ["DURATION", activeTrace.kind === "compaction" ? "0.42 s" : "1.84 s"],
              ].map(([label, value]) => (
                <div key={label}>
                  <span>{label}</span>
                  <strong>{value}</strong>
                </div>
              ))}
            </div>
            <div className="workspace-trace-content">
              <div>
                <code>TURN 0198e82c · COMPLETED</code>
                <span>
                  {activeTrace.kind === "compaction"
                    ? "event  context.compacted"
                    : "exchange  exch_01JY7F6P8S"}
                </span>
              </div>
              <h4>{activeTrace.kind === "compaction" ? "Compaction result" : "Model response"}</h4>
              <pre>
                {activeTrace.kind === "compaction"
                  ? `{"status":"completed","event":"context.compacted","dropped_messages":6,"retained_tokens":12160}`
                  : `{"status":"completed","role":"assistant"}`}
              </pre>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

const toolActivityTurns = [
  {
    id: "0198e82c",
    title: "Turn 0198e82c",
    preview: "Add the Workspace surface to the design system, but keep each major area…",
    status: "Running",
    tools: [
      {
        title: "Read ProjectWorkspace.axaml",
        state: "Completed",
        tone: "success",
        request: "apps/desktop-avalonia/Views/ProjectWorkspace/ProjectWorkspace.axaml",
        result: "218 lines read",
      },
      {
        title: "Run mvn compile for the workspace shell specimen",
        state: "Running",
        tone: "running",
        request: "mvn -pl design-system compile",
        result: "Command still running",
        live: [
          "[INFO] Scanning for projects...",
          "[INFO] Building design-system 0.0.0-review",
          "> vite build",
          "transforming modules...",
          "rendering chunks...",
        ],
      },
      {
        title: "Update workspace routes and modules",
        state: "Queued",
        tone: "queued",
        request: "design-system/src/app/navigation.js",
        result: "Waiting for the running compile command to finish",
      },
      {
        title: "Read missing workspace manifest",
        state: "Failed",
        tone: "danger",
        request: "design-system/src/projects/desktop/workspace/manifest.json",
        result: "The file could not be found",
        error: "file_not_found",
      },
    ],
  },
];

const completedToolActivityTurns = [
  {
    id: "0198e7f1",
    title: "Turn 0198e7f1",
    preview: "Review the current Avalonia workspace layout and summarize the supporting bays…",
    status: "Completed",
    tools: [
      {
        title: "List design-system files",
        state: "Completed",
        tone: "success",
        request: "design-system/src/projects/desktop/workspace",
        result: "42 files found",
      },
      {
        title: "Search provider trace bindings",
        state: "Completed",
        tone: "success",
        request: "rg ProviderTrace apps/desktop-avalonia",
        result: "14 provider trace bindings found",
      },
      {
        title: "Read ProviderTraceViewer.axaml",
        state: "Completed",
        tone: "success",
        request: "apps/desktop-avalonia/Views/ProjectWorkspace/Review/ProviderTraceViewer.axaml",
        result: "302 lines read",
      },
    ],
  },
];

export function ToolActivityPanel({ onClose, standalone = false, state = "running" }) {
  const empty = state === "empty";
  const turns = empty ? [] : state === "completed" ? completedToolActivityTurns : toolActivityTurns;
  const initialTurn = 0;
  const initialTool = state === "running" ? 1 : 0;
  const [selectedTurn, setSelectedTurn] = useState(initialTurn);
  const [selectedTool, setSelectedTool] = useState(initialTool);
  const [expandedTurns, setExpandedTurns] = useState(() => new Set([initialTurn]));
  const [visibleOutputLines, setVisibleOutputLines] = useState(2);
  const turn = turns[selectedTurn] ?? turns[0];
  const tool = turn?.tools[selectedTool] ?? turn?.tools[0];
  useEffect(() => {
    setVisibleOutputLines(2);
    if (!tool?.live) return undefined;
    const timer = window.setInterval(
      () =>
        setVisibleOutputLines((current) => {
          if (current >= tool.live.length) {
            window.clearInterval(timer);
            return current;
          }
          return current + 1;
        }),
      680,
    );
    return () => window.clearInterval(timer);
  }, [selectedTurn, selectedTool, tool?.live]);
  return (
    <section
      className={`workspace-drawer workspace-tool-activity ${standalone ? "is-standalone" : ""}`}
    >
      <header>
        <Icon name="tool" size={16} />
        <strong>Tool activity</strong>
        <span>
          {empty
            ? "0 turns"
            : `${turns.length} turn · ${turns.reduce((sum, item) => sum + item.tools.length, 0)} calls`}
        </span>
        <div />
        <div className="workspace-tool-activity-actions">
          <IconButton
            icon="copy"
            label="Copy tool activity"
            onClick={() => navigator.clipboard?.writeText("Tool activity preview")}
          />
          <IconButton
            icon="close"
            label="Close tool activity"
            onClick={onClose}
            disabled={!onClose}
          />
        </div>
      </header>
      {empty ? (
        <div className="workspace-tool-empty">
          <Icon name="tool" size={24} />
          <strong>No turns yet</strong>
          <span>Tool calls will appear here after the agent starts its first turn.</span>
        </div>
      ) : (
        <div className="workspace-tool-body">
          <div className="workspace-tool-tree">
            <div className="workspace-drawer-label">CURRENT SESSION</div>
            {turns.map((item, turnIndex) => (
              <div key={item.id}>
                <button
                  type="button"
                  className={`workspace-tool-turn ${selectedTurn === turnIndex ? "is-selected" : ""}`}
                  aria-expanded={expandedTurns.has(turnIndex)}
                  onClick={() => {
                    setSelectedTurn(turnIndex);
                    setSelectedTool(0);
                    setExpandedTurns((current) => {
                      const next = new Set(current);
                      if (next.has(turnIndex)) next.delete(turnIndex);
                      else next.add(turnIndex);
                      return next;
                    });
                  }}
                >
                  <Icon
                    name="chevron-right"
                    className={expandedTurns.has(turnIndex) ? "is-open" : ""}
                    size={11}
                  />
                  <span>
                    <strong>{item.title}</strong>
                    <small>{item.preview}</small>
                  </span>
                  <b>{item.tools.length}</b>
                </button>
                {expandedTurns.has(turnIndex) && (
                  <div className="workspace-tool-children">
                    {item.tools.map((entry, toolIndex) => (
                      <button
                        type="button"
                        key={entry.title}
                        className={`workspace-tool-tree-row is-${entry.tone} ${selectedTurn === turnIndex && selectedTool === toolIndex ? "is-selected" : ""}`}
                        onClick={() => {
                          setSelectedTurn(turnIndex);
                          setSelectedTool(toolIndex);
                        }}
                      >
                        <Icon name={entry.tone === "running" ? "tool" : "activity"} size={13} />
                        <span>
                          <strong>{entry.title}</strong>
                        </span>
                        {entry.tone !== "success" && entry.tone !== "danger" && (
                          <i className={`is-${entry.tone}`} aria-label={entry.state} />
                        )}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
          <div className="workspace-tool-detail">
            <div className="workspace-tool-detail-heading">
              <div>
                <code>
                  {turn.title} · {tool.state.toUpperCase()}
                </code>
                <h3>{tool.title}</h3>
              </div>
              <span className={`workspace-tool-state is-${tool.tone}`}>{tool.state}</span>
            </div>
            <div className="workspace-tool-meta">
              <span>TOOL CALL</span>
              <code>
                {turn.id} · {selectedTool + 1} of {turn.tools.length}
              </code>
              <span>STATUS</span>
              <code>{tool.state}</code>
            </div>
            <div className="workspace-tool-detail-section">
              <span>Request</span>
              <code>{tool.request}</code>
            </div>
            {tool.live && (
              <div className="workspace-tool-detail-section">
                <div className="workspace-tool-live-heading">
                  <span>Live output</span>
                  <small>Following tail · {visibleOutputLines} lines</small>
                </div>
                <pre className="workspace-tool-live-output" aria-live="polite">
                  <code>{tool.live.slice(0, visibleOutputLines).join("\n")}</code>
                </pre>
              </div>
            )}
            <div className="workspace-tool-detail-section">
              <span>{tool.live ? "Latest status" : "Result"}</span>
              <code>{tool.result}</code>
            </div>
            {tool.error && (
              <div className="workspace-tool-detail-section is-error">
                <span>Error</span>
                <code>{tool.error}</code>
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  );
}

export function ProjectSwitcher({ projects = workspaceRecentProjects }) {
  const [open, setOpen] = useState(true);
  const switcherRef = useRef(null);
  const currentProject = projects.find((project) => project.current) ??
    projects[0] ?? { name: "No project", path: "" };

  const closeMenu = () => setOpen(false);
  useEffect(() => {
    if (!open) return undefined;
    const handlePointerDown = (event) => {
      if (!switcherRef.current?.contains(event.target)) closeMenu();
    };
    const handleKeyDown = (event) => {
      if (event.key === "Escape") closeMenu();
    };
    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  return (
    <div className="project-switcher" ref={switcherRef}>
      <button
        type="button"
        className={`project-switcher-trigger ${open ? "is-open" : ""}`}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={`Current project: ${currentProject.name}. Switch project`}
        onClick={() => setOpen((value) => !value)}
      >
        <Icon name="folder" size={14} />
        <strong>{currentProject.name}</strong>
        <Icon name="chevron-right" size={12} className="project-switcher-chevron" />
      </button>
      {open && (
        <div className="project-switcher-menu" role="menu" aria-label="Project actions">
          <button
            type="button"
            className="project-switcher-action"
            role="menuitem"
            onClick={closeMenu}
          >
            <Icon name="folder" size={14} />
            <span>
              <strong>Open project</strong>
              <small>Choose a local folder…</small>
            </span>
            <Icon name="arrow" size={13} />
          </button>
          <div className="project-switcher-divider" role="separator" />
          <div className="project-switcher-label">RECENT PROJECTS</div>
          {projects.length ? (
            <div className="project-switcher-list">
              {projects.map((project) => (
                <button
                  type="button"
                  className="project-switcher-project"
                  role="menuitem"
                  key={project.id}
                  onClick={closeMenu}
                >
                  <Icon name="project" size={14} />
                  <span>
                    <strong>{project.name}</strong>
                    <small>{project.path}</small>
                  </span>
                </button>
              ))}
            </div>
          ) : (
            <div className="project-switcher-empty">No recent projects</div>
          )}
        </div>
      )}
    </div>
  );
}

const initialRecentContent = [
  {
    id: "session:workspace-information-architecture",
    kind: "session",
    title: "Workspace information architecture",
    detail: "2 min ago",
    session: sessions[0],
  },
  {
    id: "child-session:child-swe-contracts",
    kind: "child-session",
    title: "Map session contract changes",
    detail: "Software Engineering Agent · Completed",
    child: childSessions[1],
  },
  {
    id: "file:workspace-file",
    kind: "file",
    title: "ProjectWorkspace.axaml",
    detail: "apps/desktop-avalonia/Views/Projects/ProjectWorkspace.axaml",
    file: { id: "workspace-file", name: "ProjectWorkspace.axaml" },
  },
  {
    id: "session:provider-migration-review",
    kind: "session",
    title: "Provider migration review",
    detail: "Yesterday",
    session: sessions[1],
  },
  {
    id: "file:shared-readme",
    kind: "file",
    title: "README.md",
    detail: "../shared-ui/README.md",
    file: { id: "shared-readme", name: "README.md" },
  },
  {
    id: "session:desktop-navigation-polish",
    kind: "session",
    title: "Desktop navigation polish",
    detail: "Aug 26",
    session: sessions[2],
  },
];

const contentItemForSession = (session) => ({
  id: `session:${session.id ?? session.title}`,
  kind: "session",
  title: session.title,
  detail: session.time ?? "Recently viewed",
  session,
});

const contentItemForChildSession = (child) => ({
  id: `child-session:${child.id}`,
  kind: "child-session",
  title: child.title,
  detail: `${child.agentDisplayName} · ${childSessionStateLabels[child.state]}`,
  child,
});

const contentItemForFile = (file) => {
  const document = editorDocuments[file.id] ?? {
    name: file.name,
    path: file.path,
  };
  return {
    id: `file:${file.id ?? file.path}`,
    kind: "file",
    title: document.name ?? file.name,
    detail: document.path ?? file.path,
    file,
  };
};

export function ContentSwitcher({
  currentItem,
  items = initialRecentContent,
  onSelect,
  initialOpen = false,
}) {
  const [open, setOpen] = useState(initialOpen);
  const switcherRef = useRef(null);
  const recentItems = items.filter((item) => item.id !== currentItem.id).slice(0, 20);
  const iconForKind = (kind) =>
    kind === "file" ? "file-text" : kind === "child-session" ? "agent" : "message";
  const labelForKind = (kind) =>
    kind === "file" ? "FILE" : kind === "child-session" ? "CHILD SESSION" : "SESSION";

  useEffect(() => {
    if (!open) return undefined;
    const handlePointerDown = (event) => {
      if (!switcherRef.current?.contains(event.target)) setOpen(false);
    };
    const handleKeyDown = (event) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  return (
    <div className="content-switcher" ref={switcherRef}>
      <button
        type="button"
        className={`content-switcher-trigger ${open ? "is-open" : ""}`}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={`Current ${currentItem.kind}: ${currentItem.title}. Show recently viewed content`}
        onClick={() => setOpen((value) => !value)}
      >
        <Icon name={iconForKind(currentItem.kind)} size={13} />
        <strong>{currentItem.title}</strong>
        <Icon name="chevron-right" size={11} className="content-switcher-chevron" />
      </button>
      {open && (
        <div
          className="content-switcher-menu"
          role="menu"
          aria-label="Recently viewed files and sessions"
        >
          <div className="content-switcher-heading">
            <span>RECENTLY VIEWED</span>
            <small>{recentItems.length} / 20</small>
          </div>
          {recentItems.length ? (
            <div className="content-switcher-list">
              {recentItems.map((item) => (
                <button
                  type="button"
                  className={`content-switcher-item is-${item.kind}`}
                  role="menuitem"
                  key={item.id}
                  onClick={() => {
                    onSelect?.(item);
                    setOpen(false);
                  }}
                >
                  <Icon name={iconForKind(item.kind)} size={14} />
                  <span>
                    <strong>{item.title}</strong>
                    <small title={`${labelForKind(item.kind)} · ${item.detail}`}>
                      <b>{labelForKind(item.kind)}</b>
                      <span> · {item.detail}</span>
                    </small>
                  </span>
                </button>
              ))}
            </div>
          ) : (
            <div className="content-switcher-empty">
              <Icon name="activity" size={18} />
              <span>No recently viewed files or sessions</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export function WorkspaceWindow({ projectSwitcherProjects = workspaceRecentProjects }) {
  const [navigation, setNavigation] = useState("sessions");
  const [activeFile, setActiveFile] = useState(null);
  const [activeChild, setActiveChild] = useState(null);
  const [activeSessionId, setActiveSessionId] = useState(sessions[0].id);
  const [currentContent, setCurrentContent] = useState(() => contentItemForSession(sessions[0]));
  const [recentContent, setRecentContent] = useState(initialRecentContent);
  const [rightRegion, setRightRegion] = useState("review");
  const [drawer, setDrawer] = useState("tools");
  const [archiveRequest, setArchiveRequest] = useState(null);
  const activePrimarySession =
    sessions.find((session) => session.id === activeSessionId) ?? sessions[0];
  const activeSessionArchived = currentContent.kind === "session" && currentContent.session?.archived;
  const visibleChildSessions = childSessions.filter(
    (child) => child.parentSessionId === activePrimarySession.id,
  );
  const toggleDrawer = (next) => setDrawer((current) => (current === next ? null : next));
  const selectContent = (item) => {
    setCurrentContent(item);
    setActiveFile(item.kind === "file" ? item.file : null);
    setActiveChild(item.kind === "child-session" ? item.child : null);
    if (item.kind === "session") setActiveSessionId(item.session?.id ?? item.session?.title);
    if (item.kind === "child-session") {
      setActiveSessionId(item.child.parentSessionId);
      setRightRegion("children");
    }
    setRecentContent((current) =>
      [item, ...current.filter((entry) => entry.id !== item.id)].slice(0, 20),
    );
  };
  return (
    <div className="workspace-window">
      <div className="workspace-titlebar">
        <TrafficLights />
        <ProjectSwitcher projects={projectSwitcherProjects} />
        <ContentSwitcher
          currentItem={currentContent}
          items={recentContent}
          onSelect={selectContent}
        />
        <div className="workspace-titlebar-actions">
          <IconButton
            icon="settings"
            label="Open settings"
            onClick={() => {
              window.location.hash = "/projects/desktop/settings";
            }}
          />
        </div>
      </div>
      <div className="workspace-window-body">
        <aside className="workspace-gutter">
          <div>
            <IconButton
              icon="panel-left"
              label="Show sessions"
              active={navigation === "sessions"}
              onClick={() => setNavigation(navigation === "sessions" ? null : "sessions")}
            />
            <IconButton
              icon="files"
              label="Show explorer"
              active={navigation === "explorer"}
              onClick={() => setNavigation(navigation === "explorer" ? null : "explorer")}
            />
          </div>
          <div>
            <IconButton
              icon="git"
              label="Show source control"
              active={drawer === "git"}
              onClick={() => toggleDrawer("git")}
            />
            <IconButton
              icon="activity"
              label="Show provider trace"
              active={drawer === "trace"}
              onClick={() => toggleDrawer("trace")}
            />
            <IconButton
              icon="tool"
              label="Show tool activity"
              active={drawer === "tools"}
              onClick={() => toggleDrawer("tools")}
            />
          </div>
        </aside>
        <div className="workspace-main-stack">
          <div className="workspace-main-row">
            {navigation === "sessions" && (
              <SessionPanel
                compact
                selectedSessionId={activeSessionId}
                onArchiveRequest={setArchiveRequest}
                onSelectSession={(session) => selectContent(contentItemForSession(session))}
              />
            )}
            {navigation === "explorer" && (
              <ExplorerPanel
                compact
                selectedFileId={activeFile?.id ?? ""}
                onFileSelect={(file) => selectContent(contentItemForFile(file))}
              />
            )}
            {activeFile ? (
              <EditorPanel compact file={activeFile} />
            ) : activeChild ? (
              <ChildSessionDetail
                child={activeChild}
                onBack={() => selectContent(contentItemForSession(activePrimarySession))}
              />
            ) : (
              <ConversationPanel
                compact
                state={activeSessionArchived ? "archived" : "content-updating"}
                onViewChanges={() => setDrawer("git")}
                onOpenToolActivity={() => setDrawer("tools")}
              />
            )}
            {rightRegion === "review" && <ReviewPanel compact />}
            {rightRegion === "children" && (
              <ChildSessionsPanel
                compact
                sessions={visibleChildSessions}
                selectedChildId={activeChild?.id}
                parentTitle={activePrimarySession.title}
                onSelectChild={(child) => selectContent(contentItemForChildSession(child))}
              />
            )}
          </div>
          {drawer === "git" && (
            <SourceControlPanel
              onClose={() => setDrawer(null)}
              changeSet={completedTurnChangeSet}
            />
          )}
          {drawer === "trace" && <ProviderTracePanel onClose={() => setDrawer(null)} />}
          {drawer === "tools" && <ToolActivityPanel onClose={() => setDrawer(null)} />}
        </div>
        <aside className="workspace-gutter workspace-gutter-right">
          <IconButton
            icon="panel-right"
            label="Show review"
            active={rightRegion === "review"}
            onClick={() => setRightRegion(rightRegion === "review" ? null : "review")}
          />
          <IconButton
            icon="agent"
            label="Show child sessions"
            active={rightRegion === "children"}
            onClick={() => setRightRegion(rightRegion === "children" ? null : "children")}
          />
        </aside>
      </div>
      <footer className="workspace-statusbar">
        <div>
          <code>codex/workspace-design</code>
          <b>3 changes</b>
          <span>+308</span>
          <i>−8</i>
        </div>
        <div>
          <McpLoadingStatus settled={3} total={5} connected={2} failed={1} />
          <span className="workspace-running-session-status is-running" title="2 running">
            <i aria-hidden="true" />
            <code>2 running</code>
          </span>
          <code>gpt-5.6-sol</code>
          <span>19.7k tokens</span>
          <span>3 calls · 4.2s</span>
        </div>
      </footer>
      <DialogWindowConfirmation
        open={Boolean(archiveRequest)}
        sessionTitle={archiveRequest?.session?.title}
        onCancel={() => setArchiveRequest(null)}
        onConfirm={() => {
          archiveRequest?.confirm();
          setArchiveRequest(null);
        }}
      />
    </div>
  );
}

/**
 * Compact IntelliJ-style MCP startup feedback for the workspace footer.
 * `settled` includes both connected and terminally failed servers so the
 * indicator can always reach completion even when one server is unavailable.
 */
export function McpLoadingStatus({ settled, total, connected = 0, failed = 0 }) {
  if (total <= 0 || settled >= total) return null;
  const progress = Math.min(100, Math.max(0, (settled / total) * 100));
  const remaining = Math.max(0, total - settled);
  const label = `MCP ${settled}/${total}`;
  const detail = `${connected} connected${failed ? ` · ${failed} failed` : ""} · ${remaining} starting`;
  return (
    <span
      className={`workspace-mcp-status${failed ? " has-failures" : ""}`}
      title={detail}
      role="status"
      aria-label={`${label} · ${detail}`}
    >
      <Icon name="server" size={11} />
      <span className="workspace-mcp-status-label">{label}</span>
      <span className="workspace-mcp-progress" aria-hidden="true">
        <span style={{ width: `${progress}%` }} />
      </span>
    </span>
  );
}

export function FocusedWorkspaceFrame({ title, children, className = "" }) {
  return (
    <div className={`workspace-focused-frame ${className}`}>
      <div className="workspace-focused-titlebar">
        <TrafficLights />
        <strong>suncode</strong>
        <span>{title}</span>
        <Icon name="settings" size={14} />
      </div>
      {children}
    </div>
  );
}
