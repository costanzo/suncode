import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const areas = [
  {
    path: "/agents/general/prompt",
    icon: "message",
    title: "Prompt",
    description:
      "System, repository, tool, question, and recovery prompts used by the General agent.",
    pathLabel: "agents/general/prompt/",
  },
];

export function GeneralAgentPage() {
  return (
    <>
      <PageHeader
        title="General"
        description="The default coding agent surface for shared behavior and the prompt contracts that shape each turn."
        status="Agent"
        tone="implemented"
      />
      <Section
        id="agent-areas"
        title="General agent areas"
        description="Prompt is the first dedicated area; additional behavior surfaces can be added here as the agent grows."
      >
        <div className="module-card-grid">
          {areas.map((area) => (
            <ModuleLink
              key={area.path}
              to={area.path}
              icon={area.icon}
              title={area.title}
              description={area.description}
              path={area.pathLabel}
              status="Review"
              tone="review"
            />
          ))}
        </div>
      </Section>
    </>
  );
}
