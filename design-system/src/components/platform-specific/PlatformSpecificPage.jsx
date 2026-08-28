import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function PlatformSpecificPage() {
  return (
    <>
      <PageHeader title="Platform-specific components" description="Components that cannot remain universal are grouped behind explicit platform ownership boundaries." status="Module index" tone="reserved" />
      <Section id="module-index" title="Platform-specific modules">
        <div className="module-card-grid">
          <ModuleLink to="/components/platform-specific/platform-indexes" icon="platform" title="Platform indexes" description="Desktop, mobile, and terminal ownership boundaries." />
        </div>
      </Section>
    </>
  );
}
