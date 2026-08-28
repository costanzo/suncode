import { useState } from "react";
import { Button } from "../../../components/universal/button/index.js";
import { ModelDropdown, SingleDropdown } from "../../../components/universal/dropdown/index.js";
import { Modal } from "../../../components/universal/modal/index.js";
import { Icon } from "../../../shared/Icon.jsx";

const sessions = [
  { title: "Workspace information architecture", time: "2 min ago", pinned: true },
  { title: "Provider migration review", time: "Yesterday" },
  { title: "Desktop navigation polish", time: "Aug 26" },
];

const explorerNodes = [
  { id: "agents", name: ".agents", kind: "folder", depth: 0 },
  { id: "apps", name: "apps", kind: "folder", depth: 0 },
  { id: "desktop", parent: "apps", name: "desktop-avalonia", kind: "folder", depth: 1 },
  { id: "views", parent: "desktop", name: "Views", kind: "folder", depth: 2 },
  { id: "workspace-file", parent: "views", name: "ProjectWorkspace.axaml", kind: "file", depth: 3, selected: true },
  { id: "agent", name: "agent", kind: "folder", depth: 0 },
  { id: "design-system", name: "design-system", kind: "folder", depth: 0 },
];
const dependencyNodes = [
  { id: "deps", name: "Dependencies", kind: "folder", depth: 0 },
  { id: "shared", parent: "deps", name: "shared-ui", kind: "folder", depth: 1 },
  { id: "shared-readme", parent: "shared", name: "README.md", kind: "file", depth: 2 },
];

const changes = [
  { status: "M", kind: "modified", path: "design-system/src/projects/desktop/workspace/WorkspacePrimitives.jsx", additions: 45, deletions: 8, staged: false, unstaged: true },
  { status: "M", kind: "modified", path: "design-system/src/styles/review.css", additions: 62, deletions: 24, staged: false, unstaged: true },
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

function IconButton({ icon, label, active = false, onClick, disabled = false }) {
  return <button type="button" className={`workspace-icon-button ${active ? "is-active" : ""}`} aria-label={label} aria-pressed={active} onClick={onClick} disabled={disabled}><Icon name={icon} size={15} /></button>;
}

export function TrafficLights() {
  return <div className="traffic-lights" aria-label="Window controls"><span className="traffic-light close" /><span className="traffic-light minimize" /><span className="traffic-light maximize" /></div>;
}

export function SessionPanel({ compact = false, standalone = false, initialSessions = sessions }) {
  const [selected, setSelected] = useState(0);
  const [items, setItems] = useState(initialSessions);
  const [menu, setMenu] = useState(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [newTitle, setNewTitle] = useState("");
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
  const togglePin = (index) => {
    setItems((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, pinned: !item.pinned } : item));
    setMenu(null);
  };
  const archive = (index) => {
    setItems((current) => current.filter((_, itemIndex) => itemIndex !== index));
    setSelected(0);
    setMenu(null);
  };
  return <aside className={`workspace-panel workspace-sessions ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}>
    <header className="workspace-panel-header"><span>SESSIONS</span><IconButton icon="plus" label="New session" onClick={createSession} /></header>
    <div className="workspace-session-list">
      {items.map((session, index) => <div className="workspace-session-wrap" key={`${session.title}-${index}`}>
        <button type="button" className={`workspace-session ${selected === index ? "is-selected" : ""}`} onClick={() => setSelected(index)}>
          <span className="workspace-session-pin">{session.pinned && <Icon name="pin" size={12} />}</span>
          <span><strong>{session.title}</strong><small>{session.time}</small></span>
        </button>
        <button type="button" className="workspace-session-more" aria-label={`Actions for ${session.title}`} aria-expanded={menu === index} onClick={() => setMenu(menu === index ? null : index)}><Icon name="more" size={14} /></button>
        {menu === index && <div className="workspace-session-menu"><button type="button" onClick={() => togglePin(index)}>{session.pinned ? "Unpin" : "Pin"}</button><button type="button" onClick={() => archive(index)}>Archive</button></div>}
      </div>)}
      {!items.length && <div className="workspace-session-empty"><Icon name="components" size={22} /><strong>No sessions yet</strong><span>Use + to create one.</span></div>}
    </div>
    <Modal open={createOpen} onClose={() => setCreateOpen(false)} title="New session" description="Give this conversation a name before you begin." actions={<><button className="btn" onClick={() => setCreateOpen(false)}>Cancel</button><button className="btn btn-primary" onClick={confirmCreate} disabled={!newTitle.trim()}>Create session</button></>}><label className="modal-field-label" htmlFor="new-session-name">Session name</label><input id="new-session-name" className="field" value={newTitle} onChange={(event) => setNewTitle(event.target.value)} placeholder="e.g. Provider migration review" onKeyDown={(event) => { if (event.key === "Enter") confirmCreate(); }} /></Modal>
  </aside>;
}

export function ExplorerPanel({ compact = false, standalone = false, hasDependency = true }) {
  const nodes = hasDependency ? [...explorerNodes, ...dependencyNodes] : explorerNodes;
  const [expanded, setExpanded] = useState(() => new Set(["apps", "desktop", "views", "deps", "shared"]));
  const visibleNodes = nodes.filter((node) => {
    let parentId = node.parent;
    while (parentId) {
      if (!expanded.has(parentId)) return false;
      parentId = nodes.find((candidate) => candidate.id === parentId)?.parent;
    }
    return true;
  });
  const toggleNode = (node) => {
    if (node.kind !== "folder") return;
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(node.id)) next.delete(node.id); else next.add(node.id);
      return next;
    });
  };
  return <aside className={`workspace-panel workspace-explorer ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}>
    <header className="workspace-panel-header"><span>EXPLORER</span><span className="workspace-panel-actions"><IconButton icon="refresh" label="Refresh explorer" disabled /><IconButton icon="plus" label="Add dependency folder" disabled /></span></header>
    <div className="workspace-tree" role="tree" aria-label="Project files">
      {!hasDependency && <div className="workspace-tree-empty"><strong>No dependencies</strong><span>Add a read-only dependency folder to browse it here.</span></div>}
      {visibleNodes.map((node) => <button key={node.id} type="button" role="treeitem" aria-level={node.depth + 1} aria-expanded={node.kind === "folder" ? expanded.has(node.id) : undefined} aria-selected={node.selected || undefined} className={`workspace-tree-row ${node.selected ? "is-selected" : ""}`} style={{ "--tree-depth": node.depth }} onClick={() => toggleNode(node)}>
        {node.kind === "folder" ? <Icon name="chevron-right" className={expanded.has(node.id) ? "is-open" : ""} size={12} /> : <span />}
        <Icon name={node.kind === "folder" ? "project" : "files"} size={13} />
        <span>{node.name}</span>
      </button>)}
    </div>
  </aside>;
}

export function ConversationPanel({ compact = false, standalone = false, state = "content-waiting" }) {
  const [message, setMessage] = useState("");
  const [processOpen, setProcessOpen] = useState(true);
  const [toolOpen, setToolOpen] = useState(null);
  const hasSession = state !== "no-session";
  const hasContent = state !== "new-session" && hasSession;
  const updating = state === "content-updating";
  return <section className={`workspace-conversation ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} workspace-conversation-${state}`}>
    {!hasSession && <div className="workspace-conversation-empty"><Icon name="workspace" size={24} /><strong>No session selected</strong><span>Create or select a session to start a conversation.</span></div>}
    {hasSession && !hasContent && <div className="workspace-conversation-empty"><Icon name="plus" size={24} /><strong>New session</strong><span>Send a message to start this conversation.</span></div>}
    {hasContent && <>
    <div className="workspace-message workspace-message-user">Add the Workspace surface to the design system, but keep each major area independently reachable.</div>
    <div className="workspace-process">
      <button type="button" className="workspace-process-toggle" aria-expanded={processOpen} onClick={() => setProcessOpen(!processOpen)}><Icon name="chevron-right" className={processOpen ? "is-open" : ""} size={12} /> Worked for 42s</button>
      {processOpen && <><button type="button" className="workspace-tool-row" aria-expanded={toolOpen === 0} onClick={() => setToolOpen(toolOpen === 0 ? null : 0)}><Icon name="activity" size={14} /><span>Read ProjectWorkspace.axaml</span><small>Succeeded</small><Icon name="chevron-right" className={toolOpen === 0 ? "is-open" : ""} size={12} /></button>{toolOpen === 0 && <pre className="workspace-tool-detail">request  apps/desktop-avalonia/Views/Projects/ProjectWorkspace.axaml{"\n"}result   218 lines read</pre>}<button type="button" className="workspace-tool-row" aria-expanded={toolOpen === 1} onClick={() => setToolOpen(toolOpen === 1 ? null : 1)}><Icon name="files" size={14} /><span>Updated workspace routes and modules</span><small>Succeeded</small><Icon name="chevron-right" className={toolOpen === 1 ? "is-open" : ""} size={12} /></button>{toolOpen === 1 && <pre className="workspace-tool-detail">result   8 modules updated</pre>}</>}
    </div>
    <div className="workspace-message workspace-message-assistant">
      <p>I split Workspace into a complete composition and focused pages for sessions, explorer, conversation, review, source control, and provider trace.</p>
      <button type="button" className="workspace-copy" aria-label="Copy response"><Icon name="copy" size={13} /></button>
    </div>
    {updating && <div className="workspace-running-indicator" role="status" aria-label="Agent is working"><i /><i /><i /></div>}
    </>}
    {hasSession && <div className="workspace-composer">
      <textarea value={message} onChange={(event) => setMessage(event.target.value)} placeholder="Ask SunCode to work on this project" aria-label="Message SunCode" />
      <div className="workspace-composer-footer"><ModelDropdown groups={workspaceModelGroups} initialValue="gpt-5.6-sol" className="workspace-model-dropdown" /><SingleDropdown options={["Medium", "High"]} initialValue="High" ariaLabel="Reasoning effort" className="workspace-reasoning-dropdown" /><Button variant="primary" className="workspace-send" icon="arrow" aria-label="Send message" disabled={!message.trim()} onClick={() => setMessage("")} /></div>
    </div>}
  </section>;
}

export function ReviewPanel({ compact = false, standalone = false, state = "approval" }) {
  const running = state === "running";
  const waiting = state === "approval" || state === "question";
  const idle = state === "idle";
  return <aside className={`workspace-panel workspace-review ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}>
    <div className="workspace-review-heading"><span>AGENT PROCESSES</span><i /></div>
    <h3>{idle ? "No active process" : running ? "1 active process" : "Awaiting input"}</h3>
    {idle && <div className="workspace-review-empty"><Icon name="activity" size={22} /><strong>Agent is idle</strong><span>Start a turn from the conversation composer.</span></div>}
    {running && <><div className="workspace-process-card"><div><i /><strong>Agent loop</strong><small>Running</small></div><code>Turn turn_01JY7F3K9M</code><span>Model&nbsp; gpt-5.6-sol</span><span>Latest&nbsp; Editing workspace modules</span></div><div className="workspace-todo-card"><div><span>CURRENT TURN TODO</span><small>3 items</small></div><p><Icon name="check" size={11} />Inspect Avalonia workspace</p><p><Icon name="activity" size={11} />Build focused modules</p><p><Icon name="more" size={11} />Verify responsive routes</p></div></>}
    <div className="workspace-review-divider" />
    {waiting && <><span className="workspace-label">REVIEW QUEUE</span>{state === "approval" ? <div className="workspace-approval-card"><div><span>Approval required</span><b>REVIEW</b></div><strong>Run the production design build</strong><code>vite build</code><div className="workspace-approval-actions"><Button variant="primary" size="sm">Allow once</Button><Button variant="danger" size="sm">Deny</Button></div><Button size="sm">Allow for session</Button></div> : <div className="workspace-question-card"><div><span>Clarification needed</span><b>ANSWER</b></div><strong>Which responsive behavior should the focused trace use?</strong><label><input type="radio" name="trace-layout" defaultChecked /> Stack list above detail</label><label><input type="radio" name="trace-layout" /> Keep a narrow split view</label><Button variant="primary" size="sm">Submit answer</Button></div>}</>}
    {!compact && running && <div className="workspace-checkpoint-card"><div><span>CHECKPOINT</span><small>3 files</small></div><strong>Workspace route implementation</strong><code>navigation.js{"\n"}WorkspacePrimitives.jsx{"\n"}review.css</code><Button size="sm">Undo</Button></div>}
  </aside>;
}

export function SourceControlPanel({ onClose, standalone = false, clean = false }) {
  const [scope, setScope] = useState("all");
  const [selectedPath, setSelectedPath] = useState(changes[1].path);
  const [filter, setFilter] = useState("");
  const filtered = (clean ? [] : changes).filter((change) => {
    const matchesScope = scope === "all" || (scope === "staged" ? change.staged : change.unstaged);
    return matchesScope && change.path.toLowerCase().includes(filter.toLowerCase());
  });
  const selected = filtered.find((change) => change.path === selectedPath) ?? filtered[0] ?? null;
  return <section className={`workspace-drawer workspace-git ${standalone ? "is-standalone" : ""}`}>
    <header>
      <Icon className="workspace-git-icon" name="git" size={16} />
      <strong>main</strong>
      <span className="workspace-git-divider" aria-hidden="true" />
      <div className="workspace-scope">{[["all", "All"], ["staged", "Staged"], ["unstaged", "Unstaged"]].map(([value, label]) => <button key={value} type="button" className={scope === value ? "is-selected" : ""} onClick={() => setScope(value)}>{label}</button>)}</div>
      <input value={filter} onChange={(event) => setFilter(event.target.value)} placeholder="Filter changed files" aria-label="Filter changed files" />
      <IconButton icon="refresh" label="Refresh Git status" onClick={() => setFilter("")} />
      <IconButton icon="copy" label="Copy patch" onClick={() => navigator.clipboard?.writeText(diffLines.map((line) => `${line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : " "}${line.text}`).join("\n"))} />
      <IconButton icon="close" label="Close source control" onClick={onClose} disabled={!onClose} />
    </header>
    <div className="workspace-git-body">
      <div className="workspace-change-list">
        <div className="workspace-drawer-label">{filtered.length} {filtered.length === 1 ? "file" : "files"}</div>
        {filtered.map((change) => <button key={change.path} type="button" className={selected?.path === change.path ? "is-selected" : ""} onClick={() => setSelectedPath(change.path)}>
          <b className={`workspace-change-status is-${change.kind}`}>{change.status}</b>
          <span><code>{change.path}</code>{change.oldPath && <small>from {change.oldPath}</small>}</span>
          <small>+{change.additions}  -{change.deletions}</small>
        </button>)}
        {!filtered.length && <div className="workspace-change-empty">{clean ? "Working tree clean." : "No changed files match this filter."}</div>}
      </div>
      <div className="workspace-diff">
        {selected ? <><div className="workspace-diff-heading"><code>{selected.path}</code><span><b>+{selected.additions}</b> <i>-{selected.deletions}</i></span></div>
          <pre aria-label={`Diff for ${selected.path}`}>{diffLines.map((line, index) => <span key={`${line.kind}-${index}`} className={`workspace-diff-line diff-${line.kind}`}><span>{line.oldLine}</span><span>{line.newLine}</span><i aria-hidden="true" /><code>{line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : line.kind === "hunk" ? "" : " "}{line.text}</code></span>)}</pre></>
          : <div className="workspace-diff-empty">{clean ? "Working tree clean." : "No changed files match this filter."}</div>}
      </div>
    </div>
  </section>;
}

export function ProviderTracePanel({ onClose, standalone = false, state = "expanded" }) {
  const [selected, setSelected] = useState(0);
  const [expandedTurn, setExpandedTurn] = useState(state !== "turn-collapsed");
  const [expandedCall, setExpandedCall] = useState(state === "expanded");
  const traces = [
    { title: "Response · gpt-5.6-sol", time: "14:32:18", status: "Completed", tokens: "18,420 → 1,284" },
    { title: "Tool continuation", time: "14:31:46", status: "Completed", tokens: "12,918 → 826" },
    { title: "Initial request", time: "14:30:09", status: "Completed", tokens: "8,204 → 612" },
  ];
  const noTurns = state === "no-turns";
  return <section className={`workspace-drawer workspace-trace ${standalone ? "is-standalone" : ""}`}>
    <header><Icon name="activity" size={16} /><strong>Provider trace</strong><span>{noTurns ? "0 turns" : "1 turn · 3 exchanges"}</span><div /><IconButton icon="refresh" label="Refresh provider trace" disabled /><IconButton icon="copy" label="Copy trace" onClick={() => navigator.clipboard?.writeText("Provider trace preview")} /><IconButton icon="close" label="Close provider trace" onClick={onClose} disabled={!onClose} /></header>
    {noTurns ? <div className="workspace-trace-empty"><Icon name="activity" size={24} /><strong>No turns yet</strong><span>Provider requests will appear here after the agent starts a turn.</span></div> : <div className="workspace-trace-body"><div className="workspace-trace-list"><div className="workspace-drawer-label">CURRENT SESSION</div><button type="button" className={`workspace-trace-turn ${expandedTurn ? "is-expanded" : ""}`} onClick={() => setExpandedTurn(!expandedTurn)}><Icon name="chevron-right" className={expandedTurn ? "is-open" : ""} size={11} /><span><strong>Turn 0198e82c</strong><small>Completed · 1.84 s</small></span><b>3 calls</b></button>{expandedTurn && <div className="workspace-trace-children">{traces.map((trace, index) => <div key={trace.title}><button type="button" className={`workspace-trace-call ${selected === index ? "is-selected" : ""}`} onClick={() => { setSelected(index); setExpandedCall(state === "expanded"); }}><Icon name="chevron-right" className={selected === index && expandedCall ? "is-open" : ""} size={11} /><span><strong>{trace.title}</strong><small>{trace.time}</small></span><span><b>{trace.status}</b><small>{trace.tokens}</small></span></button>{selected === index && expandedCall && <div className="workspace-trace-content-row"><span>USER</span><code>Add the Workspace surface to the design system.</code></div>}</div>)}</div>}</div><div className="workspace-trace-detail"><div className="workspace-trace-title"><code>{traces[selected].title}</code><span>1.84 s&nbsp;&nbsp; gpt-5.6-sol</span></div><div className="workspace-trace-metrics">{[["INPUT", "18,420"], ["OUTPUT", "1,284"], ["CACHE READ", "12,160"], ["CACHE WRITE", "0"], ["CACHE HIT", "66%"], ["DURATION", "1.84 s"]].map(([label, value]) => <div key={label}><span>{label}</span><strong>{value}</strong></div>)}</div><div className="workspace-trace-content"><div><code>TURN 0198e82c · COMPLETED</code><span>exchange&nbsp; exch_01JY7F6P8S</span></div><h4>Messages</h4><p><b>USER</b><span>Add the Workspace surface to the design system.</span></p><p><b>ASSISTANT</b><span>I’ll inspect the current Avalonia composition and preserve its product boundaries.</span></p><h4>Model response</h4><pre>&#123;"status":"completed","finish_reason":"tool_calls"&#125;</pre></div></div></div>}
  </section>;
}

export function WorkspaceWindow() {
  const [navigation, setNavigation] = useState("sessions");
  const [reviewVisible, setReviewVisible] = useState(true);
  const [drawer, setDrawer] = useState(null);
  const toggleDrawer = (next) => setDrawer((current) => current === next ? null : next);
  return <div className="workspace-window">
    <div className="workspace-titlebar"><TrafficLights /><strong className="workspace-project-title">suncode</strong><span>Workspace information architecture</span><IconButton icon="settings" label="Open settings" onClick={() => { window.location.hash = "/projects/desktop/settings"; }} /></div>
    <div className="workspace-window-body">
      <aside className="workspace-gutter"><div><IconButton icon="panel-left" label="Show sessions" active={navigation === "sessions"} onClick={() => setNavigation(navigation === "sessions" ? null : "sessions")} /><IconButton icon="files" label="Show explorer" active={navigation === "explorer"} onClick={() => setNavigation(navigation === "explorer" ? null : "explorer")} /></div><div><IconButton icon="git" label="Show source control" active={drawer === "git"} onClick={() => toggleDrawer("git")} /><IconButton icon="activity" label="Show provider trace" active={drawer === "trace"} onClick={() => toggleDrawer("trace")} /></div></aside>
      <div className="workspace-main-stack"><div className="workspace-main-row">{navigation === "sessions" && <SessionPanel compact />}{navigation === "explorer" && <ExplorerPanel compact />}<ConversationPanel compact />{reviewVisible && <ReviewPanel compact />}</div>{drawer === "git" && <SourceControlPanel onClose={() => setDrawer(null)} />}{drawer === "trace" && <ProviderTracePanel onClose={() => setDrawer(null)} />}</div>
      <aside className="workspace-gutter workspace-gutter-right"><IconButton icon="panel-right" label="Show review" active={reviewVisible} onClick={() => setReviewVisible(!reviewVisible)} /></aside>
    </div>
    <footer className="workspace-statusbar"><div><Icon name="git" size={11} /><code>codex/workspace-design</code><b>3 changes</b><span>+308</span><i>−8</i></div><div><code>gpt-5.6-sol</code><span>19.7k tokens</span><span>3 calls · 4.2s</span></div></footer>
  </div>;
}

export function FocusedWorkspaceFrame({ title, children, className = "" }) {
  return <div className={`workspace-focused-frame ${className}`}><div className="workspace-focused-titlebar"><TrafficLights /><strong>suncode</strong><span>{title}</span><Icon name="settings" size={14} /></div>{children}</div>;
}
