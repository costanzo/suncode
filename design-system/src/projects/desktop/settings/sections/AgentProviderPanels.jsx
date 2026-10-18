import { useState } from "react";
import { builtInAgentCatalog as agentCatalog } from "../../../../agents/catalog.js";
import { Button } from "../../../../components/universal/button/index.js";
import { ProviderCredential } from "./ProviderCredential.jsx";

import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";
import { providerCatalog } from "../data/catalog.js";

export function AgentsPanel({ onSelect }) {
  return (
    <div className="settings-panel-content settings-agents-content">
      <div className="settings-panel-heading">
        <h2>Agents</h2>
        <p>Built-in agents available for delegation from the main session.</p>
      </div>
      <div className="settings-agent-overview">
        {agentCatalog.map((agent) => (
          <button key={agent.id} type="button" onClick={() => onSelect(agent.id)}>
            <span className="settings-agent-overview-icon">
              <Icon name="agent" size={15} />
            </span>
            <span>
              <strong>{agent.displayName}</strong>
              <small>{agent.description}</small>
              <code>{agent.name}</code>
            </span>
            <Icon name="arrow" size={14} />
          </button>
        ))}
      </div>
    </div>
  );
}

export function AgentPanel({ agentId }) {
  const selected = agentCatalog.find((agent) => agent.id === agentId) ?? agentCatalog[0];
  return (
    <div className="settings-panel-content settings-agents-content">
      <div className="settings-heading-row">
        <div className="settings-panel-heading">
          <h2>{selected.displayName}</h2>
          <p>{selected.description}</p>
        </div>
        <span className="settings-read-only-badge">Read only</span>
      </div>
      <div className="settings-agent-heading">
        <span className="settings-agent-detail-icon">
          <Icon name="agent" size={19} />
        </span>
        <div>
          <strong>{selected.displayName}</strong>
          <code>{selected.name}</code>
        </div>
      </div>

      <dl className="settings-agent-identity">
        <div>
          <dt>Stable ID</dt>
          <dd>
            <code>{selected.id}</code>
          </dd>
        </div>
        <div>
          <dt>Version</dt>
          <dd>
            <code>{selected.version}</code>
          </dd>
        </div>
        <div>
          <dt>Model</dt>
          <dd>{selected.modelPolicy}</dd>
        </div>
        <div>
          <dt>Tool limit</dt>
          <dd>{selected.toolLimit}</dd>
        </div>
      </dl>

      <div className="settings-agent-section">
        <span className="settings-section-label">Allowed tools</span>
        <div className="settings-agent-tools">
          {selected.tools.map((tool) => (
            <code key={tool}>{tool}</code>
          ))}
        </div>
      </div>

      <div className="settings-agent-section">
        <span className="settings-section-label">Boundaries</span>
        <div className="settings-agent-boundaries">
          <div>
            <span>MCP tools</span>
            <strong>{selected.mcpPolicy}</strong>
          </div>
          <div>
            <span>Delegate again</span>
            <strong>{selected.delegation}</strong>
          </div>
          <div>
            <span>Direct user chat</span>
            <strong>Not allowed</strong>
          </div>
        </div>
      </div>
    </div>
  );
}

export function ProvidersPanel({ onSelect, endpoints, credentials }) {
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Model providers</h2>
        <p>Built-in providers available to the local agent.</p>
      </div>
      <div className="settings-provider-overview">
        {Object.entries(providerCatalog).map(([id, provider]) => (
          <button key={id} type="button" onClick={() => onSelect(id)}>
            <span
              className={`settings-status-dot ${credentials[id].mask ? "is-configured" : ""}`}
            />
            <span>
              <strong>{provider.label}</strong>
              <code>{endpoints[id]}</code>
            </span>
            <small className={credentials[id].mask ? "is-configured" : ""}>
              {credentials[id].mask
                ? credentials[id].mode === "keychain"
                  ? "System keychain"
                  : "Plaintext"
                : "API key needed"}
            </small>
            <Icon name="arrow" size={14} />
          </button>
        ))}
      </div>
    </div>
  );
}

export function ProviderPanel({
  providerId,
  onSave,
  endpoint,
  onEndpointChange,
  credential,
  onCredentialChange,
  platform = "macos",
  issue = "none",
  onRetry,
}) {
  const provider = providerCatalog[providerId];
  const [draftEndpoint, setDraftEndpoint] = useState(endpoint);
  const configured = Boolean(credential.mask);
  const usable =
    configured &&
    (credential.mode === "plaintext" ||
      !["locked", "denied", "unavailable", "missing"].includes(issue));
  const saveEndpoint = () => {
    const normalized = draftEndpoint.trim().replace(/\/+$/, "");
    onEndpointChange(normalized);
    setDraftEndpoint(normalized);
    onSave();
  };
  const resetEndpoint = () => {
    setDraftEndpoint(provider.endpoint);
    onEndpointChange(provider.endpoint);
    onSave();
  };
  const endpointIsDefault = endpoint === provider.endpoint && draftEndpoint === provider.endpoint;
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>{provider.label}</h2>
        <p>Configure the provider URL and credential used by the local agent.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Provider URL</span>
        <SettingRow label="API base URL" hint="OpenAI-compatible URL used for subsequent requests.">
          <input
            className="field settings-key mono"
            type="url"
            value={draftEndpoint}
            onChange={(event) => setDraftEndpoint(event.target.value)}
            aria-label={`${provider.label} provider URL`}
            spellCheck="false"
          />
        </SettingRow>
        <div className="settings-actions">
          <Button
            variant="primary"
            size="sm"
            disabled={!draftEndpoint.trim() || draftEndpoint.trim() === endpoint}
            onClick={saveEndpoint}
          >
            Save URL
          </Button>
          <Button size="sm" disabled={endpointIsDefault} onClick={resetEndpoint}>
            Reset default
          </Button>
        </div>
      </div>
      <div className="settings-divider" />
      <ProviderCredential
        provider={provider}
        credential={credential}
        onChange={onCredentialChange}
        platform={platform}
        issue={issue}
        onRetry={onRetry}
      />
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Available models</span>
        <div className="settings-model-list">
          {provider.models.map((model) => (
            <div key={model} className={`settings-model-item ${usable ? "is-ready" : ""}`}>
              <code>{model}</code>
              <span>
                {usable
                  ? "Ready to use"
                  : configured
                    ? "Key access required"
                    : "Add API key to use"}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
