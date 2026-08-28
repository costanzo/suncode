import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function AvaloniaProjectPage() {
  return (
    <>
      <PageHeader title="Avalonia Desktop" description="The project mapping that connects approved design semantics to the only Phase 1 production client." status="Module index" tone="implemented" showOwnership={false} />
      <Section id="module-index" title="Avalonia modules">
        <div className="module-card-grid">
          <ModuleLink to="/projects/avalonia-desktop/design-to-runtime" title="Design-to-runtime" description="How reviewed semantics map into native production resources." />
          <ModuleLink to="/projects/avalonia-desktop/review-paths" title="Review paths" description="The design-system references used by the desktop client." />
          <ModuleLink to="/projects/avalonia-desktop/runtime-boundary" title="Runtime boundary" description="The ownership split between Avalonia, React review tooling, and Rust." />
        </div>
      </Section>
    </>
  );
}
