import { ModuleLink, PageHeader, Section } from "../../../core/react/PagePrimitives.jsx";

export function PlatformSpecificPage() {
  return (
    <>
      <PageHeader title="Platform-specific components" description="An explicit index for components that cannot remain universal. A platform-only label is an ownership constraint, not a shortcut around reusable semantics." path="components/platform-specific/" status="Boundary" tone="reserved" />
      <Section title="Platform indexes" description="Only Desktop has an implemented Phase 1 product surface.">
        <div className="layer-list">
          <ModuleLink to="/platforms/desktop" icon="platform" title="Desktop-only" description="Sidebar, review inspector, desktop chrome, dropdown menu, and dense data table." path="components/platform-specific/desktop-only/" status="Review" tone="implemented" />
          <ModuleLink to="/platforms/mobile" icon="mobile" title="Mobile-only" description="Bottom navigation, swipe cell, and floating action patterns remain unapproved." path="components/platform-specific/mobile-only/" status="Deferred" tone="deferred" />
          <ModuleLink to="/platforms/tui" icon="terminal" title="TUI-only" description="Command palette, status bar, and tree view remain unapproved." path="components/platform-specific/tui-only/" status="Deferred" tone="deferred" />
        </div>
      </Section>
    </>
  );
}
