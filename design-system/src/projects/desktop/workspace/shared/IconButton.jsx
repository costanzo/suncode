import { Icon } from "../../../../shared/Icon.jsx";

export function IconButton({ icon, label, active = false, onClick, disabled = false }) {
  return (
    <button
      type="button"
      className={`workspace-icon-button ${active ? "is-active" : ""}`}
      aria-label={label}
      aria-pressed={active}
      onClick={onClick}
      disabled={disabled}
    >
      <Icon name={icon} size={15} />
    </button>
  );
}
