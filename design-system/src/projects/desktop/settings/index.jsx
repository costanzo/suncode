import { useEffect, useId, useRef, useState } from "react";
import { builtInAgentCatalog as agentCatalog } from "../../../agents/catalog.js";
import { Button } from "../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../components/universal/dropdown/index.js";
import { ConfirmationDialog } from "../../../components/universal/modal/index.js";
import { Icon } from "../../../shared/Icon.jsx";
import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { NativeWindowFrame } from "../../../platforms/desktop/components/titlebar/index.js";
import { WorkspaceGuideState } from "../workspace/WorkspaceGuide.jsx";
import { WindowSizeNote } from "../WindowSizeNote.jsx";

const providerCatalog = {
  deepseek: {
    label: "DeepSeek",
    endpoint: "https://api.deepseek.com",
    placeholder: "Paste DeepSeek API key",
    keyPreview: "sk-d••••••••7K2m",
    models: ["deepseek-v4-flash", "deepseek-v4-pro"],
  },
  zhipu: {
    label: "Zhipu GLM",
    endpoint: "https://open.bigmodel.cn/api/paas/v4",
    placeholder: "Paste Zhipu API key",
    // Keep one provider unconfigured in the review surface so the recovery path
    // is visible without needing a seeded secret.
    keyPreview: "",
    models: ["glm-5.2", "glm-5.3"],
  },
  openai: {
    label: "OpenAI",
    endpoint: "https://api.openai.com/v1",
    placeholder: "Paste OpenAI API key",
    keyPreview: "sk-p••••••••9Xc4",
    models: ["gpt-5.5", "gpt-5.6-sol"],
  },
  kimi: {
    label: "Kimi",
    endpoint: "https://api.moonshot.ai/v1",
    placeholder: "Paste Kimi API key",
    keyPreview: "sk-k••••••••3FvP",
    models: ["kimi-k2.7-code", "kimi-k3"],
  },
  claude: {
    label: "Claude",
    endpoint: "https://api.anthropic.com/v1",
    placeholder: "Paste Anthropic API key",
    keyPreview: "sk-a••••••••6NwQ",
    models: ["claude-sonnet-5", "claude-opus-5"],
  },
  gemini: {
    label: "Gemini",
    endpoint: "https://generativelanguage.googleapis.com/v1beta/openai",
    placeholder: "Paste Gemini API key",
    keyPreview: "AIza••••••••2Lm8",
    models: ["gemini-3.5", "gemini-3.6-flash"],
  },
};

const navItems = [
  { id: "defaults", label: "Defaults", icon: "foundation" },
  { id: "appearance", label: "Appearance", icon: "sun" },
  { id: "shortcuts", label: "Keyboard shortcuts", icon: "keyboard" },
  { id: "network", label: "Network", icon: "platform" },
  { id: "browser", label: "Browser use", icon: "tool" },
  { id: "mcp", label: "MCP servers", icon: "server" },
  { id: "lsp", label: "Language servers", icon: "file-code" },
  { id: "logging", label: "Logging", icon: "assets" },
];

const settingsGuide = {
  tabs: {
    actions: [
      "Choose Defaults, Appearance, Keyboard shortcuts, Network, Browser use, MCP servers, Language servers, Agents, or Logging from the left navigation.",
      "Use the chevrons beside Agents and Model providers to collapse or expand their fixed catalogs.",
      "Select an agent beneath Agents to inspect its immutable identity, tool allowlist, and authority boundaries.",
      "Select a provider to edit its OpenAI-compatible URL or credential.",
      "A provider is shown without a stored key so its recovery path and available models can be reviewed.",
      "Use Reset default to restore a provider's built-in URL.",
      "Use Network to choose no proxy, system proxy, or a custom proxy and to review certificate verification and trust states.",
      "Use Browser use to inspect the bundled runtime, enable the capability, and hand an active project browser between the agent and the user.",
      "Use the folder buttons in Logging to choose log and image storage directories.",
      "Use MCP servers to add, edit, delete, enable, disable, and retry local or remote servers.",
      "Use Language servers to configure project-scoped semantic analysis for diagnostics, definitions, references, hover details, and symbols.",
      "Edit a control and use its save action; use Done to return to ProjectHub.",
      "Keyboard shortcuts are shown as read-only key combinations; customization is reserved for a future release.",
      "Agents is a read-only catalog of the built-in identities and exact tool allowlists compiled into SunCode.",
    ],
    style: [
      "The operating system owns the title bar and window controls; the client toolbar is 58px high with 22px horizontal padding.",
      "The settings body uses a 238px navigation column and a content panel with 28px top / 32px side padding.",
      "Rows use 12px labels, 11px hints, 36px controls, 24px column gaps, and 16px section gaps.",
    ],
    logic: [
      "Settings are local to the embedded agent and are grouped by defaults, appearance, keyboard shortcuts, network, language tooling, logging, agents, and providers.",
      "Built-in agents cannot be created, edited, enabled, disabled, or deleted from Settings.",
      "Provider URL changes and default resets are persisted and applied to subsequent requests without changing credentials or models.",
      "Custom proxy controls are subordinate to the selected proxy mode, preserve write-only password state, and apply to every Rust-owned HTTP client.",
      "Certificate-source controls are subordinate to HTTPS verification and switch between system trust and custom certificate-file input.",
      "Provider credentials are masked; only the first and last four characters are shown for recognition.",
      "A provider without a key keeps its model catalog visible but pauses sending until the key is saved.",
      "Saving updates local configuration state and does not grant new machine authority.",
      "MCP configuration is persisted by the embedded Rust agent; the effective tool catalog is refreshed without starting a new session.",
      "Connected, connecting, failed, and disabled are runtime states. Enabled is the persisted desired state.",
      "MCP server definitions are global, while the status shown belongs to the current project connection.",
      "Language server definitions are global desired state, while indexing and readiness belong to the current project runtime.",
      "Language servers run as local processes with user authority. Their caches and toolchain side effects are not covered by SunCode undo.",
      "Browser use is globally enabled but lazily started per project. Browser profiles and remote site changes are outside filesystem undo.",
      "Node.js, Playwright, and Chromium paths are fixed read-only installation facts; Settings never selects an external runtime.",
      "Server-requested file edits and command execution are refused; the surface specifies semantic read capabilities only.",
      "Settings reopens to the last valid destination, including the selected provider and its expanded navigation state.",
    ],
  },
};

function SettingRow({ label, hint, children, className = "" }) {
  return (
    <div className={`settings-row ${className}`}>
      <div className="settings-row-copy">
        <strong>{label}</strong>
        {hint && <span>{hint}</span>}
      </div>
      <div className="settings-row-control">{children}</div>
    </div>
  );
}

const shortcutCatalog = [
  { action: "Open Settings", keys: ["⌘", ","], ariaLabel: "Command comma" },
  { action: "Toggle project navigation", keys: ["⌘", "1"], ariaLabel: "Command 1" },
  { action: "Toggle Git viewer", keys: ["⌘", "9"], ariaLabel: "Command 9" },
  { action: "Send message", keys: ["Enter"], ariaLabel: "Enter" },
  { action: "Submit session dialog", keys: ["Enter"], ariaLabel: "Enter" },
  { action: "Cancel current turn or close dialog", keys: ["Escape"], ariaLabel: "Escape" },
];

function SettingsNav({
  page,
  setPage,
  agentsExpanded,
  setAgentsExpanded,
  providersExpanded,
  setProvidersExpanded,
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

function DefaultsPanel({ onSave }) {
  const models = Object.values(providerCatalog).flatMap((provider) => provider.models);
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Defaults</h2>
        <p>Configure model and turn defaults.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Default model</span>
        <SettingRow label="Model" hint="Only models registered by the local agent appear here.">
          <SingleDropdown
            options={models}
            initialValue="deepseek-v4-flash"
            ariaLabel="Default model"
            className="settings-dropdown"
          />
        </SettingRow>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Turn execution</span>
        <SettingRow
          label="Tool-call limit"
          hint="Maximum number of tool calls allowed in one turn."
        >
          <input
            className="field settings-number"
            type="number"
            min="1"
            max="256"
            defaultValue="64"
            aria-label="Tool-call limit"
          />
        </SettingRow>
        <div className="settings-actions">
          <Button variant="primary" size="sm" onClick={onSave}>
            Save project limit
          </Button>
          <span className="settings-save-status" role="status">
            Project: suncode
          </span>
        </div>
      </div>
    </div>
  );
}

function AppearancePanel({ onSave }) {
  const [theme, setTheme] = useState(() => document.documentElement.dataset.theme || "light");
  const applyTheme = (nextTheme) => {
    setTheme(nextTheme);
    document.documentElement.dataset.theme = nextTheme;
    try {
      window.localStorage.setItem("suncode-design-theme", nextTheme);
    } catch {
      /* non-fatal */
    }
    onSave();
  };
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Appearance</h2>
        <p>Adjust how SunCode looks across every open window.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Theme</span>
        <SettingRow label="Color theme" hint="Changes apply immediately.">
          <SingleDropdown
            options={[
              { value: "dark", label: "Dark" },
              { value: "light", label: "Light" },
            ]}
            value={theme}
            onChange={applyTheme}
            ariaLabel="Color theme"
            className="settings-dropdown"
          />
        </SettingRow>
      </div>
    </div>
  );
}

function ShortcutsPanel() {
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <div className="settings-heading-row">
          <div>
            <h2>Keyboard shortcuts</h2>
            <p>View the shortcuts available throughout the desktop application.</p>
          </div>
          <span className="settings-read-only-badge">Read-only</span>
        </div>
      </div>
      <div className="settings-read-only-note" role="note">
        <Icon name="keyboard" size={16} />
        <span>
          Shortcut customization is not available yet. Editing will be added in a future release.
          The preview uses macOS notation; Windows and Linux use Ctrl where applicable.
        </span>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Available shortcuts</span>
        <div className="settings-shortcut-list" aria-label="Available keyboard shortcuts">
          {shortcutCatalog.map((shortcut) => (
            <div className="settings-shortcut-row" key={shortcut.action}>
              <span className="settings-shortcut-action">{shortcut.action}</span>
              <span className="settings-key-chord" aria-label={shortcut.ariaLabel}>
                {shortcut.keys.map((key) => (
                  <kbd key={key}>{key}</kbd>
                ))}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function NetworkPanel({ onSave }) {
  const [proxyMode, setProxyMode] = useState("custom");
  const [proxyUrl, setProxyUrl] = useState("http://proxy.company.test:8080");
  const [proxyUsername, setProxyUsername] = useState("developer");
  const [proxyPassword, setProxyPassword] = useState("");
  const [proxyPasswordStored, setProxyPasswordStored] = useState(true);
  const [proxyBypass, setProxyBypass] = useState("localhost\n.internal.company.test\n10.0.0.0/8");
  const [verify, setVerify] = useState(true);
  const [useSystemCertificates, setUseSystemCertificates] = useState(true);
  const [certificatePath, setCertificatePath] = useState(
    "/Users/shuyi/.config/suncode/certs/dev-ca.pem",
  );
  const certificateInputRef = useRef(null);
  const chooseCertificate = () => {
    certificateInputRef.current?.click();
  };
  const chooseFallbackCertificate = (event) => {
    const file = event.target.files?.[0];
    if (file) setCertificatePath(`/Users/shuyi/Downloads/${file.name}`);
    event.target.value = "";
  };
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Network</h2>
        <p>Configure how the embedded Rust agent reaches HTTP and HTTPS services.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Proxy</span>
        <SettingRow
          label="Proxy mode"
          hint="Applies to model providers, WebFetch, and remote HTTP MCP servers."
        >
          <SingleDropdown
            options={[
              { value: "no_proxy", label: "No proxy" },
              { value: "system", label: "System proxy" },
              { value: "custom", label: "Custom proxy" },
            ]}
            value={proxyMode}
            onChange={setProxyMode}
            ariaLabel="Proxy mode"
            className="settings-dropdown"
          />
        </SettingRow>
        {proxyMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom proxy</span>
            <SettingRow
              label="Proxy URL"
              hint="Use an HTTP or HTTPS URL without embedded credentials."
            >
              <input
                className="field mono"
                value={proxyUrl}
                onChange={(event) => setProxyUrl(event.target.value)}
                aria-label="Proxy URL"
                spellCheck="false"
              />
            </SettingRow>
            <SettingRow label="Username" hint="Optional Basic proxy authentication username.">
              <input
                className="field mono"
                value={proxyUsername}
                onChange={(event) => setProxyUsername(event.target.value)}
                aria-label="Proxy username"
                autoComplete="off"
              />
            </SettingRow>
            <SettingRow
              label="Password"
              hint={
                proxyPasswordStored
                  ? "Password stored. Leave empty to keep it or remove it explicitly."
                  : "Optional Basic proxy authentication password."
              }
            >
              <div className="settings-secret-field">
                <input
                  className="field mono"
                  type="password"
                  value={proxyPassword}
                  onChange={(event) => setProxyPassword(event.target.value)}
                  aria-label="Proxy password"
                  placeholder={proxyPasswordStored ? "Password stored" : "Optional"}
                  autoComplete="new-password"
                />
                {proxyPasswordStored && (
                  <button type="button" onClick={() => setProxyPasswordStored(false)}>
                    Remove
                  </button>
                )}
              </div>
            </SettingRow>
            <SettingRow
              label="Bypass rules"
              hint="One hostname, domain suffix, IP address, CIDR range, or * per line. Loopback is always direct."
            >
              <textarea
                className="field mono settings-proxy-bypass"
                value={proxyBypass}
                onChange={(event) => setProxyBypass(event.target.value)}
                aria-label="Proxy bypass rules"
                spellCheck="false"
              />
            </SettingRow>
          </div>
        )}
        <div className="settings-actions">
          <Button variant="primary" size="sm" onClick={() => onSave("Proxy settings applied.")}>
            Save proxy settings
          </Button>
          <span className="settings-save-status" role="status">
            {proxyMode === "no_proxy"
              ? "Direct connections"
              : proxyMode === "system"
                ? "System proxy"
                : "Custom proxy"}
          </span>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">HTTPS security</span>
        <SettingRow
          label="Verify server certificates"
          hint="Validate certificate chains and hostnames for model provider and WebFetch requests."
        >
          <label className="settings-switch">
            <input
              type="checkbox"
              checked={verify}
              onChange={(event) => setVerify(event.target.checked)}
              aria-label="Verify server certificates"
            />
            <span className="settings-switch-track">
              <span />
            </span>
            <b>{verify ? "On" : "Off"}</b>
          </label>
        </SettingRow>
        {verify && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Certificate trust source</span>
            <SettingRow
              label="Use system certificates"
              hint="Trust the operating system certificate store for provider and WebFetch HTTPS requests."
            >
              <label className="settings-switch">
                <input
                  type="checkbox"
                  checked={useSystemCertificates}
                  onChange={(event) => setUseSystemCertificates(event.target.checked)}
                  aria-label="Use system certificates"
                />
                <span className="settings-switch-track">
                  <span />
                </span>
                <b>{useSystemCertificates ? "On" : "Off"}</b>
              </label>
            </SettingRow>
            <SettingRow
              label="Certificate path"
              hint={
                useSystemCertificates
                  ? "Disable system certificates to provide a custom certificate file."
                  : "Choose a PEM, CRT, CER, or DER certificate file for custom trust."
              }
            >
              <div
                className={`settings-directory-field settings-file-selector ${useSystemCertificates ? "is-disabled" : ""}`}
              >
                <input
                  className="field mono"
                  value={useSystemCertificates ? "" : certificatePath}
                  onChange={(event) => setCertificatePath(event.target.value)}
                  aria-label="Custom certificate path"
                  placeholder={
                    useSystemCertificates
                      ? "Using system certificates"
                      : "/path/to/custom-certificate.pem"
                  }
                  spellCheck="false"
                  disabled={useSystemCertificates}
                />
                <button
                  type="button"
                  aria-label="Choose certificate file"
                  title="Choose file"
                  onClick={chooseCertificate}
                  disabled={useSystemCertificates}
                >
                  <Icon name="folder" size={16} />
                </button>
                <input
                  ref={certificateInputRef}
                  type="file"
                  accept=".pem,.crt,.cer,.der"
                  aria-hidden="true"
                  tabIndex="-1"
                  onChange={chooseFallbackCertificate}
                />
              </div>
            </SettingRow>
          </div>
        )}
        {!verify && (
          <div className="settings-warning">
            <Icon name="platform" size={16} />
            <div>
              <strong>Certificate verification is off</strong>
              <span>
                SunCode will accept invalid certificates and hostnames, similar to{" "}
                <code>curl -k</code>. This can expose provider credentials and fetched content to
                man-in-the-middle attacks.
              </span>
            </div>
          </div>
        )}
      </div>
      <div className="settings-actions">
        <Button variant="primary" size="sm" onClick={onSave}>
          Save HTTPS setting
        </Button>
        <span className="settings-save-status" role="status">
          {verify
            ? useSystemCertificates
              ? "System trust store"
              : "Custom certificate required"
            : "Review required"}
        </span>
      </div>
    </div>
  );
}

const browserRuntimeComponents = [
  {
    label: "Target",
    value: "darwin-arm64",
    detail: "Bundled for this SunCode installation",
  },
  {
    label: "Node.js",
    value: "v24.11.1",
    detail: "/Applications/SunCode.app/Contents/Resources/browser-runtime/node/bin/node",
  },
  {
    label: "Playwright",
    value: "1.55.0",
    detail: "Exact production package",
  },
  {
    label: "Chromium",
    value: "140.0.7339.16 · revision 1187",
    detail:
      "/Applications/SunCode.app/Contents/Resources/browser-runtime/browsers/chromium-1187/Chromium.app",
  },
  {
    label: "Worker protocol",
    value: "1",
    detail: "Rust-managed framed stdio",
  },
];

function BrowserUsePanel({ onSave }) {
  const [enabled, setEnabled] = useState(true);
  const [installationState, setInstallationState] = useState("ready");
  const [runtimeState, setRuntimeState] = useState("background");
  const [clearOpen, setClearOpen] = useState(false);

  const installationLabel = {
    disabled: "Disabled",
    ready: "Ready",
    verifying: "Verifying",
    invalid: "Integrity failed",
  }[enabled ? installationState : "disabled"];
  const runtimeLabel = {
    not_started: "Not started",
    starting: "Starting",
    background: "Running in background",
    user_controlled: "User controlled",
    failed: "Failed",
  }[enabled ? runtimeState : "not_started"];

  const toggleEnabled = (nextEnabled) => {
    setEnabled(nextEnabled);
    if (!nextEnabled) setRuntimeState("not_started");
    onSave(
      nextEnabled
        ? "Browser use enabled. Chromium starts only when a project needs it."
        : "Browser use disabled. Active browser runtimes were stopped.",
    );
  };
  const verifyRuntime = () => {
    setInstallationState("verifying");
    window.setTimeout(() => {
      setInstallationState("ready");
      onSave("Bundled browser runtime verified.");
    }, 700);
  };
  const takeControl = () => {
    setRuntimeState("user_controlled");
    onSave("Browser tools paused while you control Chromium.");
  };
  const returnControl = () => {
    setRuntimeState("background");
    onSave("Control returned. The agent must take a fresh page snapshot.");
  };
  const restartRuntime = () => {
    setRuntimeState("starting");
    window.setTimeout(() => {
      setRuntimeState("background");
      onSave("Project browser restarted with its persistent profile.");
    }, 700);
  };
  const stopRuntime = () => {
    setRuntimeState("not_started");
    onSave("Project browser stopped. Its profile was preserved.");
  };
  const clearProfile = () => {
    setRuntimeState("not_started");
    setClearOpen(false);
    onSave("Browser data cleared for project suncode.");
  };

  return (
    <div className="settings-panel-content settings-browser-content">
      <div className="settings-panel-heading">
        <h2>Browser use</h2>
        <p>
          Run the bundled Playwright Chromium for dynamic web inspection and interaction. Website
          changes and browser data are outside filesystem undo.
        </p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Availability</span>
        <SettingRow
          label="Enable Browser Use"
          hint="Makes browser tools available. Node.js and Chromium start lazily when a project needs them."
        >
          <label className="settings-switch">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(event) => toggleEnabled(event.target.checked)}
              aria-label="Enable Browser Use"
            />
            <span className="settings-switch-track">
              <span />
            </span>
            <b>{enabled ? "On" : "Off"}</b>
          </label>
        </SettingRow>
        <div className="browser-runtime-summary" aria-live="polite">
          <div className={`browser-runtime-state is-${enabled ? installationState : "disabled"}`}>
            <span className="settings-status-dot" />
            <div>
              <strong>{installationLabel}</strong>
              <span>Bundled installation</span>
            </div>
          </div>
          <div className={`browser-runtime-state is-${enabled ? runtimeState : "not_started"}`}>
            <span className="settings-status-dot" />
            <div>
              <strong>{runtimeLabel}</strong>
              <span>Project: suncode</span>
            </div>
          </div>
        </div>
        <div className="settings-actions">
          <Button size="sm" disabled={!enabled || installationState === "verifying"} onClick={verifyRuntime}>
            {installationState === "verifying" ? "Verifying…" : "Verify runtime"}
          </Button>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Bundled runtime</span>
        <div className="browser-component-list" aria-label="Bundled Browser Use components">
          {browserRuntimeComponents.map((component) => (
            <div className="browser-component-row" key={component.label}>
              <span>{component.label}</span>
              <div>
                <code>{component.value}</code>
                <small title={component.detail}>{component.detail}</small>
              </div>
              <button
                type="button"
                className="btn btn-icon btn-quiet"
                aria-label={`Copy ${component.label}`}
                title={`Copy ${component.label}`}
                onClick={() => navigator.clipboard?.writeText(component.detail)}
              >
                <Icon name="copy" size={14} />
              </button>
            </div>
          ))}
        </div>
        <div className="browser-integrity-note is-ready">
          <Icon name="lock" size={15} />
          <div>
            <strong>Integrity verified</strong>
            <span>Versions, packaged paths, and runtime trees match the application-protected manifest.</span>
          </div>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Current project browser</span>
        <SettingRow label="Profile scope" hint="Cookies and site data persist only for this project.">
          <code className="browser-readonly-value">Persistent per project</code>
        </SettingRow>
        <SettingRow label="Profile path" hint="Stored under the local SunCode data directory.">
          <code className="browser-readonly-value" title="~/.suncode/data/browser/profiles/prj_suncode">
            ~/.suncode/data/browser/profiles/prj_suncode
          </code>
        </SettingRow>
        <SettingRow label="Profile usage" hint="Browser data is not part of the opened project.">
          <code className="browser-readonly-value">38.4 MB · 2 active pages</code>
        </SettingRow>
        <SettingRow
          label="Window control"
          hint="The full Chromium window is minimized in background mode. Wayland may require manual foreground selection."
        >
          <code className="browser-readonly-value">Full · macOS</code>
        </SettingRow>
        <div className="browser-control-note">
          <Icon name={runtimeState === "user_controlled" ? "unlock" : "lock"} size={15} />
          <div>
            <strong>
              {runtimeState === "user_controlled" ? "You control Chromium" : "Agent control is active"}
            </strong>
            <span>
              {runtimeState === "user_controlled"
                ? "Browser tools are paused. Returning control invalidates previous element references."
                : "Showing the browser transfers exclusive control to you and pauses browser tools."}
            </span>
          </div>
        </div>
        <div className="settings-actions browser-runtime-actions">
          {runtimeState === "user_controlled" ? (
            <Button variant="primary" size="sm" onClick={returnControl}>
              Return control to agent
            </Button>
          ) : (
            <Button size="sm" disabled={!enabled || runtimeState !== "background"} onClick={takeControl}>
              Show browser and take control
            </Button>
          )}
          <Button size="sm" disabled={!enabled || runtimeState === "not_started"} onClick={restartRuntime}>
            Restart
          </Button>
          <Button size="sm" disabled={!enabled || runtimeState === "not_started"} onClick={stopRuntime}>
            Stop
          </Button>
          <Button variant="danger" size="sm" disabled={runtimeState !== "not_started"} onClick={() => setClearOpen(true)}>
            Clear browser data
          </Button>
        </div>
      </div>
      <div className="browser-authority-note">
        <Icon name="lock" size={15} />
        <div>
          <strong>Browser access is not a sandbox or an authority grant</strong>
          <span>
            Page content is untrusted. Sensitive transmission and consequential web actions still
            require confirmation at the point of risk.
          </span>
        </div>
      </div>
      <ConfirmationDialog
        open={clearOpen}
        title="Clear browser data?"
        description="Saved logins, cookies, site storage, and browsing state for this project will be removed. Project files are unchanged."
        confirmLabel="Clear browser data"
        onCancel={() => setClearOpen(false)}
        onConfirm={clearProfile}
      >
        <div className="confirmation-dialog-target">
          <span>PROJECT BROWSER PROFILE</span>
          <strong>suncode</strong>
        </div>
      </ConfirmationDialog>
    </div>
  );
}

function LoggingPanel({ onSave }) {
  const [directory, setDirectory] = useState("~/.suncode/logs");
  const [imageDirectory, setImageDirectory] = useState("~/.suncode/images");
  const directoryInputRef = useRef(null);
  const imageDirectoryInputRef = useRef(null);
  const chooseDirectory = async (setValue, fallbackInputRef) => {
    if (!("showDirectoryPicker" in window)) {
      fallbackInputRef.current?.click();
      return;
    }
    try {
      const handle = await window.showDirectoryPicker({ mode: "readwrite" });
      setValue(`~/${handle.name}`);
    } catch (error) {
      if (error?.name !== "AbortError") fallbackInputRef.current?.click();
    }
  };
  const chooseFallbackDirectory = (event, setValue) => {
    const relativePath = event.target.files?.[0]?.webkitRelativePath;
    const folderName = relativePath?.split("/")[0];
    if (folderName) setValue(`~/${folderName}`);
    event.target.value = "";
  };
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Logging</h2>
        <p>Control diagnostic detail and how long local log files are kept.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Diagnostic output</span>
        <SettingRow label="Minimum level" hint="Lower levels include more diagnostic detail.">
          <SingleDropdown
            options={["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"]}
            initialValue="INFO"
            ariaLabel="Minimum log level"
            className="settings-dropdown"
          />
        </SettingRow>
        <SettingRow
          label="Log directory"
          hint="Agent and desktop log files are written to this folder."
        >
          <div className="settings-directory-field">
            <input
              className="field mono"
              value={directory}
              onChange={(event) => setDirectory(event.target.value)}
              aria-label="Log directory"
              spellCheck="false"
            />
            <button
              type="button"
              aria-label="Choose log directory"
              title="Choose folder"
              onClick={() => chooseDirectory(setDirectory, directoryInputRef)}
            >
              <Icon name="folder" size={16} />
            </button>
            <input
              ref={directoryInputRef}
              type="file"
              webkitdirectory=""
              aria-hidden="true"
              tabIndex="-1"
              onChange={(event) => chooseFallbackDirectory(event, setDirectory)}
            />
          </div>
        </SettingRow>
        <SettingRow
          label="Maximum file size"
          hint="Rotate each log file when it reaches this size."
        >
          <label className="settings-unit-field">
            <input
              className="field mono"
              type="number"
              min="1"
              max="1000"
              step="1"
              defaultValue="10"
              aria-label="Maximum file size in megabytes"
            />
            <span>MB</span>
          </label>
        </SettingRow>
        <SettingRow
          label="Retained backups"
          hint="Number of rotated backups to keep, from 0 to 100."
        >
          <input
            className="field settings-number"
            type="number"
            min="0"
            max="100"
            defaultValue="5"
            aria-label="Retained backups"
          />
        </SettingRow>
      </div>
      <div className="settings-actions">
        <Button variant="primary" size="sm" onClick={onSave}>
          Save logging settings
        </Button>
        <span className="settings-save-status" role="status">
          Local settings
        </span>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Image storage</span>
        <SettingRow label="Image directory" hint="Directory for saving images.">
          <div className="settings-directory-field">
            <input
              className="field mono"
              value={imageDirectory}
              onChange={(event) => setImageDirectory(event.target.value)}
              aria-label="Image directory"
              spellCheck="false"
            />
            <button
              type="button"
              aria-label="Choose image directory"
              title="Choose folder"
              onClick={() => chooseDirectory(setImageDirectory, imageDirectoryInputRef)}
            >
              <Icon name="folder" size={16} />
            </button>
            <input
              ref={imageDirectoryInputRef}
              type="file"
              webkitdirectory=""
              aria-hidden="true"
              tabIndex="-1"
              onChange={(event) => chooseFallbackDirectory(event, setImageDirectory)}
            />
          </div>
        </SettingRow>
      </div>
      <div className="settings-actions">
        <Button variant="primary" size="sm" onClick={onSave}>
          Save image location
        </Button>
        <span className="settings-save-status" role="status">
          Local settings
        </span>
      </div>
    </div>
  );
}

const initialMcpServers = [
  {
    id: "mcp_filesystem",
    name: "Project filesystem",
    transport: "stdio",
    command: "uvx",
    arguments: "mcp-server-filesystem\n.",
    workingDirectory: "Project directory",
    enabled: true,
    status: "connected",
    toolCount: 11,
  },
  {
    id: "mcp_github",
    name: "GitHub",
    transport: "http",
    url: "https://mcp.github.example/v1",
    enabled: true,
    status: "failed",
    toolCount: 0,
    error: "Authentication failed. Replace the Authorization header and retry.",
  },
  {
    id: "mcp_postgres",
    name: "Local Postgres",
    transport: "stdio",
    command: "docker",
    arguments: "run\n--rm\nmcp/postgres",
    workingDirectory: "Project directory",
    enabled: false,
    status: "disabled",
    toolCount: 0,
  },
];

const emptyMcpDraft = {
  name: "",
  transport: "stdio",
  command: "",
  url: "",
  workingDirectory: "Project directory",
  arguments: "",
  secrets: "",
  startupTimeout: "30",
  requestTimeout: "60",
  enabled: true,
};

const mcpStatusLabels = {
  connected: "Connected",
  connecting: "Connecting",
  failed: "Failed",
  disabled: "Disabled",
};

function McpSwitch({ server, onToggle }) {
  const pending = server.status === "connecting";
  return (
    <label className="settings-switch mcp-server-switch">
      <input
        type="checkbox"
        checked={server.enabled}
        disabled={pending}
        onChange={(event) => onToggle(server.id, event.target.checked)}
        aria-label={`${server.enabled ? "Disable" : "Enable"} ${server.name}`}
      />
      <span className="settings-switch-track">
        <span />
      </span>
      <b>{server.enabled ? "On" : "Off"}</b>
    </label>
  );
}

function McpServerEditorWindow({ open, server, onClose, onSubmit }) {
  const windowRef = useRef(null);
  const descriptionId = useId();
  const [draft, setDraft] = useState(() =>
    server
      ? {
          ...emptyMcpDraft,
          name: server.name,
          transport: server.transport,
          command: server.command ?? "",
          arguments: server.arguments ?? "",
          url: server.url ?? "",
          workingDirectory: server.workingDirectory ?? "Project directory",
          enabled: server.enabled,
        }
      : emptyMcpDraft,
  );
  const editing = Boolean(server);

  const update = (key, value) => setDraft((current) => ({ ...current, [key]: value }));
  const valid =
    draft.name.trim() && (draft.transport === "stdio" ? draft.command.trim() : draft.url.trim());

  useEffect(() => {
    if (!open) return undefined;
    const priorFocus = document.activeElement;
    windowRef.current?.querySelector("input, button, textarea")?.focus();
    const handleKeyDown = (event) => {
      if (event.key === "Escape") onClose();
      if (event.key !== "Tab") return;
      const focusable = [
        ...windowRef.current.querySelectorAll("button, input, textarea, [href]"),
      ].filter((element) => !element.disabled);
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      priorFocus?.focus();
    };
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="mcp-editor-window-shell">
      <NativeWindowFrame
        platform="macos"
        title={editing ? "Edit MCP server" : "Add MCP server"}
        width="620px"
        height="610px"
        onClose={onClose}
        className="mcp-editor-window"
      >
        <div
          ref={windowRef}
          className="mcp-editor-window-client"
          role="dialog"
          aria-modal="false"
          aria-label={editing ? "Edit MCP server" : "Add MCP server"}
          aria-describedby={descriptionId}
        >
          <header className="mcp-editor-heading">
            <h2>{editing ? "Edit MCP server" : "Add MCP server"}</h2>
            <p id={descriptionId}>
              Changes are persisted locally and reconciled with the embedded agent immediately.
            </p>
          </header>
          <div className="mcp-editor-scroll">
            <div className="mcp-server-form">
              <label className="mcp-form-field">
                <span>Server name</span>
                <input
                  className="field"
                  value={draft.name}
                  onChange={(event) => update("name", event.target.value)}
                  placeholder="GitHub"
                  aria-label="MCP server name"
                />
              </label>
              <label className="mcp-form-field">
                <span>Transport</span>
                <SingleDropdown
                  options={[
                    { value: "stdio", label: "Local process (stdio)" },
                    { value: "http", label: "Remote (Streamable HTTP)" },
                  ]}
                  value={draft.transport}
                  onChange={(value) => update("transport", value)}
                  ariaLabel="MCP transport"
                  className="settings-dropdown"
                />
              </label>
              {draft.transport === "stdio" ? (
                <>
                  <label className="mcp-form-field mcp-form-wide">
                    <span>Executable</span>
                    <input
                      className="field mono"
                      value={draft.command}
                      onChange={(event) => update("command", event.target.value)}
                      placeholder="uvx"
                      spellCheck="false"
                      aria-label="Local MCP executable"
                    />
                  </label>
                  <label className="mcp-form-field">
                    <span>Working directory</span>
                    <SingleDropdown
                      options={["Project directory", "Application data directory"]}
                      value={draft.workingDirectory}
                      onChange={(value) => update("workingDirectory", value)}
                      ariaLabel="MCP working directory"
                      className="settings-dropdown"
                    />
                  </label>
                  <label className="mcp-form-field">
                    <span>Arguments</span>
                    <textarea
                      className="field mono"
                      value={draft.arguments}
                      onChange={(event) => update("arguments", event.target.value)}
                      placeholder="One argument per line"
                      aria-label="MCP command arguments"
                    />
                  </label>
                </>
              ) : (
                <label className="mcp-form-field mcp-form-wide">
                  <span>Server URL</span>
                  <input
                    className="field mono"
                    type="url"
                    value={draft.url}
                    onChange={(event) => update("url", event.target.value)}
                    placeholder="https://mcp.example.com/v1"
                    spellCheck="false"
                    aria-label="Remote MCP server URL"
                  />
                </label>
              )}
              <label className="mcp-form-field mcp-form-wide">
                <span>{draft.transport === "stdio" ? "Environment" : "HTTP headers"}</span>
                <textarea
                  className="field mono"
                  value={draft.secrets}
                  onChange={(event) => update("secrets", event.target.value)}
                  placeholder={
                    editing
                      ? "NAME=value replaces; -NAME removes; other stored values remain unchanged"
                      : draft.transport === "stdio"
                        ? "NAME=value"
                        : "Authorization=Bearer ..."
                  }
                  aria-label={
                    draft.transport === "stdio" ? "MCP environment entries" : "MCP HTTP headers"
                  }
                />
                <small>
                  Values are stored locally and hidden after saving. Use -NAME to remove a stored
                  key.
                </small>
              </label>
              <label className="mcp-form-field">
                <span>Startup timeout</span>
                <span className="settings-unit-field">
                  <input
                    className="field mono"
                    type="number"
                    min="1"
                    max="120"
                    value={draft.startupTimeout}
                    onChange={(event) => update("startupTimeout", event.target.value)}
                    aria-label="MCP startup timeout in seconds"
                  />
                  <span>SEC</span>
                </span>
              </label>
              <label className="mcp-form-field">
                <span>Request timeout</span>
                <span className="settings-unit-field">
                  <input
                    className="field mono"
                    type="number"
                    min="1"
                    max="600"
                    value={draft.requestTimeout}
                    onChange={(event) => update("requestTimeout", event.target.value)}
                    aria-label="MCP request timeout in seconds"
                  />
                  <span>SEC</span>
                </span>
              </label>
              <div className="mcp-form-enable mcp-form-wide">
                <div>
                  <strong>Enable server</strong>
                  <span>
                    {draft.transport === "stdio"
                      ? "Starts a local process with your user authority."
                      : "Connects to the remote endpoint after saving."}
                  </span>
                </div>
                <label className="settings-switch">
                  <input
                    type="checkbox"
                    checked={draft.enabled}
                    onChange={(event) => update("enabled", event.target.checked)}
                    aria-label="Enable MCP server after saving"
                  />
                  <span className="settings-switch-track">
                    <span />
                  </span>
                  <b>{draft.enabled ? "On" : "Off"}</b>
                </label>
              </div>
            </div>
          </div>
          <footer className="mcp-editor-actions">
            <Button size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              disabled={!valid}
              onClick={() => onSubmit({ ...draft, id: server?.id })}
            >
              {editing ? "Save changes" : "Add server"}
            </Button>
          </footer>
        </div>
      </NativeWindowFrame>
    </div>
  );
}

function McpServersPanel({ servers, setServers, onSave }) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingId, setEditingId] = useState(null);
  const [deletingId, setDeletingId] = useState(null);
  const editingServer = servers.find((server) => server.id === editingId);
  const deletingServer = servers.find((server) => server.id === deletingId);

  const setTemporaryStatus = (id, finalStatus = "connected") => {
    setServers((current) =>
      current.map((server) =>
        server.id === id
          ? { ...server, enabled: true, status: "connecting", error: undefined, toolCount: 0 }
          : server,
      ),
    );
    window.setTimeout(() => {
      setServers((current) =>
        current.map((server) =>
          server.id === id
            ? {
                ...server,
                status: finalStatus,
                toolCount: finalStatus === "connected" ? server.toolCount || 6 : 0,
              }
            : server,
        ),
      );
    }, 650);
  };

  const toggleServer = (id, enabled) => {
    if (!enabled) {
      setServers((current) =>
        current.map((server) =>
          server.id === id
            ? { ...server, enabled: false, status: "disabled", toolCount: 0, error: undefined }
            : server,
        ),
      );
      onSave("Server disabled. Its tools are no longer available to new calls.");
      return;
    }
    setTemporaryStatus(id);
    onSave("Server configuration saved. Connecting now.");
  };

  const saveServer = (draft) => {
    if (draft.id) {
      setServers((current) =>
        current.map((server) =>
          server.id === draft.id
            ? {
                ...server,
                name: draft.name.trim(),
                transport: draft.transport,
                command: draft.transport === "stdio" ? draft.command.trim() : undefined,
                arguments: draft.transport === "stdio" ? draft.arguments.trim() : undefined,
                url: draft.transport === "http" ? draft.url.trim() : undefined,
                workingDirectory: draft.workingDirectory,
                enabled: draft.enabled,
                status: draft.enabled ? "connecting" : "disabled",
                toolCount: 0,
                error: undefined,
              }
            : server,
        ),
      );
      if (draft.enabled) setTemporaryStatus(draft.id);
    } else {
      const id = `mcp_${Date.now()}`;
      const next = {
        id,
        name: draft.name.trim(),
        transport: draft.transport,
        command: draft.transport === "stdio" ? draft.command.trim() : undefined,
        arguments: draft.transport === "stdio" ? draft.arguments.trim() : undefined,
        url: draft.transport === "http" ? draft.url.trim() : undefined,
        workingDirectory: draft.workingDirectory,
        enabled: draft.enabled,
        status: draft.enabled ? "connecting" : "disabled",
        toolCount: 0,
      };
      setServers((current) => [...current, next]);
      if (draft.enabled) setTemporaryStatus(id);
    }
    setDialogOpen(false);
    setEditingId(null);
    onSave(draft.enabled ? "Saved to SQLite. Connecting now." : "Saved to SQLite as disabled.");
  };

  const deleteServer = () => {
    if (!deletingServer) return;
    setServers((current) => current.filter((server) => server.id !== deletingServer.id));
    setDeletingId(null);
    onSave("Server deleted. Its tools were removed from new model requests.");
  };

  return (
    <div className="settings-panel-content settings-mcp-content">
      <div className="settings-panel-heading settings-heading-row">
        <div>
          <h2>MCP servers</h2>
          <p>Manage local and remote tools available to the embedded agent.</p>
        </div>
        <Button
          variant="primary"
          size="sm"
          icon="plus"
          onClick={() => {
            setEditingId(null);
            setDialogOpen(true);
          }}
        >
          Add server
        </Button>
      </div>
      <span className="mcp-project-context">Project connection: suncode</span>
      <div className="mcp-runtime-note" role="note">
        <Icon name="refresh" size={15} />
        <span>
          Changes update existing sessions. The next model request uses the effective tool list.
        </span>
      </div>
      {servers.length ? (
        <div className="mcp-server-list" aria-label="Configured MCP servers">
          <div className="mcp-server-list-header" aria-hidden="true">
            <span>Server</span>
            <span>Status</span>
            <span>Enabled</span>
            <span>Actions</span>
          </div>
          {servers.map((server) => (
            <div className={`mcp-server-row is-${server.status}`} key={server.id}>
              <div className="mcp-server-identity">
                <strong>{server.name}</strong>
                <code>
                  {server.transport === "stdio"
                    ? [server.command, server.arguments?.replaceAll("\n", " ")]
                        .filter(Boolean)
                        .join(" ")
                    : server.url}
                </code>
                {server.error && <span className="mcp-server-error">{server.error}</span>}
              </div>
              <div className={`mcp-server-status is-${server.status}`}>
                <span className="settings-status-dot" />
                <div>
                  <strong>{mcpStatusLabels[server.status]}</strong>
                  <span>
                    {server.status === "connected"
                      ? `${server.toolCount} tools`
                      : server.status === "connecting"
                        ? "Discovering tools"
                        : server.status === "failed"
                          ? "No tools available"
                          : "Not running"}
                  </span>
                </div>
              </div>
              <McpSwitch server={server} onToggle={toggleServer} />
              <div className="mcp-server-actions">
                {server.status === "failed" && (
                  <button
                    type="button"
                    className="btn btn-icon btn-quiet"
                    aria-label={`Retry ${server.name}`}
                    title="Retry connection"
                    onClick={() => setTemporaryStatus(server.id)}
                  >
                    <Icon name="refresh" size={14} />
                  </button>
                )}
                <button
                  type="button"
                  className="btn btn-icon btn-quiet"
                  aria-label={`Edit ${server.name}`}
                  title="Edit server"
                  onClick={() => {
                    setEditingId(server.id);
                    setDialogOpen(true);
                  }}
                >
                  <Icon name="edit" size={14} />
                </button>
                <button
                  type="button"
                  className="btn btn-icon btn-quiet mcp-delete-button"
                  aria-label={`Delete ${server.name}`}
                  title="Delete server"
                  onClick={() => setDeletingId(server.id)}
                >
                  <Icon name="trash" size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="mcp-empty-state">
          <Icon name="server" size={22} />
          <strong>No MCP servers configured</strong>
          <span>Add a local process or remote endpoint to make its tools available.</span>
        </div>
      )}
      <div className="mcp-authority-note">
        <Icon name="lock" size={15} />
        <div>
          <strong>MCP tools run outside SunCode's undo boundary</strong>
          <span>
            Tool calls still require policy approval, but changes made by an MCP server may not be
            reversible by SunCode.
          </span>
        </div>
      </div>
      <McpServerEditorWindow
        key={`${editingId ?? "new"}:${dialogOpen}`}
        open={dialogOpen}
        server={editingServer}
        onClose={() => {
          setDialogOpen(false);
          setEditingId(null);
        }}
        onSubmit={saveServer}
      />
      <ConfirmationDialog
        open={Boolean(deletingServer)}
        title="Delete MCP server?"
        description="Its tools will be removed from new model requests. An already executing call may finish."
        confirmLabel="Delete server"
        onCancel={() => setDeletingId(null)}
        onConfirm={deleteServer}
      >
        <div className="confirmation-dialog-target">
          <span>MCP SERVER</span>
          <strong>{deletingServer?.name}</strong>
        </div>
      </ConfirmationDialog>
    </div>
  );
}

const initialLanguageServers = [
  {
    id: "lsp_rust_analyzer",
    name: "rust-analyzer",
    command: "rust-analyzer",
    arguments: "",
    languages: ["Rust"],
    languageIds: "rust",
    rootMarkers: "Cargo.toml\nrust-project.json",
    enabled: true,
    status: "ready",
    capabilityCount: 5,
  },
  {
    id: "lsp_typescript",
    name: "TypeScript language server",
    command: "typescript-language-server",
    arguments: "--stdio",
    languages: ["TypeScript", "JavaScript"],
    languageIds: "typescript\ntypescriptreact\njavascript\njavascriptreact",
    rootMarkers: "tsconfig.json\njsconfig.json\npackage.json",
    enabled: true,
    status: "indexing",
    progress: 68,
    capabilityCount: 5,
  },
  {
    id: "lsp_gopls",
    name: "gopls",
    command: "gopls",
    arguments: "serve",
    languages: ["Go"],
    languageIds: "go\ngomod\ngowork\ngotmpl",
    rootMarkers: "go.work\ngo.mod",
    enabled: true,
    status: "failed",
    capabilityCount: 0,
    error: "Executable not found. Install gopls or update the executable path.",
  },
  {
    id: "lsp_python",
    name: "Python LSP",
    command: "pylsp",
    arguments: "",
    languages: ["Python"],
    languageIds: "python",
    rootMarkers: "pyproject.toml\nsetup.py\nrequirements.txt",
    enabled: false,
    status: "disabled",
    capabilityCount: 0,
  },
];

const emptyLanguageServerDraft = {
  name: "",
  command: "",
  arguments: "",
  languageIds: "",
  rootMarkers: "",
  initializationOptions: "",
  environment: "",
  startupTimeout: "30",
  requestTimeout: "30",
  enabled: true,
};

const languageServerStatusLabels = {
  ready: "Ready",
  indexing: "Indexing",
  failed: "Failed",
  disabled: "Disabled",
  not_started: "Not started",
};

function LanguageServerSwitch({ server, onToggle }) {
  const pending = server.status === "indexing";
  return (
    <label className="settings-switch lsp-server-switch">
      <input
        type="checkbox"
        checked={server.enabled}
        disabled={pending}
        onChange={(event) => onToggle(server.id, event.target.checked)}
        aria-label={`${server.enabled ? "Disable" : "Enable"} ${server.name}`}
      />
      <span className="settings-switch-track">
        <span />
      </span>
      <b>{server.enabled ? "On" : "Off"}</b>
    </label>
  );
}

function LanguageServerEditorWindow({ open, server, onClose, onSubmit }) {
  const windowRef = useRef(null);
  const descriptionId = useId();
  const [draft, setDraft] = useState(() =>
    server
      ? {
          ...emptyLanguageServerDraft,
          name: server.name,
          command: server.command,
          arguments: server.arguments ?? "",
          languageIds: server.languageIds ?? "",
          rootMarkers: server.rootMarkers ?? "",
          initializationOptions: server.initializationOptions ?? "",
          startupTimeout: server.startupTimeout ?? emptyLanguageServerDraft.startupTimeout,
          requestTimeout: server.requestTimeout ?? emptyLanguageServerDraft.requestTimeout,
          enabled: server.enabled,
        }
      : emptyLanguageServerDraft,
  );
  const editing = Boolean(server);
  const update = (key, value) => setDraft((current) => ({ ...current, [key]: value }));
  const valid = draft.name.trim() && draft.command.trim() && draft.languageIds.trim();

  useEffect(() => {
    if (!open) return undefined;
    const priorFocus = document.activeElement;
    windowRef.current?.querySelector("input, button, textarea")?.focus();
    const handleKeyDown = (event) => {
      if (event.key === "Escape") onClose();
      if (event.key !== "Tab") return;
      const focusable = [
        ...windowRef.current.querySelectorAll("button, input, textarea, [href]"),
      ].filter((element) => !element.disabled);
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      priorFocus?.focus();
    };
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="lsp-editor-window-shell">
      <NativeWindowFrame
        platform="macos"
        title={editing ? "Edit language server" : "Add language server"}
        width="640px"
        height="660px"
        onClose={onClose}
        className="lsp-editor-window"
      >
        <div
          ref={windowRef}
          className="lsp-editor-window-client"
          role="dialog"
          aria-modal="false"
          aria-label={editing ? "Edit language server" : "Add language server"}
          aria-describedby={descriptionId}
        >
          <header className="lsp-editor-heading">
            <h2>{editing ? "Edit language server" : "Add language server"}</h2>
            <p id={descriptionId}>
              Configure a structured local process. SunCode never invokes the executable through a
              shell.
            </p>
          </header>
          <div className="lsp-editor-scroll">
            <div className="lsp-server-form">
              <label className="lsp-form-field">
                <span>Server name</span>
                <input
                  className="field"
                  value={draft.name}
                  onChange={(event) => update("name", event.target.value)}
                  placeholder="rust-analyzer"
                  aria-label="Language server name"
                />
              </label>
              <label className="lsp-form-field">
                <span>Executable</span>
                <input
                  className="field mono"
                  value={draft.command}
                  onChange={(event) => update("command", event.target.value)}
                  placeholder="rust-analyzer"
                  spellCheck="false"
                  aria-label="Language server executable"
                />
              </label>
              <label className="lsp-form-field">
                <span>Arguments</span>
                <textarea
                  className="field mono"
                  value={draft.arguments}
                  onChange={(event) => update("arguments", event.target.value)}
                  placeholder="One argument per line"
                  aria-label="Language server arguments"
                />
              </label>
              <label className="lsp-form-field">
                <span>Language IDs</span>
                <textarea
                  className="field mono"
                  value={draft.languageIds}
                  onChange={(event) => update("languageIds", event.target.value)}
                  placeholder="One LSP language ID per line"
                  aria-label="Language IDs"
                />
                <small>At least one language ID is required.</small>
              </label>
              <label className="lsp-form-field">
                <span>Root markers</span>
                <textarea
                  className="field mono"
                  value={draft.rootMarkers}
                  onChange={(event) => update("rootMarkers", event.target.value)}
                  placeholder="Cargo.toml"
                  aria-label="Project root markers"
                />
                <small>One project-relative file or directory name per line.</small>
              </label>
              <label className="lsp-form-field">
                <span>Initialization options</span>
                <textarea
                  className="field mono"
                  value={draft.initializationOptions}
                  onChange={(event) => update("initializationOptions", event.target.value)}
                  placeholder="Optional JSON object"
                  aria-label="Language server initialization options"
                  spellCheck="false"
                />
              </label>
              <label className="lsp-form-field lsp-form-wide">
                <span>Environment</span>
                <textarea
                  className="field mono"
                  value={draft.environment}
                  onChange={(event) => update("environment", event.target.value)}
                  placeholder={editing ? "NAME=value replaces; -NAME removes" : "NAME=value"}
                  aria-label="Language server environment entries"
                />
                <small>
                  Values are stored locally and hidden after saving. Use -NAME to remove a stored
                  key.
                </small>
              </label>
              <label className="lsp-form-field">
                <span>Startup timeout</span>
                <span className="settings-unit-field">
                  <input
                    className="field mono"
                    type="number"
                    min="1"
                    max="120"
                    value={draft.startupTimeout}
                    onChange={(event) => update("startupTimeout", event.target.value)}
                    aria-label="Language server startup timeout in seconds"
                  />
                  <span>SEC</span>
                </span>
              </label>
              <label className="lsp-form-field">
                <span>Request timeout</span>
                <span className="settings-unit-field">
                  <input
                    className="field mono"
                    type="number"
                    min="1"
                    max="600"
                    value={draft.requestTimeout}
                    onChange={(event) => update("requestTimeout", event.target.value)}
                    aria-label="Language server request timeout in seconds"
                  />
                  <span>SEC</span>
                </span>
              </label>
              <div className="lsp-form-enable lsp-form-wide">
                <div>
                  <strong>Enable server</strong>
                  <span>Starts one project-scoped process when a matching project is active.</span>
                </div>
                <label className="settings-switch">
                  <input
                    type="checkbox"
                    checked={draft.enabled}
                    onChange={(event) => update("enabled", event.target.checked)}
                    aria-label="Enable language server after saving"
                  />
                  <span className="settings-switch-track">
                    <span />
                  </span>
                  <b>{draft.enabled ? "On" : "Off"}</b>
                </label>
              </div>
            </div>
          </div>
          <footer className="lsp-editor-actions">
            <Button size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              disabled={!valid}
              onClick={() => onSubmit({ ...draft, id: server?.id })}
            >
              {editing ? "Save changes" : "Add server"}
            </Button>
          </footer>
        </div>
      </NativeWindowFrame>
    </div>
  );
}

function LanguageServersPanel({ servers, setServers, onSave }) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingId, setEditingId] = useState(null);
  const [deletingId, setDeletingId] = useState(null);
  const editingServer = servers.find((server) => server.id === editingId);
  const deletingServer = servers.find((server) => server.id === deletingId);

  const startIndexing = (id) => {
    setServers((current) =>
      current.map((server) =>
        server.id === id
          ? {
              ...server,
              enabled: true,
              status: "indexing",
              progress: 18,
              error: undefined,
              capabilityCount: 0,
            }
          : server,
      ),
    );
    window.setTimeout(() => {
      setServers((current) =>
        current.map((server) =>
          server.id === id
            ? { ...server, status: "ready", progress: undefined, capabilityCount: 5 }
            : server,
        ),
      );
    }, 850);
  };

  const toggleServer = (id, enabled) => {
    if (!enabled) {
      setServers((current) =>
        current.map((server) =>
          server.id === id
            ? {
                ...server,
                enabled: false,
                status: "disabled",
                progress: undefined,
                capabilityCount: 0,
                error: undefined,
              }
            : server,
        ),
      );
      onSave("Language server disabled for new project runtimes.");
      return;
    }
    startIndexing(id);
    onSave("Language server enabled. Starting and indexing the current project.");
  };

  const saveServer = (draft) => {
    const languages = draft.languageIds
      .split("\n")
      .map((value) => value.trim())
      .filter(Boolean)
      .slice(0, 3)
      .map((value) => value.replace(/react$/i, " React"))
      .map((value) => value.charAt(0).toUpperCase() + value.slice(1));
    if (draft.id) {
      setServers((current) =>
        current.map((server) =>
          server.id === draft.id
            ? {
                ...server,
                name: draft.name.trim(),
                command: draft.command.trim(),
                arguments: draft.arguments.trim(),
                languageIds: draft.languageIds.trim(),
                rootMarkers: draft.rootMarkers.trim(),
                initializationOptions: draft.initializationOptions.trim(),
                startupTimeout: draft.startupTimeout,
                requestTimeout: draft.requestTimeout,
                languages,
                enabled: draft.enabled,
                status: draft.enabled ? "indexing" : "disabled",
                progress: draft.enabled ? 18 : undefined,
                capabilityCount: 0,
                error: undefined,
              }
            : server,
        ),
      );
      if (draft.enabled) startIndexing(draft.id);
    } else {
      const id = `lsp_${Date.now()}`;
      const next = {
        id,
        name: draft.name.trim(),
        command: draft.command.trim(),
        arguments: draft.arguments.trim(),
        languageIds: draft.languageIds.trim(),
        rootMarkers: draft.rootMarkers.trim(),
        initializationOptions: draft.initializationOptions.trim(),
        startupTimeout: draft.startupTimeout,
        requestTimeout: draft.requestTimeout,
        languages,
        enabled: draft.enabled,
        status: draft.enabled ? "indexing" : "disabled",
        progress: draft.enabled ? 18 : undefined,
        capabilityCount: 0,
      };
      setServers((current) => [...current, next]);
      if (draft.enabled) window.setTimeout(() => startIndexing(id), 0);
    }
    setDialogOpen(false);
    setEditingId(null);
    onSave(
      draft.enabled
        ? "Language server saved. Starting the current project runtime."
        : "Language server saved as disabled.",
    );
  };

  const deleteServer = () => {
    if (!deletingServer) return;
    setServers((current) => current.filter((server) => server.id !== deletingServer.id));
    setDeletingId(null);
    onSave("Language server deleted. Its project runtimes were stopped.");
  };

  return (
    <div className="settings-panel-content settings-lsp-content">
      <div className="settings-panel-heading settings-heading-row">
        <div>
          <h2>Language servers</h2>
          <p>Manage project-scoped semantic analysis used by the coding agent.</p>
        </div>
        <Button
          variant="primary"
          size="sm"
          icon="plus"
          onClick={() => {
            setEditingId(null);
            setDialogOpen(true);
          }}
        >
          Add server
        </Button>
      </div>
      {servers.length ? (
        <div className="lsp-server-list" aria-label="Configured language servers">
          <div className="lsp-server-list-header" aria-hidden="true">
            <span>Server</span>
            <span>Languages</span>
            <span>Status</span>
            <span>Enabled</span>
            <span>Actions</span>
          </div>
          {servers.map((server) => (
            <div className={`lsp-server-row is-${server.status}`} key={server.id}>
              <div className="lsp-server-identity">
                <strong>{server.name}</strong>
                <code>
                  {[server.command, server.arguments?.replaceAll("\n", " ")]
                    .filter(Boolean)
                    .join(" ")}
                </code>
                {server.error && <span className="lsp-server-error">{server.error}</span>}
              </div>
              <div
                className="lsp-language-list"
                aria-label={`Languages: ${server.languages.join(", ")}`}
              >
                {server.languages.map((language) => (
                  <code key={language}>{language}</code>
                ))}
              </div>
              <div className={`lsp-server-status is-${server.status}`}>
                <span className="settings-status-dot" />
                <div>
                  <strong>{languageServerStatusLabels[server.status]}</strong>
                  <span>
                    {server.status === "ready"
                      ? `${server.capabilityCount} capabilities`
                      : server.status === "indexing"
                        ? `${server.progress ?? 0}% indexed`
                        : server.status === "failed"
                          ? "No semantic results"
                          : server.status === "not_started"
                            ? "Open a project"
                            : "Not running"}
                  </span>
                  {server.status === "indexing" && (
                    <span className="lsp-index-progress" aria-hidden="true">
                      <span style={{ width: `${server.progress ?? 0}%` }} />
                    </span>
                  )}
                </div>
              </div>
              <LanguageServerSwitch server={server} onToggle={toggleServer} />
              <div className="lsp-server-actions">
                {server.status === "failed" && (
                  <button
                    type="button"
                    className="btn btn-icon btn-quiet"
                    aria-label={`Retry ${server.name}`}
                    title="Retry server"
                    onClick={() => startIndexing(server.id)}
                  >
                    <Icon name="refresh" size={14} />
                  </button>
                )}
                <button
                  type="button"
                  className="btn btn-icon btn-quiet"
                  aria-label={`Edit ${server.name}`}
                  title="Edit server"
                  onClick={() => {
                    setEditingId(server.id);
                    setDialogOpen(true);
                  }}
                >
                  <Icon name="edit" size={14} />
                </button>
                <button
                  type="button"
                  className="btn btn-icon btn-quiet lsp-delete-button"
                  aria-label={`Delete ${server.name}`}
                  title="Delete server"
                  onClick={() => setDeletingId(server.id)}
                >
                  <Icon name="trash" size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="lsp-empty-state">
          <Icon name="file-code" size={22} />
          <strong>No language servers configured</strong>
          <span>Add a local language server to give the agent project-aware semantic context.</span>
        </div>
      )}
      <LanguageServerEditorWindow
        key={`${editingId ?? "new"}:${dialogOpen}`}
        open={dialogOpen}
        server={editingServer}
        onClose={() => {
          setDialogOpen(false);
          setEditingId(null);
        }}
        onSubmit={saveServer}
      />
      <ConfirmationDialog
        open={Boolean(deletingServer)}
        title="Delete language server?"
        description="Its project runtimes will stop and semantic results will no longer be available to new agent turns."
        confirmLabel="Delete server"
        onCancel={() => setDeletingId(null)}
        onConfirm={deleteServer}
      >
        <div className="confirmation-dialog-target">
          <span>LANGUAGE SERVER</span>
          <strong>{deletingServer?.name}</strong>
        </div>
      </ConfirmationDialog>
    </div>
  );
}

function AgentsPanel({ onSelect }) {
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

function AgentPanel({ agentId }) {
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

function ProvidersPanel({ onSelect, endpoints }) {
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

function ProviderPanel({ providerId, onSave, endpoint, onEndpointChange }) {
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

export function SettingsPage({ initialPage = "defaults" }) {
  const [page, setPage] = useState(initialPage);
  const [agentsExpanded, setAgentsExpanded] = useState(false);
  const [providersExpanded, setProvidersExpanded] = useState(false);
  const [providerEndpoints, setProviderEndpoints] = useState(() =>
    Object.fromEntries(
      Object.entries(providerCatalog).map(([id, provider]) => [id, provider.endpoint]),
    ),
  );
  const [status, setStatus] = useState("");
  const [mcpServers, setMcpServers] = useState(initialMcpServers);
  const [languageServers, setLanguageServers] = useState(initialLanguageServers);
  const [guideOpen, setGuideOpen] = useState(false);
  const navigateBack = () => {
    window.location.hash = "/projects/desktop/project-hub";
  };
  const save = (message = "Saved to the local agent.") => setStatus(message);
  const renderPanel = () => {
    if (page === "appearance") return <AppearancePanel onSave={save} />;
    if (page === "shortcuts") return <ShortcutsPanel />;
    if (page === "network") return <NetworkPanel onSave={save} />;
    if (page === "browser") return <BrowserUsePanel onSave={save} />;
    if (page === "mcp")
      return <McpServersPanel servers={mcpServers} setServers={setMcpServers} onSave={save} />;
    if (page === "lsp")
      return (
        <LanguageServersPanel
          servers={languageServers}
          setServers={setLanguageServers}
          onSave={save}
        />
      );
    if (page === "agents")
      return (
        <AgentsPanel
          onSelect={(agentId) => {
            setPage(`agent:${agentId}`);
            setStatus("");
          }}
        />
      );
    if (page.startsWith("agent:")) return <AgentPanel agentId={page.slice("agent:".length)} />;
    if (page === "logging") return <LoggingPanel onSave={save} />;
    if (page === "providers")
      return (
        <ProvidersPanel
          endpoints={providerEndpoints}
          onSelect={(providerId) => {
            setPage(`provider:${providerId}`);
            setStatus("");
          }}
        />
      );
    if (page.startsWith("provider:")) {
      const providerId = page.split(":")[1];
      return (
        <ProviderPanel
          key={page}
          providerId={providerId}
          endpoint={providerEndpoints[providerId]}
          onEndpointChange={(endpoint) =>
            setProviderEndpoints((current) => ({ ...current, [providerId]: endpoint }))
          }
          onSave={save}
        />
      );
    }
    return <DefaultsPanel onSave={save} />;
  };
  return (
    <>
      <PageHeader
        title="Settings"
        description="The Avalonia desktop settings window for local defaults, keyboard shortcuts, security, Browser Use, MCP servers, language servers, built-in agents, diagnostics, and provider credentials."
        path="projects/desktop/settings/"
      />
      <WindowSizeNote width="900" height="672" minimumWidth="720" minimumHeight="552" />
      <Section
        id="settings-window"
        title="Desktop settings"
        description="A focused settings window whose outer title bar and controls are native to the operating system."
      >
        <WorkspaceGuideState
          className="settings-guide-state"
          title="Settings controls"
          description="Navigate local defaults, keyboard shortcuts, security, Browser Use, MCP servers, language servers, built-in agents, diagnostics, and provider credentials."
          guide={settingsGuide}
          side="right"
          open={guideOpen}
          onToggle={() => setGuideOpen((open) => !open)}
          onClose={() => setGuideOpen(false)}
        >
          <NativeWindowFrame
            platform="macos"
            title="Settings"
            width="900px"
            height="672px"
            className="settings-window"
          >
            <div className="settings-toolbar">
              <strong>Settings</strong>
              <Button variant="primary" size="sm" onClick={navigateBack}>
                Done
              </Button>
            </div>
            <div className="settings-body">
              <SettingsNav
                page={page}
                setPage={(nextPage) => {
                  setPage(nextPage);
                  setStatus("");
                }}
                agentsExpanded={agentsExpanded}
                setAgentsExpanded={setAgentsExpanded}
                providersExpanded={providersExpanded}
                setProvidersExpanded={setProvidersExpanded}
              />
              <main className="settings-panel">
                {renderPanel()}
                {status && (
                  <div className="settings-global-status" role="status">
                    {status}
                  </div>
                )}
              </main>
            </div>
          </NativeWindowFrame>
        </WorkspaceGuideState>
      </Section>
    </>
  );
}
