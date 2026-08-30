import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function PlatformIndexesPage() {
  return (
    <>
      <PageHeader
        title="Platform indexes"
        description="Platform-only components are indexed by the client surface that owns their behavior."
        path="components/platform-specific/platform-indexes/"
        status="Boundary"
        tone="reserved"
      />
      <Section id="platforms" title="Platform indexes">
        <div className="layer-list">
          <ModuleLink
            to="/platforms/desktop"
            icon="platform"
            title="Desktop-only"
            description="Sidebar, review inspector, desktop chrome, dropdown menu, and dense data table."
            status="Review"
            tone="implemented"
          />
          <ModuleLink
            to="/platforms/mobile"
            icon="mobile"
            title="Mobile-only"
            description="Bottom navigation, swipe cell, and floating action patterns remain unapproved."
            status="Deferred"
            tone="deferred"
          />
          <ModuleLink
            to="/platforms/tui"
            icon="terminal"
            title="TUI-only"
            description="Command palette, status bar, and tree view remain unapproved."
            status="Deferred"
            tone="deferred"
          />
        </div>
      </Section>
    </>
  );
}
