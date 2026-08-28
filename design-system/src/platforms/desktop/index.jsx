import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function DesktopPlatformPage() {
  return (
    <>
      <PageHeader title="Desktop platform" description="The adaptation layer for the .NET 10 Avalonia client." status="Module index" tone="implemented" />
      <Section id="module-index" title="Desktop modules">
        <div className="module-card-grid">
          <ModuleLink to="/platforms/desktop/anatomy" title="Window anatomy" description="Conversation-first composition with bounded supporting bays." />
          <ModuleLink to="/platforms/desktop/ownership" title="Desktop ownership" description="Desktop-only components and responsibility boundaries." />
        </div>
      </Section>
    </>
  );
}
