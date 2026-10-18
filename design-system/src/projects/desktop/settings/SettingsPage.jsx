import { useState } from "react";
import { builtInAgentCatalog as agentCatalog } from "../../../agents/catalog.js";
import { Button } from "../../../components/universal/button/index.js";
import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { NativeWindowFrame } from "../../../platforms/desktop/components/titlebar/index.js";
import { WorkspaceGuideState } from "../workspace/WorkspaceGuide.jsx";
import { WindowSizeNote } from "../WindowSizeNote.jsx";
import { SettingsNav } from "./components/index.js";
import { providerCatalog } from "./data/catalog.js";
import { settingsGuide } from "./data/guide.js";
import { AppearancePanel } from "./sections/AppearancePanel.jsx";
import { DefaultsPanel } from "./sections/DefaultsPanel.jsx";
import { ShortcutsPanel } from "./sections/ShortcutsPanel.jsx";
import {
  AgentPanel,
  AgentsPanel,
  ProviderPanel,
  ProvidersPanel,
} from "./sections/AgentProviderPanels.jsx";
import { BrowserUsePanel } from "./sections/BrowserUsePanel.jsx";
import { ComputerUsePanel } from "./sections/ComputerUsePanel.jsx";
import { initialLanguageServers, LanguageServersPanel } from "./sections/LanguageServersPanel.jsx";
import { LoggingPanel } from "./sections/LoggingPanel.jsx";
import { initialMcpServers, McpServersPanel } from "./sections/McpServersPanel.jsx";
import { NetworkPanel } from "./sections/NetworkPanel.jsx";
import { RemoteServerPanel } from "./sections/RemoteServerPanel.jsx";
import "../styles/remote-server.css";
import "./styles/provider-credentials.css";

export function SettingsPage({
  initialPage = "defaults",
  initialLanguageServerId = null,
  languageServerHasProject = true,
  credentialReview = false,
}) {
  const [page, setPage] = useState(initialPage);
  const [agentsExpanded, setAgentsExpanded] = useState(false);
  const [providersExpanded, setProvidersExpanded] = useState(initialPage.startsWith("provider"));
  const [platform, setPlatform] = useState("macos");
  const [issue, setIssue] = useState("none");
  const [reviewVersion, setReviewVersion] = useState(0);
  const [credentials, setCredentials] = useState(() =>
    Object.fromEntries(
      Object.entries(providerCatalog).map(([id, provider]) => [
        id,
        {
          mask: provider.keyPreview,
          mode: provider.storageMode ?? "keychain",
        },
      ]),
    ),
  );
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
    if (page === "remote-server") return <RemoteServerPanel onSave={save} />;
    if (page === "computer") return <ComputerUsePanel onSave={save} />;
    if (page === "browser") return <BrowserUsePanel onSave={save} />;
    if (page === "mcp")
      return <McpServersPanel servers={mcpServers} setServers={setMcpServers} onSave={save} />;
    if (page === "lsp")
      return (
        <LanguageServersPanel
          servers={languageServers}
          setServers={setLanguageServers}
          onSave={save}
          initialSelectedId={initialLanguageServerId}
          hasProject={languageServerHasProject}
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
          credentials={credentials}
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
          key={`${page}-${reviewVersion}`}
          providerId={providerId}
          credential={credentials[providerId]}
          onCredentialChange={(credential) =>
            setCredentials((current) => ({ ...current, [providerId]: credential }))
          }
          platform={platform}
          issue={issue}
          onRetry={() => setIssue("none")}
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
        description="The Avalonia desktop settings window for local defaults, keyboard shortcuts, security, Computer Use, Browser Use, MCP servers, language servers, built-in agents, diagnostics, and provider credentials."
        path="projects/desktop/settings/"
      />
      {credentialReview && (
        <div className="credential-review-controls" aria-label="Credential specimen controls">
          <div>
            <strong>API key storage · Design preview</strong>
            <span>Fictional keys. Changes stay in memory; no operating-system access.</span>
          </div>
          <label>
            Platform
            <select
              className="field"
              value={platform}
              onChange={(event) => setPlatform(event.target.value)}
            >
              <option value="macos">macOS</option>
              <option value="windows">Windows</option>
              <option value="linux">Linux</option>
            </select>
          </label>
          <label>
            Review state
            <select
              className="field"
              value={issue}
              onChange={(event) => {
                setIssue(event.target.value);
                setReviewVersion((value) => value + 1);
              }}
            >
              <option value="none">Normal</option>
              <option value="locked">Keychain locked</option>
              <option value="denied">Access denied</option>
              <option value="unavailable">Service unavailable</option>
              <option value="missing">Saved entry missing</option>
              <option value="saving">Saving</option>
              <option value="save_failed">Save failed</option>
              <option value="remove_failed">Remove failed</option>
            </select>
          </label>
        </div>
      )}
      <WindowSizeNote width="900" height="672" minimumWidth="720" minimumHeight="552" />
      <Section
        id="settings-window"
        title="Desktop settings"
        description="A focused settings window whose outer title bar and controls are native to the operating system."
      >
        <WorkspaceGuideState
          className="settings-guide-state"
          title="Settings controls"
          description="Navigate local defaults, keyboard shortcuts, security, Computer Use, Browser Use, MCP servers, language servers, built-in agents, diagnostics, and provider credentials."
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
                credentials={credentials}
                page={page}
                agentCatalog={agentCatalog}
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
