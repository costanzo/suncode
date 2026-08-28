import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function DesktopProjectPage() {
  return <><PageHeader title="Desktop" description="The desktop project surface and its primary windows." status="Project index" tone="implemented" /><Section id="module-index" title="Desktop modules"><div className="module-card-grid"><ModuleLink to="/projects/desktop/project-hub" icon="project" title="ProjectHub" description="Welcome window for recent projects and opening a local folder." /><ModuleLink to="/projects/desktop/workspace" icon="workspace" title="Workspace" description="The active project window for sessions, conversation, review, and traceability." /><ModuleLink to="/projects/desktop/settings" icon="foundation" title="Settings" description="Configure defaults, appearance, network, logging, and provider credentials." /></div></Section></>;
}
