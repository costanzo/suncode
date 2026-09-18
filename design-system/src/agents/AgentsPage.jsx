import { ModuleLink, PageHeader, Section } from "../shared/PagePrimitives.jsx";
import { builtInAgentCatalog } from "./catalog.js";

const agents = [
  {
    path: "/agents/general",
    icon: "agent",
    title: "General",
    description: "Shared agent behavior, orchestration rules, and prompt composition.",
    pathLabel: "agents/general/",
  },
  ...builtInAgentCatalog.map((agent) => ({
    path: `/agents/${agent.slug}`,
    icon: "agent",
    title: agent.displayName,
    description: agent.description,
    pathLabel: `agents/${agent.slug}/`,
  })),
];

export function AgentsPage() {
  return (
    <>
      <PageHeader
        title="Agents"
        description="A durable home for the behavior, instructions, and interaction contracts of every agent in SunCode."
        status="Module index"
        tone="implemented"
      />
      <Section
        id="agent-index"
        title="Agent modules"
        description="Each agent owns an independent review surface so the catalog can grow without mixing responsibilities."
      >
        <div className="module-card-grid">
          {agents.map((agent) => (
            <ModuleLink
              key={agent.path}
              to={agent.path}
              icon={agent.icon}
              title={agent.title}
              description={agent.description}
              path={agent.pathLabel}
              status="Active"
              tone="implemented"
            />
          ))}
        </div>
      </Section>
    </>
  );
}
