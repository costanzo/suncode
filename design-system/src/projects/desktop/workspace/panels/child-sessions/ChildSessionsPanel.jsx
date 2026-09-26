import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";
import {
  childSessionStateLabels,
  childSessions,
  primarySessions as sessions,
} from "../../data/sessions.js";

export function ChildSessionsPanel({
  compact = false,
  standalone = false,
  sessions: items = childSessions,
  selectedChildId = items[0]?.id ?? "",
  parentTitle = "Workspace information architecture",
  onSelectChild,
}) {
  return (
    <aside
      className={`workspace-panel workspace-child-sessions ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""}`}
    >
      <div className="workspace-child-sessions-header">
        <div>
          <h3>Child sessions</h3>
          <span>{parentTitle}</span>
        </div>
        <code>{items.length}</code>
      </div>
      {items.length ? (
        <div className="workspace-child-session-list" role="group" aria-label="Child sessions">
          {items.map((child) => (
            <button
              type="button"
              key={child.id}
              className={`workspace-child-session ${selectedChildId === child.id ? "is-selected" : ""}`}
              aria-current={selectedChildId === child.id ? "true" : undefined}
              onClick={() => onSelectChild?.(child)}
            >
              <span className="workspace-child-session-icon">
                <Icon name="agent" size={14} />
              </span>
              <span className="workspace-child-session-copy">
                <span>
                  <strong>{child.title}</strong>
                  <i
                    className={`workspace-child-session-status is-${child.state}`}
                    aria-label={childSessionStateLabels[child.state]}
                    title={childSessionStateLabels[child.state]}
                  />
                </span>
                <small>{child.agentDisplayName}</small>
                <span className="workspace-child-session-meta">
                  <b>{childSessionStateLabels[child.state]}</b>
                  <span>·</span>
                  <span>{child.time}</span>
                </span>
              </span>
              <Icon name="chevron-right" size={12} className="workspace-child-session-chevron" />
            </button>
          ))}
        </div>
      ) : (
        <div className="workspace-child-session-empty">
          <Icon name="agent" size={22} />
          <strong>No child sessions yet</strong>
          <span>Child sessions appear when the main agent delegates work.</span>
        </div>
      )}
    </aside>
  );
}

export function ChildSessionDetail({ child = childSessions[0], standalone = false, onBack }) {
  if (!child)
    return (
      <section className={`workspace-child-detail ${standalone ? "is-standalone" : ""}`}>
        <div className="workspace-child-detail-empty">
          <Icon name="agent" size={24} />
          <strong>Select a child session</strong>
          <span>Choose a delegated session from the right panel to inspect its activity.</span>
        </div>
      </section>
    );

  const isRunning = child.state === "running";
  const isCompleted = child.state === "completed";
  const isApproval = child.state === "approval";
  const isFailed = child.state === "failed";

  return (
    <section className={`workspace-child-detail ${standalone ? "is-standalone" : ""}`}>
      <header className="workspace-child-detail-header">
        {onBack ? (
          <button type="button" onClick={onBack} aria-label="Back to main session">
            <Icon name="chevron-right" size={13} />
          </button>
        ) : (
          <span className="workspace-child-detail-back-placeholder" aria-hidden="true" />
        )}
        <span className="workspace-child-detail-agent-icon">
          <Icon name="agent" size={15} />
        </span>
        <div>
          <strong>{child.title}</strong>
          <span>{child.agentDisplayName}</span>
        </div>
        <span className={`workspace-child-detail-state is-${child.state}`}>
          <i />
          {childSessionStateLabels[child.state]}
        </span>
      </header>
      <div className="workspace-child-detail-scroll">
        <dl className="workspace-child-detail-meta">
          <div>
            <dt>Agent</dt>
            <dd>
              <code>{child.agentName}</code>
            </dd>
          </div>
          <div>
            <dt>Model</dt>
            <dd>
              <code>{child.model}</code>
            </dd>
          </div>
          <div>
            <dt>Duration</dt>
            <dd>{child.duration}</dd>
          </div>
          <div>
            <dt>Tool calls</dt>
            <dd>{child.toolCalls}</dd>
          </div>
        </dl>
        <div className="workspace-child-timeline">
          <article className="workspace-child-event is-delegation">
            <span className="workspace-child-event-mark">
              <Icon name="message" size={13} />
            </span>
            <div>
              <header>
                <strong>Task from Main Agent</strong>
                <time>{child.time}</time>
              </header>
              <p>{child.task}</p>
            </div>
          </article>

          <article className="workspace-child-event is-tool">
            <span className="workspace-child-event-mark">
              <Icon name="tool" size={13} />
            </span>
            <div>
              <header>
                <strong>{isApproval ? "bash" : "read · glob · grep"}</strong>
                <time>{isApproval ? "Awaiting approval" : `${child.toolCalls} calls`}</time>
              </header>
              <code>
                {isApproval
                  ? "npm run build --prefix design-system"
                  : "Inspected the relevant workspace, settings, and design-system specifications."}
              </code>
            </div>
          </article>

          {isApproval && (
            <div className="workspace-child-approval">
              <Icon name="lock" size={15} />
              <div>
                <strong>Run a shell command</strong>
                <span>Shell command · npm run build --prefix design-system</span>
                <div>
                  <button className="btn primary small" type="button">
                    Allow once
                  </button>
                  <button className="btn small" type="button">
                    Deny
                  </button>
                </div>
              </div>
            </div>
          )}

          {isFailed && (
            <div className="workspace-child-failure-note">
              <Icon name="activity" size={15} />
              <div>
                <strong>Child session stopped</strong>
                <span>
                  The compact-width inspection could not load the requested view. The main agent can
                  decide whether to delegate it again.
                </span>
              </div>
            </div>
          )}

          {(isRunning || isCompleted) && (
            <article
              className={`workspace-child-event is-response ${isRunning ? "is-running" : ""}`}
            >
              <span className="workspace-child-event-mark">
                <Icon name="agent" size={13} />
              </span>
              <div>
                <header>
                  <strong>{child.agentDisplayName}</strong>
                  <time>{isRunning ? "Working" : "Completed"}</time>
                </header>
                {isRunning ? (
                  <>
                    <p>
                      I am checking how the new right panel yields space before the central reading
                      surface becomes too narrow.
                    </p>
                    <span className="workspace-child-working">Reviewing responsive states</span>
                  </>
                ) : (
                  <>
                    <p>
                      The contract should keep the primary session as the authority root while child
                      sessions remain independently inspectable.
                    </p>
                    <ul>
                      <li>Exclude child sessions from the primary Sessions list.</li>
                      <li>Expose child snapshots through read-only detail surfaces.</li>
                      <li>Keep delegated checkpoints scoped to the child session.</li>
                    </ul>
                  </>
                )}
              </div>
            </article>
          )}
        </div>
      </div>
    </section>
  );
}
