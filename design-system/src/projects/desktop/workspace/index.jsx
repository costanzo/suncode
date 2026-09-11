import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { McpLoadingStatus, WorkspaceWindow } from "./WorkspacePrimitives.jsx";
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
              <span className="workspace-mcp-complete-note">Footer returns to its normal quiet state.</span>
              <code>gpt-5.6-sol</code>
            </div>
          </div>
        </div>
      </Section>
    </>
  );
}
