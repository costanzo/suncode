import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function DesignToRuntimePage() {
  return <><PageHeader title="Design-to-runtime" description="How reviewed semantics move into the native Avalonia presentation runtime." path="projects/avalonia-desktop/design-to-runtime/" status="Production mapping" tone="implemented" /><Section id="mapping" title="Design-to-runtime path" description="React is the review surface; Avalonia remains the presentation runtime."><div className="mapping-flow"><div><code>src/styles/tokens/</code><strong>Semantic decisions</strong><span>Color, type, spacing, shape</span></div><span aria-hidden="true">→</span><div><code>src/components/ + src/platforms/desktop/</code><strong>Reviewed behavior</strong><span>States, anatomy, platform rules</span></div><span aria-hidden="true">→</span><div><code>apps/desktop-avalonia/</code><strong>Production resources</strong><span>AXAML + C# view models</span></div></div></Section></>;
}
