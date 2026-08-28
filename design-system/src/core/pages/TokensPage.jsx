import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const modules = [
  { path: "/core/tokens/colors", title: "Colors", description: "Semantic roles and theme-aware values." },
  { path: "/core/tokens/typography", title: "Typography", description: "UI hierarchy and machine-readable type." },
  { path: "/core/tokens/spacing", title: "Spacing & shape", description: "Spacing rhythm, radii, and control dimensions." },
];

export function TokensPage() {
  return (
    <>
      <PageHeader title="Core tokens" description="The semantic foundation shared by every review route and mapped into the Avalonia runtime." status="Module index" tone="review" />
      <Section id="module-index" title="Token modules">
        <div className="module-card-grid">
          {modules.map((module) => <ModuleLink key={module.path} to={module.path} title={module.title} description={module.description} />)}
        </div>
      </Section>
    </>
  );
}
