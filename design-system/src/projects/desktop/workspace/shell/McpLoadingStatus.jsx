import { Icon } from "../../../../shared/Icon.jsx";

export function McpLoadingStatus({ settled, total, connected = 0, failed = 0 }) {
  if (total <= 0 || settled >= total) return null;
  const progress = Math.min(100, Math.max(0, (settled / total) * 100));
  const remaining = Math.max(0, total - settled);
  const label = `MCP ${settled}/${total}`;
  const detail = `${connected} connected${failed ? ` · ${failed} failed` : ""} · ${remaining} starting`;
  return (
    <span
      className={`workspace-mcp-status${failed ? " has-failures" : ""}`}
      title={detail}
      role="status"
      aria-label={`${label} · ${detail}`}
    >
      <Icon name="server" size={11} />
      <span className="workspace-mcp-status-label">{label}</span>
      <span className="workspace-mcp-progress" aria-hidden="true">
        <span style={{ width: `${progress}%` }} />
      </span>
    </span>
  );
}
