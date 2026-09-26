import { useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";

function formatContextTokens(value) {
  if (value >= 1000) return `${(value / 1000).toFixed(value >= 10000 ? 1 : 1)}k`;
  return `${value}`;
}

export function ContextUsage({
  used = 18420,
  limit = 32768,
  input = 15240,
  output = 3180,
  cached = 0,
}) {
  const [expanded, setExpanded] = useState(false);
  const percentage = limit > 0 ? Math.min(100, Math.max(0, (used / limit) * 100)) : null;
  const tone =
    percentage === null
      ? "unknown"
      : percentage >= 90
        ? "danger"
        : percentage >= 75
          ? "warning"
          : "normal";
  const valueLabel =
    percentage === null
      ? "Unavailable"
      : `${formatContextTokens(used)} / ${formatContextTokens(limit)} tokens`;

  return (
    <section className={`workspace-context-usage is-${tone}`} aria-label="Context window usage">
      <button
        type="button"
        className="workspace-context-usage-toggle"
        aria-expanded={expanded}
        onClick={() => setExpanded((open) => !open)}
      >
        <span>
          <small>CONTEXT WINDOW</small>
          <strong>{valueLabel}</strong>
        </span>
        <b>{percentage === null ? "--" : `${Math.round(percentage)}%`}</b>
        <Icon name="chevron-right" size={12} />
      </button>
      <div
        className="workspace-context-usage-bar"
        role="progressbar"
        aria-label="Context window used"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={percentage === null ? undefined : Math.round(percentage)}
      >
        <span style={{ width: `${percentage ?? 0}%` }} />
      </div>
      {expanded && percentage !== null && (
        <dl className="workspace-context-usage-details">
          <div>
            <dt>Input</dt>
            <dd>{formatContextTokens(input)}</dd>
          </div>
          <div>
            <dt>Output</dt>
            <dd>{formatContextTokens(output)}</dd>
          </div>
          <div>
            <dt>Cached</dt>
            <dd>{formatContextTokens(cached)}</dd>
          </div>
        </dl>
      )}
    </section>
  );
}
