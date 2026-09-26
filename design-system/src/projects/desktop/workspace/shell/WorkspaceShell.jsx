import { useEffect, useRef, useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";
import { TrafficLights } from "../../../../shared/TrafficLights.jsx";
import { DialogWindowConfirmation } from "../../dialog-window/index.jsx";
import { IconButton } from "../shared/IconButton.jsx";
import { McpLoadingStatus } from "./McpLoadingStatus.jsx";
import { ProjectSwitcher } from "./ProjectSwitcher.jsx";
import { ConversationPanel } from "../panels/conversation/ConversationPanel.jsx";
import { EditorPanel } from "../panels/editor/EditorPanel.jsx";
import { ExplorerPanel } from "../panels/explorer/ExplorerPanel.jsx";
import {
  ChildSessionDetail,
  ChildSessionsPanel,
} from "../panels/child-sessions/ChildSessionsPanel.jsx";
import { ReviewPanel } from "../panels/review/ReviewPanel.jsx";
import { SessionPanel } from "../panels/sessions/SessionPanel.jsx";
import { SourceControlPanel } from "../panels/source-control/SourceControlPanel.jsx";
import { ProviderTracePanel } from "../panels/provider-trace/ProviderTracePanel.jsx";
import { ToolActivityPanel } from "../panels/tool-activity/ToolActivityPanel.jsx";
import { childSessionStateLabels, childSessions } from "../data/sessions.js";
import { completedTurnChangeSet } from "../data/review.js";
import { editorDocuments } from "../data/editor.js";
import { primarySessions as sessions } from "../data/sessions.js";
import { workspaceRecentProjects } from "../data/projects.js";

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
  const activeSessionArchived =
    currentContent.kind === "session" && currentContent.session?.archived;
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
