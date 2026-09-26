import { useState } from "react";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";
import { SettingRow } from "../components/SettingRow.jsx";

export function AppearancePanel({ onSave }) {
  const [theme, setTheme] = useState(() => document.documentElement.dataset.theme || "light");
  const applyTheme = (nextTheme) => {
    setTheme(nextTheme);
    document.documentElement.dataset.theme = nextTheme;
    try {
      window.localStorage.setItem("suncode-design-theme", nextTheme);
    } catch {
      /* non-fatal */
    }
    onSave();
  };
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Appearance</h2>
        <p>Adjust how SunCode looks across every open window.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Theme</span>
        <SettingRow label="Color theme" hint="Changes apply immediately.">
          <SingleDropdown
            options={[
              { value: "dark", label: "Dark" },
              { value: "light", label: "Light" },
            ]}
            value={theme}
            onChange={applyTheme}
            ariaLabel="Color theme"
            className="settings-dropdown"
          />
        </SettingRow>
      </div>
    </div>
  );
}
