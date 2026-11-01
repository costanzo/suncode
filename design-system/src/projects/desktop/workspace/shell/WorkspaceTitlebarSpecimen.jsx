import { useState } from "react";
import { UbuntuWindowControls } from "../../../../platforms/desktop/components/titlebar/index.js";
import { WorkspaceWindow } from "./WorkspaceShell.jsx";

export function WorkspaceTitlebarSpecimen() {
  const [platform, setPlatform] = useState("ubuntu");
  const [maximized, setMaximized] = useState(false);
  const [isActive, setIsActive] = useState(true);
  const [size, setSize] = useState("default");
  const [feedback, setFeedback] = useState("");
  const ubuntu = platform === "ubuntu";
  return (
    <div className="workspace-titlebar-specimen">
      <div className="workspace-native-menu-controls">
        <label>
          Platform
          <select
            value={platform}
            onChange={(event) => {
              setPlatform(event.target.value);
              setFeedback("");
            }}
          >
            <option value="ubuntu">Ubuntu (GNOME / Yaru)</option>
            <option value="macos">macOS</option>
          </select>
        </label>
        {ubuntu && (
          <>
            <label>
              Window
              <select
                value={maximized ? "maximized" : "normal"}
                onChange={(event) => setMaximized(event.target.value === "maximized")}
              >
                <option value="normal">Normal</option>
                <option value="maximized">Maximized</option>
              </select>
            </label>
            <label>
              Focus
              <select
                value={isActive ? "active" : "inactive"}
                onChange={(event) => setIsActive(event.target.value === "active")}
              >
                <option value="active">Active</option>
                <option value="inactive">Inactive</option>
              </select>
            </label>
            <label>
              Width
              <select value={size} onChange={(event) => setSize(event.target.value)}>
                <option value="default">Available width</option>
                <option value="minimum">Minimum · 620 DIP</option>
              </select>
            </label>
          </>
        )}
      </div>
      <div className="ubuntu-workspace-scroll">
        <div
          className={`ubuntu-workspace-preview ${ubuntu && size === "minimum" ? "is-minimum" : ""}`}
        >
          <WorkspaceWindow
            key={platform}
            titlebarPlatform={platform}
            windowMaximized={maximized}
            windowActive={isActive}
            onWindowMaximize={() => setMaximized((value) => !value)}
            onWindowMinimize={() => setFeedback("Minimize requested in the preview.")}
            onWindowClose={() => setFeedback("Close requested in the preview.")}
            initialDrawer={ubuntu ? null : "tools"}
          />
        </div>
      </div>
      {ubuntu && (
        <>
          <p className="ubuntu-titlebar-feedback" role="status">
            {feedback || "Design preview only. Window controls do not affect the browser window."}
          </p>
          <div className="ubuntu-caption-states" aria-label="Ubuntu caption button states">
            {[
              ["Normal", {}],
              ["Hover", { previewState: "hover" }],
              ["Pressed", { previewState: "pressed" }],
              ["Keyboard focus", { previewState: "focus" }],
              ["Inactive", { isActive: false }],
              ["Maximized", { maximized: true }],
            ].map(([label, props]) => (
              <div className="ubuntu-caption-state" key={label}>
                <span>{label}</span>
                <UbuntuWindowControls
                  {...props}
                  onMinimize={() => setFeedback(`${label}: Minimize requested in the preview.`)}
                  onMaximize={() =>
                    setFeedback(
                      `${label}: ${props.maximized ? "Restore" : "Maximize"} requested in the preview.`,
                    )
                  }
                  onClose={() => setFeedback(`${label}: Close requested in the preview.`)}
                />
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
