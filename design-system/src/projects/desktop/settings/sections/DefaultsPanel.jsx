import { Button } from "../../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";
import { SettingRow } from "../components/SettingRow.jsx";
import { providerCatalog } from "../data/catalog.js";

export function DefaultsPanel({ onSave }) {
  const models = Object.values(providerCatalog).flatMap((provider) => provider.models);
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Defaults</h2>
        <p>Configure model and turn defaults.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Default model</span>
        <SettingRow label="Model" hint="Only models registered by the local agent appear here.">
          <SingleDropdown
            options={models}
            initialValue="deepseek-v4-flash"
            ariaLabel="Default model"
            className="settings-dropdown"
          />
        </SettingRow>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Turn execution</span>
        <SettingRow
          label="Tool-call limit"
          hint="Maximum number of tool calls allowed in one turn."
        >
          <input
            className="field settings-number"
            type="number"
            min="1"
            max="256"
            defaultValue="64"
            aria-label="Tool-call limit"
          />
        </SettingRow>
        <div className="settings-actions">
          <Button variant="primary" size="sm" onClick={onSave}>
            Save project limit
          </Button>
          <span className="settings-save-status" role="status">
            Project: suncode
          </span>
        </div>
      </div>
    </div>
  );
}
