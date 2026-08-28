import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function AvaloniaProjectPage() {
  return (
    <>
      <PageHeader title="Avalonia Desktop" description="The project mapping that connects approved design semantics to the only Phase 1 production client." path="projects/avalonia-desktop/" status="Production mapping" tone="implemented" />
      <Section id="runtime-path" title="Design-to-runtime path" description="React is the review surface; Avalonia remains the presentation runtime.">
        <div className="mapping-flow">
          <div><code>src/styles/tokens/</code><strong>Semantic decisions</strong><span>Color, type, spacing, shape</span></div><span aria-hidden="true">→</span><div><code>src/components/ + src/platforms/desktop/</code><strong>Reviewed behavior</strong><span>States, anatomy, platform rules</span></div><span aria-hidden="true">→</span><div><code>apps/desktop-avalonia/</code><strong>Production resources</strong><span>AXAML + C# view models</span></div>
        </div>
      </Section>
      <Section id="review-paths" title="Review paths">
        <div className="layer-list">
          <ModuleLink to="/components/universal" icon="components" title="Universal inventory" description="Confirm primitive semantics and states." status="Review" tone="review" />
          <ModuleLink to="/platforms/desktop" icon="platform" title="Desktop adaptation" description="Confirm shell anatomy and desktop-only behavior." status="Phase 1" tone="implemented" />
          <ModuleLink to="/core/tokens" icon="foundation" title="Token mapping" description="Keep Avalonia resource names semantically aligned." status="Source" tone="review" />
        </div>
      </Section>
      <Section id="runtime-boundary" title="Runtime boundary">
        <div className="boundary-note"><strong>Production remains native.</strong><p>Avalonia owns presentation and navigation. The embedded Rust SDK owns providers, sessions, policy, persistence, credentials, operations, recovery, and undo. This React app ships no product client behavior.</p><code>apps/desktop-avalonia/App.axaml</code></div>
      </Section>
    </>
  );
}
