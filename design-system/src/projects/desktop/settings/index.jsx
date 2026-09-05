import { useRef, useState } from "react";
import { Button } from "../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../components/universal/dropdown/index.js";
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
  { id: "network", label: "Network", icon: "platform" },
  { id: "logging", label: "Logging", icon: "assets" },
];

const settingsGuide = {
  tabs: {
    actions: [
      "Choose Defaults, Appearance, Network, or Logging from the left navigation.",
      "Use the chevron beside Model providers to collapse or expand its provider links.",
      "Select a provider to edit its OpenAI-compatible URL or credential.",
      "A provider is shown without a stored key so its recovery path and available models can be reviewed.",
      "Use Reset default to restore a provider's built-in URL.",
      "Use Network to review certificate verification, system trust, and custom certificate-path states.",
      "Use the folder buttons in Logging to choose log and image storage directories.",
      "Edit a control and use its save action; use Done to return to ProjectHub.",
    ],
    style: [
      "The operating system owns the title bar and window controls; the client toolbar is 58px high with 22px horizontal padding.",
      "The settings body uses a 238px navigation column and a content panel with 28px top / 32px side padding.",
      "Rows use 12px labels, 11px hints, 36px controls, 24px column gaps, and 16px section gaps.",
    ],
    logic: [
      "Settings are local to the embedded agent and are grouped by defaults, appearance, network, logging, and providers.",
      "Provider URL changes and default resets are persisted and applied to subsequent requests without changing credentials or models.",
      "Certificate-source controls are subordinate to HTTPS verification and switch between system trust and custom certificate-file input.",
      "Provider credentials are masked; only the first and last four characters are shown for recognition.",
      "A provider without a key keeps its model catalog visible but pauses sending until the key is saved.",
      "Saving updates local configuration state and does not grant new machine authority.",
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
                The models stay visible for selection, but sending is paused until this provider
                has a credential.
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
  const [guideOpen, setGuideOpen] = useState(false);
  const navigateBack = () => {
    window.location.hash = "/projects/desktop/project-hub";
  };
  const save = () => setStatus("Saved to the local agent.");
  const renderPanel = () => {
    if (page === "appearance") return <AppearancePanel onSave={save} />;
    if (page === "network") return <NetworkPanel onSave={save} />;
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
        description="The Avalonia desktop settings window for local defaults, security, diagnostics, and provider credentials."
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
          description="Navigate local defaults, security, diagnostics, and provider credentials."
          guide={settingsGuide}
          side="right"
          open={guideOpen}
          onToggle={() => setGuideOpen((open) => !open)}
          onClose={() => setGuideOpen(false)}
        >
          <NativeWindowFrame platform="macos" title="Settings" className="settings-window">
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
