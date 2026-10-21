import { useEffect, useId, useRef, useState } from "react";
import { Button } from "../../../../../components/universal/button/index.js";
import {
  ModelDropdown,
  SingleDropdown,
} from "../../../../../components/universal/dropdown/index.js";
import { Modal } from "../../../../../components/universal/modal/index.js";

import { Icon } from "../../../../../shared/Icon.jsx";
import { ConversationAttachments } from "./ConversationAttachments.jsx";
import { QueuedInputs } from "./QueuedInputs.jsx";

import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";
import { runningConversationToolCalls, workspaceModelGroups } from "../../data/conversation.js";
import { completedTurnChanges } from "../../data/review.js";

const createSampleAttachment = (name, title, detail) => {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200" viewBox="0 0 320 200"><rect width="320" height="200" fill="#f2f3f5"/><rect x="18" y="18" width="284" height="164" rx="8" fill="#ffffff" stroke="#d6d9de"/><rect x="34" y="38" width="68" height="10" rx="3" fill="#23262b"/><rect x="34" y="62" width="112" height="8" rx="3" fill="#d6d9de"/><rect x="34" y="82" width="210" height="8" rx="3" fill="#e7e9ed"/><rect x="34" y="112" width="${title === "Workspace" ? 152 : 184}" height="28" rx="5" fill="#${title === "Workspace" ? "dfe3e8" : "eceef1"}"/><circle cx="270" cy="48" r="8" fill="#${detail}"/><text x="34" y="164" fill="#7d848e" font-family="Arial,sans-serif" font-size="10">${title}</text></svg>`;
  return {
    id: `sample-${name}`,
    name,
    type: "image/svg+xml",
    url: `data:image/svg+xml,${encodeURIComponent(svg)}`,
  };
};

export const sampleConversationAttachments = [
  createSampleAttachment("workspace-layout.svg", "Workspace", "626a73"),
  createSampleAttachment("settings-reference.svg", "Settings", "8a919b"),
];

export function ConversationPanel({
  compact = false,
  standalone = false,
  state = "content-waiting",
  initialAttachments = [],
  initialSentAttachments = [],
  imageInputEnabled = false,
  initialQueuedSteering = null,
  initialQueuedMessages = null,
  initialQueueCollapsed = false,
  onViewChanges,
  onOpenToolActivity,
}) {
  const inputHintId = useId();
  const inputModeHint =
    "Messages queue for after this turn. Use Steer now on a queued message to guide this turn.";
  const [message, setMessage] = useState(
    state === "immersive-composer"
      ? "Please refactor the conversation layout into focused review states, preserve the existing attachment behavior, and keep the visual language aligned with Quiet Control Desk. I want the resulting specimen to stay calm even when the prompt is several paragraphs long.\n\nAlso add a clearer tool-inspection state so long-running commands can be observed without leaving the conversation surface."
      : state === "queued-input"
        ? "Change the report layout, then verify it with the existing tests."
        : "",
  );
  const [attachments, setAttachments] = useState(initialAttachments);
  const sentAttachments = initialSentAttachments;
  const [submittedMessages, setSubmittedMessages] = useState([]);
  const [previewAttachment, setPreviewAttachment] = useState(null);
  const [composerExpanded, setComposerExpanded] = useState(false);
  const [copiedResponse, setCopiedResponse] = useState(false);
  const [queuedMessages, setQueuedMessages] = useState(
    initialQueuedMessages ??
      (state === "queued-input"
        ? [
            {
              id: "queued-1",
              mode: "followup",
              text: "Keep the composer available while the build is running.",
            },
            {
              id: "queued-2",
              mode: initialQueuedSteering ? "steering" : "followup",
              steerError: initialQueuedSteering === "failed",
              text: "Also preserve the queue if I edit the next instruction.",
              attachments: [sampleConversationAttachments[0]],
            },
          ]
        : []),
  );
  const [turnPhase, setTurnPhase] = useState("idle");
  const [queueHeld, setQueueHeld] = useState(false);
  const [inputNotice, setInputNotice] = useState("");
  const [removedMessage, setRemovedMessage] = useState(null);
  const [attachmentError, setAttachmentError] = useState("");
  const [queueCollapsed, setQueueCollapsed] = useState(initialQueueCollapsed);
  const queueToggleRef = useRef(null);
  const composerRef = useRef(null);
  const expandedComposerRef = useRef(null);
  const [showScrollToBottom, setShowScrollToBottom] = useState(state === "scrolled-up");
  const conversationScrollRef = useRef(null);
  const attachmentInputRef = useRef(null);
  const localAttachmentUrls = useRef(new Set());
  const copyResetTimerRef = useRef(null);
  const hasSession = state !== "no-session";
  const hasContent = hasSession && (state !== "new-session" || submittedMessages.length > 0);
  const hasInitialHistory = hasSession && state !== "new-session";
  const archived = state === "archived";
  const updating =
    state === "content-updating" || state === "live-tool-stream" || state === "queued-input";
  const thinking = state === "content-thinking";
  const [retryStopped, setRetryStopped] = useState(false);
  const retrying = state === "content-retrying" && !retryStopped;
  const modelUnavailable = state === "model-unavailable";
  const submissionBusy = turnPhase === "stopping" || turnPhase === "submitting";
  const turnActive =
    turnPhase === "running" || ((updating || thinking || retrying) && turnPhase === "idle");
  const hasDraft = Boolean(message.trim() || attachments.length);
  const canSend =
    !submissionBusy && !modelUnavailable && hasDraft && (!attachments.length || imageInputEnabled);
  const focusComposer = (expanded = composerExpanded) => {
    window.requestAnimationFrame(() =>
      (expanded ? expandedComposerRef : composerRef).current?.focus(),
    );
  };
  const admitDraft = () => {
    setSubmittedMessages((current) => [
      ...current,
      { id: `submitted-${Date.now()}`, text: message.trim(), attachments },
    ]);
    setAttachments([]);
    setMessage("");
    setComposerExpanded(false);
    setTurnPhase("running");
    setAttachmentError("");
    window.requestAnimationFrame(scrollToBottom);
    focusComposer(false);
  };
  // Review-only timing for ordinary send and the separate Stop action.
  // Steering changes queue eligibility only; it never enters this cancellation path.
  useEffect(() => {
    if (!submissionBusy) return undefined;
    const timer = window.setTimeout(
      () => {
        if (turnPhase === "stopping") {
          setRetryStopped(true);
          setTurnPhase("stopped");
        } else {
          admitDraft();
        }
      },
      turnPhase === "stopping" ? 1800 : 900,
    );
    return () => window.clearTimeout(timer);
  }, [turnPhase]);
  const scrollToBottom = () => {
    conversationScrollRef.current?.scrollTo({
      top: conversationScrollRef.current.scrollHeight,
      behavior: "smooth",
    });
    setShowScrollToBottom(false);
  };
  const toggleQueueCollapsed = () => {
    const history = conversationScrollRef.current;
    const wasFollowing =
      history && history.scrollHeight - history.clientHeight - history.scrollTop <= 32;
    setQueueCollapsed((current) => !current);
    window.requestAnimationFrame(() => {
      if (wasFollowing) {
        history.scrollTo({ top: history.scrollHeight, behavior: "instant" });
        setShowScrollToBottom(false);
      }
      queueToggleRef.current?.focus();
    });
  };
  const activeTool =
    updating && turnPhase !== "running"
      ? runningConversationToolCalls.find((tool) => tool.tone === "running")
      : null;
  const messageCharacters = Array.from(message).length;
  const handleAttachmentChange = (event) => {
    if (!imageInputEnabled || submissionBusy) return;
    const files = Array.from(event.target.files ?? []);
    const valid = files.filter(
      (file) => file.type.startsWith("image/") && file.size <= 20 * 1024 * 1024,
    );
    const available = Math.max(0, 3 - attachments.length);
    const selected = valid.slice(0, available).map((file) => {
      const url = URL.createObjectURL(file);
      localAttachmentUrls.current.add(url);
      return { id: crypto.randomUUID(), name: file.name, type: file.type, url };
    });
    setAttachments((current) => [...current, ...selected]);
    setAttachmentError(
      valid.length !== files.length
        ? "Choose image files up to 20 MiB each."
        : valid.length > available
          ? "Maximum 3 images per message. Extra images were not added."
          : "",
    );
    event.target.value = "";
  };
  // Keep URLs alive while draft, queue, history, undo, or preview may reference them.
  useEffect(
    () => () => {
      localAttachmentUrls.current.forEach((url) => URL.revokeObjectURL(url));
      if (copyResetTimerRef.current) window.clearTimeout(copyResetTimerRef.current);
    },
    [],
  );
  const sendMessage = () => {
    if (!canSend) return;
    setInputNotice("");
    if (turnActive) {
      setQueuedMessages((current) => [
        ...current,
        { id: crypto.randomUUID(), mode: "followup", text: message.trim(), attachments },
      ]);
      setMessage("");
      setAttachments([]);
      setComposerExpanded(false);
      setAttachmentError("");
      focusComposer(false);
      return;
    }
    setTurnPhase("submitting");
  };
  const takeBackQueuedMessage = (queued) => {
    if (submissionBusy) return;
    const savedDraft = hasDraft
      ? { id: crypto.randomUUID(), mode: "followup", text: message, attachments }
      : null;
    setQueuedMessages((current) =>
      current.flatMap((item) =>
        item.id === queued.id ? (savedDraft ? [savedDraft] : []) : [item],
      ),
    );
    if (savedDraft) setQueueHeld(true);
    setMessage(queued.text);
    setAttachments(queued.attachments ?? []);
    setInputNotice(
      savedDraft
        ? "Previous draft saved in paused queue. Review before resuming."
        : "Message taken back for editing.",
    );
    setAttachmentError("");
    focusComposer();
  };
  const removeQueuedMessage = (queued) => {
    setRemovedMessage({
      message: queued,
      position: queuedMessages.findIndex((item) => item.id === queued.id),
    });
    setQueuedMessages((current) => current.filter((item) => item.id !== queued.id));
    setInputNotice("");
  };
  const undoRemove = () => {
    setQueuedMessages((current) => {
      const next = [...current];
      next.splice(Math.min(removedMessage.position, next.length), 0, removedMessage.message);
      return next;
    });
    setRemovedMessage(null);
  };
  const steerQueuedMessage = (queued) => {
    if (
      !turnActive ||
      submissionBusy ||
      queueHeld ||
      (queued.mode === "steering" && !queued.steerError)
    )
      return;
    setQueuedMessages((current) =>
      current.map((item) =>
        item.id === queued.id ? { ...item, mode: "steering", steerError: false } : item,
      ),
    );
    setInputNotice("Steering queued for the next model request. Current work continues.");
  };
  const stopTurn = () => {
    setQueueHeld(true);
    setInputNotice("");
    setTurnPhase("stopping");
  };
  const handleComposerKeyDown = (event, expanded = false) => {
    if (event.nativeEvent.isComposing || event.key !== "Enter") return;
    if (expanded ? event.metaKey || event.ctrlKey : !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  };
  const renderAttachments = () => (
    <ConversationAttachments
      attachments={attachments}
      onPreview={setPreviewAttachment}
      onRemove={
        submissionBusy
          ? undefined
          : (id) => setAttachments((current) => current.filter((item) => item.id !== id))
      }
    />
  );
  const turnFeedback = {
    stopping: "Stopping current turn…",
    submitting: "Sending your message…",
    stopped: "Turn stopped. Your draft and queued messages are kept.",
  }[turnPhase];
  const copyResponse = async () => {
    await navigator.clipboard?.writeText(
      "I split Workspace into a complete composition and focused pages for sessions, explorer, conversation, review, source control, and provider trace.",
    );
    setCopiedResponse(true);
    if (copyResetTimerRef.current) window.clearTimeout(copyResetTimerRef.current);
    copyResetTimerRef.current = window.setTimeout(() => setCopiedResponse(false), 1400);
  };
  return (
    <section
      className={`workspace-conversation ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} workspace-conversation-${state}`}
    >
      {!hasSession && (
        <div className="workspace-conversation-empty">
          <Icon name="workspace" size={24} />
          <strong>No session selected</strong>
          <span>Create or select a session to start a conversation.</span>
        </div>
      )}
      {hasSession && !hasContent && (
        <div className="workspace-conversation-empty">
          <Icon name="plus" size={24} />
          <strong>New session</strong>
          <span>Send a message to start this conversation.</span>
        </div>
      )}
      {hasContent && (
        <>
          <div
            ref={conversationScrollRef}
            className="workspace-conversation-scroll"
            onScroll={(event) => {
              const element = event.currentTarget;
              setShowScrollToBottom(
                element.scrollHeight - element.clientHeight - element.scrollTop > 32,
              );
            }}
          >
            {hasInitialHistory && (
              <>
                <div className="workspace-message workspace-message-user">
                  {sentAttachments.length > 0 && (
                    <div
                      className="workspace-message-attachments"
                      aria-label="Images sent with this message"
                    >
                      {sentAttachments.map((attachment) => (
                        <button
                          type="button"
                          className="workspace-message-attachment"
                          key={attachment.id}
                          onClick={() => setPreviewAttachment(attachment)}
                          aria-label={`View ${attachment.name}`}
                          title="View image"
                        >
                          <img src={attachment.url} alt={attachment.name} />
                        </button>
                      ))}
                    </div>
                  )}
                  <p>Review the conversation layout and keep the existing attachment behavior.</p>
                </div>
                {state === "intermediate-assistant" && (
                  <div className="workspace-message workspace-message-assistant workspace-message-assistant-intermediate">
                    <p>
                      Let me confirm the module types and dependency relationships before I
                      summarize the project structure.
                    </p>
                  </div>
                )}
                {turnActive && activeTool && (
                  <div className="workspace-assistant-duration">Working for 18s</div>
                )}
                {turnActive && activeTool && (
                  <button
                    type="button"
                    className="workspace-active-tool"
                    onClick={() =>
                      onOpenToolActivity
                        ? onOpenToolActivity("0198e82c", 1)
                        : (window.location.hash = "/projects/desktop/workspace/tool-activity")
                    }
                  >
                    <Icon name={activeTool.icon} size={14} />
                    <span>{activeTool.title}</span>
                    <small>Running · View in Tool activity</small>
                    <Icon name="arrow" size={12} />
                  </button>
                )}
                {!updating && !thinking && state !== "content-retrying" ? (
                  <div className="workspace-message workspace-message-assistant">
                    <div className="workspace-assistant-duration">Worked for 42s</div>
                    <p>
                      I split Workspace into a complete composition and focused pages for sessions,
                      explorer, conversation, review, source control, and provider trace.
                    </p>
                    <div className="workspace-message-footer">
                      <div className="workspace-message-actions">
                        <button
                          type="button"
                          className={`workspace-copy ${copiedResponse ? "is-copied" : ""}`}
                          aria-label={copiedResponse ? "Copied response" : "Copy response"}
                          title={copiedResponse ? "Copied" : "Copy response"}
                          onClick={copyResponse}
                        >
                          <Icon name={copiedResponse ? "check" : "copy"} size={13} />
                        </button>
                        <time
                          className="workspace-message-completed-time"
                          dateTime="2026-09-12T14:32:18+08:00"
                          title="Completed at 14:32"
                        >
                          14:32
                        </time>
                      </div>
                      <TurnChangeSummary {...completedTurnChanges} onViewChanges={onViewChanges} />
                    </div>
                  </div>
                ) : (
                  <div className="workspace-message workspace-message-assistant workspace-message-assistant-status">
                    <p>
                      Inspecting the workspace shell and keeping the long-running build visible in
                      the conversation timeline.
                    </p>
                  </div>
                )}
              </>
            )}
            {submittedMessages.map((submitted) => (
              <div className="workspace-message workspace-message-user" key={submitted.id}>
                <ConversationAttachments
                  attachments={submitted.attachments}
                  onPreview={setPreviewAttachment}
                />
                {submitted.text && <p>{submitted.text}</p>}
              </div>
            ))}
            {turnPhase === "running" && (
              <div className="workspace-message workspace-message-assistant workspace-message-assistant-status">
                <p>Reviewing your instruction and its visual references.</p>
              </div>
            )}
            {retrying && turnActive && turnPhase !== "running" && (
              <div className="workspace-retry-indicator" role="status">
                Retrying request after 2s · retry 1 of 2
              </div>
            )}
            {thinking && turnActive && turnPhase !== "running" && (
              <div
                className="workspace-thinking-indicator"
                role="status"
                aria-label="Assistant is thinking"
              >
                <span>Thinking</span>
              </div>
            )}
            {turnActive && (!thinking || turnPhase === "running") && !retrying && (
              <div
                className="workspace-running-indicator"
                role="status"
                aria-label="Agent is working"
              >
                <i />
                <i />
                <i />
              </div>
            )}
          </div>
        </>
      )}
      {hasSession && !archived && (
        <div className="workspace-composer-dock">
          {showScrollToBottom && (
            <button
              type="button"
              className="workspace-scroll-to-bottom"
              onClick={scrollToBottom}
              aria-label="Scroll to bottom"
              title="Scroll to bottom"
            >
              <Icon name="arrow-down" size={14} />
            </button>
          )}
          <QueuedInputs
            messages={queuedMessages}
            paused={queueHeld}
            busy={submissionBusy}
            canResume={turnActive && !submissionBusy}
            onResume={() => setQueueHeld(false)}
            onEdit={takeBackQueuedMessage}
            onRemove={removeQueuedMessage}
            onPreview={setPreviewAttachment}
            canSteer={turnActive && !submissionBusy && !queueHeld}
            onSteer={steerQueuedMessage}
            collapsed={queueCollapsed}
            onToggleCollapsed={toggleQueueCollapsed}
            toggleButtonRef={queueToggleRef}
          />
          {turnFeedback && (
            <div className="workspace-input-feedback" role="status">
              <span>{turnFeedback}</span>
            </div>
          )}
          {(inputNotice || removedMessage) && (
            <div className="workspace-input-feedback" role="status">
              <span>{inputNotice || "Message removed from Up next."}</span>
              {removedMessage && (
                <button type="button" disabled={submissionBusy} onClick={undoRemove}>
                  Undo remove
                </button>
              )}
            </div>
          )}
          <div
            className={`workspace-composer ${modelUnavailable ? "is-model-unavailable" : ""} ${attachments.length ? "has-attachments" : ""} ${turnActive ? "is-running-input" : ""}`}
          >
            {renderAttachments()}
            {attachmentError && (
              <p className="workspace-attachment-error" role="alert">
                {attachmentError}
              </p>
            )}
            {modelUnavailable && (
              <div className="workspace-composer-status" role="status">
                <span>Configure an API key in Settings to send messages.</span>
              </div>
            )}
            <textarea
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              ref={composerRef}
              onKeyDown={handleComposerKeyDown}
              placeholder={
                modelUnavailable
                  ? ""
                  : turnActive
                    ? "Add a follow-up while SunCode works"
                    : "Ask SunCode to work on this project"
              }
              aria-label="Message SunCode"
              aria-describedby={turnActive ? inputHintId : undefined}
              disabled={modelUnavailable || submissionBusy}
            />
            <div className="workspace-composer-footer">
              <div className="workspace-composer-actions">
                <button
                  type="button"
                  className="workspace-attach"
                  aria-label="Add attachment"
                  title={
                    !imageInputEnabled
                      ? "Selected model does not support image input"
                      : attachments.length >= 3
                        ? "Maximum 3 images"
                        : "Add image"
                  }
                  disabled={
                    submissionBusy ||
                    modelUnavailable ||
                    !imageInputEnabled ||
                    attachments.length >= 3
                  }
                  onClick={() => attachmentInputRef.current?.click()}
                >
                  <Icon name="plus" size={14} />
                </button>
                {turnActive && (
                  <button
                    type="button"
                    className="workspace-attach"
                    aria-label="Stop turn"
                    title="Stop current turn"
                    onClick={stopTurn}
                  >
                    <Icon name="pause" size={14} />
                  </button>
                )}
                <button
                  type="button"
                  className="workspace-expand-composer"
                  aria-label="Open expanded composer"
                  title="Open expanded composer"
                  onClick={() => setComposerExpanded(true)}
                  disabled={modelUnavailable || submissionBusy}
                >
                  <Icon name="expand" size={13} />
                </button>
              </div>
              <input
                ref={attachmentInputRef}
                className="workspace-attachment-input"
                type="file"
                accept="image/*"
                multiple
                tabIndex={-1}
                onChange={handleAttachmentChange}
              />
              <div className="workspace-composer-options">
                <fieldset className="workspace-composer-settings" disabled={submissionBusy}>
                  <ModelDropdown
                    groups={
                      imageInputEnabled
                        ? [{ id: "specimen", label: "Specimen", models: ["Vision specimen"] }]
                        : modelUnavailable
                          ? [{ id: "openai", label: "OpenAI", models: ["gpt-5.6-sol"] }]
                          : workspaceModelGroups
                    }
                    initialValue={
                      imageInputEnabled
                        ? "Vision specimen"
                        : modelUnavailable
                          ? "gpt-5.6-sol"
                          : "gpt-5.6-sol"
                    }
                    className="workspace-model-dropdown"
                  />
                  <SingleDropdown
                    options={["Medium", "High"]}
                    initialValue="High"
                    ariaLabel="Reasoning effort"
                    className="workspace-reasoning-dropdown"
                  />
                </fieldset>
                <Button
                  variant="primary"
                  className="workspace-send"
                  icon="arrow-up"
                  aria-label={turnActive ? "Queue follow-up" : "Send message"}
                  title={turnActive ? "Queue for after this turn · Enter" : "Send message · Enter"}
                  disabled={!canSend}
                  onClick={sendMessage}
                />
              </div>
            </div>
            {turnActive && (
              <p className="workspace-input-mode-hint" id={inputHintId}>
                {inputModeHint}
              </p>
            )}
          </div>
        </div>
      )}
      {archived && (
        <div className="workspace-archived-readonly" role="status">
          <Icon name="lock" size={14} />
          <span>This session is archived and read-only.</span>
          <small>Restore it from Archived sessions to continue the conversation.</small>
        </div>
      )}
      <Modal
        open={Boolean(previewAttachment)}
        title={previewAttachment?.name ?? "Image preview"}
        onClose={() => setPreviewAttachment(null)}
        className="workspace-image-modal"
      >
        <div className="workspace-image-preview">
          {previewAttachment && <img src={previewAttachment.url} alt={previewAttachment.name} />}
        </div>
      </Modal>
      <Modal
        open={composerExpanded && !previewAttachment}
        onClose={() => setComposerExpanded(false)}
        className="workspace-composer-modal"
        hideTitle
        ariaLabel="Expanded composer"
        hideClose
        actions={
          <>
            <button type="button" className="btn" onClick={() => setComposerExpanded(false)}>
              Close
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={sendMessage}
              disabled={!canSend}
              title={
                turnActive
                  ? "Queue for after this turn · Ctrl/Cmd+Enter"
                  : "Send message · Ctrl/Cmd+Enter"
              }
            >
              {turnActive ? "Queue follow-up" : "Send message"}
            </button>
          </>
        }
      >
        <div className="workspace-composer-modal-content">
          {renderAttachments()}
          {attachmentError && (
            <p className="workspace-attachment-error" role="alert">
              {attachmentError}
            </p>
          )}
          {turnFeedback && (
            <div className="workspace-input-feedback" role="status">
              {turnFeedback}
            </div>
          )}
          <textarea
            ref={expandedComposerRef}
            data-dialog-initial-focus
            disabled={submissionBusy}
            onKeyDown={(event) => handleComposerKeyDown(event, true)}
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            placeholder="Ask SunCode to work on this project"
            aria-label="Expanded message composer"
            aria-describedby={turnActive ? `${inputHintId}-expanded` : undefined}
          />
          {turnActive && (
            <p className="workspace-input-mode-hint" id={`${inputHintId}-expanded`}>
              {inputModeHint}
            </p>
          )}
          <div className="workspace-composer-modal-footer">
            <strong aria-live="polite">
              {messageCharacters} characters · {attachments.length}/3 images
            </strong>
            <button
              type="button"
              className="btn btn-quiet"
              disabled={submissionBusy || !imageInputEnabled || attachments.length >= 3}
              onClick={() => attachmentInputRef.current?.click()}
            >
              Add image
            </button>
          </div>
        </div>
      </Modal>
    </section>
  );
}
