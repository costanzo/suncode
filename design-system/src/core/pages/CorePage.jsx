import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const modules = [
  { path: "/core/tokens", icon: "foundation", title: "Tokens", description: "Semantic color, typography, spacing, shape, and theme foundations." },
  { path: "/core/assets", icon: "assets", title: "Assets", description: "Approved brand marks, interface icons, and font guidance." },
];

export function CorePage() {
  return (
    <>
      <PageHeader title="Core" description="Foundational decisions shared by components, platform adaptations, and production mappings." status="Module index" tone="implemented" showOwnership={false} />
      <Section id="module-index" title="Core modules">
        <div className="module-card-grid">
          {modules.map((module) => <ModuleLink key={module.path} to={module.path} icon={module.icon} title={module.title} description={module.description} />)}
        </div>
      </Section>
    </>
  );
}
