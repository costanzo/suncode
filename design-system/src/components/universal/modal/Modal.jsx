import { useEffect, useId, useRef } from "react";
import { Icon } from "../../../shared/Icon.jsx";

export function Modal({
  open,
  title,
  description,
  onClose,
  children,
  actions,
  className = "",
  hideTitle = false,
  ariaLabel,
  hideClose = false,
}) {
  const dialogRef = useRef(null);
  const titleId = useId();
  const descriptionId = useId();
  useEffect(() => {
    if (!open) return undefined;
    const priorFocus = document.activeElement;
    const initialFocus =
      dialogRef.current?.querySelector("[data-dialog-initial-focus]") ??
      dialogRef.current?.querySelector("input, button, textarea, select");
    initialFocus?.focus();
    const handleKeyDown = (event) => {
      if (event.key === "Escape") onClose?.();
      if (event.key !== "Tab") return;
      const focusable = [
        ...dialogRef.current.querySelectorAll("button, input, textarea, select, [href]"),
      ];
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      }
      if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      priorFocus?.focus();
    };
  }, [open, onClose]);
  if (!open) return null;
  return (
    <div className="dialog-backdrop" role="presentation" onMouseDown={onClose}>
      <div
        ref={dialogRef}
        className={`review-dialog ${className}`.trim()}
        role="dialog"
        aria-modal="true"
        aria-labelledby={!hideTitle && title ? titleId : undefined}
        aria-label={ariaLabel || (hideTitle ? "Dialog" : undefined)}
        aria-describedby={description ? descriptionId : undefined}
        onMouseDown={(event) => event.stopPropagation()}
      >
        {(title || !hideClose) && (
          <div className={`dialog-title ${hideTitle ? "is-title-hidden" : ""}`.trim()}>
            {!hideTitle && (
              <div>
                <h3 id={titleId}>{title}</h3>
                {description && <p id={descriptionId}>{description}</p>}
              </div>
            )}
            {!hideClose && (
              <button
                type="button"
                className="btn btn-icon btn-quiet"
                onClick={onClose}
                aria-label="Close dialog"
              >
                <Icon name="close" />
              </button>
            )}
          </div>
        )}
        {children}
        {actions && <div className="dialog-actions">{actions}</div>}
      </div>
    </div>
  );
}
