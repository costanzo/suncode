import { useEffect, useRef, useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";
import { workspaceRecentProjects } from "../data/projects.js";

export function ProjectSwitcher({ projects = workspaceRecentProjects }) {
  const [open, setOpen] = useState(true);
  const switcherRef = useRef(null);
  const currentProject = projects.find((project) => project.current) ??
    projects[0] ?? { name: "No project", path: "" };

  const closeMenu = () => setOpen(false);
  useEffect(() => {
    if (!open) return undefined;
    const handlePointerDown = (event) => {
      if (!switcherRef.current?.contains(event.target)) closeMenu();
    };
    const handleKeyDown = (event) => {
      if (event.key === "Escape") closeMenu();
    };
    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  return (
    <div className="project-switcher" ref={switcherRef}>
      <button
        type="button"
        className={`project-switcher-trigger ${open ? "is-open" : ""}`}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={`Current project: ${currentProject.name}. Switch project`}
        onClick={() => setOpen((value) => !value)}
      >
        <Icon name="folder" size={14} />
        <strong>{currentProject.name}</strong>
        <Icon name="chevron-right" size={12} className="project-switcher-chevron" />
      </button>
      {open && (
        <div className="project-switcher-menu" role="menu" aria-label="Project actions">
          <button
            type="button"
            className="project-switcher-action"
            role="menuitem"
            onClick={closeMenu}
          >
            <Icon name="folder" size={14} />
            <span>
              <strong>Open project</strong>
              <small>Choose a local folder…</small>
            </span>
            <Icon name="arrow" size={13} />
          </button>
          <div className="project-switcher-divider" role="separator" />
          <div className="project-switcher-label">RECENT PROJECTS</div>
          {projects.length ? (
            <div className="project-switcher-list">
              {projects.map((project) => (
                <button
                  type="button"
                  className="project-switcher-project"
                  role="menuitem"
                  key={project.id}
                  onClick={closeMenu}
                >
                  <Icon name="project" size={14} />
                  <span>
                    <strong>{project.name}</strong>
                    <small>{project.path}</small>
                  </span>
                </button>
              ))}
            </div>
          ) : (
            <div className="project-switcher-empty">No recent projects</div>
          )}
        </div>
      )}
    </div>
  );
}
