import { useEffect, useState } from "react";

import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";

const toolActivityTurns = [
  {
    id: "0198e82c",
    title: "Turn 0198e82c",
    preview: "Add the Workspace surface to the design system, but keep each major area…",
    status: "Running",
    tools: [
      {
        title: "Read ProjectWorkspace.axaml",
        state: "Completed",
        tone: "success",
        request: "apps/desktop-avalonia/Views/ProjectWorkspace/ProjectWorkspace.axaml",
        result: "218 lines read",
      },
      {
        title: "Run mvn compile for the workspace shell specimen",
        state: "Running",
        tone: "running",
        request: "mvn -pl design-system compile",
        result: "Command still running",
        live: [
          "[INFO] Scanning for projects...",
          "[INFO] Building design-system 0.0.0-review",
          "> vite build",
          "transforming modules...",
          "rendering chunks...",
        ],
      },
      {
        title: "Update workspace routes and modules",
        state: "Queued",
        tone: "queued",
        request: "design-system/src/app/navigation.js",
        result: "Waiting for the running compile command to finish",
      },
      {
        title: "Read missing workspace manifest",
        state: "Failed",
        tone: "danger",
        request: "design-system/src/projects/desktop/workspace/manifest.json",
        result: "The file could not be found",
        error: "file_not_found",
      },
    ],
  },
];

const completedToolActivityTurns = [
  {
    id: "0198e7f1",
    title: "Turn 0198e7f1",
    preview: "Review the current Avalonia workspace layout and summarize the supporting bays…",
    status: "Completed",
    tools: [
      {
        title: "List design-system files",
        state: "Completed",
        tone: "success",
        request: "design-system/src/projects/desktop/workspace",
        result: "42 files found",
      },
      {
        title: "Search provider trace bindings",
        state: "Completed",
        tone: "success",
        request: "rg ProviderTrace apps/desktop-avalonia",
        result: "14 provider trace bindings found",
      },
      {
        title: "Read ProviderTraceViewer.axaml",
        state: "Completed",
        tone: "success",
        request: "apps/desktop-avalonia/Views/ProjectWorkspace/Review/ProviderTraceViewer.axaml",
        result: "302 lines read",
      },
    ],
  },
];

export function ToolActivityPanel({ onClose, standalone = false, state = "running" }) {
  const empty = state === "empty";
  const turns = empty ? [] : state === "completed" ? completedToolActivityTurns : toolActivityTurns;
  const initialTurn = 0;
  const initialTool = state === "running" ? 1 : 0;
  const [selectedTurn, setSelectedTurn] = useState(initialTurn);
  const [selectedTool, setSelectedTool] = useState(initialTool);
  const [expandedTurns, setExpandedTurns] = useState(() => new Set([initialTurn]));
  const [visibleOutputLines, setVisibleOutputLines] = useState(2);
  const turn = turns[selectedTurn] ?? turns[0];
  const tool = turn?.tools[selectedTool] ?? turn?.tools[0];
  useEffect(() => {
    setVisibleOutputLines(2);
    if (!tool?.live) return undefined;
    const timer = window.setInterval(
      () =>
        setVisibleOutputLines((current) => {
          if (current >= tool.live.length) {
            window.clearInterval(timer);
            return current;
          }
          return current + 1;
        }),
      680,
    );
    return () => window.clearInterval(timer);
  }, [selectedTurn, selectedTool, tool?.live]);
  return (
    <section
      className={`workspace-drawer workspace-tool-activity ${standalone ? "is-standalone" : ""}`}
    >
      <header>
        <Icon name="tool" size={16} />
        <strong>Tool activity</strong>
        <span>
          {empty
            ? "0 turns"
            : `${turns.length} turn · ${turns.reduce((sum, item) => sum + item.tools.length, 0)} calls`}
        </span>
        <div />
        <div className="workspace-tool-activity-actions">
          <IconButton
            icon="copy"
            label="Copy tool activity"
            onClick={() => navigator.clipboard?.writeText("Tool activity preview")}
          />
          <IconButton
            icon="close"
            label="Close tool activity"
            onClick={onClose}
            disabled={!onClose}
          />
        </div>
      </header>
      {empty ? (
        <div className="workspace-tool-empty">
          <Icon name="tool" size={24} />
          <strong>No turns yet</strong>
          <span>Tool calls will appear here after the agent starts its first turn.</span>
        </div>
      ) : (
        <div className="workspace-tool-body">
          <div className="workspace-tool-tree">
            <div className="workspace-drawer-label">CURRENT SESSION</div>
            {turns.map((item, turnIndex) => (
              <div key={item.id}>
                <button
                  type="button"
                  className={`workspace-tool-turn ${selectedTurn === turnIndex ? "is-selected" : ""}`}
                  aria-expanded={expandedTurns.has(turnIndex)}
                  onClick={() => {
                    setSelectedTurn(turnIndex);
                    setSelectedTool(0);
                    setExpandedTurns((current) => {
                      const next = new Set(current);
                      if (next.has(turnIndex)) next.delete(turnIndex);
                      else next.add(turnIndex);
                      return next;
                    });
                  }}
                >
                  <Icon
                    name="chevron-right"
                    className={expandedTurns.has(turnIndex) ? "is-open" : ""}
                    size={11}
                  />
                  <span>
                    <strong>{item.title}</strong>
                    <small>{item.preview}</small>
                  </span>
                  <b>{item.tools.length}</b>
                </button>
                {expandedTurns.has(turnIndex) && (
                  <div className="workspace-tool-children">
                    {item.tools.map((entry, toolIndex) => (
                      <button
                        type="button"
                        key={entry.title}
                        className={`workspace-tool-tree-row is-${entry.tone} ${selectedTurn === turnIndex && selectedTool === toolIndex ? "is-selected" : ""}`}
                        onClick={() => {
                          setSelectedTurn(turnIndex);
                          setSelectedTool(toolIndex);
                        }}
                      >
                        <Icon name={entry.tone === "running" ? "tool" : "activity"} size={13} />
                        <span>
                          <strong>{entry.title}</strong>
                        </span>
                        {entry.tone !== "success" && entry.tone !== "danger" && (
                          <i className={`is-${entry.tone}`} aria-label={entry.state} />
                        )}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
          <div className="workspace-tool-detail">
            <div className="workspace-tool-detail-heading">
              <div>
                <code>
                  {turn.title} · {tool.state.toUpperCase()}
                </code>
                <h3>{tool.title}</h3>
              </div>
              <span className={`workspace-tool-state is-${tool.tone}`}>{tool.state}</span>
            </div>
            <div className="workspace-tool-meta">
              <span>TOOL CALL</span>
              <code>
                {turn.id} · {selectedTool + 1} of {turn.tools.length}
              </code>
              <span>STATUS</span>
              <code>{tool.state}</code>
            </div>
            <div className="workspace-tool-detail-section">
              <span>Request</span>
              <code>{tool.request}</code>
            </div>
            {tool.live && (
              <div className="workspace-tool-detail-section">
                <div className="workspace-tool-live-heading">
                  <span>Live output</span>
                  <small>Following tail · {visibleOutputLines} lines</small>
                </div>
                <pre className="workspace-tool-live-output" aria-live="polite">
                  <code>{tool.live.slice(0, visibleOutputLines).join("\n")}</code>
                </pre>
              </div>
            )}
            <div className="workspace-tool-detail-section">
              <span>{tool.live ? "Latest status" : "Result"}</span>
              <code>{tool.result}</code>
            </div>
            {tool.error && (
              <div className="workspace-tool-detail-section is-error">
                <span>Error</span>
                <code>{tool.error}</code>
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  );
}
