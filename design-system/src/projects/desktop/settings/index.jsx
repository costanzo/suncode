import { useState } from "react";
import { Button } from "../../../components/universal/button/index.js";
import { Icon } from "../../../shared/Icon.jsx";
import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";

const providerCatalog = {
  deepseek: { label: "DeepSeek", placeholder: "Paste DeepSeek API key", keyPreview: "sk-d••••••••7K2m", models: ["deepseek-v4-flash", "deepseek-v4-pro"] },
  zhipu: { label: "Zhipu GLM", placeholder: "Paste Zhipu API key", keyPreview: "zhip••••••••M8qR", models: ["glm-5.2", "glm-5.3"] },
  openai: { label: "OpenAI", placeholder: "Paste OpenAI API key", keyPreview: "sk-p••••••••9Xc4", models: ["gpt-5.5", "gpt-5.6-sol"] },
  kimi: { label: "Kimi", placeholder: "Paste Kimi API key", keyPreview: "sk-k••••••••3FvP", models: ["kimi-k2.7-code", "kimi-k3"] },
  claude: { label: "Claude", placeholder: "Paste Anthropic API key", keyPreview: "sk-a••••••••6NwQ", models: ["claude-sonnet-5", "claude-opus-5"] },
  gemini: { label: "Gemini", placeholder: "Paste Gemini API key", keyPreview: "AIza••••••••2Lm8", models: ["gemini-3.5", "gemini-3.6-flash"] },
};

const navGroups = [
  { label: "General", items: [{ id: "defaults", label: "Defaults", icon: "foundation" }, { id: "appearance", label: "Appearance", icon: "sun" }, { id: "network", label: "Network", icon: "platform" }, { id: "logging", label: "Logging", icon: "assets" }] },
];

function TrafficLights({ onClose, onAction }) {
  return <div className="settings-traffic-lights" aria-label="Window controls"><button type="button" className="traffic-light close" aria-label="Close settings" title="Close settings" onClick={onClose} /><button type="button" className="traffic-light minimize" aria-label="Minimize settings" title="Minimize settings" onClick={() => onAction("Minimize is available in the desktop window.")} /><button type="button" className="traffic-light maximize" aria-label="Maximize settings" title="Maximize settings" onClick={() => onAction("Maximize is available in the desktop window.")} /></div>;
}

function SettingRow({ label, hint, children, className = "" }) {
  return <div className={`settings-row ${className}`}><div className="settings-row-copy"><strong>{label}</strong>{hint && <span>{hint}</span>}</div><div className="settings-row-control">{children}</div></div>;
}

function SettingsNav({ page, setPage, providersExpanded, setProvidersExpanded }) {
  return <aside className="settings-nav"><nav aria-label="Settings sections">
    {navGroups.map((group) => <div className="settings-nav-group" key={group.label}><span className="settings-nav-label" role="heading" aria-level="3">{group.label}</span>{group.items.map((item) => <button key={item.id} type="button" aria-current={page === item.id ? "page" : undefined} className={`settings-nav-item ${page === item.id ? "is-selected" : ""}`} onClick={() => setPage(item.id)}><Icon name={item.icon} size={15} /><span>{item.label}</span></button>)}</div>)}
    <div className="settings-nav-group settings-nav-models"><span className="settings-nav-label" role="heading" aria-level="3">Models</span><button type="button" className="settings-nav-item settings-nav-parent" aria-expanded={providersExpanded} aria-controls="settings-provider-list" onClick={() => setProvidersExpanded((open) => !open)}><Icon name="chevron-right" size={14} className={providersExpanded ? "is-rotated" : ""} /><span>Model providers</span></button>{providersExpanded && <div className="settings-provider-list" id="settings-provider-list">{Object.entries(providerCatalog).map(([id, provider]) => <button key={id} type="button" aria-current={page === `provider:${id}` ? "page" : undefined} className={`settings-nav-item settings-nav-provider ${page === `provider:${id}` ? "is-selected" : ""}`} onClick={() => setPage(`provider:${id}`)}><span>{provider.label}</span></button>)}</div>}</div>
  </nav></aside>;
}

function DefaultsPanel({ onSave }) {
  const models = Object.values(providerCatalog).flatMap((provider) => provider.models);
  return <div className="settings-panel-content"><div className="settings-panel-heading"><h2>Defaults</h2><p>Configure model and turn defaults.</p></div><div className="settings-panel-section"><span className="settings-section-label">Default model</span><SettingRow label="Model" hint="Only models registered by the local agent appear here."><select className="field settings-select" defaultValue="deepseek-v4-flash" aria-label="Default model">{models.map((model) => <option value={model} key={model}>{model}</option>)}</select></SettingRow></div><div className="settings-divider" /><div className="settings-panel-section"><span className="settings-section-label">Turn execution</span><SettingRow label="Tool-call limit" hint="Maximum number of tool calls allowed in one turn."><input className="field settings-number" type="number" min="1" max="256" defaultValue="64" aria-label="Tool-call limit" /></SettingRow><div className="settings-actions"><Button variant="primary" size="sm" onClick={onSave}>Save project limit</Button><span className="settings-save-status" role="status">Project: suncode</span></div></div></div>;
}

function AppearancePanel({ onSave }) {
  const [theme, setTheme] = useState(() => document.documentElement.dataset.theme || "light");
  const applyTheme = (event) => { const nextTheme = event.target.value; setTheme(nextTheme); document.documentElement.dataset.theme = nextTheme; try { window.localStorage.setItem("suncode-design-theme", nextTheme); } catch { /* non-fatal */ } onSave(); };
  return <div className="settings-panel-content"><div className="settings-panel-heading"><h2>Appearance</h2><p>Adjust how SunCode looks across every open window.</p></div><div className="settings-panel-section"><span className="settings-section-label">Theme</span><SettingRow label="Color theme" hint="Changes apply immediately."><select className="field settings-select" value={theme} onChange={applyTheme} aria-label="Color theme"><option value="dark">Dark</option><option value="light">Light</option></select></SettingRow></div><div className="settings-actions"><span className="settings-save-status" role="status">{theme === "light" ? "Light" : "Dark"} theme active</span></div></div>;
}

function NetworkPanel({ onSave }) {
  const [verify, setVerify] = useState(true);
  return <div className="settings-panel-content"><div className="settings-panel-heading"><h2>Network</h2><p>Configure how Rust HTTPS clients establish secure connections.</p></div><div className="settings-panel-section"><span className="settings-section-label">HTTPS security</span><SettingRow label="Verify server certificates" hint="Validate certificate chains and hostnames for model provider and WebFetch requests."><label className="settings-switch"><input type="checkbox" checked={verify} onChange={(event) => setVerify(event.target.checked)} aria-label="Verify server certificates" /><span className="settings-switch-track"><span /></span><b>{verify ? "On" : "Off"}</b></label></SettingRow>{!verify && <div className="settings-warning"><Icon name="platform" size={16} /><div><strong>Certificate verification is off</strong><span>SunCode will accept invalid certificates and hostnames, similar to <code>curl -k</code>. This can expose provider credentials and fetched content to man-in-the-middle attacks.</span></div></div>}</div><div className="settings-actions"><Button variant="primary" size="sm" onClick={onSave}>Save HTTPS setting</Button><span className="settings-save-status" role="status">{verify ? "Verification enabled" : "Review required"}</span></div></div>;
}

function LoggingPanel({ onSave }) {
  return <div className="settings-panel-content"><div className="settings-panel-heading"><h2>Logging</h2><p>Control diagnostic detail and how long local log files are kept.</p></div><div className="settings-panel-section"><span className="settings-section-label">Diagnostic output</span><SettingRow label="Minimum level" hint="Lower levels include more diagnostic detail."><select className="field settings-select" defaultValue="INFO" aria-label="Minimum log level"><option>TRACE</option><option>DEBUG</option><option>INFO</option><option>WARN</option><option>ERROR</option><option>OFF</option></select></SettingRow><SettingRow label="Log directory" hint="Leave empty to use the data directory's logs folder."><input className="field settings-text" placeholder="Default logs folder" aria-label="Log directory" /></SettingRow><SettingRow label="Maximum file size" hint="Rotate each file after it reaches this many bytes. Minimum 1024."><input className="field settings-text mono" defaultValue="10485760" aria-label="Maximum file size" /></SettingRow><SettingRow label="Retained backups" hint="Number of rotated backups to keep, from 0 to 100."><input className="field settings-number" type="number" min="0" max="100" defaultValue="5" aria-label="Retained backups" /></SettingRow></div><div className="settings-actions"><Button variant="primary" size="sm" onClick={onSave}>Save logging settings</Button><span className="settings-save-status" role="status">Local settings</span></div></div>;
}

function maskApiKey(value) {
  const key = value.trim();
  if (key.length <= 8) return `${key.slice(0, 2)}••••${key.slice(-2)}`;
  return `${key.slice(0, 4)}••••••••${key.slice(-4)}`;
}

function ProviderPanel({ providerId, onSave }) {
  const provider = providerCatalog[providerId];
  const [maskedKey, setMaskedKey] = useState(provider.keyPreview);
  const [apiKey, setApiKey] = useState("");
  const configured = Boolean(maskedKey);
  const saveKey = () => { setMaskedKey(maskApiKey(apiKey)); setApiKey(""); onSave(); };
  const removeKey = () => { setMaskedKey(""); setApiKey(""); onSave(); };
  return <div className="settings-panel-content"><div className="settings-panel-heading"><h2>{provider.label}</h2></div><div className="settings-panel-section"><span className="settings-section-label">Credential</span><div className={`settings-credential-status ${configured ? "is-configured" : ""}`}><span className="settings-status-dot" /><div>{configured ? <><strong>API key configured</strong><code aria-label={`API key starts with ${maskedKey.slice(0, 4)} and ends with ${maskedKey.slice(-4)}`}>{maskedKey}</code></> : <><strong>No API key configured</strong><small>Add a key to enable this provider.</small></>}</div></div><input className="field settings-key mono" type="password" value={apiKey} onChange={(event) => setApiKey(event.target.value)} placeholder={configured ? "Paste a new key to replace it" : provider.placeholder} aria-label={`${provider.label} API key`} autoComplete="new-password" /><div className="settings-actions"><Button variant="primary" size="sm" disabled={!apiKey.trim()} onClick={saveKey}>{configured ? "Replace key" : "Save key"}</Button><Button variant="danger" size="sm" onClick={removeKey} disabled={!configured}>Remove key</Button></div></div><div className="settings-divider" /><div className="settings-panel-section"><span className="settings-section-label">Available models</span><div className="settings-model-list">{provider.models.map((model) => <code key={model}>{model}</code>)}</div></div></div>;
}

export function SettingsPage() {
  const [page, setPage] = useState("defaults");
  const [providersExpanded, setProvidersExpanded] = useState(true);
  const [status, setStatus] = useState("");
  const navigateBack = () => { window.location.hash = "/projects/desktop/project-hub"; };
  const save = () => setStatus("Saved to the local agent.");
  const renderPanel = () => {
    if (page === "appearance") return <AppearancePanel onSave={save} />;
    if (page === "network") return <NetworkPanel onSave={save} />;
    if (page === "logging") return <LoggingPanel onSave={save} />;
    if (page.startsWith("provider:")) return <ProviderPanel key={page} providerId={page.split(":")[1]} onSave={save} />;
    return <DefaultsPanel onSave={save} />;
  };
  return <><PageHeader title="Settings" description="The Avalonia desktop settings window for local defaults, security, diagnostics, and provider credentials." status="Phase 1" tone="implemented" path="projects/desktop/settings/" /><Section id="settings-window" title="Desktop settings" description="A focused settings window with stable pages for each configuration area."><div className="settings-window"><div className="settings-titlebar"><TrafficLights onClose={navigateBack} onAction={setStatus} /><strong>Settings</strong></div><div className="settings-toolbar"><strong>Settings</strong><Button variant="primary" size="sm" onClick={navigateBack}>Done</Button></div><div className="settings-body"><SettingsNav page={page} setPage={(nextPage) => { setPage(nextPage); setStatus(""); }} providersExpanded={providersExpanded} setProvidersExpanded={setProvidersExpanded} /><main className="settings-panel">{renderPanel()}{status && <div className="settings-global-status" role="status">{status}</div>}</main></div></div></Section></>;
}
