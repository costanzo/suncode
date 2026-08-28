import { Icon } from "../../../shared/Icon.jsx";

export function EmptyState({ title, description, icon = "assets" }) {
  return <div className="empty project-hub-empty"><Icon name={icon} size={28} /><strong>{title}</strong><span>{description}</span></div>;
}
