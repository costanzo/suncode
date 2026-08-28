import { PageHeader, Section } from "./PagePrimitives.jsx";

const colors = [
  ["Canvas", "var(--canvas)", "#f3f5f7 / #0d0f12"],
  ["Surface", "var(--surface)", "#ffffff / #121519"],
  ["Raised", "var(--surface-raised)", "#f8fafc / #181c21"],
  ["Active", "var(--surface-active)", "#e7edf1 / #242a31"],
  ["Text", "var(--text)", "#17202a / #edf0f3"],
  ["Secondary", "var(--text-secondary)", "#44515e / #a7afb9"],
  ["Muted", "var(--text-muted)", "#65717d / #7f8994"],
  ["Action", "var(--accent)", "#2c3742 / #d9e0e6"],
  ["Healthy", "var(--success)", "#4f6d82 / #9fb3c3"],
  ["Approval", "var(--warning)", "#966a25 / #d5ad6f"],
  ["Risk", "var(--danger)", "#b6463f / #e68a83"],
];

export function TokensPage() {
  return (
    <>
      <PageHeader title="Core tokens" description="The semantic foundation shared by every review route and mapped into the Avalonia runtime." path="core/tokens/" status="Source" tone="review" />
      <Section title="Color roles" description="The current theme changes values, never meaning.">
        <div className="token-swatches">
          {colors.map(([name, value, hex]) => (
            <div className="token-swatch" key={name}>
              <span style={{ background: value }} />
              <strong>{name}</strong>
              <code>{hex}</code>
            </div>
          ))}
        </div>
      </Section>
      <Section title="Typography" description="Product UI stays sans; machine-readable values alone use mono.">
        <div className="type-specimens">
          <div><code>Display · 34</code><span className="type-display">Reviewable authority</span></div>
          <div><code>Title · 22</code><span className="type-title">Current session</span></div>
          <div><code>Body · 14</code><span>Review the proposed changes before approval.</span></div>
          <div><code>Label · 12</code><span className="type-label">PROJECT SETTINGS</span></div>
          <div><code>Data · 12</code><span className="mono">src/agent/context.rs:148</span></div>
        </div>
      </Section>
      <Section title="Spacing, shape, and controls">
        <div className="measurement-grid">
          {[['space-1','4px'],['space-2','8px'],['space-4','16px'],['space-6','24px'],['radius-sm','6px'],['radius-md','10px'],['radius-lg','14px'],['control-md','36px']].map(([name, value]) => <div key={name}><code>{name}</code><strong>{value}</strong></div>)}
        </div>
      </Section>
    </>
  );
}
