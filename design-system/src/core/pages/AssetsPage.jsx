import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const modules = [
  { path: "/core/assets/brand", title: "Brand marks", description: "Full and compact SunCode identity assets." },
  { path: "/core/assets/icons", title: "Interface icons", description: "The approved monochrome symbol inventory." },
  { path: "/core/assets/fonts", title: "Fonts", description: "UI and code typeface guidance and packaging rules." },
];

export function AssetsPage() {
  return (
    <>
      <PageHeader title="Core assets" description="Approved brand marks and interface symbols live in one stable catalog before runtime packaging copies them." status="Module index" tone="review" />
      <Section id="module-index" title="Asset modules">
        <div className="module-card-grid">
          {modules.map((module) => <ModuleLink key={module.path} to={module.path} title={module.title} description={module.description} />)}
        </div>
      </Section>
    </>
  );
}
