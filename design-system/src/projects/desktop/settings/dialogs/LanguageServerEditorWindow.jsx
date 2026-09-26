import { useEffect, useId, useRef, useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { NativeWindowFrame } from "../../../../platforms/desktop/components/titlebar/index.js";

const emptyLanguageServerDraft = {
  name: "",
  command: "",
  arguments: "",
  languageIds: "",
  rootMarkers: "",
  initializationOptions: "",
  environment: "",
  startupTimeout: "30",
  requestTimeout: "30",
  enabled: true,
};

export function LanguageServerEditorWindow({ open, server, onClose, onSubmit }) {
  const windowRef = useRef(null);
  const descriptionId = useId();
  const [draft, setDraft] = useState(() =>
    server
      ? {
          ...emptyLanguageServerDraft,
          name: server.name,
          command: server.command,
          arguments: server.arguments ?? "",
          languageIds: server.languageIds ?? "",
          rootMarkers: server.rootMarkers ?? "",
          initializationOptions: server.initializationOptions ?? "",
          startupTimeout: server.startupTimeout ?? emptyLanguageServerDraft.startupTimeout,
          requestTimeout: server.requestTimeout ?? emptyLanguageServerDraft.requestTimeout,
          enabled: server.enabled,
        }
      : emptyLanguageServerDraft,
  );
  const editing = Boolean(server);
  const update = (key, value) => setDraft((current) => ({ ...current, [key]: value }));
  const valid = draft.name.trim() && draft.command.trim() && draft.languageIds.trim();

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
    <div className="lsp-editor-window-shell">
      <NativeWindowFrame
        platform="macos"
        title={editing ? "Edit language server" : "Add language server"}
        width="640px"
        height="660px"
        onClose={onClose}
        className="lsp-editor-window"
      >
        <div
          ref={windowRef}
          className="lsp-editor-window-client"
          role="dialog"
          aria-modal="false"
          aria-label={editing ? "Edit language server" : "Add language server"}
          aria-describedby={descriptionId}
        >
          <header className="lsp-editor-heading">
            <h2>{editing ? "Edit language server" : "Add language server"}</h2>
            <p id={descriptionId}>
              Configure a structured local process. SunCode never invokes the executable through a
              shell.
            </p>
          </header>
          <div className="lsp-editor-scroll">
            <div className="lsp-server-form">
              <label className="lsp-form-field">
                <span>Server name</span>
                <input
                  className="field"
                  value={draft.name}
                  onChange={(event) => update("name", event.target.value)}
                  placeholder="rust-analyzer"
                  aria-label="Language server name"
                />
              </label>
              <label className="lsp-form-field">
                <span>Executable</span>
                <input
                  className="field mono"
                  value={draft.command}
                  onChange={(event) => update("command", event.target.value)}
                  placeholder="rust-analyzer"
                  spellCheck="false"
                  aria-label="Language server executable"
                />
              </label>
              <label className="lsp-form-field">
                <span>Arguments</span>
                <textarea
                  className="field mono"
                  value={draft.arguments}
                  onChange={(event) => update("arguments", event.target.value)}
                  placeholder="One argument per line"
                  aria-label="Language server arguments"
                />
              </label>
              <label className="lsp-form-field">
                <span>Language IDs</span>
                <textarea
                  className="field mono"
                  value={draft.languageIds}
                  onChange={(event) => update("languageIds", event.target.value)}
                  placeholder="One LSP language ID per line"
                  aria-label="Language IDs"
                />
                <small>At least one language ID is required.</small>
              </label>
              <label className="lsp-form-field">
                <span>Root markers</span>
                <textarea
                  className="field mono"
                  value={draft.rootMarkers}
                  onChange={(event) => update("rootMarkers", event.target.value)}
                  placeholder="Cargo.toml"
                  aria-label="Project root markers"
                />
                <small>One project-relative file or directory name per line.</small>
              </label>
              <label className="lsp-form-field">
                <span>Initialization options</span>
                <textarea
                  className="field mono"
                  value={draft.initializationOptions}
                  onChange={(event) => update("initializationOptions", event.target.value)}
                  placeholder="Optional JSON object"
                  aria-label="Language server initialization options"
                  spellCheck="false"
                />
              </label>
              <label className="lsp-form-field lsp-form-wide">
                <span>Environment</span>
                <textarea
                  className="field mono"
                  value={draft.environment}
                  onChange={(event) => update("environment", event.target.value)}
                  placeholder={editing ? "NAME=value replaces; -NAME removes" : "NAME=value"}
                  aria-label="Language server environment entries"
                />
                <small>
                  Values are stored locally and hidden after saving. Use -NAME to remove a stored
                  key.
                </small>
              </label>
              <label className="lsp-form-field">
                <span>Startup timeout</span>
                <span className="settings-unit-field">
                  <input
                    className="field mono"
                    type="number"
                    min="1"
                    max="120"
                    value={draft.startupTimeout}
                    onChange={(event) => update("startupTimeout", event.target.value)}
                    aria-label="Language server startup timeout in seconds"
                  />
                  <span>SEC</span>
                </span>
              </label>
              <label className="lsp-form-field">
                <span>Request timeout</span>
                <span className="settings-unit-field">
                  <input
                    className="field mono"
                    type="number"
                    min="1"
                    max="600"
                    value={draft.requestTimeout}
                    onChange={(event) => update("requestTimeout", event.target.value)}
                    aria-label="Language server request timeout in seconds"
                  />
                  <span>SEC</span>
                </span>
              </label>
              <div className="lsp-form-enable lsp-form-wide">
                <div>
                  <strong>Enable server</strong>
                  <span>Starts one project-scoped process when a matching project is active.</span>
                </div>
                <label className="settings-switch">
                  <input
                    type="checkbox"
                    checked={draft.enabled}
                    onChange={(event) => update("enabled", event.target.checked)}
                    aria-label="Enable language server after saving"
                  />
                  <span className="settings-switch-track">
                    <span />
                  </span>
                  <b>{draft.enabled ? "On" : "Off"}</b>
                </label>
              </div>
            </div>
          </div>
          <footer className="lsp-editor-actions">
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
