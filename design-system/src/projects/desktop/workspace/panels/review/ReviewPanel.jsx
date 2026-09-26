import { useState } from "react";
import { Button } from "../../../../../components/universal/button/index.js";

import { Radio } from "../../../../../components/universal/radio/index.js";
import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";
import {
  changes,
  completedTurnChangeSet,
  completedTurnChanges,
  currentTurnTodos,
} from "../../data/review.js";

export function ReviewPanel({ compact = false, standalone = false, state = "approval" }) {
  const running = state === "running" || state === "running-no-changes";
  const compacting = state === "compacting";
  const waiting = state === "approval" || state === "question";
  const idle = state === "idle";
  const failed = state === "failed";
  const noChanges = state === "running-no-changes";
  const inactive = idle;
  const statusTone = idle
    ? "idle"
    : failed
      ? "failed"
      : compacting
        ? "compacting"
        : running
          ? "running"
          : state === "approval"
            ? "approval"
            : "question";
  const statusLabel = idle
    ? "Agent idle"
    : failed
      ? "Turn failed"
      : compacting
        ? "Compacting conversation context"
        : running
          ? noChanges
            ? "Agent running, no file changes"
            : "Agent running"
          : state === "approval"
            ? "Waiting for approval"
            : "Waiting for answer";
  const [questionOption, setQuestionOption] = useState(null);
  const [customAnswer, setCustomAnswer] = useState("");
  const questionOptions = [
    {
      id: "a",
      label: "A · Stack list above detail",
      description: "Keep the focused trace easy to scan on narrow screens.",
    },
    {
      id: "b",
      label: "B · Keep a narrow split view",
      description: "Preserve side-by-side context when the viewport allows it.",
    },
    {
      id: "c",
      label: "C · Custom answer",
      description: "Provide a different behavior in your own words.",
    },
  ];
  const showTurnChanges = !inactive && !noChanges && !failed && !compacting;
  const turnChangeRows = completedTurnChangeSet;
  const [turnChangesOpen, setTurnChangesOpen] = useState(false);
  return (
    <aside
      className={`workspace-panel workspace-review ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}
    >
      <div className={`workspace-review-heading is-${statusTone}`}>
        <h3>
          {idle
            ? "No active process"
            : failed
              ? "Turn stopped"
              : compacting
                ? "Compacting context"
                : running
                  ? "1 active process"
                  : "Awaiting input"}
        </h3>
        <i role="status" aria-label={statusLabel} title={statusLabel} />
      </div>
      <ContextUsage />
      {inactive && (
        <div className="workspace-review-empty">
          <Icon name="activity" size={22} />
          <strong>Agent is idle</strong>
          <span>Start a turn from the conversation composer.</span>
        </div>
      )}
      {failed && (
        <div className="workspace-failure-card">
          <div>
            <span>TURN STOPPED</span>
            <b>FAILED</b>
          </div>
          <strong>Provider request failed</strong>
          <p>The turn ended before completion. No further tool calls will run.</p>
          <dl>
            <div>
              <dt>Reason</dt>
              <dd>Network unavailable</dd>
            </div>
            <div>
              <dt>Turn</dt>
              <dd>
                <code>turn_01JY7F3K9M</code>
              </dd>
            </div>
          </dl>
          <Button variant="primary" size="sm">
            Retry turn
          </Button>
        </div>
      )}
      {compacting && (
        <div className="workspace-process-card workspace-compaction-card">
          <div>
            <i />
            <strong>Context compaction</strong>
            <small>Compacting</small>
          </div>
          <code>Turn turn_01JY7F3K9M</code>
          <span>Context&nbsp; Summarizing earlier turns</span>
          <span>Next&nbsp; Resume model call</span>
        </div>
      )}
      {running && (
        <>
          <div className="workspace-process-card">
            <div>
              <i />
              <strong>Agent loop</strong>
              <small>Running</small>
            </div>
            <code>Turn turn_01JY7F3K9M</code>
            <span>Model&nbsp; gpt-5.6-sol</span>
            <span>
              Latest&nbsp;{" "}
              {noChanges ? "Inspecting project, no file changes yet" : "Editing workspace modules"}
            </span>
          </div>
          <div className="workspace-todo-card">
            <div>
              <span>TODO</span>
              <small>{currentTurnTodos.length} items</small>
            </div>
            {currentTurnTodos.map((todo) => (
              <p key={todo.content} className={`workspace-todo-item is-${todo.status}`}>
                <span className="workspace-todo-marker">
                  <Icon name={todo.icon} size={11} />
                </span>
                <span>{todo.content}</span>
              </p>
            ))}
          </div>
        </>
      )}
      {showTurnChanges && (
        <div className="workspace-turn-changes">
          <button
            type="button"
            className="workspace-turn-changes-summary"
            aria-expanded={turnChangesOpen}
            onClick={() => setTurnChangesOpen((open) => !open)}
          >
            <span className="workspace-turn-changes-summary-title">
              <strong>CHANGES</strong>
              <small>{turnChangeRows.length} files</small>
            </span>
            <span className="workspace-turn-changes-summary-meta">
              <small className="is-added">{completedTurnChanges.added} added</small>
              <small className="is-deleted">{completedTurnChanges.deleted} deleted</small>
              <small className="is-edited">{completedTurnChanges.edited} edited</small>
              {running && <small className="is-live">LIVE</small>}
            </span>
          </button>
          {turnChangesOpen && (
            <div className="workspace-turn-changes-list">
              {turnChangeRows.map((change) => (
                <div className="workspace-turn-change" key={change.path}>
                  <b className={`workspace-change-status is-${change.kind}`}>{change.status}</b>
                  <code title={change.path}>{change.path}</code>
                  <small>
                    +{change.additions} &nbsp;−{change.deletions}
                  </small>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
      {!compacting && <div className="workspace-review-divider" />}
      {waiting && (
        <>
          <span className="workspace-label">REVIEW QUEUE</span>
          {state === "approval" ? (
            <div className="workspace-approval-card">
              <div>
                <span>Approval required</span>
                <b>REVIEW</b>
              </div>
              <strong>Run the production design build</strong>
              <code>vite build</code>
              <div className="workspace-approval-actions">
                <Button variant="primary" size="sm">
                  Allow once
                </Button>
                <Button variant="danger" size="sm">
                  Deny
                </Button>
              </div>
              <Button size="sm">Allow for session</Button>
            </div>
          ) : (
            <div className="workspace-question-card">
              <div>
                <span>Clarification needed</span>
                <b>ANSWER</b>
              </div>
              <div className="workspace-question-prompt">
                <span>Scope</span>
                <strong>Which responsive behavior should the focused trace use?</strong>
              </div>
              <div className="workspace-question-options">
                {questionOptions.map((option) => (
                  <label
                    key={option.id}
                    className={`workspace-question-option ${questionOption === option.id ? "is-selected" : ""}`}
                  >
                    <Radio
                      className="workspace-question-radio"
                      name="trace-layout"
                      value={option.id}
                      checked={questionOption === option.id}
                      onChange={() => setQuestionOption(option.id)}
                    />
                    <span>
                      <strong>{option.label}</strong>
                      <small>{option.description}</small>
                    </span>
                  </label>
                ))}
              </div>
              <input
                className="workspace-question-custom"
                value={customAnswer}
                onChange={(event) => setCustomAnswer(event.target.value)}
                placeholder="Add a custom answer"
                aria-label="Custom answer"
              />
              <div className="workspace-question-actions">
                <Button
                  variant="primary"
                  size="sm"
                  disabled={!questionOption && !customAnswer.trim()}
                >
                  Submit answers
                </Button>
                <Button variant="danger" size="sm">
                  Skip
                </Button>
              </div>
            </div>
          )}
        </>
      )}
      {!compact && running && !noChanges && (
        <div className="workspace-checkpoint-card">
          <div>
            <span>CHECKPOINT</span>
            <small>3 files</small>
          </div>
          <strong>Workspace route implementation</strong>
          <code>
            navigation.js{"\n"}WorkspacePrimitives.jsx{"\n"}review.css
          </code>
          <Button size="sm">Undo</Button>
        </div>
      )}
    </aside>
  );
}
