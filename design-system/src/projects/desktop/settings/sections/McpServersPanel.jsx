import { useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { ConfirmationDialog } from "../../../../components/universal/modal/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import { McpServerEditorWindow } from "../dialogs/McpServerEditorWindow.jsx";
import { initialMcpServers } from "../data/mcpServers.js";

export { initialMcpServers };

const mcpStatusLabels = {
  connected: "Connected",
  connecting: "Connecting",
  failed: "Failed",
  disabled: "Disabled",
};

export function McpSwitch({ server, onToggle }) {
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

export function McpServersPanel({ servers, setServers, onSave }) {
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
