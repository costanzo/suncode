import { Icon } from "../../../../../shared/Icon.jsx";
import { ConversationAttachments } from "./ConversationAttachments.jsx";

export function QueuedInputs({
  messages,
  paused,
  busy,
  canResume,
  onResume,
  onEdit,
  onRemove,
  onPreview,
  canSteer,
  onSteer,
  collapsed,
  onToggleCollapsed,
  toggleButtonRef,
}) {
  if (!messages.length) return null;
  const steeringCount = messages.filter((message) => message.mode === "steering").length;
  return (
    <section className="workspace-queued-inputs" aria-label="Queued messages">
      <header>
        <span>
          <Icon name="message" size={12} /> Up next <small>{messages.length}</small>
        </span>
        <span>
          {paused ? (
            <>
              <small>Paused</small>
              <button
                type="button"
                className="workspace-queue-resume"
                disabled={!canResume}
                onClick={onResume}
              >
                Resume queue
              </button>
            </>
          ) : (
            <small>{steeringCount ? `${steeringCount} steering · ${messages.length - steeringCount} follow-up` : "After this turn"}</small>
          )}
          <button
            ref={toggleButtonRef}
            type="button"
            onClick={onToggleCollapsed}
            aria-label={collapsed ? "Expand queued messages" : "Collapse queued messages"}
            aria-expanded={!collapsed}
            title={
              collapsed ? "Expand queued messages" : "Collapse queued messages to see more output"
            }
          >
            <Icon name={collapsed ? "arrow-up" : "arrow-down"} size={12} />
          </button>
        </span>
      </header>
      <ol hidden={collapsed}>
        {messages.map((queued, index) => (
          <li key={queued.id}>
            <span className="workspace-queue-position" aria-hidden="true">
              {index + 1}
            </span>
            <div className="workspace-queue-content">
              <p>
                {queued.text ||
                  `${queued.attachments.length} image${queued.attachments.length === 1 ? "" : "s"}`}
              </p>
              <small className={queued.steerError ? "workspace-queue-error" : ""}>
                {queued.steerError
                  ? "Steering failed · Message kept"
                  : queued.mode === "steering"
                    ? "Steering · Next model request"
                    : "Follow-up · After this turn"}
              </small>
              <ConversationAttachments attachments={queued.attachments} onPreview={onPreview} />
            </div>
            <div className="workspace-queue-actions">
              <button
                type="button"
                className="workspace-queue-steer"
                disabled={!canSteer}
                aria-disabled={queued.mode === "steering" && !queued.steerError}
                onClick={() => onSteer(queued)}
                aria-label={queued.steerError
                  ? `Retry steering queued message ${index + 1}`
                  : queued.mode === "steering"
                    ? `Steering queued for message ${index + 1}`
                    : `Steer now with queued message ${index + 1}`}
                title={queued.mode === "steering" && !queued.steerError
                  ? "Waiting for the next model request; current work continues"
                  : !canSteer
                    ? "Steering needs an active turn and a resumed queue"
                    : "Include this message in the next model request without stopping current work"}
              >
                {queued.mode === "steering" && !queued.steerError ? "Steering queued" : queued.steerError ? "Retry steer" : "Steer now"}
              </button>
              <div>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => onEdit(queued)}
                  aria-label={`Take back and edit queued message ${index + 1}`}
                  title="Take back and edit"
                >
                  <Icon name="edit" size={12} />
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => onRemove(queued)}
                  aria-label={`Remove queued message ${index + 1}`}
                  title="Remove from queue"
                >
                  <Icon name="close" size={12} />
                </button>
              </div>
            </div>
          </li>
        ))}
      </ol>
    </section>
  );
}
