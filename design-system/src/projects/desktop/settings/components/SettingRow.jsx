export function SettingRow({ label, hint, children, className = "" }) {
  return (
    <div className={`settings-row ${className}`}>
      <div className="settings-row-copy">
        <strong>{label}</strong>
        {hint && <span>{hint}</span>}
      </div>
      <div className="settings-row-control">{children}</div>
    </div>
  );
}
