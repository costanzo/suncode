import { Icon } from "../../../../shared/Icon.jsx";
import { navItems, providerCatalog } from "../data/catalog.js";

export function SettingsNav({
  page,
  setPage,
  agentsExpanded,
  setAgentsExpanded,
  providersExpanded,
  setProvidersExpanded,
  agentCatalog,
}) {
  return (
    <aside className="settings-nav">
      <nav aria-label="Settings sections">
        <div className="settings-nav-list">
          {navItems.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-current={page === item.id ? "page" : undefined}
              className={`settings-nav-item ${page === item.id ? "is-selected" : ""}`}
              onClick={() => setPage(item.id)}
            >
              <Icon name={item.icon} size={15} />
              <span>{item.label}</span>
            </button>
          ))}
          <div className="settings-nav-models">
            <button
              type="button"
              className={`settings-nav-item settings-nav-parent ${page === "agents" ? "is-selected" : ""}`}
              aria-current={page === "agents" ? "page" : undefined}
              aria-label={`${agentsExpanded ? "Collapse" : "Expand"} agents`}
              aria-expanded={agentsExpanded}
              aria-controls="settings-agent-list"
              onClick={() => {
                setPage("agents");
                setAgentsExpanded((expanded) => !expanded);
              }}
            >
              <Icon name="agent" size={15} />
              <span>Agents</span>
              <Icon
                name="chevron-right"
                size={13}
                className={`settings-nav-chevron ${agentsExpanded ? "is-rotated" : ""}`}
              />
            </button>
            {agentsExpanded && (
              <div className="settings-provider-list" id="settings-agent-list">
                {agentCatalog.map((agent) => (
                  <button
                    key={agent.id}
                    type="button"
                    aria-current={page === `agent:${agent.id}` ? "page" : undefined}
                    className={`settings-nav-item settings-nav-provider ${page === `agent:${agent.id}` ? "is-selected" : ""}`}
                    onClick={() => setPage(`agent:${agent.id}`)}
                  >
                    <span>{agent.displayName}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
          <div className="settings-nav-models">
            <button
              type="button"
              className={`settings-nav-item settings-nav-parent ${page === "providers" ? "is-selected" : ""}`}
              aria-current={page === "providers" ? "page" : undefined}
              aria-label={`${providersExpanded ? "Collapse" : "Expand"} model providers`}
              aria-expanded={providersExpanded}
              aria-controls="settings-provider-list"
              onClick={() => {
                setPage("providers");
                setProvidersExpanded((expanded) => !expanded);
              }}
            >
              <Icon name="components" size={15} />
              <span>Model providers</span>
              <Icon
                name="chevron-right"
                size={13}
                className={`settings-nav-chevron ${providersExpanded ? "is-rotated" : ""}`}
              />
            </button>
            {providersExpanded && (
              <div className="settings-provider-list" id="settings-provider-list">
                {Object.entries(providerCatalog).map(([id, provider]) => (
                  <button
                    key={id}
                    type="button"
                    aria-current={page === `provider:${id}` ? "page" : undefined}
                    className={`settings-nav-item settings-nav-provider ${page === `provider:${id}` ? "is-selected" : ""}`}
                    onClick={() => setPage(`provider:${id}`)}
                  >
                    <span>{provider.label}</span>
                    <span
                      className={`settings-nav-provider-status ${provider.keyPreview ? "is-configured" : ""}`}
                      aria-label={provider.keyPreview ? "API key configured" : "API key needed"}
                    />
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </nav>
    </aside>
  );
}
