import "./ubuntu-window-controls.css";

// Yaru caption symbols stay visible; maximize restores a maximized window.
export function UbuntuWindowControls({
  maximized = false,
  isActive = true,
  previewState,
  onMinimize,
  onMaximize,
  onClose,
}) {
  const controls = [
    { action: "Minimize", path: "M4 8.5h8", onClick: onMinimize },
    {
      action: maximized ? "Restore" : "Maximize",
      path: maximized ? "M6 4.5h6v6M4 6.5h6v6H4z" : "M4 4.5h8v8H4z",
      onClick: onMaximize,
    },
    { action: "Close", path: "m4.5 4.5 7 7m0-7-7 7", onClick: onClose },
  ];
  return (
    <div
      className={`ubuntu-window-controls ${isActive ? "" : "is-inactive"}`}
      role="group"
      aria-label="Ubuntu window controls (design preview)"
    >
      {controls.map(({ action, path, onClick }) => (
        <button
          key={action}
          type="button"
          className={previewState ? `is-${previewState}` : undefined}
          aria-label={`${action} preview window`}
          title={`${action} (preview)`}
          onClick={onClick}
        >
          <span>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d={path} stroke="currentColor" strokeWidth="1.25" />
            </svg>
          </span>
        </button>
      ))}
    </div>
  );
}
