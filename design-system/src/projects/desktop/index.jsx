import { Button } from "../../components/universal/button/index.js";
import { ProjectCard } from "../../components/universal/card/index.js";
import { EmptyState } from "../../components/universal/feedback/index.js";
import { Icon } from "../../shared/Icon.jsx";
import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const recentProjects = [
  { name: "suncode", path: "~/Projects/suncode" },
  { name: "Avalonia desktop", path: "~/Projects/suncode/apps/desktop-avalonia" },
];

function ProjectHubWindow({ projects = [] }) {
  const hasProjects = projects.length > 0;

  return <div className="project-hub-frame">
    <div className="project-hub-titlebar">
      <div className="traffic-lights" aria-label="Window controls"><span className="traffic-light close" /><span className="traffic-light minimize" /><span className="traffic-light maximize" /></div>
      <strong>Welcome to SunCode</strong>
    </div>
    <div className="project-hub-toolbar">
      <div className="project-hub-brand"><span className="project-hub-logo"><Icon name="components" size={20} /></span><strong>SunCode</strong></div>
      <div className="project-hub-actions"><Button variant="quiet" onClick={() => { window.location.hash = "/projects/desktop/settings"; }}>Settings</Button><Button variant="primary" icon="plus">Open project</Button></div>
    </div>
    <div className="project-hub-content">
      <div className="project-hub-heading"><span className="type-label">RECENT PROJECTS</span></div>
      {hasProjects ? <div className="project-hub-list">{projects.map((project) => <ProjectCard key={project.path} {...project} />)}</div> : <EmptyState title="No projects yet" description="Open a local folder to create your first project window." />}
    </div>
  </div>;
}

export function ProjectHubPage() {
  return (
    <>
      <PageHeader title="ProjectHub" description="The project landing surface from the Avalonia desktop client: reconnect to a recent project or open a local folder." path="projects/desktop/project-hub/" status="Phase 1" tone="implemented" />
      <Section id="project-hub-recent" title="With recent projects" description="The hub lists local projects that can be reopened.">
        <ProjectHubWindow projects={recentProjects} />
      </Section>
      <Section id="project-hub-empty" title="Without recent projects" description="The first-run state appears automatically when no project has been opened.">
        <ProjectHubWindow />
      </Section>
    </>
  );
}
