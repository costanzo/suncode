import { useEffect, useRef, useState } from "react";
import { Button } from "../../../../../components/universal/button/index.js";
import {
  ModelDropdown,
  SingleDropdown,
} from "../../../../../components/universal/dropdown/index.js";
import { Modal } from "../../../../../components/universal/modal/index.js";

import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";
import { runningConversationToolCalls, workspaceModelGroups } from "../../data/conversation.js";
import { completedTurnChanges } from "../../data/review.js";
import { primarySessions as sessions } from "../../data/sessions.js";

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
  onViewChanges,
  onOpenToolActivity,
}) {
  const [message, setMessage] = useState(
    state === "immersive-composer"
      ? "Please refactor the conversation layout into focused review states, preserve the existing attachment behavior, and keep the visual language aligned with Quiet Control Desk. I want the resulting specimen to stay calm even when the prompt is several paragraphs long.\n\nAlso add a clearer tool-inspection state so long-running commands can be observed without leaving the conversation surface."
      : "",
  );
  const [attachments, setAttachments] = useState(initialAttachments);
  const [sentAttachments, setSentAttachments] = useState(initialSentAttachments);
  const [previewAttachment, setPreviewAttachment] = useState(null);
  const [composerExpanded, setComposerExpanded] = useState(false);
  const [copiedResponse, setCopiedResponse] = useState(false);
  const [showScrollToBottom, setShowScrollToBottom] = useState(state === "scrolled-up");
  const conversationScrollRef = useRef(null);
  const attachmentInputRef = useRef(null);
  const localAttachmentUrls = useRef(new Set());
  const copyResetTimerRef = useRef(null);
  const hasSession = state !== "no-session";
  const hasContent = state !== "new-session" && hasSession;
  const archived = state === "archived";
  const updating = state === "content-updating" || state === "live-tool-stream";
  const thinking = state === "content-thinking";
  const modelUnavailable = state === "model-unavailable";
  const turnActive = updating || thinking;
  const scrollToBottom = () => {
    conversationScrollRef.current?.scrollTo({
      top: conversationScrollRef.current.scrollHeight,
      behavior: "smooth",
    });
    setShowScrollToBottom(false);
  };
  const activeTool = updating
    ? runningConversationToolCalls.find((tool) => tool.tone === "running")
    : null;
  const messageCharacters = Array.from(message).length;
  const handleAttachmentChange = (event) => {
    if (!imageInputEnabled) return;
    const selectedImages = Array.from(event.target.files ?? []).filter((file) =>
      file.type.startsWith("image/"),
    );
    if (selectedImages.length)
      setAttachments((current) => {
        const newAttachments = selectedImages
          .slice(0, Math.max(0, 3 - current.length))
          .map((file, index) => ({
            id: `${file.name}-${file.lastModified}-${index}`,
            name: file.name,
            type: file.type,
            url: URL.createObjectURL(file),
            local: true,
          }));
        newAttachments.forEach((attachment) => localAttachmentUrls.current.add(attachment.url));
        return [...current, ...newAttachments];
      });
    event.target.value = "";
  };
  const removeAttachment = (id) => {
    setAttachments((current) => {
      const removed = current.find((attachment) => attachment.id === id);
      if (removed?.local) {
        URL.revokeObjectURL(removed.url);
        localAttachmentUrls.current.delete(removed.url);
      }
      return current.filter((attachment) => attachment.id !== id);
    });
  };
  useEffect(
    () => () => {
      localAttachmentUrls.current.forEach((url) => URL.revokeObjectURL(url));
      if (copyResetTimerRef.current) window.clearTimeout(copyResetTimerRef.current);
    },
    [],
  );
  const sendMessage = () => {
    if (!message.trim() && !attachments.length) return;
    if (attachments.length) setSentAttachments((current) => [...current, ...attachments]);
    setAttachments([]);
    setMessage("");
    setComposerExpanded(false);
  };
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
                  Let me confirm the module types and dependency relationships before I summarize
                  the project structure.
                </p>
              </div>
            )}
            {activeTool && <div className="workspace-assistant-duration">Working for 18s</div>}
            {activeTool && (
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
            {!turnActive ? (
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
                  Inspecting the workspace shell and keeping the long-running build visible in the
                  conversation timeline.
                </p>
              </div>
            )}
            {thinking && (
              <div
                className="workspace-thinking-indicator"
                role="status"
                aria-label="Assistant is thinking"
              >
                <span>Thinking</span>
              </div>
            )}
            {updating && !thinking && (
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
        </>
      )}
      {hasSession && !archived && (
        <div className={`workspace-composer ${attachments.length ? "has-attachments" : ""}`}>
          {attachments.length > 0 && (
            <div className="workspace-attachment-strip" aria-label="Attached images">
              {attachments.map((attachment) => (
                <div className="workspace-attachment" key={attachment.id}>
                  <button
                    type="button"
                    className="workspace-attachment-preview"
                    onClick={() => setPreviewAttachment(attachment)}
                    aria-label={`View ${attachment.name}`}
                    title="View image"
                  >
                    <img src={attachment.url} alt={attachment.name} />
                  </button>
                  <button
                    type="button"
                    className="workspace-attachment-remove"
                    aria-label={`Remove ${attachment.name}`}
                    title="Remove image"
                    onClick={() => removeAttachment(attachment.id)}
                  >
                    <Icon name="close" size={10} />
                  </button>
                </div>
              ))}
            </div>
          )}
          {modelUnavailable && (
            <div className="workspace-composer-status" role="status">
              <span>Configure an API key in Settings to send messages.</span>
            </div>
          )}
          <textarea
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            placeholder={modelUnavailable ? "" : "Ask SunCode to work on this project"}
            aria-label="Message SunCode"
            disabled={modelUnavailable}
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
                disabled={!imageInputEnabled || attachments.length >= 3}
                onClick={() => attachmentInputRef.current?.click()}
              >
                <Icon name="plus" size={14} />
              </button>
              <button
                type="button"
                className="workspace-expand-composer"
                aria-label="Open expanded composer"
                title="Open expanded composer"
                onClick={() => setComposerExpanded(true)}
                disabled={modelUnavailable}
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
              <ModelDropdown
                groups={
                  imageInputEnabled
                    ? [{ id: "specimen", label: "Specimen", models: ["vision-input specimen"] }]
                    : modelUnavailable
                      ? [{ id: "openai", label: "OpenAI", models: ["gpt-5.6-sol"] }]
                      : workspaceModelGroups
                }
                initialValue={
                  imageInputEnabled
                    ? "vision-input specimen"
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
              <Button
                variant="primary"
                className="workspace-send"
                icon="arrow-up"
                aria-label="Send message"
                disabled={modelUnavailable || (!message.trim() && !attachments.length)}
                onClick={sendMessage}
              />
            </div>
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
        open={composerExpanded}
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
              disabled={!message.trim() && !attachments.length}
            >
              Send message
            </button>
          </>
        }
      >
        <div className="workspace-composer-modal-content">
          <textarea
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            placeholder="Ask SunCode to work on this project"
            aria-label="Expanded message composer"
          />
          <div className="workspace-composer-modal-footer">
            <strong aria-live="polite">{messageCharacters} characters</strong>
          </div>
        </div>
      </Modal>
    </section>
  );
}
