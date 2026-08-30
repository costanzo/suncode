import { useState } from "react";
import { Button } from "../../components/universal/button/index.js";
import { ProjectCard } from "../../components/universal/card/index.js";
import { EmptyState } from "../../components/universal/feedback/index.js";
import { Icon } from "../../shared/Icon.jsx";
import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "./workspace/WorkspaceGuide.jsx";

const recentProjects = [
  { name: "suncode", path: "~/Projects/suncode" },
  { name: "Avalonia desktop", path: "~/Projects/suncode/apps/desktop-avalonia" },
];

const projectHubGuides = {
  recent: { tabs: {
    actions: ["Select a recent project card to reopen its local project window.", "Use Open project to choose a different local folder.", "Open Settings when defaults, providers, or logging need attention."],
    style: ["The operating system owns the title bar, window buttons, shadow, and outer resize behavior.", "The client toolbar is 62px high with 22px horizontal padding and an 18px brand title.", "Project cards are 70px minimum height with 14px horizontal padding and an 8px list gap."],
    logic: ["Recent projects are local paths that can be reopened without provisioning a remote project.", "Selecting a card transitions to the active desktop project window.", "The list is a convenience index; the opened folder remains the project boundary."],
  } },
  empty: { tabs: {
    actions: ["Use Open project to select the first local folder.", "After opening a folder, return to ProjectHub to see it in Recents.", "Use Settings before opening a project when provider setup is required."],
    style: ["The empty content area uses a 130px minimum height and 18px inset padding.", "Empty state copy is centered with a compact icon, 12px title, and 10px supporting text.", "The client area starts with the 62px toolbar beneath the operating system title bar."],
    logic: ["No local project has been opened in the current recent-project projection.", "Open project creates the first project entry and navigates to its workspace.", "An empty state does not imply an error or missing provider configuration."],
  } },
};

function ProjectHubWindow({ projects = [] }) {
  const hasProjects = projects.length > 0;

  return <div className="project-hub-frame">
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
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    { id: "recent", title: "With recent projects", description: "The hub lists local projects that can be reopened.", side: "right", projects: recentProjects },
    { id: "empty", title: "Without recent projects", description: "The first-run state appears automatically when no project has been opened.", side: "left", projects: [] },
  ];
  return (
    <>
      <PageHeader title="ProjectHub" description="The project landing surface from the Avalonia desktop client: reconnect to a recent project or open a local folder." path="projects/desktop/project-hub/" />
      {states.map((state) => <Section key={state.id} id={`project-hub-${state.id}`} title={state.id === "recent" ? "Recent project flow" : "First project flow"}><WorkspaceGuideState className="project-hub-guide-state" title={state.title} description={state.description} guide={projectHubGuides[state.id]} side={state.side} open={openGuide === state.id} onToggle={() => setOpenGuide(openGuide === state.id ? null : state.id)} onClose={() => setOpenGuide(null)}><ProjectHubWindow projects={state.projects} /></WorkspaceGuideState></Section>)}
    </>
  );
}
