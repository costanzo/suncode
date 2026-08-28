import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const modules = [
  { path: "actions", title: "Actions", description: "Buttons, action hierarchy, and authority decisions." },
  { path: "fields", title: "Fields", description: "Text entry, selection fields, and validation." },
  { path: "selection", title: "Selection", description: "Checkboxes, radios, and toggles." },
  { path: "surfaces", title: "Surfaces", description: "Cards and authority-focused containers." },
  { path: "overlays", title: "Overlays", description: "Avatars, modals, and contextual tooltips." },
  { path: "navigation", title: "Navigation", description: "Tabs, segmented views, and local context." },
  { path: "feedback", title: "Feedback", description: "Status, alerts, progress, loading, and empty states." },
  { path: "data", title: "Data", description: "Code blocks, tables, and precise technical content." },
  { path: "markdown", title: "Markdown", description: "The complete assistant reading surface." },
];

export function UniversalComponentsPage() {
  return (
    <>
      <PageHeader title="Universal components" description="Cross-platform primitives are organized into independently addressable modules so each inventory can grow without turning this page into one continuous document." status="Module index" tone="implemented" showOwnership={false} />
      <Section id="module-index" title="Component modules">
        <div className="component-module-grid">
          {modules.map((module) => <ModuleLink key={module.path} to={`/components/universal/${module.path}`} title={module.title} description={module.description} />)}
        </div>
      </Section>
    </>
  );
}
