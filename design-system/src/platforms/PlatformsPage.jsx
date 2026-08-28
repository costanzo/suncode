import { ModuleLink, PageHeader, Section } from "../shared/PagePrimitives.jsx";

const modules = [
  { path: "/platforms/desktop", icon: "platform", title: "Desktop", description: "Avalonia window anatomy and desktop-only adaptation rules." },
  { path: "/platforms/mobile", icon: "mobile", title: "Mobile", description: "Reserved adaptation boundary for a future mobile client." },
  { path: "/platforms/tui", icon: "terminal", title: "TUI", description: "Reserved adaptation boundary for a future terminal client." },
  { path: "/platforms/web", icon: "platform", title: "Web", description: "Reserved adaptation boundary for a future browser client." },
];

export function PlatformsPage() {
  return (
    <>
      <PageHeader title="Platforms" description="Adaptation layers translate shared semantics into the constraints of each client surface." status="Module index" tone="implemented" />
      <Section id="module-index" title="Platform modules">
        <div className="module-card-grid">
          {modules.map((module) => <ModuleLink key={module.path} to={module.path} icon={module.icon} title={module.title} description={module.description} />)}
        </div>
      </Section>
    </>
  );
}
