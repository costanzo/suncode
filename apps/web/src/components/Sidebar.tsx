import { useEffect, useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { selectHost, useWebStore } from "../store";
import type { SessionSummary } from "../types";
import { Connection } from "./Connection";
import { Icon } from "./Icon";
import { ProjectRow } from "./ProjectRow";

export function Sidebar({ onSettings }: { onSettings: () => void }) {
  const { host, projects, sessions, selectedSessionId, selectSession, error } = useWebStore(
    useShallow((state) => ({
      host: selectHost(state),
      projects: state.projects,
      sessions: state.sessions,
      selectedSessionId: state.selectedSessionId,
      selectSession: state.selectSession,
      error: state.error,
    })),
  );
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  useEffect(() => {
    if (projects[0] && Object.keys(expanded).length === 0) setExpanded({ [projects[0].id]: true });
  }, [projects, expanded]);
  const grouped = useMemo(
    () =>
      sessions.reduce<Record<string, SessionSummary[]>>((result, session) => {
        (result[session.project.id] ??= []).push(session);
        return result;
      }, {}),
    [sessions],
  );
  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="brand-mark">
          <Icon name="monitor" />
        </div>
        <strong>SunCode</strong>
        <span>WEB</span>
      </div>
      <button className="host-picker" type="button" onClick={onSettings}>
        <div className="host-icon">
          <Icon name="monitor" />
        </div>
        <span>
          <strong>{host?.displayName ?? "No Desktop paired"}</strong>
          <small>{host ? "Open Settings" : "Pair in Settings"}</small>
        </span>
        <Icon name="chevron" />
      </button>
      <div className="sidebar-heading">
        <span>PROJECTS</span>
        <span>{projects.length}</span>
      </div>
      <div className="project-list">
        {projects.length ? (
          projects.map((project) => (
            <ProjectRow
              key={project.id}
              project={project}
              sessions={grouped[project.id] ?? []}
              expanded={!!expanded[project.id]}
              selectedId={selectedSessionId}
              onToggle={() =>
                setExpanded((state) => ({ ...state, [project.id]: !state[project.id] }))
              }
              onSelect={(id) => void selectSession(id)}
            />
          ))
        ) : (
          <div className="sidebar-empty">
            {host ? "No projects available" : "Pair a Desktop in Settings to load projects."}
          </div>
        )}
      </div>
      <div className="sidebar-footer">
        {error && <small className="sidebar-error">{error}</small>}
        <Connection state={host?.connectionState ?? "offline"} />
        <button className="settings-link" type="button" onClick={onSettings}>
          <Icon name="settings" /> Settings
        </button>
        <small>remote.v1 · browser E2E</small>
      </div>
    </aside>
  );
}
