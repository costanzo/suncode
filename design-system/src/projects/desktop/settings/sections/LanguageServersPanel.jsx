import { useEffect, useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";
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

function ServerSwitch({ server, onToggle, compact = false, disabled = false }) {
  return (
    <label className={`settings-switch ${compact ? "lsp-server-switch" : ""}`}>
      <input
        type="checkbox"
        checked={server.enabled}
        disabled={disabled || server.status === "indexing"}
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

function LanguageServerCatalog({ servers, onConfigure, onToggle, hasProject }) {
  return (
    <div className="settings-panel-content settings-lsp-content">
      <div className="settings-panel-heading">
        <h2>Language servers</h2>
        <p>
          Enable SunCode-supported language servers for the current project. Each project keeps its
          own process, index, and configuration.
        </p>
      </div>
      {hasProject ? (
        <div className="lsp-project-context">
          <Icon name="project" size={16} />
          <div>
            <span>Current project</span>
            <strong>suncode</strong>
            <code>~/Projects/suncode/suncode</code>
          </div>
        </div>
      ) : (
        <div className="lsp-project-context is-unscoped">
          <Icon name="project" size={16} />
          <div>
            <span>Project configuration unavailable</span>
            <strong>
              Open Settings from a project window to enable and configure language servers.
            </strong>
          </div>
        </div>
      )}
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
            <RuntimeStatus
              server={
                hasProject
                  ? server
                  : { ...server, enabled: false, status: "not_started", capabilityCount: 0 }
              }
            />
            <ServerSwitch
              server={hasProject ? server : { ...server, enabled: false, status: "not_started" }}
              compact
              disabled={!hasProject}
              onToggle={(enabled) => onToggle(server.id, enabled)}
            />
            <Button size="sm" disabled={!hasProject} onClick={() => onConfigure(server.id)}>
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
  const isDirty = JSON.stringify(draft) !== JSON.stringify(server);

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
          <SingleDropdown
            options={[
              { value: "managed", label: "Managed by SunCode" },
              { value: "custom", label: "Custom" },
            ]}
            value={draft.dataPathMode}
            onChange={(value) => update("dataPathMode", value)}
            ariaLabel="jdtls data directory mode"
            className="settings-dropdown"
            wide
          />
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
        <SettingRow
          label="Root markers"
          hint="One file or directory name per line. The first match identifies a Java project."
        >
          <textarea
            className="field mono lsp-multiline-field"
            value={draft.rootMarkers}
            onChange={(event) => update("rootMarkers", event.target.value)}
            spellCheck="false"
            aria-label="jdtls project root markers"
          />
        </SettingRow>
      </div>

      <details className="lsp-advanced-settings">
        <summary>Advanced configuration</summary>
        <div className="lsp-advanced-content">
          <SettingRow
            label="Environment"
            hint="One NAME=value entry per line. Values are stored locally and hidden after saving."
          >
            <textarea
              className="field mono lsp-multiline-field"
              value={draft.environment}
              onChange={(event) => update("environment", event.target.value)}
              placeholder="NAME=value"
              spellCheck="false"
              aria-label="jdtls environment variables"
            />
          </SettingRow>
          <SettingRow label="Startup timeout">
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
          </SettingRow>
          <SettingRow label="Request timeout">
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
          </SettingRow>
        </div>
      </details>

      <div className="lsp-detail-actions">
        <Button size="sm" disabled={!isDirty} onClick={() => setDraft(server)}>
          Reset changes
        </Button>
        <Button variant="primary" size="sm" disabled={!isDirty} onClick={() => onSave(draft)}>
          Save configuration
        </Button>
      </div>
    </div>
  );
}

function VtslsSettings({ server, onBack, onSave, onRetry }) {
  const [draft, setDraft] = useState(server);
  const [detecting, setDetecting] = useState(false);
  const update = (key, value) => setDraft((current) => ({ ...current, [key]: value }));
  const isDirty = JSON.stringify(draft) !== JSON.stringify(server);

  useEffect(() => setDraft(server), [server]);

  const detectInstallation = () => {
    setDetecting(true);
    window.setTimeout(() => {
      setDraft((current) => ({
        ...current,
        detected: true,
        executable: "/usr/local/bin/vtsls",
        nodePath: "/usr/local/bin/node",
        nodeVersion: "22.14.0",
        version: "0.3.0",
      }));
      setDetecting(false);
    }, 650);
  };

  const tsdkHint =
    draft.tsdkMode === "workspace-preferred"
      ? "Uses workspace TypeScript when available, then falls back to the version bundled with vtsls."
      : draft.tsdkMode === "bundled"
        ? "Always uses the TypeScript version bundled with vtsls."
        : "Uses a specific TypeScript lib directory selected for this project.";

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
          <p>Project configuration for JavaScript and TypeScript semantic analysis.</p>
        </div>
        <RuntimeStatus server={server} />
      </div>

      {server.status === "failed" && (
        <div className="lsp-error-banner" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>vtsls could not start</strong>
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
          hint="Starts one vtsls process for JavaScript and TypeScript files in this project."
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
          label="vtsls executable"
          hint="The @vtsls/language-server executable SunCode starts with --stdio."
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.executable}
              onChange={(event) => update("executable", event.target.value)}
              spellCheck="false"
              aria-label="vtsls executable path"
            />
            <button type="button" aria-label="Choose vtsls executable" title="Choose executable">
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
        <SettingRow
          label="Node.js runtime"
          hint={`Node 18 or newer. Detected ${draft.nodeVersion || "runtime version unavailable"}.`}
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.nodePath}
              onChange={(event) => update("nodePath", event.target.value)}
              spellCheck="false"
              aria-label="Node.js runtime path"
            />
            <button
              type="button"
              aria-label="Choose Node.js runtime"
              title="Choose Node.js runtime"
            >
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Project matching</span>
        <SettingRow
          label="Root markers"
          hint="One file name per line. The first match identifies a JavaScript or TypeScript project."
        >
          <textarea
            className="field mono lsp-multiline-field"
            value={draft.rootMarkers}
            onChange={(event) => update("rootMarkers", event.target.value)}
            spellCheck="false"
            aria-label="vtsls project root markers"
          />
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">TypeScript service</span>
        <SettingRow label="TypeScript SDK" hint={tsdkHint}>
          <SingleDropdown
            options={[
              { value: "workspace-preferred", label: "Workspace preferred" },
              { value: "bundled", label: "Bundled with vtsls" },
              { value: "custom", label: "Custom SDK directory" },
            ]}
            value={draft.tsdkMode}
            onChange={(value) => update("tsdkMode", value)}
            ariaLabel="TypeScript SDK policy"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        {draft.tsdkMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom TypeScript SDK</span>
            <SettingRow
              label="TypeScript lib directory"
              hint="Choose the directory containing tsserverlibrary.js."
            >
              <div className="lsp-path-control">
                <input
                  className="field mono"
                  value={draft.customTsdk}
                  onChange={(event) => update("customTsdk", event.target.value)}
                  placeholder="/usr/local/lib/node_modules/typescript/lib"
                  spellCheck="false"
                  aria-label="Custom TypeScript SDK directory"
                />
                <button
                  type="button"
                  aria-label="Choose TypeScript SDK directory"
                  title="Choose directory"
                >
                  <Icon name="folder" size={14} />
                </button>
              </div>
            </SettingRow>
          </div>
        )}
        <SettingRow
          label="TypeScript server memory"
          hint="Maximum memory for the tsserver process. Increase this for large monorepos."
        >
          <span className="settings-unit-field">
            <input
              className="field mono"
              type="number"
              min="512"
              max="16384"
              step="256"
              value={draft.maxTsServerMemory}
              onChange={(event) => update("maxTsServerMemory", event.target.value)}
            />
            <span>MB</span>
          </span>
        </SettingRow>
      </div>

      <details className="lsp-advanced-settings">
        <summary>Advanced configuration</summary>
        <div className="lsp-advanced-content">
          <SettingRow
            label="Environment"
            hint="One NAME=value entry per line. Values are stored locally and hidden after saving."
          >
            <textarea
              className="field mono lsp-multiline-field"
              value={draft.environment}
              onChange={(event) => update("environment", event.target.value)}
              placeholder="NAME=value"
              spellCheck="false"
              aria-label="vtsls environment variables"
            />
          </SettingRow>
          <SettingRow label="Startup timeout">
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
          </SettingRow>
          <SettingRow label="Request timeout">
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
          </SettingRow>
        </div>
      </details>

      <div className="lsp-detail-actions">
        <Button size="sm" disabled={!isDirty} onClick={() => setDraft(server)}>
          Reset changes
        </Button>
        <Button variant="primary" size="sm" disabled={!isDirty} onClick={() => onSave(draft)}>
          Save configuration
        </Button>
      </div>
    </div>
  );
}

export function LanguageServersPanel({
  servers,
  setServers,
  onSave,
  initialSelectedId = null,
  hasProject = true,
}) {
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
    const server = servers.find((candidate) => candidate.id === id);
    const label = server?.shortName ?? id;
    if (enabled) {
      startServer(id, `${label} enabled for suncode. Indexing the current project.`);
      return;
    }
    setServers((current) =>
      current.map((server) =>
        server.id === id
          ? { ...server, enabled: false, status: "disabled", capabilityCount: 0 }
          : server,
      ),
    );
    onSave(`${label} disabled for suncode. Its project process was stopped.`);
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
      onSave(`${draft.shortName} configuration saved as disabled for suncode.`);
      return;
    }
    onSave(`${draft.shortName} configuration saved. Restarting the project language server.`);
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
    const detailProps = {
      server: selectedServer,
      onBack: () => setSelectedId(null),
      onSave: saveConfiguration,
      onRetry: () =>
        startServer(selectedServer.id, `Retrying ${selectedServer.shortName} for suncode.`),
    };
    if (selectedServer.kind === "vtsls") return <VtslsSettings {...detailProps} />;
    return <JdtlsSettings {...detailProps} />;
  }

  return (
    <LanguageServerCatalog
      servers={servers}
      onConfigure={setSelectedId}
      onToggle={toggleServer}
      hasProject={hasProject}
    />
  );
}
