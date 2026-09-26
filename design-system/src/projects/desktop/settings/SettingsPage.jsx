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
        description="The Avalonia desktop settings window for local defaults, keyboard shortcuts, security, Computer Use, Browser Use, MCP servers, language servers, built-in agents, diagnostics, and provider credentials."
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
