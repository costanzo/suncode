import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { MarkdownMessage } from "./MarkdownMessage";
import { useWebStore } from "./store";
import type { Project, SessionSummary } from "./types";

const icon = (name: string) => {
  const paths: Record<string, string> = {
    folder: "M3 6h7l2 2h9v11H3z",
    chevron: "m9 5 7 7-7 7",
    monitor: "M3 4h18v13H3zM8 21h8M12 17v4",
    settings:
      "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8m8.5 4a8.5 8.5 0 0 0-.1-1.2l2-1.2-2-3.4-2.2 1a8 8 0 0 0-2.1-1.2L15.8 3h-4l-.3 2.4a8 8 0 0 0-2.1 1.2l-2.2-1-2 3.4 2 1.2A8.5 8.5 0 0 0 7 12c0 .4 0 .8.1 1.2l-2 1.2 2 3.4 2.2-1a8 8 0 0 0 2.1 1.2l.3 2.4h4l.3-2.4a8 8 0 0 0 2.1-1.2l2.2 1 2-3.4-2-1.2c.1-.4.2-.8.2-1.2",
    plus: "M12 5v14M5 12h14",
    send: "M5 12h14M13 6l6 6-6 6",
    lock: "M5 10h14v11H5zM8 10V7a4 4 0 0 1 8 0v3",
    panel: "M3 4h18v16H3zM15 4v16",
  };
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d={paths[name] ?? paths.folder} />
    </svg>
  );
};

function Connection({ state }: { state: string }) {
  return (
    <span className={`connection connection-${state}`}>
      <i />
      {state === "connected"
        ? "Connected"
        : state === "reconnecting"
          ? "Reconnecting"
          : state === "offline"
            ? "Offline"
            : "Degraded"}
    </span>
  );
}

function ProjectRow({
  project,
  sessions,
  expanded,
  selectedId,
  onToggle,
  onSelect,
}: {
  project: Project;
  sessions: SessionSummary[];
  expanded: boolean;
  selectedId: string | null;
  onToggle: () => void;
  onSelect: (id: string) => void;
}) {
  return (
    <div className={`project-group ${expanded ? "expanded" : ""}`}>
      <button className="project-row" type="button" aria-expanded={expanded} onClick={onToggle}>
        {icon("folder")}
        <span>
          <strong>{project.displayName}</strong>
          <small>{project.activeSessionCount ?? 0} active sessions</small>
        </span>
        {icon("chevron")}
      </button>
      {expanded && (
        <div className="project-sessions">
          {sessions.length ? (
            sessions.map((session) => (
              <button
                className={`session-row ${selectedId === session.id ? "selected" : ""}`}
                key={session.id}
                type="button"
                onClick={() => onSelect(session.id)}
              >
                <i className={`state-dot state-${session.state}`} />
                <span>
                  <strong>{session.title}</strong>
                  <small>
                    {session.state.replaceAll("_", " ")} · {timeAgo(session.updatedAt)}
                  </small>
                </span>
              </button>
            ))
          ) : (
            <span className="empty-sessions">No active sessions</span>
          )}
        </div>
      )}
    </div>
  );
}

function Sidebar({ onSettings }: { onSettings: () => void }) {
  const { host, projects, sessions, selectedSessionId, selectSession, error } = useWebStore();
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  useEffect(() => {
    if (projects[0] && Object.keys(expanded).length === 0) setExpanded({ [projects[0].id]: true });
  }, [projects, expanded]);
  const grouped = useMemo(
    () =>
      sessions.reduce<Record<string, SessionSummary[]>>((result, session) => {
        (result[session.project.id] ??= []).push(session);
        return result;
      }, {}),
    [sessions],
  );
  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="brand-mark">{icon("monitor")}</div>
        <strong>SunCode</strong>
        <span>WEB</span>
      </div>
      <button className="host-picker" type="button" onClick={onSettings}>
        <div className="host-icon">{icon("monitor")}</div>
        <span>
          <strong>{host?.displayName ?? "No Desktop paired"}</strong>
          <small>{host ? "Open Settings" : "Pair in Settings"}</small>
        </span>
        {icon("chevron")}
      </button>
      <div className="sidebar-heading">
        <span>PROJECTS</span>
        <span>{projects.length}</span>
      </div>
      <div className="project-list">
        {projects.length ? (
          projects.map((project) => (
            <ProjectRow
              key={project.id}
              project={project}
              sessions={grouped[project.id] ?? []}
              expanded={!!expanded[project.id]}
              selectedId={selectedSessionId}
              onToggle={() =>
                setExpanded((state) => ({ ...state, [project.id]: !state[project.id] }))
              }
              onSelect={(id) => void selectSession(id)}
            />
          ))
        ) : (
          <div className="sidebar-empty">
            {host ? "No projects available" : "Pair a Desktop in Settings to load projects."}
          </div>
        )}
      </div>
      <div className="sidebar-footer">
        {error && <small className="sidebar-error">{error}</small>}
        <Connection state={host?.connectionState ?? "offline"} />
        <button className="settings-link" type="button" onClick={onSettings}>
          {icon("settings")} Settings
        </button>
        <small>remote.v1 · browser E2E</small>
      </div>
    </aside>
  );
}

function Composer() {
  const sendMessage = useWebStore((state) => state.sendMessage);
  const [value, setValue] = useState("");
  const submit = async () => {
    if (!value.trim()) return;
    const submittedValue = value;
    setValue("");
    try {
      await sendMessage(submittedValue);
    } catch {
      setValue((current) => (current.trim() ? current : submittedValue));
    }
  };
  return (
    <div className="composer">
      <button className="composer-icon" type="button" aria-label="Attach image">
        {icon("plus")}
      </button>
      <textarea
        value={value}
        onChange={(event) => setValue(event.target.value)}
        onKeyDown={(event) => {
          const composing = event.nativeEvent.isComposing || event.nativeEvent.keyCode === 229;
          if (event.key === "Enter" && !event.shiftKey && !composing) {
            event.preventDefault();
            void submit();
          }
        }}
        placeholder="Message SunCode…"
        rows={1}
      />
      <button
        className="send-button"
        type="button"
        aria-label="Send message"
        onClick={() => void submit()}
      >
        {icon("send")}
      </button>
    </div>
  );
}

function Conversation() {
  const {
    snapshot,
    streamState,
    reviewOpen,
    toggleReview,
    resolveApproval,
    cancelTurn,
    retryTurn,
    loading,
    error,
  } = useWebStore();
  const cleanup = useWebStore((state) => state.connectStream);
  const messagesRef = useRef<HTMLDivElement>(null);
  const followLatestRef = useRef(true);
  useEffect(() => cleanup(), [cleanup, snapshot?.id]);
  useLayoutEffect(() => {
    const messages = messagesRef.current;
    if (messages && followLatestRef.current) messages.scrollTop = messages.scrollHeight;
  }, [snapshot?.id, snapshot?.messages.length, snapshot?.messages.at(-1)?.text, snapshot?.state]);
  if (!snapshot)
    return (
      <main className="conversation empty-conversation">
        {loading ? (
          <>
            <h2>Loading session</h2>
            <p>Fetching the live session from the paired Desktop.</p>
          </>
        ) : (
          <>
            <h2>{error ? "Unable to load session" : "Select a Session"}</h2>
            <p>{error ?? "Choose a Project and Session from the left."}</p>
          </>
        )}
      </main>
    );
  return (
    <main className="conversation">
      <header className="conversation-header">
        <div>
          <span className="eyebrow">PRIMARY SESSION</span>
          <h1>{snapshot.title}</h1>
          <small>
            {snapshot.project?.displayName ?? "Project"} · updated {timeAgo(snapshot.updatedAt)}
          </small>
        </div>
        <div className="header-actions">
          <Connection
            state={
              streamState === "live"
                ? "connected"
                : streamState === "connecting"
                  ? "reconnecting"
                  : streamState
            }
          />
          <button
            className={`review-button ${reviewOpen ? "active" : ""}`}
            type="button"
            onClick={toggleReview}
          >
            {icon("panel")} Review
          </button>
        </div>
      </header>
      <div className="stream-status">
        <i />
        {streamState === "live"
          ? "Live session stream"
          : streamState === "connecting"
            ? "Connecting to Session"
            : streamState === "failed"
              ? "Session stream failed"
              : "Reconnecting with a fresh snapshot"}
        <code>event {snapshot.eventSequence}</code>
      </div>
      {error && <div className="workspace-error">{error}</div>}
      <div
        className="messages"
        ref={messagesRef}
        onScroll={(event) => {
          const element = event.currentTarget;
          followLatestRef.current =
            element.scrollHeight - element.scrollTop - element.clientHeight < 96;
        }}
      >
        {snapshot.messages.map((message) => (
          <article className={`message message-${message.role}`} key={message.id}>
            <span className="message-author">{message.role === "user" ? "You" : "SunCode"}</span>
            {message.role === "user" ? (
              <div>{message.text}</div>
            ) : (
              <MarkdownMessage text={message.text} />
            )}
          </article>
        ))}
        {snapshot.pendingApproval && (
          <div className="approval-card">
            <div>
              <strong>Approval required</strong>
              <small>{snapshot.pendingApproval.summary} · policy decision</small>
            </div>
            <code>{snapshot.pendingApproval.scope}</code>
            <p>
              {snapshot.pendingApproval.detail ?? "SunCode is waiting for your policy decision."}
            </p>
            <div>
              <button
                className="primary-button"
                type="button"
                onClick={() => void resolveApproval("allow_once")}
              >
                Allow once
              </button>
              <button
                className="quiet-button"
                type="button"
                onClick={() => void resolveApproval("deny")}
              >
                Deny
              </button>
            </div>
          </div>
        )}
        {snapshot.pendingQuestion && <QuestionCard />}
        {snapshot.state === "running" && (
          <div className="thinking-indicator" role="status" aria-live="polite">
            <span>SunCode is thinking</span>
            <i />
            <i />
            <i />
          </div>
        )}
        {snapshot.state === "failed" && (
          <div className="approval-card">
            <strong>Turn failed</strong>
            <p>The Desktop reported a failed turn. Retry it when you are ready.</p>
            <button className="primary-button" type="button" onClick={() => void retryTurn()}>
              Retry turn
            </button>
          </div>
        )}
      </div>
      <Composer />
      <footer className="conversation-footer">
        <button type="button" onClick={() => void cancelTurn()}>
          Cancel turn
        </button>
        <span>
          Last-Event-ID: {useWebStore.getState().lastEventId ?? `session:${snapshot.eventSequence}`}
        </span>
      </footer>
    </main>
  );
}

function QuestionCard() {
  const question = useWebStore((state) => state.snapshot?.pendingQuestion);
  const replyQuestion = useWebStore((state) => state.replyQuestion);
  const [answer, setAnswer] = useState("");
  if (!question) return null;
  return (
    <div className="approval-card question-card">
      <strong>{question.prompt}</strong>
      {question.options.map((option) => (
        <button
          className="quiet-button"
          key={option}
          type="button"
          onClick={() => void replyQuestion(option)}
        >
          {option}
        </button>
      ))}
      <input
        value={answer}
        onChange={(event) => setAnswer(event.target.value)}
        placeholder="Your answer"
      />
      <button
        className="primary-button"
        type="button"
        disabled={!answer.trim()}
        onClick={() => void replyQuestion(answer)}
      >
        Reply
      </button>
    </div>
  );
}

function ReviewPanel() {
  const snapshot = useWebStore((state) => state.snapshot);
  return (
    <aside className="review-panel">
      <div className="review-heading">
        <span>
          <span className="eyebrow">SESSION CONTEXT</span>
          <strong>Review</strong>
        </span>
        <button type="button" onClick={useWebStore.getState().toggleReview}>
          ×
        </button>
      </div>
      <section>
        <span className="eyebrow">CURRENT SESSION</span>
        <p>{snapshot?.title ?? "No session selected"}</p>
        <small>
          {snapshot
            ? `${snapshot.messages.length} messages · revision ${snapshot.revision}`
            : "Select a session to inspect it."}
        </small>
      </section>
      <section>
        <span className="eyebrow">RUNTIME</span>
        <p>
          <Connection state={snapshot?.state === "running" ? "connected" : "degraded"} />
        </p>
        <small>Policy and approvals remain local to Desktop.</small>
      </section>
      <div className="undo-note">
        Filesystem changes are controlled by the paired Desktop agent.
      </div>
    </aside>
  );
}

function Settings({ onBack, onPair }: { onBack: () => void; onPair: () => void }) {
  const { credential, host, unpair, setEncryptionEnabled } = useWebStore();
  return (
    <main className="settings-page">
      <header>
        <button className="back-button" type="button" onClick={onBack}>
          ‹ Sessions
        </button>
        <Connection state={host?.connectionState ?? "offline"} />
      </header>
      <div className="settings-content">
        <span className="eyebrow">BROWSER SETTINGS</span>
        <h1>Settings</h1>
        <p>Manage the paired Host, browser-side encryption, and connection recovery.</p>
        <div className="settings-list">
          <div>
            <span>{icon("lock")}</span>
            <strong>
              Encrypt outgoing requests
              <small>Send request bodies as AES-256-GCM payloads when enabled</small>
            </strong>
            <label className="settings-toggle">
              <input
                type="checkbox"
                checked={Boolean(credential?.encryptionEnabled && credential.e2eKey)}
                disabled={!credential?.e2eKey}
                onChange={(event) => setEncryptionEnabled(event.target.checked)}
              />
              <span>
                {credential?.e2eKey ? (credential.encryptionEnabled ? "On" : "Off") : "Unavailable"}
              </span>
            </label>
          </div>
          <div>
            <span>{icon("monitor")}</span>
            <strong>
              {host?.displayName ?? "No Desktop paired"}
              <small>{host ? "Remote Server host projection" : "Pair a Desktop to start"}</small>
            </strong>
            <b>{host ? host.connectionState : "Not paired"}</b>
          </div>
          <div>
            <span>{icon("plus")}</span>
            <strong>
              Pair another Desktop<small>Use a one-time Desktop pairing URL</small>
            </strong>
            <button className="quiet-button" type="button" onClick={onPair}>
              Pair
            </button>
          </div>
          <div>
            <span>{icon("panel")}</span>
            <strong>
              Session stream recovery<small>Reconnect with Last-Event-ID after cursor expiry</small>
            </strong>
            <code>automatic</code>
          </div>
        </div>
        {credential && (
          <div className="security-note">
            Access credentials and the browser E2E key are kept in this browser session. The key is
            never sent to the relay.
          </div>
        )}
        <button className="danger-button" type="button" onClick={() => void unpair()}>
          Unpair this browser
        </button>
      </div>
    </main>
  );
}

function Pairing({ onBack, onPaired }: { onBack: () => void; onPaired: () => void }) {
  const pairFromUrl = useWebStore((state) => state.pairFromUrl);
  const pairingError = useWebStore((state) => state.pairingError);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  return (
    <main className="settings-page">
      <header>
        <button className="back-button" type="button" onClick={onBack}>
          ‹ Settings
        </button>
        <span className="eyebrow">SECURE DEVICE LINK</span>
      </header>
      <div className="pairing-content">
        <span className="eyebrow">PAIR A DESKTOP</span>
        <h1>Pair this browser</h1>
        <p>
          Paste the one-time pairing URL from SunCode Desktop. The encryption key remains in this
          browser and is never sent to the relay.
        </p>
        <label>
          Pairing URL
          <textarea
            value={value}
            onChange={(event) => setValue(event.target.value)}
            placeholder="https://relay…?hostId=…&code=…&k=…&e2e=1"
          />
        </label>
        {pairingError && <div className="error-note">{pairingError}</div>}
        <button
          className="primary-button"
          type="button"
          disabled={busy || !value.trim()}
          onClick={async () => {
            setBusy(true);
            try {
              await pairFromUrl(value);
              onPaired();
            } catch {
              /* pairing error is rendered above */
            } finally {
              setBusy(false);
            }
          }}
        >
          {busy ? "Pairing…" : "Confirm pairing"}
        </button>
      </div>
    </main>
  );
}

function timeAgo(value: string) {
  const seconds = Math.max(1, Math.floor((Date.now() - new Date(value).getTime()) / 1000));
  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

export function App() {
  const hydrate = useWebStore((state) => state.hydrate);
  const hydrated = useWebStore((state) => state.hydrated);
  const reviewOpen = useWebStore((state) => state.reviewOpen);
  const [page, setPage] = useState<"workspace" | "settings" | "pairing">("workspace");
  useEffect(() => {
    void hydrate();
  }, [hydrate]);
  if (!hydrated) return <div className="loading-screen">Restoring secure browser session…</div>;
  if (page === "settings")
    return <Settings onBack={() => setPage("workspace")} onPair={() => setPage("pairing")} />;
  if (page === "pairing")
    return <Pairing onBack={() => setPage("settings")} onPaired={() => setPage("workspace")} />;
  return (
    <div className={`app-shell ${reviewOpen ? "review-open" : ""}`}>
      <Sidebar onSettings={() => setPage("settings")} />
      <Conversation />
      {reviewOpen && <ReviewPanel />}
    </div>
  );
}
