import { FileTree, ModuleLink, PageHeader, Section, Status } from "../../shared/PagePrimitives.jsx";

const structure = `design-system/
├── index.html              # React entry
├── core/                   # tokens, assets, shell
├── components/             # shared + platform-only
├── platforms/              # adaptation layers
└── projects/               # product mappings`;

export function OverviewPage() {
  return (
    <>
      <PageHeader
        title="One entry. Every design decision."
        description="Browse SunCode foundations, reusable components, platform adaptations, and production mappings without leaving the catalog. The structure stays layered; the experience is now continuous."
        status="Review tool"
        tone="review"
      />

      <section className="start-panel">
        <div className="start-copy">
          <h2>Start with what you need to decide</h2>
          <p>Inspect semantic foundations before introducing a component. Review universal behavior before adding a platform override. Use project mappings to confirm what ships.</p>
        </div>
        <div className="start-actions">
          <ModuleLink to="/components/universal" icon="components" title="Review components" description="Complete shared state inventory" status="Complete" tone="implemented" />
          <ModuleLink to="/core/tokens" icon="foundation" title="Inspect tokens" description="Color, typography, spacing, shape" status="Source" tone="review" />
        </div>
      </section>

      <Section title="Four layers, one browsing model" description="Each module renders from the directory that owns it.">
        <div className="layer-list">
          <ModuleLink to="/core" icon="foundation" title="Core" description="Semantic color, typography, spacing, icons, logos, and fonts." path="core/" status="Source" tone="review" />
          <ModuleLink to="/components/universal" icon="components" title="Components" description="Cross-platform primitives plus explicit platform-only boundaries." path="components/" status="Review" tone="review" />
          <ModuleLink to="/platforms" icon="platform" title="Platforms" description="Desktop adaptations now; mobile and TUI boundaries remain deferred." path="platforms/" status="Mixed" tone="reserved" />
          <ModuleLink to="/projects" icon="project" title="Projects" description="Map approved design semantics into the Phase 1 Avalonia client." path="projects/" status="Mapped" tone="implemented" />
        </div>
      </Section>

      <Section title="Repository shape" description="Build tooling is local to this review surface and does not change production topology.">
        <div className="overview-split">
          <FileTree>{structure}</FileTree>
          <div className="principles-list">
            <div><Status tone="implemented">01</Status><p><strong>Conversation first</strong><br />Supporting UI yields before the primary work surface does.</p></div>
            <div><Status tone="review">02</Status><p><strong>Status lamp rule</strong><br />Color explains action, state, or change—never decoration.</p></div>
            <div><Status tone="reserved">03</Status><p><strong>Native mapping</strong><br />Review semantics here; ship them through Avalonia resources.</p></div>
          </div>
        </div>
      </Section>
    </>
  );
}
