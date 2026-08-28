import { Icon } from "../../core/react/Icon.jsx";
import { PageHeader, Section } from "../../core/react/PagePrimitives.jsx";

export function DesktopPlatformPage() {
  return (
    <>
      <PageHeader title="Desktop platform" description="The Phase 1 adaptation layer for the .NET 10 Avalonia client: conversation-first composition, native window authority, and independently collapsible side bays." path="platforms/desktop/" status="Phase 1" tone="implemented" />
      <Section title="Project window anatomy" description="The center remains fluid; supporting bays have bounded widths and can retreat.">
        <div className="desktop-window">
          <div className="desktop-titlebar"><div className="desktop-brand"><img src="./core/assets/logos/suncode-logo-small.svg" alt="" /><strong>SunCode</strong><span>suncode</span></div><div className="window-actions"><i /><i /><i /></div></div>
          <div className="desktop-layout">
            <aside className="desktop-nav"><span className="desktop-label">Project</span><strong>suncode</strong><nav><span className="active"><Icon name="components" />Current session</span><span><Icon name="assets" />Files</span><span><Icon name="project" />Changes</span></nav></aside>
            <main className="desktop-conversation"><header><strong>Provider migration</strong><span>gpt-5.6-sol</span></header><div className="desktop-message"><span className="avatar">S</span><div><strong>SunCode</strong><p>I’ll inspect the affected Rust boundary and keep the client contract unchanged.</p><div className="tool-row"><Icon name="foundation" /><span>Reading agent/crates/core</span><code>complete</code></div></div></div><div className="desktop-composer"><span>Ask SunCode to make a change…</span><button aria-label="Send"><Icon name="arrow" /></button></div></main>
            <aside className="desktop-review"><span className="desktop-label">Review</span><div className="approval-card"><strong>Approval required</strong><p>Run focused Rust tests in the current project.</p><div><button className="btn btn-sm">Deny</button><button className="btn btn-primary btn-sm">Approve</button></div></div><dl><div><dt>Files touched</dt><dd>4</dd></div><div><dt>Checkpoint</dt><dd>Ready</dd></div><div><dt>Agent</dt><dd>Healthy</dd></div></dl></aside>
          </div>
        </div>
      </Section>
      <Section title="Desktop-only ownership">
        <div className="measurement-grid"><div><code>sidebar/</code><strong>Navigation bay</strong></div><div><code>review-inspector/</code><strong>Authority + changes</strong></div><div><code>dropdown-menu/</code><strong>Native menu behavior</strong></div><div><code>data-table/</code><strong>Dense project data</strong></div></div>
      </Section>
    </>
  );
}
