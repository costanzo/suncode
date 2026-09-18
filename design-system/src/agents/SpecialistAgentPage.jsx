import { Icon } from "../shared/Icon.jsx";
import { PageHeader, Section } from "../shared/PagePrimitives.jsx";

export function SpecialistAgentPage({ agent }) {
  return (
    <>
      <PageHeader
        title={agent.displayName}
        description={agent.description}
        status="Built-in agent"
        tone="implemented"
      />
      <Section
        id="agent-definition"
        title="Definition"
        description="The stable identity and inherited execution policy compiled into the Rust agent."
      >
        <dl className="agent-definition-grid">
          <div>
            <dt>Unique name</dt>
            <dd><code>{agent.name}</code></dd>
          </div>
          <div>
            <dt>Stable ID</dt>
            <dd><code>{agent.id}</code></dd>
          </div>
          <div>
            <dt>Version</dt>
            <dd><code>{agent.version}</code></dd>
          </div>
          <div>
            <dt>Model</dt>
            <dd>{agent.modelPolicy}</dd>
          </div>
          <div>
            <dt>Tool-call limit</dt>
            <dd>{agent.toolLimit}</dd>
          </div>
        </dl>
      </Section>
      <Section
        id="agent-responsibilities"
        title="Role responsibilities"
        description="The specialist receives a bounded task from the main agent and stays within this role."
      >
        <div className="agent-responsibility-list">
          {agent.responsibilities.map((responsibility) => (
            <div key={responsibility}>
              <span><Icon name="check" size={14} /></span>
              <p>{responsibility}</p>
            </div>
          ))}
        </div>
      </Section>
      <Section
        id="agent-tools"
        title="Allowed tools"
        description="This allowlist is a capability ceiling; normal scope, policy, approval, audit, and checkpoint rules still apply."
      >
        <div className="agent-tool-list">
          {agent.tools.map((tool) => <code key={tool}>{tool}</code>)}
        </div>
      </Section>
      <Section
        id="agent-boundaries"
        title="Boundaries"
        description="Every child session remains linked to the primary session and cannot become a peer conversation."
      >
        <dl className="agent-boundary-list">
          <div><dt>MCP tools</dt><dd>{agent.mcpPolicy}</dd></div>
          <div><dt>Delegate again</dt><dd>{agent.delegation}</dd></div>
          <div><dt>Direct user chat</dt><dd>{agent.directChat}</dd></div>
        </dl>
      </Section>
    </>
  );
}
