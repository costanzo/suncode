import { useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { ConfirmationDialog } from "../../../../components/universal/modal/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import { LanguageServerEditorWindow } from "../dialogs/LanguageServerEditorWindow.jsx";
import { initialLanguageServers } from "../data/languageServers.js";

export { initialLanguageServers };

const languageServerStatusLabels = {
  ready: "Ready",
  indexing: "Indexing",
  failed: "Failed",
  disabled: "Disabled",
  not_started: "Not started",
};

export function LanguageServerSwitch({ server, onToggle }) {
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

export function LanguageServersPanel({ servers, setServers, onSave }) {
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
