import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function TuiPlatformPage() {
  return (
    <>
      <PageHeader
        title="TUI platform"
        description="A reserved adaptation layer for a future terminal client."
        status="Module index"
        tone="deferred"
      />
      <Section id="module-index" title="TUI modules">
        <div className="module-card-grid">
          <ModuleLink
            to="/platforms/tui/boundary"
            icon="terminal"
            title="Adaptation boundary"
            description="What is reserved without implying implementation."
          />
          <ModuleLink
            to="/platforms/tui/ownership"
            icon="terminal"
            title="Ownership contract"
            description="Where future terminal tokens and components belong."
          />
        </div>
      </Section>
    </>
  );
}
