import { ModuleLink, PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { Icon } from "../../../shared/Icon.jsx";
import { McpLoadingStatus, WorkspaceWindow } from "./shell/index.js";
import { WindowSizeNote } from "../WindowSizeNote.jsx";

export function WorkspacePage() {
  return (
    <>
      <PageHeader
        title="Workspace"
        description="The active project window, composed from the Avalonia session, conversation, review, and observability surfaces."
      />
      <WindowSizeNote width="1440" height="900" minimumWidth="620" minimumHeight="620" />
      <Section id="workspace-window" title="Project workspace">
        <WorkspaceWindow />
      </Section>
      <Section
        id="workspace-window-tabs"
        title="Project window tabs"
        description="Window > Merge All Windows groups open project windows into one workspace. Tabs keep each project's independent session and panel state, expose a close action on hover, and can be dragged beyond the tear-off threshold to return to a separate window."
      >
        <div className="workspace-tabs-specimen" role="tablist" aria-label="Open projects">
          <div className="workspace-project-tab-shell">
            <button className="workspace-project-tab is-active" role="tab" aria-selected="true">
              <span className="workspace-project-tab-mark" />
              <span>suncode</span>
            </button>
            <button
              className="workspace-project-tab-close"
              type="button"
              aria-label="Close suncode project"
              title="Close project"
            >
              <Icon name="close" size={12} />
            </button>
          </div>
          <div className="workspace-project-tab-shell">
            <button className="workspace-project-tab" role="tab" aria-selected="false">
              <span className="workspace-project-tab-mark" />
              <span>desktop-avalonia</span>
            </button>
            <button
              className="workspace-project-tab-close"
              type="button"
              aria-label="Close desktop-avalonia project"
              title="Close project"
            >
              <Icon name="close" size={12} />
            </button>
          </div>
          <div className="workspace-project-tab-shell">
            <button className="workspace-project-tab" role="tab" aria-selected="false">
              <span className="workspace-project-tab-mark" />
              <span>design-system</span>
            </button>
            <button
              className="workspace-project-tab-close"
              type="button"
              aria-label="Close design-system project"
              title="Close project"
            >
              <Icon name="close" size={12} />
            </button>
          </div>
        </div>
        <p className="workspace-tabs-note">Window menu: Merge All Windows</p>
      </Section>
      <Section id="workspace-modules" title="Focused workspace surfaces">
        <div className="module-card-grid">
          <ModuleLink
            to="/projects/desktop/workspace/child-sessions"
            icon="agent"
            title="Child sessions"
            description="Linked delegated sessions, their right-side list, read-only detail, and recent-content behavior."
            path="Right bay + central detail"
          />
        </div>
      </Section>
      <Section
        id="workspace-mcp-loading"
        title="MCP startup feedback"
        description="A compact footer indicator keeps the workspace usable while project-scoped MCP servers connect in the background."
      >
        <div className="workspace-mcp-specimen-grid">
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Loading</strong>
              <span>Two of five servers have settled.</span>
            </div>
            <div className="workspace-mcp-specimen-footer">
              <McpLoadingStatus settled={2} total={5} connected={2} />
              <code>gpt-5.6-sol</code>
            </div>
          </div>
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Partial failure</strong>
              <span>Failed servers count as settled; chat remains available.</span>
            </div>
            <div className="workspace-mcp-specimen-footer">
              <McpLoadingStatus settled={4} total={5} connected={3} failed={1} />
              <code>gpt-5.6-sol</code>
            </div>
          </div>
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Complete</strong>
              <span>All servers are connected or terminally failed.</span>
            </div>
            <div className="workspace-mcp-specimen-footer is-complete">
              <span className="workspace-mcp-complete-note">
                Footer returns to its normal quiet state.
              </span>
              <code>gpt-5.6-sol</code>
            </div>
          </div>
        </div>
      </Section>
      <Section
        id="workspace-provider-traffic"
        title="Provider transfer status"
        description="Live HTTP body transfer rates for active LLM requests, refreshed every second using a one-second averaging interval."
      >
        <div className="workspace-mcp-specimen-grid">
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Active exchange</strong>
              <span>Rates use provider request and response body bytes.</span>
            </div>
            <div
              className="workspace-mcp-specimen-footer workspace-provider-traffic-specimen"
              title="OpenAI / gpt-5.6-sol: 12.4 KiB uploaded, 84.2 KiB downloaded"
            >
              <span>↑ 12.4 KiB/s&nbsp;&nbsp; ↓ 84.2 KiB/s</span>
              <code>gpt-5.6-sol</code>
            </div>
          </div>
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Idle</strong>
              <span>Zero rates stay visible between provider exchanges.</span>
            </div>
            <div className="workspace-mcp-specimen-footer is-complete">
              <span className="workspace-provider-traffic-zero">↑ 0 B/s&nbsp;&nbsp; ↓ 0 B/s</span>
              <code>gpt-5.6-sol</code>
            </div>
          </div>
        </div>
      </Section>
      <Section
        id="workspace-running-session-status"
        title="Running session count"
        description="The project footer keeps the session count visible while a project is open, including when no session is running."
      >
        <div className="workspace-mcp-specimen-grid">
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Idle</strong>
              <span>A single static dot precedes the zero count.</span>
            </div>
            <div className="workspace-mcp-specimen-footer workspace-running-session-specimen">
              <span className="workspace-running-session-status">
                <i aria-hidden="true" />
                <code>0 running</code>
              </span>
            </div>
          </div>
          <div className="workspace-mcp-specimen">
            <div className="workspace-mcp-specimen-label">
              <strong>Active</strong>
              <span>The original running pulse returns while sessions are active.</span>
            </div>
            <div className="workspace-mcp-specimen-footer workspace-running-session-specimen">
              <span className="workspace-running-session-status is-running">
                <i aria-hidden="true" />
                <code>2 running</code>
              </span>
            </div>
          </div>
        </div>
      </Section>
    </>
  );
}
