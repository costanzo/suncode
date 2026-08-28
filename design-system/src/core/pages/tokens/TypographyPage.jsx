import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";

export function TypographyPage() {
  return <><PageHeader title="Typography" description="A compact type system keeps product UI readable and technical values precise." path="core/tokens/typography/" status="Token source" tone="review" /><Section id="type-scale" title="Type hierarchy" description="Product UI stays sans; machine-readable values alone use mono."><div className="type-specimens"><div><code>Display · 34</code><span className="type-display">Reviewable authority</span></div><div><code>Title · 22</code><span className="type-title">Current session</span></div><div><code>Body · 14</code><span>Review the proposed changes before approval.</span></div><div><code>Label · 12</code><span className="type-label">PROJECT SETTINGS</span></div><div><code>Data · 12</code><span className="mono">src/agent/context.rs:148</span></div></div></Section></>;
}
