import { useState } from "react";
import { Button } from "../../components/universal/button/index.js";
import { Icon } from "../../shared/Icon.jsx";
import { ModuleLink, PageHeader, Section, Status } from "../../shared/PagePrimitives.jsx";

const projects = [
  { name: "suncode", detail: "2 active sessions", active: true },
  { name: "mobile-app", detail: "1 idle session" },
  { name: "design-system", detail: "No active sessions" },
];

const sessions = [
  {
    title: "Fix login redirect",
    project: "suncode",
    state: "Waiting for approval",
    tone: "warning",
    time: "2 min ago",
    preview: "I found one file write that needs your approval.",
  },
  {
    title: "Refactor API client",
    project: "mobile-app",
    state: "Running",
    tone: "running",
    time: "12 min ago",
    preview: "Updating the request retry policy and its tests.",
  },
  {
    title: "Review release notes",
    project: "design-system",
    state: "Idle",
    tone: "neutral",
    time: "Yesterday",
    preview: "The latest response is ready to review.",
  },
];

function ConnectionLamp({ state = "connected" }) {
  const labels = {
    connected: "Desktop connected",
    degraded: "Desktop unavailable",
    reconnecting: "Reconnecting",
    offline: "Offline",
    unauthorized: "Unauthorized",
  };
  return (
    <span className={`web-connection is-${state}`}>
      <i aria-hidden="true" /> {labels[state]}
    </span>
  );
}

function ProjectSessionRail({ selected, onSelect, onOpenSettings }) {
  const [expandedProjects, setExpandedProjects] = useState({ suncode: true });
  const sessionsByProject = sessions.reduce((groups, session) => {
    groups[session.project] = [...(groups[session.project] ?? []), session];
    return groups;
  }, {});

  return (
    <aside className="web-rail">
      <div className="web-brand">
        <span className="web-brand-mark"><Icon name="foundation" size={17} /></span>
        <strong>SunCode</strong>
        <span className="web-beta">WEB</span>
      </div>
      <div className="web-host-switcher">
        <span className="web-host-icon"><Icon name="computer" size={17} /></span>
        <span><strong>MacBook Pro</strong><small>Development PC</small></span>
        <Icon name="chevron-right" size={14} />
      </div>
      <div className="web-project-session-list">
        <div className="web-sidebar-heading"><span className="web-kicker">PROJECTS</span><button type="button" aria-label="Refresh projects"><Icon name="refresh" size={14} /></button></div>
        {projects.map((project) => {
          const expanded = expandedProjects[project.name];
          const projectSessions = sessionsByProject[project.name] ?? [];
          return (
            <div className={`web-project-group${expanded ? " is-expanded" : ""}`} key={project.name}>
              <button type="button" className="web-project-picker" onClick={() => setExpandedProjects((current) => ({ ...current, [project.name]: !expanded }))} aria-expanded={expanded}>
                <Icon name="folder" size={15} /><span><strong>{project.name}</strong><small>{project.detail}</small></span><Icon name="chevron-right" size={13} />
              </button>
              {expanded && projectSessions.length > 0 && <div className="web-project-sessions">
                {projectSessions.map((session) => <button type="button" key={session.title} className={`web-sidebar-session${selected === session.title ? " is-active" : ""}`} onClick={() => onSelect?.(session.title)}><span className={`web-state-dot is-${session.tone}`} /><span><strong>{session.title}</strong><small>{session.state} · {session.time}</small></span></button>)}
              </div>}
              {expanded && projectSessions.length === 0 && <span className="web-sidebar-empty">No active sessions</span>}
            </div>
          );
        })}
      </div>
      <div className="web-rail-footer">
        <ConnectionLamp />
        <button type="button" className="web-rail-settings" onClick={onOpenSettings}>
          <Icon name="settings" size={16} /> Settings
        </button>
        <span className="web-protocol">remote.v1 · E2E enabled</span>
      </div>
    </aside>
  );
}

function ProjectColumn({ selectedProject = "suncode" }) {
  return (
    <aside className="web-project-column">
      <div className="web-column-heading"><div><span className="web-kicker">HOST PROJECTS</span><h3>Projects</h3></div><button type="button" aria-label="Refresh projects"><Icon name="refresh" size={15} /></button></div>
      <div className="web-project-list">
        {projects.map((project) => (
          <button type="button" key={project.name} className={selectedProject === project.name ? "is-active" : ""}>
            <Icon name="folder" size={16} /><span><strong>{project.name}</strong><small>{project.detail}</small></span><Icon name="chevron-right" size={14} />
          </button>
        ))}
      </div>
      <div className="web-column-note"><Icon name="lock" size={14} /><span>Projects and paths are read from the paired Desktop. The relay never sees absolute paths.</span></div>
    </aside>
  );
}

function SessionColumn({ selected = "Fix login redirect", onSelect }) {
  return (
    <section className="web-session-column">
      <div className="web-column-heading"><div><span className="web-kicker">SUNCODE</span><h3>Sessions</h3></div><button type="button" aria-label="Create session"><Icon name="plus" size={16} /></button></div>
      <div className="web-session-toolbar"><button type="button" className="is-selected">All</button><button type="button">Needs attention <b>1</b></button><button type="button" aria-label="Search sessions"><Icon name="search" size={14} /></button></div>
      <div className="web-session-list">
        {sessions.map((session) => (
          <button type="button" key={session.title} className={`web-session-row${selected === session.title ? " is-active" : ""}`} onClick={() => onSelect?.(session.title)}>
            <span className="web-session-row-title"><strong>{session.title}</strong><i className={`web-state-dot is-${session.tone}`} /></span>
            <span className="web-session-preview">{session.preview}</span>
            <span className="web-session-meta"><span>{session.project}</span><time>{session.time}</time></span>
            <span className={`web-state-text is-${session.tone}`}>{session.state}</span>
          </button>
        ))}
      </div>
      <button type="button" className="web-new-session"><Icon name="plus" size={15} /> New session</button>
    </section>
  );
}

function Message({ role, children }) {
  return <div className={`web-message is-${role}`}><span className="web-message-label">{role === "user" ? "You" : "SunCode"}</span><div>{children}</div></div>;
}

function Conversation({ state = "approval", sessionTitle = "Fix login redirect", reviewOpen, onToggleReview }) {
  const [streamState, setStreamState] = useState("live");
  return (
    <main className="web-conversation">
      <header className="web-conversation-header">
        <div><span className="web-kicker">PRIMARY SESSION</span><h2>{sessionTitle}</h2><span className="web-session-path"><Icon name="folder" size={13} /> suncode <span>·</span> updated 2 min ago</span></div>
        <div className="web-conversation-actions"><ConnectionLamp state={streamState === "live" ? "connected" : "reconnecting"} /><button type="button" className={`web-review-toggle${reviewOpen ? " is-active" : ""}`} onClick={onToggleReview}><Icon name="panel-right" size={15} /> Review</button><button type="button" aria-label="Session actions"><Icon name="more" size={17} /></button></div>
      </header>
      <div className="web-stream-banner">
        <span><i /> {streamState === "live" ? "Live session stream" : "Reconnecting with a fresh snapshot"}</span>
        <code>event 42</code>
        {streamState !== "live" && <Button variant="quiet" size="sm" onClick={() => setStreamState("live")}>Retry</Button>}
      </div>
      <div className="web-message-scroll">
        <Message role="user">Fix the redirect loop after login and add a regression test.</Message>
        <Message role="assistant"><p>I found the redirect loop in <code>auth/redirect.ts</code>. I’m checking the existing test coverage before writing the fix.</p><div className="web-activity-row"><span className="web-activity-icon"><Icon name="search" size={14} /></span><span><strong>Read</strong> <code>src/auth/redirect.ts</code></span><em>completed</em></div></Message>
        <Message role="assistant"><p>The fix is ready. Writing one file needs your approval.</p>{state === "approval" && <ApprovalCard />}{state === "question" && <QuestionCard />}</Message>
      </div>
      <div className="web-composer"><button type="button" className="web-attach" aria-label="Attach image" title="Image attachments are not available in the web client yet" disabled><Icon name="plus" size={16} /></button><span>Message SunCode…</span><button type="button" className="web-send" aria-label="Send message"><Icon name="arrow-up" size={15} /></button></div>
      <footer className="web-conversation-footer"><span>Cancel turn</span><span><code>Last-Event-ID: host-01:42</code> · SSE session stream</span></footer>
    </main>
  );
}

function ApprovalCard() {
  return <div className="web-authority-card is-approval"><div className="web-authority-heading"><span className="web-authority-icon"><Icon name="lock" size={15} /></span><span><strong>Approval required</strong><small>Write 1 file · policy decision</small></span><Status tone="warning">Pending</Status></div><code>src/auth/redirect.ts</code><p>This operation changes a project file. SunCode will create a checkpoint before writing.</p><div className="web-inline-actions"><Button variant="primary" size="sm">Allow once</Button><Button variant="quiet" size="sm">Deny</Button></div></div>;
}

function QuestionCard() {
  return <div className="web-authority-card is-question"><div className="web-authority-heading"><span className="web-authority-icon"><Icon name="message" size={15} /></span><span><strong>SunCode needs an answer</strong><small>Question from the current turn</small></span><Status tone="warning">Waiting</Status></div><p>Which redirect behavior should I preserve for an already authenticated user?</p><div className="web-question-options"><button type="button" className="is-selected"><i /> Return to the requested page</button><button type="button"><i /> Always open the dashboard</button><button type="button"><i /> Let me answer with text</button></div><Button variant="primary" size="sm">Send answer</Button></div>;
}

function AuthorityPanel({ state = "approval" }) {
  return <aside className="web-authority-panel"><div className="web-panel-heading"><div><span className="web-kicker">SESSION CONTEXT</span><h3>Review</h3></div><button type="button" aria-label="Close review panel"><Icon name="close" size={15} /></button></div><div className="web-context-row"><span>Context window</span><strong>68% <small>of 128k tokens</small></strong><div className="web-progress"><i style={{ width: "68%" }} /></div></div><div className="web-review-section"><span className="web-kicker">CURRENT TURN</span><div className="web-review-item"><span className="web-review-icon is-running"><Icon name="activity" size={15} /></span><span><strong>{state === "approval" ? "Waiting for approval" : "Waiting for answer"}</strong><small>Turn 8 · 2 min ago</small></span></div></div><div className="web-review-section"><span className="web-kicker">TOUCHED FILES</span><div className="web-file-row"><Icon name="file-code" size={15} /><code>src/auth/redirect.ts</code><span>+12 −4</span></div><div className="web-file-row"><Icon name="file-code" size={15} /><code>tests/auth.test.ts</code><span>+18</span></div></div><div className="web-review-section"><span className="web-kicker">RUNTIME</span><div className="web-runtime-row"><ConnectionLamp /><span>Rust agent 0.1.0</span></div><div className="web-runtime-row"><Icon name="lock" size={14} /><span>Policy and approvals local to Desktop</span></div></div><div className="web-undo-note"><Icon name="refresh" size={14} /><span>Filesystem changes can be undone from the Desktop checkpoint. External side effects are not covered.</span></div></aside>;
}

function WebRemoteShell({ view = "shell", initialState = "approval" }) {
  const [activeNav, setActiveNav] = useState(view === "pairing" ? "Pairing" : view === "security" ? "Security" : "Sessions");
  const [sessionTitle, setSessionTitle] = useState("Fix login redirect");
  const [state, setState] = useState(initialState);
  const [reviewOpen, setReviewOpen] = useState(false);
  if (activeNav === "Pairing") return <PairingPanel onBack={() => setActiveNav("Security")} />;
  if (activeNav === "Security") return <SecurityPanel onBack={() => setActiveNav("Sessions")} onPair={() => setActiveNav("Pairing")} />;
  return <div className={`web-app-shell${reviewOpen ? " is-review-open" : ""}`}><ProjectSessionRail selected={sessionTitle} onSelect={setSessionTitle} onOpenSettings={() => setActiveNav("Security")} /><Conversation state={state} sessionTitle={sessionTitle} reviewOpen={reviewOpen} onToggleReview={() => setReviewOpen((open) => !open)} />{reviewOpen && <AuthorityPanel state={state} />}<div className="web-state-switcher"><span>Review states</span><button type="button" className={state === "approval" ? "is-selected" : ""} onClick={() => setState("approval")}>Approval</button><button type="button" className={state === "question" ? "is-selected" : ""} onClick={() => setState("question")}>Question</button></div></div>;
}

function PairingPanel({ onBack }) {
  return <div className="web-focused-page"><div className="web-focused-header"><button type="button" className="web-back-button" onClick={onBack}><Icon name="chevron-right" size={15} /> Sessions</button><ConnectionLamp /><span className="web-beta">WEB REVIEW</span></div><div className="web-pairing-layout"><div className="web-pairing-main"><span className="web-kicker">SECURE DEVICE LINK</span><h2>Pair this browser with a Desktop</h2><p>Use the one-time pairing QR code shown in SunCode Desktop. The Remote Server only relays bounded requests; it never receives the encryption key.</p><div className="web-qr-placeholder" aria-label="QR code preview"><span className="web-qr-corner top-left" /><span className="web-qr-corner top-right" /><span className="web-qr-corner bottom-left" /><span className="web-qr-corner bottom-right" /><div className="web-qr-dots">▦</div><span>Drop a pairing QR image or use your camera</span></div><div className="web-pairing-actions"><Button variant="primary" icon="computer">Choose QR image</Button><Button variant="quiet">Use camera</Button></div><p className="web-manual-fallback"><Icon name="link" size={14} /> Prefer a manual flow? Paste the pairing URL instead.</p></div><div className="web-trust-panel"><div className="web-trust-heading"><span className="web-authority-icon"><Icon name="lock" size={16} /></span><div><strong>What you are trusting</strong><span>Verify before confirming</span></div></div><div className="web-trust-row"><span>Desktop</span><strong>MacBook Pro</strong><Status tone="success">Verified</Status></div><div className="web-trust-row"><span>Remote Server</span><code>relay.suncode.dev</code></div><div className="web-trust-row"><span>Host ID</span><code>host-01J8YQ6M7A</code></div><div className="web-trust-row"><span>Key fingerprint</span><code>SHA256: 7A:31:…:D9</code></div><div className="web-e2e-note"><Icon name="lock" size={15} /><span><strong>End-to-end encryption enabled</strong>The AES-256 key stays in this browser and the Desktop. It is never sent to the relay. Rotating it requires pairing again.</span></div><Button variant="primary" className="web-confirm-pairing">Confirm pairing</Button></div></div></div>;
}

function SecurityPanel({ onBack, onPair }) {
  return <div className="web-focused-page"><div className="web-focused-header"><button type="button" className="web-back-button" onClick={onBack}><Icon name="chevron-right" size={15} /> Sessions</button><ConnectionLamp /></div><div className="web-settings-page"><span className="web-kicker">BROWSER SETTINGS</span><h2>Settings</h2><p>Manage the paired Host, browser-side encryption, and connection recovery. Pairing lives here rather than in the primary workspace navigation.</p><div className="web-settings-section"><div className="web-setting-row"><span className="web-setting-icon"><Icon name="lock" size={16} /></span><span><strong>End-to-end encryption</strong><small>AES-256-GCM payloads and Session SSE events</small></span><Status tone="success">Enabled</Status></div><div className="web-setting-row"><span className="web-setting-icon"><Icon name="server" size={16} /></span><span><strong>MacBook Pro</strong><small>relay.suncode.dev · last sync just now</small></span><Status tone="success">Connected</Status><Button variant="quiet" size="sm">Manage</Button></div><div className="web-setting-row"><span className="web-setting-icon"><Icon name="link" size={16} /></span><span><strong>Pair another Desktop</strong><small>Scan a new one-time QR code without adding another workspace tab</small></span><Button variant="quiet" size="sm" onClick={onPair}>Pair</Button></div><div className="web-setting-row"><span className="web-setting-icon"><Icon name="refresh" size={16} /></span><span><strong>Session stream recovery</strong><small>Reconnect with Last-Event-ID and request a fresh snapshot after cursor expiry</small></span><code>automatic</code></div></div><div className="web-security-warning"><Icon name="warning" size={16} /><span><strong>Transport reminder</strong><br />Prefer an HTTPS Remote Server endpoint. HTTP exposes tokens and plaintext routing data to the network.</span></div><Button variant="danger">Unpair this browser</Button></div></div>;
}

export function WebProjectPage() {
  return <><PageHeader title="Web remote control" description="A browser-first control desk for the same paired Desktop sessions and Remote Server contract used by Mobile." status="Design reference" tone="implemented" /><Section id="module-index" title="Web modules" description="Review the complete shell and the security-critical entry flows before production implementation in apps/web."><div className="module-card-grid"><ModuleLink to="/projects/web/shell" icon="platform" title="Control desk" description="Hosts, Projects, Sessions, live conversation, and the authority review bay." status="Primary surface" tone="implemented" /><ModuleLink to="/projects/web/pairing" icon="link" title="Secure pairing" description="QR/manual pairing, Host identity confirmation, and E2E trust language." /><ModuleLink to="/projects/web/session" icon="message" title="Session detail" description="Streaming conversation with approval, question, reconnect, and composer states." /><ModuleLink to="/projects/web/security" icon="lock" title="Security settings" description="Paired Host, encryption, token-safe recovery, and unpairing states." /></div></Section><Section id="contract-notes" title="Contract alignment"><div className="web-contract-notes"><div><strong>Same API surface</strong><span>Uses the mobile remote-control paths, DTOs, request IDs, and one Session SSE stream.</span></div><div><strong>Browser E2E</strong><span>WebCrypto will own the AES key locally; the relay sees only routing headers and opaque payloads.</span></div><div><strong>Review-only</strong><span>This route contains fixture data only. It does not create apps/web or make network calls.</span></div></div></Section></>;
}

export function WebShellPage() { return <><PageHeader title="Control desk" description="Desktop-first browser composition with supporting bays that yield as the viewport narrows." status="Review reference" tone="implemented" /><Section id="web-shell" title="Primary shell"><WebRemoteShell /></Section></>; }
export function WebPairingPage() { return <><PageHeader title="Secure pairing" description="A browser pairing flow that makes Host identity and E2E trust explicit before the first authenticated request." status="Review reference" tone="implemented" /><Section id="web-pairing" title="Pair a Desktop"><WebRemoteShell view="pairing" /></Section></>; }
export function WebSessionPage() { return <><PageHeader title="Session detail" description="One live Session stream, with approval and question variants sharing the same conversation surface." status="Review reference" tone="implemented" /><Section id="web-session" title="Live Session"><WebRemoteShell initialState="question" /></Section></>; }
export function WebSecurityPage() { return <><PageHeader title="Security settings" description="Connection health, browser-side E2E state, recovery semantics, and safe unpairing." status="Review reference" tone="implemented" /><Section id="web-security" title="Connection and encryption"><WebRemoteShell view="security" /></Section></>; }
