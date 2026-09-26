import { useState } from "react";
import { builtInAgentCatalog as agentCatalog } from "../../../../agents/catalog.js";
import { Button } from "../../../../components/universal/button/index.js";

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

export function ProvidersPanel({ onSelect, endpoints }) {
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Model providers</h2>
        <p>Built-in providers available to the local agent.</p>
      </div>
      <div className="settings-provider-overview">
        {Object.entries(providerCatalog).map(([id, provider]) => (
          <button key={id} type="button" onClick={() => onSelect(id)}>
            <span className={`settings-status-dot ${provider.keyPreview ? "is-configured" : ""}`} />
            <span>
              <strong>{provider.label}</strong>
              <code>{endpoints[id]}</code>
            </span>
            <small className={provider.keyPreview ? "is-configured" : ""}>
              {provider.keyPreview ? "Ready" : "API key needed"}
            </small>
            <Icon name="arrow" size={14} />
          </button>
        ))}
      </div>
    </div>
  );
}

function maskApiKey(value) {
  const key = value.trim();
  if (key.length <= 8) return `${key.slice(0, 2)}••••${key.slice(-2)}`;
  return `${key.slice(0, 4)}••••••••${key.slice(-4)}`;
}

export function ProviderPanel({ providerId, onSave, endpoint, onEndpointChange }) {
  const provider = providerCatalog[providerId];
  const [maskedKey, setMaskedKey] = useState(provider.keyPreview);
  const [apiKey, setApiKey] = useState("");
  const [draftEndpoint, setDraftEndpoint] = useState(endpoint);
  const configured = Boolean(maskedKey);
  const saveKey = () => {
    setMaskedKey(maskApiKey(apiKey));
    setApiKey("");
    onSave();
  };
  const removeKey = () => {
    setMaskedKey("");
    setApiKey("");
    onSave();
  };
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
      <div className="settings-panel-section">
        <span className="settings-section-label">Credential</span>
        <div className={`settings-credential-status ${configured ? "is-configured" : ""}`}>
          <span className="settings-status-dot" />
          <div>
            {configured ? (
              <>
                <strong>API key configured</strong>
                <code
                  aria-label={`API key starts with ${maskedKey.slice(0, 4)} and ends with ${maskedKey.slice(-4)}`}
                >
                  {maskedKey}
                </code>
              </>
            ) : (
              <>
                <strong>No API key configured</strong>
                <small>Add a key to enable this provider.</small>
              </>
            )}
          </div>
        </div>
        {!configured && (
          <div className="settings-provider-unconfigured" role="status">
            <div>
              <strong>Add an API key to use this provider</strong>
              <span>
                The models stay visible for selection, but sending is paused until this provider has
                a credential.
              </span>
            </div>
            <code>API key required</code>
          </div>
        )}
        <input
          className="field settings-key mono"
          type="password"
          value={apiKey}
          onChange={(event) => setApiKey(event.target.value)}
          placeholder={configured ? "Paste a new key to replace it" : provider.placeholder}
          aria-label={`${provider.label} API key`}
          autoComplete="new-password"
        />
        <div className="settings-actions">
          <Button variant="primary" size="sm" disabled={!apiKey.trim()} onClick={saveKey}>
            {configured ? "Replace key" : "Save key"}
          </Button>
          <Button variant="danger" size="sm" onClick={removeKey} disabled={!configured}>
            Remove key
          </Button>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Available models</span>
        <div className="settings-model-list">
          {provider.models.map((model) => (
            <div key={model} className={`settings-model-item ${configured ? "is-ready" : ""}`}>
              <code>{model}</code>
              <span>{configured ? "Ready to use" : "Add API key to use"}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
