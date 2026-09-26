import { Icon } from "../../../../shared/Icon.jsx";
import { shortcutCatalog } from "../data/catalog.js";

export function ShortcutsPanel() {
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <div className="settings-heading-row">
          <div>
            <h2>Keyboard shortcuts</h2>
            <p>View the shortcuts available throughout the desktop application.</p>
          </div>
          <span className="settings-read-only-badge">Read-only</span>
        </div>
      </div>
      <div className="settings-read-only-note" role="note">
        <Icon name="keyboard" size={16} />
        <span>
          Shortcut customization is not available yet. Editing will be added in a future release.
          The preview uses macOS notation; Windows and Linux use Ctrl where applicable.
        </span>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Available shortcuts</span>
        <div className="settings-shortcut-list" aria-label="Available keyboard shortcuts">
          {shortcutCatalog.map((shortcut) => (
            <div className="settings-shortcut-row" key={shortcut.action}>
              <span className="settings-shortcut-action">{shortcut.action}</span>
              <span className="settings-key-chord" aria-label={shortcut.ariaLabel}>
                {shortcut.keys.map((key) => (
                  <kbd key={key}>{key}</kbd>
                ))}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
