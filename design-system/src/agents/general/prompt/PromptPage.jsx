import { PageHeader, Section, Status } from "../../../shared/PagePrimitives.jsx";

const promptEntries = [
  {
    id: "system",
    title: "System prompt",
    scope: "Every model request",
    owner: "General agent",
    status: "Design draft",
    tone: "review",
    description:
      "The canonical behavioral contract for the General agent. It establishes the agent identity, response posture, authority boundaries, and the order in which it should approach coding work.",
    content: `You are SunCode, a general-purpose coding agent.

Help the user understand, change, review, and maintain the opened project.
Prefer clear, reviewable actions and keep the user informed about consequential work.
Use the available tools according to their declared scope and never imply authority
that has not been granted.`,
  },
  {
    id: "repository",
    title: "Repository instructions",
    scope: "Project and directory scope",
    owner: "Project AGENTS.md",
    status: "Implemented",
    tone: "implemented",
    description:
      "Project-owned instructions are injected as system context. More specific AGENTS.md files discovered while reading a file refine the guidance for that directory tree.",
    content: `Repository instructions from AGENTS.md (scope: the entire opened project):
<project>/AGENTS.md

More specific AGENTS.md files reported by the read tool override conflicting
broader instructions for files in their directory tree.`,
  },
  {
    id: "host",
    title: "Host environment",
    scope: "Every model request",
    owner: "Agent runtime",
    status: "Implemented",
    tone: "implemented",
    description:
      "The runtime gives the model stable execution context so commands and paths are expressed for the actual host rather than an imagined environment.",
    content: `SunCode host environment:
OS=<host OS>, architecture=<host architecture>,
shell tool dialect=<shell>, path style=<path style>,
session started at=<timestamp>.

Use the bash tool for terminal commands and use glob, grep, and read for
file discovery and content search.`,
  },
  {
    id: "tool-guidance",
    title: "Tool guidance",
    scope: "Tool selection and arguments",
    owner: "General agent",
    status: "Review",
    tone: "review",
    description:
      "Short guidance attached to the agent's tool contract. It keeps tool selection predictable, preserves the audited operation path, and explains when a failed call may be corrected within the same turn.",
    content: `Choose the narrowest tool that matches the task.
Read and inspect before mutating.
Keep project paths relative and respect the tool's declared scope.
When an argument is invalid but recoverable, correct it and try again.
Do not bypass approval or policy controls.`,
  },
  {
    id: "question",
    title: "Question prompt",
    scope: "Interactive clarification",
    owner: "Question tool",
    status: "Implemented",
    tone: "implemented",
    description:
      "Structured questions pause a turn when the agent needs a user decision. The prompt carries a concise header, one question, and ordered options without pretending the agent can decide on the user's behalf.",
    content: `Question:
<one decision the user can answer>

Options:
- <clear option> - <short consequence>
- <clear option> - <short consequence>

The turn remains paused until the user replies or rejects the question.`,
  },
  {
    id: "context",
    title: "Context summary",
    scope: "Context compaction",
    owner: "Context builder",
    status: "Implemented",
    tone: "implemented",
    description:
      "When the conversation exceeds its configured working budget, the retained context begins with a structured system summary so the agent can continue without silently losing the objective or active constraints.",
    content: `{
  "type": "suncode_context_summary",
  "objective": "...",
  "important_constraints": [],
  "completed_work": [],
  "active_work": [],
  "blockers": [],
  "next_action": "..."
}`,
  },
  {
    id: "recovery",
    title: "Recovery and continuation",
    scope: "Approval or question resume",
    owner: "Turn continuation",
    status: "Implemented",
    tone: "implemented",
    description:
      "When a turn pauses for approval or a question, continuation resumes the same turn with the user's decision and remaining calls instead of starting a new conversation branch.",
    content: `Continuation:
Preserve the original session and turn identity.
Apply the user's approval, answer, or rejection exactly once.
Resume the remaining tool calls with the existing context.
Record the outcome before requesting the next model response.`,
  },
];

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

export function PromptPage() {
  return (
    <>
      <PageHeader
        title="Prompt"
        description="The General agent's prompt catalog: each entry records where guidance enters the model context, who owns it, and how it should evolve."
        status="Prompt catalog"
        tone="review"
      />
      <Section
        id="prompt-principles"
        title="Prompt contract"
        description="Prompt content is treated as an inspectable product surface, with explicit scope and ownership."
      >
        <div className="agent-prompt-principles">
          <div>
            <strong>Layered composition</strong>
            <span>
              Keep global behavior, project guidance, runtime context, and turn-specific prompts
              distinguishable.
            </span>
          </div>
          <div>
            <strong>Reviewable authority</strong>
            <span>
              Prompts must not imply permissions that are absent from policy, approval, or tool
              scope.
            </span>
          </div>
          <div>
            <strong>Bounded context</strong>
            <span>
              Every injected instruction has a defined source, size boundary, and replacement or
              compaction rule.
            </span>
          </div>
        </div>
      </Section>
      <Section
        id="prompt-catalog"
        title="Prompt entries"
        description="Current and proposed prompt surfaces for the General agent."
      >
        <div className="agent-prompt-list">
          {promptEntries.map((entry) => (
            <PromptEntry key={entry.id} entry={entry} />
          ))}
        </div>
      </Section>
    </>
  );
}
