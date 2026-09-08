import { useEffect, useId, useRef, useState } from "react";
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
  { id: "mcp", label: "MCP servers", icon: "server" },
  { id: "logging", label: "Logging", icon: "assets" },
];

const settingsGuide = {
  tabs: {
    actions: [
      "Choose Defaults, Appearance, Keyboard shortcuts, Network, MCP servers, or Logging from the left navigation.",
      "Use the chevron beside Model providers to collapse or expand its provider links.",
      "Select a provider to edit its OpenAI-compatible URL or credential.",
      "A provider is shown without a stored key so its recovery path and available models can be reviewed.",
      "Use Reset default to restore a provider's built-in URL.",
      "Use Network to review certificate verification, system trust, and custom certificate-path states.",
      "Use the folder buttons in Logging to choose log and image storage directories.",
      "Use MCP servers to add, edit, delete, enable, disable, and retry local or remote servers.",
      "Edit a control and use its save action; use Done to return to ProjectHub.",
      "Keyboard shortcuts are shown as read-only key combinations; customization is reserved for a future release.",
    ],
    style: [
      "The operating system owns the title bar and window controls; the client toolbar is 58px high with 22px horizontal padding.",
      "The settings body uses a 238px navigation column and a content panel with 28px top / 32px side padding.",
      "Rows use 12px labels, 11px hints, 36px controls, 24px column gaps, and 16px section gaps.",
    ],
    logic: [
      "Settings are local to the embedded agent and are grouped by defaults, appearance, keyboard shortcuts, network, logging, and providers.",
      "Provider URL changes and default resets are persisted and applied to subsequent requests without changing credentials or models.",
      "Certificate-source controls are subordinate to HTTPS verification and switch between system trust and custom certificate-file input.",
      "Provider credentials are masked; only the first and last four characters are shown for recognition.",
      "A provider without a key keeps its model catalog visible but pauses sending until the key is saved.",
      "Saving updates local configuration state and does not grant new machine authority.",
      "MCP configuration is persisted by the embedded Rust agent; the effective tool catalog is refreshed without starting a new session.",
      "Connected, connecting, failed, and disabled are runtime states. Enabled is the persisted desired state.",
      "MCP server definitions are global, while the status shown belongs to the current project connection.",
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

function SettingsNav({ page, setPage, providersExpanded, setProvidersExpanded }) {
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
        <p>Configure how Rust HTTPS clients establish secure connections.</p>
      </div>
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
                <small>Values are stored locally and hidden after saving. Use -NAME to remove a stored key.</small>
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

export function SettingsPage() {
  const [page, setPage] = useState("defaults");
  const [providersExpanded, setProvidersExpanded] = useState(true);
  const [providerEndpoints, setProviderEndpoints] = useState(() =>
    Object.fromEntries(
      Object.entries(providerCatalog).map(([id, provider]) => [id, provider.endpoint]),
    ),
  );
  const [status, setStatus] = useState("");
  const [mcpServers, setMcpServers] = useState(initialMcpServers);
  const [guideOpen, setGuideOpen] = useState(false);
  const navigateBack = () => {
    window.location.hash = "/projects/desktop/project-hub";
  };
  const save = (message = "Saved to the local agent.") => setStatus(message);
  const renderPanel = () => {
    if (page === "appearance") return <AppearancePanel onSave={save} />;
    if (page === "shortcuts") return <ShortcutsPanel />;
    if (page === "network") return <NetworkPanel onSave={save} />;
    if (page === "mcp")
      return <McpServersPanel servers={mcpServers} setServers={setMcpServers} onSave={save} />;
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
        description="The Avalonia desktop settings window for local defaults, keyboard shortcuts, security, MCP servers, diagnostics, and provider credentials."
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
          description="Navigate local defaults, keyboard shortcuts, security, MCP servers, diagnostics, and provider credentials."
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
