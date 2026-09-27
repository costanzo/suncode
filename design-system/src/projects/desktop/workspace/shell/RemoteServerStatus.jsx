import { Icon } from "../../../../shared/Icon.jsx";

/** Footer status for a configured Remote Server connection. */
export function RemoteServerStatus({ configured = false, connected = false }) {
  if (!configured) return null;

  const state = connected ? "connected" : "disconnected";
  const label = connected ? "Remote Server connected" : "Remote Server disconnected";

  return (
    <span
      className={`workspace-remote-server-status is-${state}`}
      role="status"
      aria-label={label}
      title={label}
    >
      <Icon name="server" size={12} />
      <i aria-hidden="true" />
      <code>{connected ? "remote connected" : "remote disconnected"}</code>
    </span>
  );
}
