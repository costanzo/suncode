import { ModuleLink, PageHeader, Section } from "../shared/PagePrimitives.jsx";

export function ProjectsPage() {
  return (
    <>
      <PageHeader
        title="Projects"
        description="Business-project mappings connect approved design decisions to real production clients."
        status="Module index"
        tone="implemented"
      />
      <Section id="module-index" title="Project modules">
        <div className="module-card-grid">
          <ModuleLink
            to="/projects/desktop"
            icon="project"
            title="Desktop"
            description="ProjectHub and the Avalonia desktop entry experience."
          />
          <ModuleLink
            to="/projects/mobile"
            icon="mobile"
            title="Mobile"
            description="CMP mobile client for cross-host sessions, secure pairing, and offline review."
            status="Review reference"
            tone="implemented"
          />
        </div>
      </Section>
    </>
  );
}
