import { useShallow } from "zustand/react/shallow";
import { selectHost, useWebStore } from "../store";
import { Connection, streamConnectionState, type ConnectionDisplayState } from "./Connection";

export function ReviewPanel() {
  const { title, messageCount, revision, toggleReview, connection } = useWebStore(
    useShallow((state) => {
      const hostState = selectHost(state)?.connectionState ?? "offline";
      // A connected Host is only as live as the selected Session stream.
      const connection: ConnectionDisplayState =
        hostState === "connected" && state.snapshot
          ? streamConnectionState(state.streamState)
          : hostState;
      return {
        title: state.snapshot?.title,
        messageCount: state.snapshot?.messages.length,
        revision: state.snapshot?.revision,
        toggleReview: state.toggleReview,
        connection,
      };
    }),
  );
  return (
    <aside className="review-panel">
      <div className="review-heading">
        <span>
          <span className="eyebrow">SESSION CONTEXT</span>
          <strong>Review</strong>
        </span>
        <button type="button" onClick={toggleReview}>
          ×
        </button>
      </div>
      <section>
        <span className="eyebrow">CURRENT SESSION</span>
        <p>{title ?? "No session selected"}</p>
        <small>
          {title !== undefined
            ? `${messageCount} messages · revision ${revision}`
            : "Select a session to inspect it."}
        </small>
      </section>
      <section>
        <span className="eyebrow">RUNTIME</span>
        <p>
          <Connection state={connection} />
        </p>
        <small>Policy and approvals remain local to Desktop.</small>
      </section>
      <div className="undo-note">
        Filesystem changes are controlled by the paired Desktop agent.
      </div>
    </aside>
  );
}
