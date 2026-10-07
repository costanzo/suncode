import { useLayoutEffect, useRef } from "react";
import { useShallow } from "zustand/react/shallow";
import { useSessionStream } from "../hooks/useSessionStream";
import { timeAgo } from "../lib/time";
import { MarkdownMessage } from "../MarkdownMessage";
import { useWebStore, type StreamState } from "../store";
import { ApprovalCard } from "./ApprovalCard";
import { Composer } from "./Composer";
import { Connection, streamConnectionState } from "./Connection";
import { Icon } from "./Icon";
import type { SessionState } from "../types";
import { QuestionCard } from "./QuestionCard";

const streamLabels: Record<StreamState, string> = {
  live: "Live session stream",
  connecting: "Connecting to Session",
  failed: "Session stream failed",
  reconnecting: "Reconnecting with a fresh snapshot",
  idle: "Reconnecting with a fresh snapshot",
};

/** A turn is in progress while it runs or waits on the user; only then can it be cancelled. */
const turnActive = (state: SessionState) =>
  state === "running" || state === "waiting_for_approval" || state === "waiting_for_answer";

export function Conversation() {
  useSessionStream();
  const {
    snapshot,
    streamState,
    reviewOpen,
    toggleReview,
    cancelTurn,
    retryTurn,
    loading,
    error,
    lastEventId,
  } = useWebStore(
    useShallow((state) => ({
      snapshot: state.snapshot,
      streamState: state.streamState,
      reviewOpen: state.reviewOpen,
      toggleReview: state.toggleReview,
      cancelTurn: state.cancelTurn,
      retryTurn: state.retryTurn,
      loading: state.loading,
      error: state.error,
      lastEventId: state.lastEventId,
    })),
  );
  const messagesRef = useRef<HTMLDivElement>(null);
  const followLatestRef = useRef(true);
  useLayoutEffect(() => {
    const messages = messagesRef.current;
    if (messages && followLatestRef.current) messages.scrollTop = messages.scrollHeight;
  }, [snapshot?.id, snapshot?.messages.length, snapshot?.messages.at(-1)?.text, snapshot?.state]);
  if (!snapshot)
    return (
      <main className="conversation empty-conversation">
        {loading ? (
          <>
            <h2>Loading session</h2>
            <p>Fetching the live session from the paired Desktop.</p>
          </>
        ) : (
          <>
            <h2>{error ? "Unable to load session" : "Select a Session"}</h2>
            <p>{error ?? "Choose a Project and Session from the left."}</p>
          </>
        )}
      </main>
    );
  return (
    <main className="conversation">
      <header className="conversation-header">
        <div>
          <span className="eyebrow">PRIMARY SESSION</span>
          <h1>{snapshot.title}</h1>
          <small>
            {snapshot.project?.displayName ?? "Project"} · updated {timeAgo(snapshot.updatedAt)}
          </small>
        </div>
        <div className="header-actions">
          <Connection state={streamConnectionState(streamState)} />
          <button
            className={`review-button ${reviewOpen ? "active" : ""}`}
            type="button"
            onClick={toggleReview}
          >
            <Icon name="panel" /> Review
          </button>
        </div>
      </header>
      <div className="stream-status">
        <i />
        {streamLabels[streamState]}
        <code>event {snapshot.eventSequence}</code>
      </div>
      {error && <div className="workspace-error">{error}</div>}
      <div
        className="messages"
        ref={messagesRef}
        onScroll={(event) => {
          const element = event.currentTarget;
          followLatestRef.current =
            element.scrollHeight - element.scrollTop - element.clientHeight < 96;
        }}
      >
        {snapshot.messages.map((message) => (
          <article className={`message message-${message.role}`} key={message.id}>
            <span className="message-author">{message.role === "user" ? "You" : "SunCode"}</span>
            {message.role === "user" ? (
              <div>{message.text}</div>
            ) : (
              <MarkdownMessage text={message.text} />
            )}
          </article>
        ))}
        {snapshot.pendingApproval && <ApprovalCard />}
        {snapshot.pendingQuestion && <QuestionCard />}
        {snapshot.state === "running" && (
          <div className="thinking-indicator" role="status" aria-live="polite">
            <span>SunCode is thinking</span>
            <i />
            <i />
            <i />
          </div>
        )}
        {snapshot.state === "failed" && (
          <div className="approval-card">
            <strong>Turn failed</strong>
            <p>The Desktop reported a failed turn. Retry it when you are ready.</p>
            <button className="primary-button" type="button" onClick={() => void retryTurn()}>
              Retry turn
            </button>
          </div>
        )}
      </div>
      <Composer />
      <footer className="conversation-footer">
        <button
          type="button"
          disabled={!turnActive(snapshot.state)}
          onClick={() => void cancelTurn()}
        >
          Cancel turn
        </button>
        <span>Last-Event-ID: {lastEventId ?? `session:${snapshot.eventSequence}`}</span>
      </footer>
    </main>
  );
}
