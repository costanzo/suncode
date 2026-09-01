import { Modal } from "./Modal.jsx";

export function ConfirmationDialog({
  open,
  title,
  description,
  children,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  confirmVariant = "danger",
  onConfirm,
  onCancel,
  className = "",
}) {
  return (
    <Modal
      open={open}
      title={title}
      description={description}
      onClose={onCancel}
      className={`confirmation-dialog ${className}`.trim()}
      actions={
        <>
          <button
            type="button"
            className="btn"
            data-dialog-initial-focus
            onClick={onCancel}
          >
            {cancelLabel}
          </button>
          <button
            type="button"
            className={`btn btn-${confirmVariant}`}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </>
      }
    >
      {children && <div className="confirmation-dialog-content">{children}</div>}
    </Modal>
  );
}
