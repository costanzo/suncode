import { timeAgo } from "../lib/time";
import type { Project, SessionSummary } from "../types";
import { Icon } from "./Icon";

export function ProjectRow({
  project,
  sessions,
  expanded,
  selectedId,
  onToggle,
  onSelect,
}: {
  project: Project;
  sessions: SessionSummary[];
  expanded: boolean;
  selectedId: string | null;
  onToggle: () => void;
  onSelect: (id: string) => void;
}) {
  return (
    <div className={`project-group ${expanded ? "expanded" : ""}`}>
      <button className="project-row" type="button" aria-expanded={expanded} onClick={onToggle}>
        <Icon name="folder" />
        <span>
          <strong>{project.displayName}</strong>
          <small>{project.activeSessionCount ?? 0} active sessions</small>
        </span>
        <Icon name="chevron" />
      </button>
      {expanded && (
        <div className="project-sessions">
          {sessions.length ? (
            sessions.map((session) => (
              <button
                className={`session-row ${selectedId === session.id ? "selected" : ""}`}
                key={session.id}
                type="button"
                onClick={() => onSelect(session.id)}
              >
                <i className={`state-dot state-${session.state}`} />
                <span>
                  <strong>{session.title}</strong>
                  <small>
                    {session.state.replaceAll("_", " ")} · {timeAgo(session.updatedAt)}
                  </small>
                </span>
              </button>
            ))
          ) : (
            <span className="empty-sessions">No active sessions</span>
          )}
        </div>
      )}
    </div>
  );
}
