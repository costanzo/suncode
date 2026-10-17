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

function SettingsSwitch({ checked, onChange, label, disabled = false, compact = false }) {
  return (
    <label className={`settings-switch ${compact ? "lsp-server-switch" : ""}`}>
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
        aria-label={label}
      />
      <span className="settings-switch-track">
        <span />
      </span>
      <b>{checked ? "On" : "Off"}</b>
    </label>
  );
}

function ServerSwitch({ server, onToggle, compact = false, disabled = false }) {
  return (
    <SettingsSwitch
      checked={server.enabled}
      disabled={disabled || server.status === "indexing"}
      onChange={onToggle}
      label={`${server.enabled ? "Disable" : "Enable"} ${server.name} for this project`}
      compact={compact}
    />
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

function BasedpyrightSettings({ server, onBack, onSave, onRetry }) {
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
        executable: "/opt/homebrew/bin/basedpyright-langserver",
        pythonPath: "~/Projects/suncode/suncode/.venv/bin/python",
        pythonVersion: "3.12.7",
        version: "1.40.2",
      }));
      setDetecting(false);
    }, 650);
  };

  const diagnosticHint =
    draft.diagnosticMode === "workspace"
      ? "Analyzes every Python file selected by the project configuration."
      : "Analyzes files after SunCode opens them for semantic work.";
  const configPathHint =
    draft.configPathMode === "automatic"
      ? "Finds pyproject.toml or pyrightconfig.json from the project root."
      : "Uses a specific configuration file or directory for this project.";

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
          <p>Project configuration for Python semantic analysis.</p>
        </div>
        <RuntimeStatus server={server} />
      </div>

      {server.status === "failed" && (
        <div className="lsp-error-banner" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>basedpyright could not start</strong>
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
          hint="Starts one basedpyright process for Python files in this project."
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
          label="basedpyright executable"
          hint="The basedpyright-langserver executable SunCode starts with --stdio."
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.executable}
              onChange={(event) => update("executable", event.target.value)}
              spellCheck="false"
              aria-label="basedpyright executable path"
            />
            <button
              type="button"
              aria-label="Choose basedpyright executable"
              title="Choose executable"
            >
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Python environment</span>
        <SettingRow
          label="Python interpreter"
          hint={`Used to resolve installed packages and infer the target version. Detected Python ${draft.pythonVersion}.`}
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.pythonPath}
              onChange={(event) => update("pythonPath", event.target.value)}
              spellCheck="false"
              aria-label="Python interpreter path"
            />
            <button type="button" aria-label="Choose Python interpreter" title="Choose interpreter">
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
          hint="One file name per line. The first match identifies a Python project."
        >
          <textarea
            className="field mono lsp-multiline-field"
            value={draft.rootMarkers}
            onChange={(event) => update("rootMarkers", event.target.value)}
            spellCheck="false"
            aria-label="basedpyright project root markers"
          />
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Python analysis</span>
        <SettingRow label="Diagnostic scope" hint={diagnosticHint}>
          <SingleDropdown
            options={[
              { value: "openFilesOnly", label: "Open files only" },
              { value: "workspace", label: "Entire workspace" },
            ]}
            value={draft.diagnosticMode}
            onChange={(value) => update("diagnosticMode", value)}
            ariaLabel="basedpyright diagnostic scope"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        <SettingRow
          label="Type checking"
          hint="Project pyproject.toml or pyrightconfig.json settings take precedence."
        >
          <SingleDropdown
            options={[
              { value: "recommended", label: "Recommended" },
              { value: "standard", label: "Standard" },
              { value: "basic", label: "Basic" },
              { value: "strict", label: "Strict" },
              { value: "off", label: "Off" },
              { value: "all", label: "All rules" },
            ]}
            value={draft.typeCheckingMode}
            onChange={(value) => update("typeCheckingMode", value)}
            ariaLabel="basedpyright type checking mode"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        <SettingRow label="Configuration location" hint={configPathHint}>
          <SingleDropdown
            options={[
              { value: "automatic", label: "Automatic from project" },
              { value: "custom", label: "Custom location" },
            ]}
            value={draft.configPathMode}
            onChange={(value) => update("configPathMode", value)}
            ariaLabel="basedpyright configuration location"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        {draft.configPathMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom configuration</span>
            <SettingRow
              label="Configuration file or directory"
              hint="Choose a pyproject.toml, pyrightconfig.json, or the directory containing it."
            >
              <div className="lsp-path-control">
                <input
                  className="field mono"
                  value={draft.customConfigPath}
                  onChange={(event) => update("customConfigPath", event.target.value)}
                  placeholder="~/Projects/suncode/suncode/backend/pyproject.toml"
                  spellCheck="false"
                  aria-label="Custom basedpyright configuration location"
                />
                <button
                  type="button"
                  aria-label="Choose basedpyright configuration location"
                  title="Choose configuration"
                >
                  <Icon name="folder" size={14} />
                </button>
              </div>
            </SettingRow>
          </div>
        )}
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
              aria-label="basedpyright environment variables"
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

function RustAnalyzerSettings({ server, onBack, onSave, onRetry }) {
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
        executable: "/opt/homebrew/opt/rustup/bin/rust-analyzer",
        toolchainVersion: "Rust 1.95.0 · Cargo 1.95.0",
        version: "1.95.0",
      }));
      setDetecting(false);
    }, 650);
  };

  const featureHint =
    draft.featureMode === "all"
      ? "Activates every Cargo feature while rust-analyzer loads the project."
      : draft.featureMode === "custom"
        ? "Activates only the listed Cargo features in addition to default features."
        : "Uses the default Cargo feature set declared by the project.";
  const targetHint =
    draft.targetMode === "custom"
      ? "Analyzes the project for a specific Rust target triple."
      : "Uses the host target reported by the selected Rust toolchain.";

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
          <p>Project configuration for Rust semantic analysis.</p>
        </div>
        <RuntimeStatus server={server} />
      </div>

      {server.status === "failed" && (
        <div className="lsp-error-banner" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>rust-analyzer could not start</strong>
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
          hint="Starts one rust-analyzer process for Rust files in this project."
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
          label="rust-analyzer executable"
          hint="The rust-analyzer executable SunCode starts over standard input and output."
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.executable}
              onChange={(event) => update("executable", event.target.value)}
              spellCheck="false"
              aria-label="rust-analyzer executable path"
            />
            <button
              type="button"
              aria-label="Choose rust-analyzer executable"
              title="Choose executable"
            >
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
        <SettingRow
          label="Detected toolchain"
          hint="Read-only versions reported by the Rust toolchain available to this project."
        >
          <span className="lsp-toolchain-summary">
            <span className="settings-status-dot" />
            <code>{draft.toolchainVersion}</code>
          </span>
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Project matching</span>
        <SettingRow
          label="Root markers"
          hint="One file name per line. The first match identifies a Rust project."
        >
          <textarea
            className="field mono lsp-multiline-field"
            value={draft.rootMarkers}
            onChange={(event) => update("rootMarkers", event.target.value)}
            spellCheck="false"
            aria-label="rust-analyzer project root markers"
          />
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Cargo analysis</span>
        <SettingRow label="Cargo features" hint={featureHint}>
          <SingleDropdown
            options={[
              { value: "default", label: "Default features" },
              { value: "all", label: "All features" },
              { value: "custom", label: "Custom features" },
            ]}
            value={draft.featureMode}
            onChange={(value) => update("featureMode", value)}
            ariaLabel="rust-analyzer Cargo features"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        {draft.featureMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom features</span>
            <SettingRow
              label="Feature names"
              hint="One Cargo feature per line. Default features remain enabled."
            >
              <textarea
                className="field mono lsp-multiline-field"
                value={draft.customFeatures}
                onChange={(event) => update("customFeatures", event.target.value)}
                placeholder="feature-name"
                spellCheck="false"
                aria-label="Custom Cargo features"
              />
            </SettingRow>
          </div>
        )}
        <SettingRow
          label="All targets"
          hint="Loads library, binary, example, test, and benchmark targets from Cargo."
        >
          <SettingsSwitch
            checked={draft.allTargets}
            onChange={(value) => update("allTargets", value)}
            label="Analyze all Cargo targets"
          />
        </SettingRow>
        <SettingRow
          label="Build scripts"
          hint="Runs Cargo build scripts to discover generated code and compiler settings."
        >
          <SettingsSwitch
            checked={draft.buildScripts}
            onChange={(value) => update("buildScripts", value)}
            label="Run Cargo build scripts"
          />
        </SettingRow>
        <SettingRow
          label="Procedural macros"
          hint="Expands procedural macros for more accurate semantic results."
        >
          <SettingsSwitch
            checked={draft.procMacros}
            onChange={(value) => update("procMacros", value)}
            label="Enable procedural macro expansion"
          />
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Diagnostics</span>
        <SettingRow
          label="Check on save"
          hint="Runs the selected Cargo command after a Rust file is saved."
        >
          <SettingsSwitch
            checked={draft.checkOnSave}
            onChange={(value) => update("checkOnSave", value)}
            label="Run Cargo diagnostics on save"
          />
        </SettingRow>
        <SettingRow
          label="Check command"
          hint="Clippy adds lint diagnostics and may take longer than Cargo check."
        >
          <SingleDropdown
            options={[
              { value: "check", label: "Cargo check" },
              { value: "clippy", label: "Clippy" },
            ]}
            value={draft.checkCommand}
            onChange={(value) => update("checkCommand", value)}
            ariaLabel="rust-analyzer check command"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        <SettingRow label="Compilation target" hint={targetHint}>
          <SingleDropdown
            options={[
              { value: "host", label: "Host toolchain target" },
              { value: "custom", label: "Custom target" },
            ]}
            value={draft.targetMode}
            onChange={(value) => update("targetMode", value)}
            ariaLabel="rust-analyzer compilation target"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        {draft.targetMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom target</span>
            <SettingRow
              label="Target triple"
              hint="Use a target installed for the selected Rust toolchain."
            >
              <input
                className="field mono"
                value={draft.customTarget}
                onChange={(event) => update("customTarget", event.target.value)}
                placeholder="wasm32-unknown-unknown"
                spellCheck="false"
                aria-label="Custom Rust target triple"
              />
            </SettingRow>
          </div>
        )}
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
              aria-label="rust-analyzer environment variables"
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

function RoslynLanguageServerSettings({ server, onBack, onSave, onRetry }) {
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
        executable: "~/.dotnet/tools/roslyn-language-server",
        dotnetVersion: ".NET SDK 10.0.400 · Runtime 10.0.11",
        version: "5.13.0-1.26514.10",
      }));
      setDetecting(false);
    }, 650);
  };

  const projectLoadHint =
    draft.projectLoadMode === "custom"
      ? "Limits recursive project discovery when no single solution can be selected."
      : "Uses Roslyn's recommended limit of 500 discovered projects.";
  const solutionHint =
    draft.solutionMode === "custom"
      ? "Loads one solution relative to the project root."
      : "Loads a single root solution, or discovers C# projects recursively.";
  const sourceGeneratorHint =
    draft.sourceGeneratorMode === "automatic"
      ? "Runs source generators whenever Roslyn needs generated code for analysis."
      : "Balances generated-code freshness with project loading cost.";

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
          <p>Project configuration for C# semantic analysis.</p>
        </div>
        <RuntimeStatus server={server} />
      </div>

      {server.status === "failed" && (
        <div className="lsp-error-banner" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>Roslyn Language Server could not start</strong>
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
          hint="Starts one Roslyn language-server process for C# files in this project."
        >
          <ServerSwitch server={draft} onToggle={(enabled) => update("enabled", enabled)} />
        </SettingRow>
        <SettingRow label="Project detection" hint="Matched against the opened project tree.">
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
          label="roslyn-language-server executable"
          hint="The .NET tool executable SunCode starts with --stdio."
        >
          <div className="lsp-path-control">
            <input
              className="field mono"
              value={draft.executable}
              onChange={(event) => update("executable", event.target.value)}
              spellCheck="false"
              aria-label="roslyn-language-server executable path"
            />
            <button
              type="button"
              aria-label="Choose roslyn-language-server executable"
              title="Choose executable"
            >
              <Icon name="folder" size={14} />
            </button>
          </div>
        </SettingRow>
        <SettingRow
          label="Detected .NET"
          hint="Read-only SDK and runtime versions available from the project root."
        >
          <span className="lsp-toolchain-summary">
            <span className="settings-status-dot" />
            <code>{draft.dotnetVersion}</code>
          </span>
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Project matching</span>
        <SettingRow
          label="Root patterns"
          hint="One project-relative file pattern per line. The first match identifies a C# project."
        >
          <textarea
            className="field mono lsp-multiline-field"
            value={draft.rootMarkers}
            onChange={(event) => update("rootMarkers", event.target.value)}
            spellCheck="false"
            aria-label="Roslyn project root patterns"
          />
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Project loading</span>
        <SettingRow label="Project discovery limit" hint={projectLoadHint}>
          <SingleDropdown
            options={[
              { value: "recommended", label: "Recommended · 500 projects" },
              { value: "custom", label: "Custom limit" },
            ]}
            value={draft.projectLoadMode}
            onChange={(value) => update("projectLoadMode", value)}
            ariaLabel="Roslyn project discovery limit"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        {draft.projectLoadMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Discovery limit</span>
            <SettingRow
              label="Maximum projects"
              hint="Must be greater than zero. Test projects are dropped first when the limit is reached."
            >
              <span className="settings-unit-field">
                <input
                  className="field mono"
                  type="number"
                  min="1"
                  max="5000"
                  value={draft.maxProjects}
                  onChange={(event) => update("maxProjects", event.target.value)}
                  aria-label="Maximum Roslyn projects"
                />
                <span>PROJECTS</span>
              </span>
            </SettingRow>
          </div>
        )}
        <SettingRow label="Solution selection" hint={solutionHint}>
          <SingleDropdown
            options={[
              { value: "automatic", label: "Automatic from project" },
              { value: "custom", label: "Custom solution" },
            ]}
            value={draft.solutionMode}
            onChange={(value) => update("solutionMode", value)}
            ariaLabel="Roslyn solution selection"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        {draft.solutionMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom solution</span>
            <SettingRow
              label="Solution file"
              hint="Choose a .sln or .slnx file inside the current project."
            >
              <div className="lsp-path-control">
                <input
                  className="field mono"
                  value={draft.customSolution}
                  onChange={(event) => update("customSolution", event.target.value)}
                  placeholder="src/MyApp.slnx"
                  spellCheck="false"
                  aria-label="Custom Roslyn solution file"
                />
                <button
                  type="button"
                  aria-label="Choose Roslyn solution file"
                  title="Choose solution"
                >
                  <Icon name="folder" size={14} />
                </button>
              </div>
            </SettingRow>
          </div>
        )}
        <SettingRow
          label="Automatic restore"
          hint="Allows Roslyn to restore missing project dependencies while loading."
        >
          <SettingsSwitch
            checked={draft.automaticRestore}
            onChange={(value) => update("automaticRestore", value)}
            label="Automatically restore C# projects"
          />
        </SettingRow>
      </div>

      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Code analysis</span>
        <SettingRow label="Source generators" hint={sourceGeneratorHint}>
          <SingleDropdown
            options={[
              { value: "balanced", label: "Balanced" },
              { value: "automatic", label: "Automatic" },
            ]}
            value={draft.sourceGeneratorMode}
            onChange={(value) => update("sourceGeneratorMode", value)}
            ariaLabel="Roslyn source generator execution"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        <SettingRow
          label="Compiler diagnostics"
          hint="Controls how much of the loaded solution Roslyn checks in the background."
        >
          <SingleDropdown
            options={[
              { value: "openFiles", label: "Open files" },
              { value: "fullSolution", label: "Full solution" },
              { value: "none", label: "Off" },
            ]}
            value={draft.compilerDiagnosticScope}
            onChange={(value) => update("compilerDiagnosticScope", value)}
            ariaLabel="Roslyn compiler diagnostic scope"
            className="settings-dropdown"
            wide
          />
        </SettingRow>
        <SettingRow
          label="Analyzer diagnostics"
          hint="Controls background diagnostics from project analyzers and analyzer packages."
        >
          <SingleDropdown
            options={[
              { value: "openFiles", label: "Open files" },
              { value: "fullSolution", label: "Full solution" },
              { value: "none", label: "Off" },
            ]}
            value={draft.analyzerDiagnosticScope}
            onChange={(value) => update("analyzerDiagnosticScope", value)}
            ariaLabel="Roslyn analyzer diagnostic scope"
            className="settings-dropdown"
            wide
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
              aria-label="roslyn-language-server environment variables"
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

      <p className="lsp-detail-footnote">
        Project restore and source generators run with your user permissions. Their network,
        package-cache, and generated-code effects are outside filesystem undo.
      </p>

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
    if (selectedServer.kind === "basedpyright") {
      return <BasedpyrightSettings {...detailProps} />;
    }
    if (selectedServer.kind === "rust-analyzer") {
      return <RustAnalyzerSettings {...detailProps} />;
    }
    if (selectedServer.kind === "roslyn-language-server") {
      return <RoslynLanguageServerSettings {...detailProps} />;
    }
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
