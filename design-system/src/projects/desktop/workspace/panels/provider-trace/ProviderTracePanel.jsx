import { useState } from "react";

import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";

export function ProviderTracePanel({ onClose, standalone = false, state = "expanded" }) {
  const [selected, setSelected] = useState(state === "context-compaction" ? 1 : 0);
  const [expandedTurn, setExpandedTurn] = useState(state !== "turn-collapsed");
  const traces = [
    {
      title: "Response · gpt-5.6-sol",
      time: "14:32:18",
      status: "Completed",
      tokens: "18,420 → 1,284",
      contents: [
        {
          kind: "user",
          label: "USER",
          title: "User message",
          summary: "Add the Workspace surface to the design system.",
          time: "14:32:18",
        },
        {
          kind: "assistant",
          label: "ASSISTANT",
          title: "Assistant message",
          summary: "I’ll inspect the current Avalonia composition first.",
          time: "14:32:19",
        },
        {
          kind: "tool",
          label: "TOOL CALL",
          title: "Read ProjectWorkspace.axaml",
          summary: "Read 218 lines from the project workspace view.",
          time: "14:32:24",
        },
      ],
    },
    {
      title: "Context compaction",
      kind: "compaction",
      time: "14:32:07",
      status: "Completed",
      tokens: "18,420 → 12,160",
      contents: [
        {
          kind: "system",
          label: "CONTEXT",
          title: "Context summary",
          summary:
            "Reduced the earlier conversation to 12,160 retained tokens and dropped 6 messages.",
          time: "14:32:07",
        },
      ],
    },
    {
      title: "Tool continuation",
      time: "14:31:46",
      status: "Completed",
      tokens: "12,918 → 826",
      contents: [
        {
          kind: "user",
          label: "USER",
          title: "Tool result",
          summary: "The workspace view and its review drawer are available.",
          time: "14:31:46",
        },
        {
          kind: "assistant",
          label: "ASSISTANT",
          title: "Assistant message",
          summary: "I’ll split each workspace area into a focused route.",
          time: "14:31:48",
        },
        {
          kind: "tool",
          label: "TOOL CALL",
          title: "Update workspace routes",
          summary: "Updated 8 route modules and navigation entries.",
          time: "14:31:55",
        },
      ],
    },
    {
      title: "Initial request",
      time: "14:30:09",
      status: "Completed",
      tokens: "8,204 → 612",
      contents: [
        {
          kind: "user",
          label: "USER",
          title: "User message",
          summary: "Keep the major areas independently reachable from the sidebar.",
          time: "14:30:09",
        },
        {
          kind: "assistant",
          label: "ASSISTANT",
          title: "Assistant message",
          summary: "I’ll preserve the desktop composition and add stable child routes.",
          time: "14:30:12",
        },
        {
          kind: "tool",
          label: "TOOL CALL",
          title: "List design-system files",
          summary: "Found the desktop workspace modules and shared primitives.",
          time: "14:30:18",
        },
      ],
    },
  ];
  const noTurns = state === "no-turns";
  const activeTrace = traces[selected] ?? traces[0];
  return (
    <section className={`workspace-drawer workspace-trace ${standalone ? "is-standalone" : ""}`}>
      <header>
        <Icon name="activity" size={16} />
        <strong>Provider trace</strong>
        <span>{noTurns ? "0 turns" : `1 turn · ${traces.length} calls`}</span>
        <div />
        <IconButton icon="refresh" label="Refresh provider trace" disabled />
        <IconButton
          icon="copy"
          label="Copy trace"
          onClick={() => navigator.clipboard?.writeText("Provider trace preview")}
        />
        <IconButton
          icon="close"
          label="Close provider trace"
          onClick={onClose}
          disabled={!onClose}
        />
      </header>
      {noTurns ? (
        <div className="workspace-trace-empty">
          <Icon name="activity" size={24} />
          <strong>No turns yet</strong>
          <span>Provider requests will appear here after the agent starts a turn.</span>
        </div>
      ) : (
        <div className="workspace-trace-body">
          <div className="workspace-trace-list">
            <div className="workspace-drawer-label">CURRENT SESSION</div>
            <button
              type="button"
              className={`workspace-trace-turn ${expandedTurn ? "is-expanded" : ""}`}
              onClick={() => setExpandedTurn(!expandedTurn)}
            >
              <Icon name="chevron-right" className={expandedTurn ? "is-open" : ""} size={11} />
              <span>
                <strong>Turn 0198e82c</strong>
                <small>Completed · 1.84 s</small>
              </span>
              <b>{traces.length} calls</b>
            </button>
            {expandedTurn && (
              <div className="workspace-trace-children">
                {traces.map((trace, index) => (
                  <div key={trace.title}>
                    <button
                      type="button"
                      className={`workspace-trace-call ${trace.kind ? `is-${trace.kind}` : ""} ${selected === index ? "is-selected" : ""}`}
                      onClick={() => setSelected(index)}
                    >
                      <span>
                        <strong>{trace.title}</strong>
                        <small>{trace.time}</small>
                      </span>
                      <span>
                        <b>{trace.status}</b>
                        <small>{trace.tokens}</small>
                      </span>
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
          <div className="workspace-trace-detail">
            <div className="workspace-trace-title">
              <code>{activeTrace.title}</code>
              <span>
                {activeTrace.kind === "compaction" ? "context build" : "1.84 s  gpt-5.6-sol"}
              </span>
            </div>
            <div className="workspace-trace-metrics">
              {[
                ["INPUT", activeTrace.kind === "compaction" ? "18,420" : "18,420"],
                ["OUTPUT", activeTrace.kind === "compaction" ? "12,160" : "1,284"],
                ["CACHE READ", activeTrace.kind === "compaction" ? "0" : "12,160"],
                ["CACHE WRITE", "0"],
                ["CACHE HIT", activeTrace.kind === "compaction" ? "—" : "66%"],
                ["DURATION", activeTrace.kind === "compaction" ? "0.42 s" : "1.84 s"],
              ].map(([label, value]) => (
                <div key={label}>
                  <span>{label}</span>
                  <strong>{value}</strong>
                </div>
              ))}
            </div>
            <div className="workspace-trace-content">
              <div>
                <code>TURN 0198e82c · COMPLETED</code>
                <span>
                  {activeTrace.kind === "compaction"
                    ? "event  context.compacted"
                    : "exchange  exch_01JY7F6P8S"}
                </span>
              </div>
              <h4>{activeTrace.kind === "compaction" ? "Compaction result" : "Model response"}</h4>
              <pre>
                {activeTrace.kind === "compaction"
                  ? `{"status":"completed","event":"context.compacted","dropped_messages":6,"retained_tokens":12160}`
                  : `{"status":"completed","role":"assistant"}`}
              </pre>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
