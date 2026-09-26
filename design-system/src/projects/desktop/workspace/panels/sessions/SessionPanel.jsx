import { useState } from "react";

import { ConfirmationDialog } from "../../../../../components/universal/modal/index.js";
import { Icon } from "../../../../../shared/Icon.jsx";

import {
  archivedSessions,
  primarySessions as sessions,
  sessionStatusLabels,
} from "../../data/sessions.js";
import { IconButton } from "../../shared/IconButton.jsx";

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
