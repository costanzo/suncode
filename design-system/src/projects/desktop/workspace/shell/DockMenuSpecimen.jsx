import { useState } from "react";

export function DockMenuSpecimen() {
  const [context, setContext] = useState("multiple");
  const [feedback, setFeedback] = useState("");
  const empty = context === "empty";
  const blocked = context === "modal";
  const open = empty ? [] : context === "hub" ? [] : ["suncode", "design-system"];
  const recent = empty ? [] : ["suncode", "design-system", "sample-app"];
  const action = (label) => (
    <button type="button" className="workspace-native-menu-item" key={label} disabled={blocked}
      onClick={() => setFeedback(`${label}: ${context === "merged" ? "select the project tab in its existing window" : "open or activate the project window"}.`)}>
      <span className="workspace-native-menu-label">{label}</span>
    </button>
  );
  const submenu = (label, items, emptyLabel) => (
    <details className="workspace-native-menu-submenu" open key={`${label}-${context}`}>
      <summary className="workspace-native-menu-item" aria-disabled={blocked}
        onClick={event => { if (blocked) event.preventDefault(); }}>{label}</summary>
      <div className="workspace-native-menu-nested">
        {items.length ? items.map(action) : <button type="button" className="workspace-native-menu-item" disabled>{emptyLabel}</button>}
      </div>
    </details>
  );

  return (
    <div className="workspace-native-menu-specimen">
      <div className="workspace-native-menu-controls">
        <label>Dock context<select value={context} onChange={event => { setContext(event.target.value); setFeedback(""); }}>
          <option value="multiple">Multiple project windows</option>
          <option value="merged">Merged project tabs</option>
          <option value="hub">ProjectHub only</option>
          <option value="empty">No recent projects</option>
          <option value="modal">Modal dialog open</option>
        </select></label>
      </div>
      <div className="workspace-native-menu-review">
        <div className="workspace-native-menu-popover" aria-label="SunCode Dock menu">
          {action("Open Project…")}
          {submenu("Open Windows", open, "No Open Windows")}
          {submenu("Recent Projects", recent, "No Recent Projects")}
          <div className="workspace-native-menu-divider" />
          {action("Show Project Hub")}
        </div>
        <div className="workspace-native-menu-notes">
          <p>macOS only. Secondary-click the running SunCode icon in the Dock to open this application menu.</p>
          <p>Open Windows lists open projects, including projects in merged tabs. Choosing a project restores a minimized host and selects its tab.</p>
          <p>Recent Projects uses the existing SDK catalog order and shows at most ten projects. A project already open reuses its window; matching names include paths.</p>
          <p>The OS adds its own standard Dock commands. Those commands and any automatic OS window list are outside this custom section.</p>
          <p>Modal dialogs disable these navigation commands. The review fixture shows all submenus expanded for inspection.</p>
          <p role="status">{feedback}</p>
        </div>
      </div>
    </div>
  );
}
