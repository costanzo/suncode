import type { StreamState } from "../store";
import type { ConnectionState } from "../types";

/** Host connection states plus the transient state a Session stream reports while retrying. */
export type ConnectionDisplayState = ConnectionState | "reconnecting";

const labels: Record<ConnectionDisplayState, string> = {
  connected: "Connected",
  reconnecting: "Reconnecting",
  offline: "Offline",
  degraded: "Degraded",
  unauthorized: "Unauthorized",
};

/** Maps a Session stream state onto the connection lamp. */
export function streamConnectionState(state: StreamState): ConnectionDisplayState {
  if (state === "live") return "connected";
  if (state === "connecting" || state === "reconnecting") return "reconnecting";
  return "degraded";
}

export function Connection({ state }: { state: ConnectionDisplayState }) {
  return (
    <span className={`connection connection-${state}`}>
      <i />
      {labels[state] ?? "Degraded"}
    </span>
  );
}
