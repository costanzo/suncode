import { Icon } from "../../../../../shared/Icon.jsx";

export function ConversationAttachments({ attachments = [], onPreview, onRemove }) {
  if (!attachments.length) return null;
  return (
    <div className="workspace-attachment-strip is-inline" aria-label="Attached images">
      {attachments.map((attachment) => (
        <div className="workspace-attachment" key={attachment.id}>
          <button
            type="button"
            className="workspace-attachment-preview"
            onClick={() => onPreview(attachment)}
            aria-label={`View ${attachment.name}`}
            title={attachment.name}
          >
            <img src={attachment.url} alt={attachment.name} />
          </button>
          {onRemove && (
            <button
              type="button"
              className="workspace-attachment-remove"
              aria-label={`Remove ${attachment.name}`}
              title="Remove image"
              onClick={() => onRemove(attachment.id)}
            >
              <Icon name="close" size={10} />
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
