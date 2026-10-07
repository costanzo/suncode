import { ArrowUpRight, Check, ChevronRight, Circle, GitBranch, LockKeyhole, Undo2 } from "lucide-react";
import { MobileNav } from "./components/MobileNav";

const githubUrl = "https://github.com/costanzo/suncode";

const capabilities = [
  {
    label: "Understand",
    title: "A project-aware starting point",
    copy: "Open a local project, give the agent the context it needs, and follow the work from a real session instead of a detached prompt.",
    icon: <Circle size={17} strokeWidth={1.7} />,
  },
  {
    label: "Act",
    title: "Machine access with a visible boundary",
    copy: "Filesystem, search, process, Git, web, MCP, and language-server work goes through Rust-owned policy and an audited dispatcher.",
    icon: <LockKeyhole size={17} strokeWidth={1.7} />,
  },
  {
    label: "Review",
    title: "A trail you can inspect and undo",
    copy: "Watch tool activity as it streams, approve sensitive operations with scope, review touched files, and restore checkpointed changes when needed.",
    icon: <Undo2 size={17} strokeWidth={1.7} />,
  },
];

const workflow = [
  ["01", "Open a project", "Point SunCode at the repository you are working in."],
  ["02", "Give it a turn", "Describe the change and keep relevant files in view."],
  ["03", "Stay in the loop", "Read ordered conversation and tool activity as it happens."],
  ["04", "Decide with context", "Allow, deny, inspect, or undo with the affected scope visible."],
];

function ProductConsole() {
  return (
    <div className="console" aria-label="Illustrative SunCode session interface">
      <div className="console-topbar">
        <div className="traffic-lights" aria-hidden="true"><span /><span /><span /></div>
        <div className="console-project"><span className="status-dot" /> sun-code <span className="console-path">/ agent / core</span></div>
        <span className="console-state">LOCAL SESSION</span>
      </div>
      <div className="console-body">
        <aside className="console-sidebar">
          <p className="console-label">SESSIONS</p>
          <div className="session active"><span className="session-pulse" /> Refactor context builder <small>now</small></div>
          <div className="session"><span /> Provider transfer status <small>yesterday</small></div>
          <div className="session"><span /> Add approval scope <small>Mon</small></div>
          <div className="sidebar-rule" />
          <p className="console-label">PROJECT</p>
          <div className="file-row"><GitBranch size={13} /> main</div>
          <div className="file-row muted"><ChevronRight size={11} /> agent/crates/core</div>
          <div className="file-row muted"><ChevronRight size={11} /> contracts/agent-sdk</div>
        </aside>
        <main className="console-main">
          <div className="console-heading"><div><span className="eyebrow">TURN 08</span><h3>Refactor context builder</h3></div><span className="live-badge"><span className="status-dot" /> RUNNING</span></div>
          <div className="message user-message"><span className="avatar user-avatar">you</span><p>Make context assembly easier to test, then show me the touched files.</p></div>
          <div className="message agent-message"><span className="avatar agent-avatar">sc</span><div><p>I&apos;ll inspect the current assembly path and its focused tests before changing anything.</p><div className="tool-line"><Check size={13} /> searched 14 files <span>· 1.2s</span></div></div></div>
          <div className="approval-row"><div className="approval-icon"><LockKeyhole size={14} /></div><div><strong>Approval needed</strong><p>Run focused Rust tests in <code>agent/crates/core</code></p></div><button type="button">Review scope <ChevronRight size={14} /></button></div>
          <div className="console-footer"><span><span className="status-dot" /> waiting for your decision</span><span className="mono">checkpoint: ready</span></div>
        </main>
      </div>
    </div>
  );
}

export default function Home() {
  return (
    <main>
      <header className="site-header">
        <a className="brand" href="#top" aria-label="SunCode home"><img src="/suncode-logo.svg" alt="" /><span>SunCode</span></a>
        <nav className="desktop-nav" aria-label="Primary navigation"><a href="#why">Why SunCode</a><a href="#capabilities">Capabilities</a><a href="#workflow">Workflow</a></nav>
        <a className="header-cta" href={githubUrl} target="_blank" rel="noreferrer">View on GitHub <ArrowUpRight size={15} /></a>
        <MobileNav />
      </header>

      <section className="hero section-shell" id="top">
        <div className="hero-copy">
          <p className="kicker"><span className="kicker-line" /> local-first coding agent</p>
          <h1>SunCode makes agentic coding <em>reviewable.</em></h1>
          <p className="hero-lede">Understand, change, review, and maintain software projects while every meaningful machine action stays visible, scoped, and reversible.</p>
          <div className="hero-actions"><a className="button button-primary" href={githubUrl} target="_blank" rel="noreferrer">Explore the source <ArrowUpRight size={16} /></a><a className="text-link" href="#workflow">See how it works <ChevronRight size={15} /></a></div>
          <div className="hero-note"><span className="status-dot" /> Rust core <span className="note-divider" /> .NET 10 desktop <span className="note-divider" /> open source</div>
        </div>
        <ProductConsole />
      </section>

      <section className="signal-strip" aria-label="Product principles"><div><span className="strip-value">01</span><span>one embedded agent core</span></div><div><span className="strip-value">02</span><span>approval-aware operations</span></div><div><span className="strip-value">03</span><span>turn-level filesystem undo</span></div></section>

      <section className="intro section-shell" id="why"><div className="section-marker">/ why suncode</div><div className="intro-content"><h2>A coding agent with a clear chain of custody.</h2><div className="intro-copy"><p>SunCode is built for developers who want the speed of delegation without handing their project to a black box.</p><p>The Rust agent owns the session, providers, policy, persistence, credentials, and machine operations. The desktop client gives you the surface to understand what happened and decide what happens next.</p></div></div></section>

      <section className="capabilities section-shell" id="capabilities"><div className="section-marker">/ what it does</div><div className="capability-list">{capabilities.map((item) => <article className="capability-row" key={item.label}><div className="capability-label">{item.icon}<span>{item.label}</span></div><h3>{item.title}</h3><p>{item.copy}</p><ChevronRight className="row-arrow" size={18} /></article>)}</div></section>

      <section className="workflow section-shell" id="workflow"><div className="section-marker">/ the working loop</div><div className="workflow-heading"><h2>From instruction to inspected change.</h2><p>The product stays close to the work: sessions, turns, activity, decisions, and recovery live in one local flow.</p></div><div className="workflow-list">{workflow.map(([number, title, copy]) => <div className="workflow-item" key={number}><span className="workflow-number">{number}</span><div><h3>{title}</h3><p>{copy}</p></div></div>)}</div></section>

      <section className="open-source section-shell"><div className="open-source-mark"><img src="/suncode-logo.svg" alt="" /></div><div><p className="kicker"><span className="kicker-line" /> build in the open</p><h2>See the system behind the surface.</h2><p>SunCode is being developed in public, with the agent core, contracts, clients, and design decisions in one repository.</p><a className="button button-primary" href={githubUrl} target="_blank" rel="noreferrer">Visit GitHub <ArrowUpRight size={16} /></a></div></section>

      <footer className="site-footer section-shell"><a className="brand" href="#top"><img src="/suncode-logo.svg" alt="" /><span>SunCode</span></a><p>Local-first coding work, with a trail you can trust.</p><div className="footer-links"><a href={githubUrl} target="_blank" rel="noreferrer">GitHub <ArrowUpRight size={14} /></a><a href="#top">Back to top <ChevronRight size={14} /></a></div></footer>
    </main>
  );
}
