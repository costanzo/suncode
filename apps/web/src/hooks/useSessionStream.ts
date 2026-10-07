import { useEffect } from "react";
import { useWebStore } from "../store";

/**
 * Keeps the selected Session's SSE stream open while its snapshot is loaded, and closes it when
 * the selection changes or the caller unmounts. A replaced snapshot of the same Session (for
 * example a stream-delivered `session.snapshot`) does not reconnect.
 */
export function useSessionStream(): void {
  const connectStream = useWebStore((state) => state.connectStream);
  const selectedSessionId = useWebStore((state) => state.selectedSessionId);
  const loadedSessionId = useWebStore((state) => state.snapshot?.id);
  useEffect(() => {
    if (!loadedSessionId || loadedSessionId !== selectedSessionId) return;
    return connectStream();
  }, [connectStream, selectedSessionId, loadedSessionId]);
}
