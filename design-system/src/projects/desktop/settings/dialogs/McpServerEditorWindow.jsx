import { useEffect, useId, useRef, useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";
import { NativeWindowFrame } from "../../../../platforms/desktop/components/titlebar/index.js";

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

export function McpServerEditorWindow({ open, server, onClose, onSubmit }) {
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
