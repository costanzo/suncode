import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function DesktopProjectPage() {
  return <><PageHeader title="Desktop" description="The Phase 1 desktop project surface and its entry experience." status="Project index" tone="implemented" showOwnership={false} /><Section id="module-index" title="Desktop modules"><div className="module-card-grid"><ModuleLink to="/projects/desktop/project-hub" icon="project" title="ProjectHub" description="Welcome window for recent projects and opening a local folder." /><ModuleLink to="/projects/desktop/settings" icon="foundation" title="Settings" description="Configure defaults, appearance, network, logging, and provider credentials." /></div></Section></>;
}
