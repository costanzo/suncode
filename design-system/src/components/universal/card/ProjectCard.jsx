import { Icon } from "../../../shared/Icon.jsx";

export function ProjectCard({ name, path, onClick }) {
  return <button type="button" className="project-card project-hub-project" onClick={onClick}><span className="project-mark">P</span><span className="project-card-copy"><strong>{name}</strong><small>{path}</small></span><Icon name="chevron-right" /></button>;
}
