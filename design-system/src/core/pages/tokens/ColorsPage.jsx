import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";

const colors = [["Canvas", "var(--canvas)", "#f3f5f7 / #0d0f12"], ["Surface", "var(--surface)", "#ffffff / #121519"], ["Raised", "var(--surface-raised)", "#f8fafc / #181c21"], ["Active", "var(--surface-active)", "#e7edf1 / #242a31"], ["Text", "var(--text)", "#17202a / #edf0f3"], ["Secondary", "var(--text-secondary)", "#44515e / #a7afb9"], ["Muted", "var(--text-muted)", "#65717d / #7f8994"], ["Action", "var(--accent)", "#2c3742 / #d9e0e6"], ["Healthy", "var(--success)", "#4f6d82 / #9fb3c3"], ["Approval", "var(--warning)", "#966a25 / #d5ad6f"], ["Risk", "var(--danger)", "#b6463f / #e68a83"]];

export function ColorsPage() {
  return <><PageHeader title="Colors" description="Semantic color roles preserve meaning across light and dark themes." path="core/tokens/colors/" status="Token source" tone="review" /><Section id="color-roles" title="Color roles" description="The current theme changes values, never meaning."><div className="token-swatches">{colors.map(([name, value, hex]) => <div className="token-swatch" key={name}><span style={{ background: value }} /><strong>{name}</strong><code>{hex}</code></div>)}</div></Section></>;
}
