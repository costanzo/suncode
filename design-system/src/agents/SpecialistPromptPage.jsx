import { PageHeader, Section, Status } from "../shared/PagePrimitives.jsx";

const sharedChildPrompt = {
  title: "Child-session guardrails",
  scope: "Every child model request",
  owner: "Agent runtime",
  status: "Implemented",
  tone: "implemented",
  description:
    "The runtime appends the same execution boundaries to every specialist prompt. These constraints explain why a child session is a bounded worker rather than a second user-facing agent.",
  content: `You cannot delegate, ask the user questions, or use MCP tools.
Stay within the advertised tool allowlist.
The parent session remains the conversation authority and receives your result.`,
};

function PromptEntry({ entry }) {
  return (
    <article className="agent-prompt-entry">
      <header className="agent-prompt-entry-header">
        <div className="agent-prompt-entry-title">
          <h3>{entry.title}</h3>
          <Status tone={entry.tone}>{entry.status}</Status>
        </div>
        <div className="agent-prompt-entry-meta">
          <span>{entry.scope}</span>
          <code>{entry.owner}</code>
        </div>
      </header>
      <p className="agent-prompt-entry-description">{entry.description}</p>
      <pre className="agent-prompt-code">
        <code>{entry.content}</code>
      </pre>
    </article>
  );
}

export function SpecialistPromptPage({ agent }) {
  const entries = [
    {
      title: "Role instructions",
      scope: agent.prompt.scope,
      owner: agent.prompt.owner,
      status: "Implemented",
      tone: "implemented",
      description: agent.prompt.description,
      content: agent.prompt.content,
    },
    sharedChildPrompt,
  ];

  return (
    <>
      <PageHeader
        title={`${agent.displayName} prompt`}
        description={`The prompt contract for ${agent.displayName}: one role-specific instruction layered into the shared child-session runtime.`}
        status="Prompt catalog"
        tone="implemented"
      />
      <Section
        id="specialist-prompt-principles"
        title="Prompt composition"
        description="Specialist prompts follow the same inspectable structure as the General agent while keeping role guidance separate from runtime boundaries."
      >
        <div className="agent-prompt-principles">
          <div>
            <strong>Role-specific</strong>
            <span>
              The instruction names one specialist responsibility and is selected by the stable
              agent ID.
            </span>
          </div>
          <div>
            <strong>Runtime-bounded</strong>
            <span>
              Shared child-session guardrails remain visible and cannot be widened by the role
              prompt.
            </span>
          </div>
          <div>
            <strong>Policy-aware</strong>
            <span>
              Prompt wording describes intent only; tools, approvals, scope, and audit rules still
              enforce authority.
            </span>
          </div>
        </div>
      </Section>
      <Section
        id="specialist-prompt-catalog"
        title="Prompt entries"
        description="The exact role instruction and shared runtime guardrails sent to this specialist."
      >
        <div className="agent-prompt-list">
          {entries.map((entry) => (
            <PromptEntry key={entry.title} entry={entry} />
          ))}
        </div>
      </Section>
    </>
  );
}
