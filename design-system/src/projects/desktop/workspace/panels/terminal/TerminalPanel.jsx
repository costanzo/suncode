import { useState } from "react";

import { Icon } from "../../../../../shared/Icon.jsx";

import { IconButton } from "../../shared/IconButton.jsx";

const promptLines = [
  { prompt: "~/Projects/suncode", command: "git status --short", output: " M design-system/src/projects/desktop/workspace" },
  { prompt: "~/Projects/suncode", command: "dotnet test apps/desktop-avalonia/tests", output: "Passed · 42 tests" },
];

export function TerminalPanel({ onClose, standalone = false, state = "running" }) {
  const [command, setCommand] = useState("");
  const [activeTab, setActiveTab] = useState("zsh");
  const [restarted, setRestarted] = useState(false);
  const isEmpty = state === "empty";
  const isExited = state === "exited";
  const isError = state === "error";

  const handleSubmit = (event) => {
    event.preventDefault();
    setCommand("");
  };

  return (
    <section
      className={`workspace-drawer workspace-terminal ${standalone ? "is-standalone" : ""} is-${state}`}
      aria-label="Terminal"
    >
      <header className="workspace-terminal-header">
        <div className="workspace-terminal-tabs" role="tablist" aria-label="Terminal sessions">
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "zsh"}
            className={activeTab === "zsh" ? "is-active" : ""}
            onClick={() => setActiveTab("zsh")}
          >
            <i aria-hidden="true" />
            <span>zsh</span>
          </button>
          <button
            type="button"
            className="workspace-terminal-tab-close"
            aria-label="Close zsh terminal session"
            title="Close terminal session"
          >
            <Icon name="close" size={11} />
          </button>
          <button type="button" className="workspace-terminal-new" aria-label="New terminal" title="New terminal">
            <Icon name="plus" size={13} />
          </button>
        </div>
        <code className="workspace-terminal-cwd">~/Projects/suncode</code>
        <div className="workspace-terminal-actions">
          <IconButton
            icon="refresh"
            label="Restart shell"
            onClick={() => setRestarted((value) => !value)}
          />
          <IconButton
            icon="close"
            label="Hide terminal drawer"
            onClick={onClose}
            disabled={!onClose}
          />
        </div>
      </header>

      {isEmpty ? (
        <div className="workspace-terminal-empty">
          <Icon name="terminal" size={24} />
          <strong>No project open</strong>
          <button type="button">Open project</button>
        </div>
      ) : isExited || isError ? (
        <div className="workspace-terminal-exited" role="status">
          <Icon name={isError ? "warning" : "terminal"} size={22} />
          <strong>{isError ? "Terminal failed to start" : "Shell exited"}</strong>
          <span>{isError ? "The local shell could not be started." : "Exit code 0 · 2 min ago"}</span>
          <button type="button" onClick={() => setRestarted((value) => !value)}>
            {restarted ? "Starting" : "Restart"}
          </button>
        </div>
      ) : (
        <div className="workspace-terminal-body">
          <div className="workspace-terminal-output" role="log" aria-live="polite">
            {promptLines.map((line) => (
              <div className="workspace-terminal-entry" key={line.command}>
                <div className="workspace-terminal-command">
                  <span className="workspace-terminal-prompt">{line.prompt} %</span>
                  <code>{line.command}</code>
                </div>
                <code className="workspace-terminal-result">{line.output}</code>
              </div>
            ))}
          </div>
          <form className="workspace-terminal-input" onSubmit={handleSubmit}>
            <span className="workspace-terminal-prompt">~/Projects/suncode %</span>
            <input
              aria-label="Terminal command"
              value={command}
              onChange={(event) => setCommand(event.target.value)}
              spellCheck="false"
              autoComplete="off"
              autoCapitalize="off"
            />
            <span className="workspace-terminal-caret" aria-hidden="true" />
          </form>
        </div>
      )}
    </section>
  );
}
