import { useEffect, useRef, useState } from "react";
import { Icon } from "../../../core/react/Icon.jsx";
import { PageHeader, Section } from "../../../core/react/PagePrimitives.jsx";

const componentNav = [
  ["actions", "Actions"], ["fields", "Fields"], ["selection", "Selection"],
  ["surfaces", "Surfaces"], ["overlays", "Overlays"], ["navigation", "Navigation"], ["feedback", "Feedback"],
  ["data", "Data"], ["markdown", "Markdown"],
];

export function UniversalComponentsPage() {
  const [tab, setTab] = useState("Overview");
  const [segment, setSegment] = useState("Changes");
  const [enabled, setEnabled] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const dialogRef = useRef(null);

  useEffect(() => {
    if (!dialogOpen) return undefined;
    const priorFocus = document.activeElement;
    const dialog = dialogRef.current;
    const focusable = [...dialog.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')];
    focusable[0]?.focus();
    const handleDialogKey = (event) => {
      if (event.key === "Escape") setDialogOpen(false);
      if (event.key !== "Tab" || focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener("keydown", handleDialogKey);
    return () => { document.removeEventListener("keydown", handleDialogKey); priorFocus?.focus(); };
  }, [dialogOpen]);

  const handleTabKey = (event, index) => {
    const tabs = ["Overview", "Files", "Provider trace"];
    let next = index;
    if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
    else if (event.key === "ArrowLeft") next = (index - 1 + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else return;
    event.preventDefault();
    setTab(tabs[next]);
    document.getElementById(`component-tab-${next}`)?.focus();
  };

  return (
    <>
      <PageHeader title="Universal components" description="Cross-platform primitives with the same semantic inventory in light and dark themes. Interact with controls here before mapping them into a client." path="components/universal/" status="Complete inventory" tone="implemented" />
      <nav className="anchor-nav" aria-label="Component sections">
        {componentNav.map(([id, label]) => <button key={id} onClick={() => document.getElementById(id)?.scrollIntoView({ behavior: "smooth" })}>{label}</button>)}
      </nav>

      <Section id="actions" title="Buttons and actions" description="One primary action advances the turn; quiet and destructive actions stay subordinate.">
        <div className="specimen-grid specimen-grid-2">
          <Specimen label="Variants">
            <div className="row"><button className="btn btn-primary"><Icon name="arrow" />Send turn</button><button className="btn">Cancel</button><button className="btn btn-danger">Delete project</button><button className="btn btn-quiet">Learn more</button></div>
          </Specimen>
          <Specimen label="Size and availability">
            <div className="row"><button className="btn btn-primary btn-lg">Open project</button><button className="btn btn-sm">Compact</button><button className="btn btn-icon" aria-label="Settings"><Icon name="components" /></button><button className="btn btn-icon" disabled aria-label="Disabled"><Icon name="close" /></button></div>
          </Specimen>
          <Specimen label="State reference"><div className="state-strip"><div className="state-box">Rest</div><div className="state-box hover">Hover</div><div className="state-box active">Pressed</div><div className="state-box disabled">Disabled</div></div></Specimen>
          <Specimen label="Authority decision"><div className="split"><span className="type-label">Approve this operation?</span><div className="row"><button className="btn btn-sm">Deny</button><button className="btn btn-primary btn-sm">Approve once</button></div></div></Specimen>
        </div>
      </Section>

      <Section id="fields" title="Fields" description="Labels stay visible; validation explains how to recover without moving the page.">
        <div className="specimen-grid specimen-grid-3">
          <Specimen label="Text input"><div className="stack"><label><span className="field-label">Session name</span><input className="field" defaultValue="Refactor provider cache" /></label><label><span className="field-label">Project path</span><input className="field" placeholder="/Users/name/project" /></label></div></Specimen>
          <Specimen label="Select and textarea"><div className="stack"><label><span className="field-label">Model</span><select className="field" defaultValue="deepseek"><option value="deepseek">DeepSeek V4 Pro</option><option value="claude">Claude Sonnet 5</option></select></label><label><span className="field-label">Instruction</span><textarea className="field" placeholder="Describe the change…" /></label></div></Specimen>
          <Specimen label="Validation"><label><span className="field-label">API endpoint</span><input className="field field-error" defaultValue="https://" aria-invalid="true" aria-describedby="endpoint-error" /><span className="field-help error" id="endpoint-error">Enter a complete HTTPS endpoint.</span></label></Specimen>
        </div>
      </Section>

      <Section id="selection" title="Selection controls" description="Native inputs retain their familiar behavior and visible focus.">
        <div className="specimen-grid specimen-grid-3">
          <Specimen label="Checkbox"><div className="stack"><label className="checkbox"><input type="checkbox" defaultChecked /> Save as default</label><label className="checkbox"><input type="checkbox" /> Include diagnostics</label><label className="checkbox"><input type="checkbox" disabled /> Managed by policy</label></div></Specimen>
          <Specimen label="Radio"><div className="stack"><label className="radio"><input type="radio" name="scope" defaultChecked /> Current project</label><label className="radio"><input type="radio" name="scope" /> Current session</label></div></Specimen>
          <Specimen label="Toggle"><label className="toggle"><input type="checkbox" checked={enabled} onChange={(event) => setEnabled(event.target.checked)} /><span className="toggle-track" /> Provider {enabled ? "enabled" : "disabled"}</label></Specimen>
        </div>
      </Section>

      <Section id="surfaces" title="Cards and authority surfaces" description="Containers frame repeated content or important tools; they do not become the page scaffold.">
        <div className="specimen-grid specimen-grid-3">
          <div className="card project-card"><span className="project-mark">S</span><div><strong>suncode</strong><small>~/Projects/suncode</small></div><span className="badge success">Ready</span></div>
          <div className="card"><div className="activity"><Icon name="foundation" /><div><h3>Reading project files</h3><p>Scanning <span className="mono">agent/</span> for provider usage.</p></div><small>12s</small></div></div>
          <div className="card warning"><div className="approval"><span className="approval-mark">!</span><div><h3>Approval required</h3><p>Run <span className="mono">cargo test</span> in this project.</p></div><div className="approval-actions"><button className="btn btn-sm">Deny</button><button className="btn btn-primary btn-sm">Approve</button></div></div></div>
        </div>
      </Section>

      <Section id="overlays" title="Avatar, modal, and tooltip" description="Overlays appear only when focus or compact explanation genuinely requires them.">
        <div className="specimen-grid specimen-grid-3">
          <Specimen label="Avatar group"><div className="avatar-group" aria-label="Three collaborators"><span className="avatar avatar-silver">S</span><span className="avatar avatar-steel">R</span><span className="avatar avatar-neutral">A</span></div><p className="sample-note">Initials are a fallback when no approved image exists.</p></Specimen>
          <Specimen label="Focused dialog"><button className="btn btn-primary" onClick={() => setDialogOpen(true)}>Open undo dialog</button><p className="sample-note">Dialogs interrupt only protected, consequential decisions.</p></Specimen>
          <Specimen label="Tooltip"><div className="tooltip-demo"><button className="btn btn-icon" aria-label="Copy path"><Icon name="assets" /></button><span role="tooltip">Copy project path</span></div><p className="sample-note">Hover or focus the icon action.</p></Specimen>
        </div>
        {dialogOpen && <div className="dialog-backdrop" role="presentation" onMouseDown={() => setDialogOpen(false)}><div ref={dialogRef} className="review-dialog" role="dialog" aria-modal="true" aria-labelledby="undo-dialog-title" aria-describedby="undo-dialog-description" onMouseDown={(event) => event.stopPropagation()}><div className="dialog-title"><div><h3 id="undo-dialog-title">Undo this turn?</h3><p id="undo-dialog-description">Four filesystem changes will be restored from the checkpoint.</p></div><button className="btn btn-icon btn-quiet" onClick={() => setDialogOpen(false)} aria-label="Close dialog"><Icon name="close" /></button></div><div className="dialog-list"><span><code>agent.rs</code><strong>Modified</strong></span><span><code>App.axaml</code><strong>Modified</strong></span></div><div className="dialog-actions"><button className="btn" onClick={() => setDialogOpen(false)}>Cancel</button><button className="btn btn-danger" onClick={() => setDialogOpen(false)}>Undo changes</button></div></div></div>}
      </Section>

      <Section id="navigation" title="Navigation and filters" description="Active context is visible without turning every option into a filled pill.">
        <div className="specimen-grid specimen-grid-3">
          <Specimen label="Project bay"><div className="nav-demo"><div className="nav-item active"><Icon name="assets" />Explorer</div><div className="nav-item"><Icon name="components" />Sessions<small>3</small></div><div className="nav-item"><Icon name="project" />Changes<small>8</small></div></div></Specimen>
          <Specimen label="Tabs"><div className="tabs" role="tablist" aria-label="Project details">{["Overview", "Files", "Provider trace"].map((item, index) => <button id={`component-tab-${index}`} key={item} className={`tab ${tab === item ? "active" : ""}`} onClick={() => setTab(item)} onKeyDown={(event) => handleTabKey(event, index)} role="tab" aria-selected={tab === item} aria-controls="component-tabpanel" tabIndex={tab === item ? 0 : -1}>{item}</button>)}</div><div id="component-tabpanel" className="sample-note" role="tabpanel" aria-labelledby={`component-tab-${["Overview", "Files", "Provider trace"].indexOf(tab)}`}>Selected: {tab}</div></Specimen>
          <Specimen label="Segmented view"><div className="segmented">{["Changes", "All files", "History"].map((item) => <button key={item} className={`segment ${segment === item ? "active" : ""}`} onClick={() => setSegment(item)}>{item}</button>)}</div><p className="sample-note">Selected: {segment}</p></Specimen>
        </div>
      </Section>

      <Section id="feedback" title="Status, alerts, and progress" description="Semantic color reports health, authority, failure, or active work only.">
        <div className="specimen-grid specimen-grid-2">
          <Specimen label="Badges and alerts"><div className="row"><span className="badge accent">Running</span><span className="badge success">Connected</span><span className="badge warning">Waiting approval</span><span className="badge danger">Failed</span></div><div className="stack feedback-stack"><Alert tone="success" title="Checkpoint created">This turn can be undone from review.</Alert><Alert tone="warning" title="External side effect">Process output cannot be reverted.</Alert><Alert tone="danger" title="Provider unavailable">Check credentials or choose another model.</Alert></div></Specimen>
          <Specimen label="Loading and empty"><div className="split"><span className="type-label">Analyzing repository</span><span className="mono type-caption">62%</span></div><div className="progress"><span /></div><div className="skeleton" style={{ width: "84%" }} /><div className="skeleton short" /><div className="empty"><div><Icon name="assets" /><strong>No changes yet</strong><span>Files touched by this turn will appear here.</span></div></div></Specimen>
        </div>
      </Section>

      <Section id="data" title="Code and data" description="Monospace appears where character precision changes understanding.">
        <div className="specimen-grid specimen-grid-2">
          <div className="code"><span className="comment">// provider route</span><br /><span className="keyword">const</span> model = <span className="string">&quot;deepseek-v4-pro&quot;</span>;<br /><span className="keyword">const</span> path = <span className="string">&quot;agent/context.rs&quot;</span>;<br /><span className="keyword">await</span> runTests({`{ cwd: projectRoot }`});</div>
          <div className="table-wrap"><table className="data-table"><thead><tr><th>File</th><th>Change</th><th>Status</th></tr></thead><tbody><tr><td><strong className="mono">agent.rs</strong></td><td>+42 / -8</td><td><span className="badge success">Ready</span></td></tr><tr><td><strong className="mono">App.axaml</strong></td><td>+16 / -4</td><td><span className="badge accent">Review</span></td></tr><tr><td><strong className="mono">DESIGN.md</strong></td><td>+30 / -2</td><td><span className="badge warning">Pending</span></td></tr></tbody></table></div>
        </div>
      </Section>

      <Section id="markdown" title="Markdown reading surface" description="Assistant content keeps a readable measure and complete structural hierarchy.">
        <article className="sample markdown-sample"><div className="markdown"><h1>Provider migration review</h1><p>The Rust facade now owns <strong>session state</strong> and exposes a focused <a href="#/projects/avalonia-desktop">client API</a>. Keep <code>SQLite</code> behind the agent boundary; <em>presentation stays native</em> and <del>legacy production TypeScript</del> stays removed.</p><h2>Before merging</h2><ul><li>Run focused contract tests.</li><li>Confirm errors remain redacted.</li></ul><ol><li>Verify the Rust facade.</li><li>Verify the Avalonia binding.</li></ol><ul className="markdown-task-list"><li className="markdown-task"><input type="checkbox" checked disabled /> Provider tests pass</li><li className="markdown-task"><input type="checkbox" disabled /> Full migration complete</li></ul><blockquote><p>Approval is a real authority decision, not a generic confirmation.</p></blockquote><pre><code>cargo test -p suncode-core --lib</code></pre><hr /><div className="markdown-table-wrap"><table className="markdown-table"><thead><tr><th>Owner</th><th>Responsibility</th></tr></thead><tbody><tr><td>Rust</td><td>Policy, state, operations</td></tr><tr><td>Avalonia</td><td>Presentation, navigation</td></tr></tbody></table></div><p className="markdown-footnote">Review result: focused verification only.</p></div></article>
      </Section>
    </>
  );
}

function Specimen({ label, children }) {
  return <div className="sample"><div className="sample-label">{label}</div>{children}</div>;
}

function Alert({ tone, title, children }) {
  return <div className={`alert ${tone}`}><Icon name={tone === "danger" ? "close" : "check"} /><div><strong>{title}</strong><span>{children}</span></div></div>;
}
