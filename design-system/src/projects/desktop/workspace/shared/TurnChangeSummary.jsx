import { Icon } from "../../../../shared/Icon.jsx";

export function TurnChangeSummary({ added, deleted, edited, onViewChanges }) {
  const stats = [
    ["added", added, "is-added"],
    ["deleted", deleted, "is-deleted"],
    ["edited", edited, "is-edited"],
  ];
  const content = (
    <>
      <span className="workspace-turn-summary-heading">
        <Icon name="check" size={12} />
        <strong>Changes</strong>
      </span>
      <span className="workspace-turn-summary-stats">
        {stats.map(([label, count, tone]) => (
          <span key={label} className={`workspace-turn-summary-stat ${tone}`}>
            <b>{count}</b>
            <small>{label}</small>
          </span>
        ))}
      </span>
    </>
  );
  if (!onViewChanges) {
    return (
      <div
        className="workspace-turn-summary"
        role="status"
        aria-label={`Turn complete: ${added} files added, ${deleted} files deleted, ${edited} files edited`}
      >
        {content}
      </div>
    );
  }
  return (
    <button
      type="button"
      className="workspace-turn-summary is-actionable"
      aria-label={`View changes from this turn: ${added} files added, ${deleted} files deleted, ${edited} files edited`}
      title="View turn changes"
      onClick={onViewChanges}
    >
      {content}
    </button>
  );
}
