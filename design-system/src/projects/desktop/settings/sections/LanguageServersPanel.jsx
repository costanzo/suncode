import { useEffect, useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";
import { initialLanguageServers } from "../data/languageServers.js";

export { initialLanguageServers };

const statusLabels = {
  ready: "Ready",
  indexing: "Indexing",
  failed: "Needs attention",
  disabled: "Disabled",
  not_started: "Not started",
};

function ServerSwitch({ server, onToggle, compact = false }) {
  return (
    <label className={`settings-switch ${compact ? "lsp-server-switch" : ""}`}>
      <input
        type="checkbox"
        checked={server.enabled}
        disabled={server.status === "indexing"}
        onChange={(event) => onToggle(event.target.checked)}
        aria-label={`${server.enabled ? "Disable" : "Enable"} ${server.name} for this project`}
      />
      <span className="settings-switch-track">
        <span />
      </span>
      <b>{server.enabled ? "On" : "Off"}</b>
    </label>
  );
}

function RuntimeStatus({ server }) {
  return (
    <div className={`lsp-runtime-status is-${server.status}`} aria-live="polite">
      <span className="settings-status-dot" />
      <div>
        <strong>{statusLabels[server.status]}</strong>
        <span>
          {server.status === "ready"
            ? `${server.capabilityCount} semantic capabilities`
            : server.status === "indexing"
              ? "Indexing project sources"
              : server.status === "failed"
                ? server.error
                : server.status === "not_started"
                  ? "Starts when this project opens"
                  : "Unavailable to new agent turns"}
        </span>
      </div>
    </div>
  );
}

function LanguageServerCatalog({ servers, onConfigure, onToggle }) {
  return (
    <div className="settings-panel-content settings-lsp-content">
      <div className="settings-panel-heading">
        <h2>Language servers</h2>
        <p>
          Enable SunCode-supported language servers for the current project. Each project keeps its
          own process, index, and configuration.
        </p>
      </div>
      <div className="lsp-project-context">
        <Icon name="project" size={16} />
        <div>
          <span>Current project</span>
          <strong>suncode</strong>
          <code>~/Projects/suncode/suncode</code>
        </div>
      </div>
      <div className="lsp-catalog-heading">
        <span>Available for this project</span>
        <small>{servers.length} supported</small>
      </div>
      <div className="lsp-catalog-list" aria-label="Supported language servers">
        {servers.map((server) => (
          <div className="lsp-catalog-row" key={server.id}>
            <div className="lsp-server-mark" aria-hidden="true">
              <Icon name="file-code" size={18} />
            </div>
            <div className="lsp-catalog-identity">
              <strong>{server.name}</strong>
              <span>{server.description}</span>
              <div className="lsp-server-meta">
                <code>{server.shortName}</code>
                <span>{server.language}</span>
                <span className={server.detected ? "is-detected" : ""}>
                  {server.detected ? `Detected · ${server.version}` : "Not detected"}
                </span>
              </div>
            </div>
            <RuntimeStatus server={server} />
            <ServerSwitch
              server={server}
              compact
              onToggle={(enabled) => onToggle(server.id, enabled)}
            />
            <Button size="sm" onClick={() => onConfigure(server.id)}>
              Configure
            </Button>
          </div>
        ))}
      </div>
      <p className="lsp-catalog-footnote">
        Language-server processes run locally with your user permissions. Their caches and toolchain
        effects are outside filesystem undo.
      </p>
    </div>
  );
}

function JdtlsSettings({ server, onBack, onSave, onRetry }) {
  const [draft, setDraft] = useState(server);
  const [detecting, setDetecting] = useState(false);
  const update = (key, value) => setDraft((current) => ({ ...current, [key]: value }));

  useEffect(() => setDraft(server), [server]);

  const detectInstallation = () => {
    setDetecting(true);
    window.setTimeout(() => {
      setDraft((current) => ({
        ...current,
        detected: true,
        executable: "/opt/homebrew/bin/jdtls",
        javaHome: "/Library/Java/JavaVirtualMachines/jdk-21.jdk/Contents/Home",
        version: "1.57.0",
      }));
      setDetecting(false);
    }, 650);
  };

  return (
    <div className="settings-panel-content settings-lsp-content lsp-detail-content">
      <button type="button" className="lsp-back-button" onClick={onBack}>
        <Icon name="chevron-right" size={13} />
        Language servers
      </button>
      <div className="lsp-detail-heading">
        <div className="lsp-server-mark" aria-hidden="true">
          <Icon name="file-code" size={20} />
        </div>
        <div>
          <h2>{server.name}</h2>
          <p>Project configuration for Java semantic analysis.</p>
        </div>
        <RuntimeStatus server={server} />
      </div>

      {server.status === "failed" && (
        <div className="lsp-error-banner" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>jdtls could not start</strong>
            <span>{server.error}</span>
          </div>
          <Button size="sm" onClick={onRetry} icon="refresh">
            Retry
          </Button>
        </div>
      )}

      <div className="settings-panel-section">
        <span className="settings-section-label">Project availability</span>
        <SettingRow
          label="Enable for suncode"
          hint="Starts one jdtls process and keeps a separate index for this project."
        >
          <ServerSwitch server={draft} onToggle={(enabled) => update("enabled", enabled)} />
        </SettingRow>
        <SettingRow label="Project detection" hint="Matched against the opened project root.">
          <span className="lsp-inline-status is-ready">
            <span className="settings-status-dot" />
            Matched <code>{server.matchedRootMarker}</code>
          </span>
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <div className="lsp-section-heading">
          <span className="settings-section-label">Installation</span>
          <Button size="sm" onClick={detectInstallation} disabled={detecting} icon="search">
            {detecting ? "Detecting…" : "Detect again"}
          </Button>
        </div>
        <SettingRow
          label="jdtls executable"
          hint="SunCode detected this executable on the current system."
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.executable}
              onChange={(event) => update("executable", event.target.value)}
              spellCheck="false"
              aria-label="jdtls executable path"
            />
            <button type="button" aria-label="Choose jdtls executable" title="Choose executable">
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
        <SettingRow label="Java runtime" hint="Used as JAVA_HOME when SunCode starts jdtls.">
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.javaHome}
              onChange={(event) => update("javaHome", event.target.value)}
              spellCheck="false"
              aria-label="Java runtime path"
            />
            <button type="button" aria-label="Choose Java runtime" title="Choose Java runtime">
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Project index</span>
        <SettingRow
          label="Data directory"
          hint="jdtls stores its workspace metadata here. The directory is unique to this project."
        >
          <div
            className="lsp-choice-control"
            role="radiogroup"
            aria-label="jdtls data directory mode"
          >
            <label>
              <input
                type="radio"
                name="jdtls-data-mode"
                checked={draft.dataPathMode === "managed"}
                onChange={() => update("dataPathMode", "managed")}
              />
              Managed by SunCode
            </label>
            <label>
              <input
                type="radio"
                name="jdtls-data-mode"
                checked={draft.dataPathMode === "custom"}
                onChange={() => update("dataPathMode", "custom")}
              />
              Custom
            </label>
          </div>
        </SettingRow>
        <SettingRow
          label={draft.dataPathMode === "managed" ? "Resolved data path" : "Custom data path"}
          hint={
            draft.dataPathMode === "managed"
              ? "Generated from the stable project identity."
              : "Use a directory that is not shared with another running jdtls instance."
          }
        >
          <div
            className={`lsp-path-control ${draft.dataPathMode === "managed" ? "is-readonly" : ""}`}
          >
            <input
              className="field mono"
              value={
                draft.dataPathMode === "managed" ? draft.managedDataPath : draft.customDataPath
              }
              readOnly={draft.dataPathMode === "managed"}
              onChange={(event) => update("customDataPath", event.target.value)}
              spellCheck="false"
              aria-label="jdtls data path"
            />
            {draft.dataPathMode === "custom" && (
              <button
                type="button"
                aria-label="Choose jdtls data directory"
                title="Choose directory"
              >
                <Icon name="folder" size={14} />
              </button>
            )}
          </div>
        </SettingRow>
        <label className="lsp-wide-field">
          <span>Root markers</span>
          <textarea
            className="field mono"
            value={draft.rootMarkers}
            onChange={(event) => update("rootMarkers", event.target.value)}
            spellCheck="false"
            aria-label="jdtls project root markers"
          />
          <small>
            One file or directory name per line. The first match identifies a Java project.
          </small>
        </label>
      </div>

      <details className="lsp-advanced-settings">
        <summary>Advanced configuration</summary>
        <div className="lsp-advanced-content">
          <label className="lsp-wide-field">
            <span>Environment</span>
            <textarea
              className="field mono"
              value={draft.environment}
              onChange={(event) => update("environment", event.target.value)}
              placeholder="NAME=value"
              spellCheck="false"
              aria-label="jdtls environment variables"
            />
            <small>
              One NAME=value entry per line. Values are stored locally and hidden after saving.
            </small>
          </label>
          <div className="lsp-timeout-grid">
            <label>
              <span>Startup timeout</span>
              <span className="settings-unit-field">
                <input
                  className="field mono"
                  type="number"
                  min="1"
                  max="120"
                  value={draft.startupTimeout}
                  onChange={(event) => update("startupTimeout", event.target.value)}
                />
                <span>SEC</span>
              </span>
            </label>
            <label>
              <span>Request timeout</span>
              <span className="settings-unit-field">
                <input
                  className="field mono"
                  type="number"
                  min="1"
                  max="600"
                  value={draft.requestTimeout}
                  onChange={(event) => update("requestTimeout", event.target.value)}
                />
                <span>SEC</span>
              </span>
            </label>
          </div>
        </div>
      </details>

      <div className="lsp-detail-actions">
        <Button size="sm" onClick={() => setDraft(server)}>
          Reset changes
        </Button>
        <Button variant="primary" size="sm" onClick={() => onSave(draft)}>
          Save configuration
        </Button>
      </div>
    </div>
  );
}

export function LanguageServersPanel({ servers, setServers, onSave, initialSelectedId = null }) {
  const [selectedId, setSelectedId] = useState(initialSelectedId);
  const selectedServer = servers.find((server) => server.id === selectedId);

  const startServer = (id, message) => {
    setServers((current) =>
      current.map((server) =>
        server.id === id
          ? { ...server, enabled: true, status: "indexing", error: undefined }
          : server,
      ),
    );
    onSave(message);
    window.setTimeout(() => {
      setServers((current) =>
        current.map((server) =>
          server.id === id
            ? { ...server, status: "ready", capabilityCount: 5, error: undefined }
            : server,
        ),
      );
    }, 850);
  };

  const toggleServer = (id, enabled) => {
    if (enabled) {
      startServer(id, "jdtls enabled for suncode. Indexing the current project.");
      return;
    }
    setServers((current) =>
      current.map((server) =>
        server.id === id
          ? { ...server, enabled: false, status: "disabled", capabilityCount: 0 }
          : server,
      ),
    );
    onSave("jdtls disabled for suncode. Its project process was stopped.");
  };

  const saveConfiguration = (draft) => {
    setServers((current) =>
      current.map((server) =>
        server.id === draft.id
          ? {
              ...draft,
              status: draft.enabled ? "indexing" : "disabled",
              capabilityCount: 0,
            }
          : server,
      ),
    );
    if (!draft.enabled) {
      onSave("jdtls configuration saved as disabled for suncode.");
      return;
    }
    onSave("jdtls configuration saved. Restarting the project language server.");
    window.setTimeout(() => {
      setServers((current) =>
        current.map((server) =>
          server.id === draft.id
            ? { ...server, status: "ready", capabilityCount: 5, error: undefined }
            : server,
        ),
      );
    }, 850);
  };

  if (selectedServer) {
    return (
      <JdtlsSettings
        server={selectedServer}
        onBack={() => setSelectedId(null)}
        onSave={saveConfiguration}
        onRetry={() => startServer(selectedServer.id, "Retrying jdtls for suncode.")}
      />
    );
  }

  return (
    <LanguageServerCatalog servers={servers} onConfigure={setSelectedId} onToggle={toggleServer} />
  );
}
