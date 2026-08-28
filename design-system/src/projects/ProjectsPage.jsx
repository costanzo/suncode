import { ModuleLink, PageHeader, Section } from "../shared/PagePrimitives.jsx";

export function ProjectsPage() {
  return (
    <>
      <PageHeader title="Projects" description="Business-project mappings connect approved design decisions to real production clients." status="Module index" tone="implemented" showOwnership={false} />
      <Section id="module-index" title="Project modules">
        <div className="module-card-grid">
          <ModuleLink to="/projects/avalonia-desktop" icon="project" title="Avalonia Desktop" description="Map shared tokens, components, and desktop adaptations into the Phase 1 production client." />
        </div>
      </Section>
    </>
  );
}
