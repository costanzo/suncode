import { useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";

import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";

export function ComputerUsePanel({ onSave }) {
  const [enabled, setEnabled] = useState(false);
  const [capturePermission, setCapturePermission] = useState("denied");
  const [inputPermission, setInputPermission] = useState("allowed");
  const [controlOwner, setControlOwner] = useState("user");

  const toggleEnabled = (nextEnabled) => {
    setEnabled(nextEnabled);
    setControlOwner(nextEnabled ? "agent" : "user");
    onSave(
      nextEnabled
        ? "Computer Use enabled. Desktop input still requires approval."
        : "Computer Use disabled and held input was released.",
    );
  };
  const emergencyStop = () => {
    setEnabled(false);
    setControlOwner("user");
    onSave("Computer Use stopped. Held input was released.");
  };
  const requestCapturePermission = () => {
    setCapturePermission("allowed");
    onSave("Screen capture permission is available.");
  };
  const requestInputPermission = () => {
    setInputPermission("allowed");
    onSave("Input control permission is available.");
  };
  const takeControl = () => {
    setControlOwner("user");
    onSave("You control the desktop. Computer Use tools are paused.");
  };
  const returnControl = () => {
    setControlOwner("agent");
    onSave("Control returned. The agent must take a fresh screenshot before coordinate input.");
  };

  return (
    <div className="settings-panel-content settings-computer-content">
      <div className="settings-panel-heading">
        <h2>Computer use</h2>
        <p>
          Let supported models observe and operate the real primary display. Screenshots may be sent
          to the selected provider, and changes in other applications are outside filesystem undo.
        </p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Availability</span>
        <SettingRow
          label="Enable Computer Use"
          hint="Makes the built-in desktop tools available when the selected model and operating system support them."
        >
          <label className="settings-switch">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(event) => toggleEnabled(event.target.checked)}
              aria-label="Enable Computer Use"
            />
            <span className="settings-switch-track">
              <span />
            </span>
            <b>{enabled ? "On" : "Off"}</b>
          </label>
        </SettingRow>
        <div className="computer-runtime-summary" aria-live="polite">
          <div className="computer-runtime-state is-ready">
            <span className="settings-status-dot" />
            <div>
              <strong>Supported</strong>
              <span>Selected model · claude-sonnet-5</span>
            </div>
          </div>
          <div className={`computer-runtime-state ${enabled ? "is-ready" : "is-disabled"}`}>
            <span className="settings-status-dot" />
            <div>
              <strong>{enabled ? "Ready" : "Disabled"}</strong>
              <span>Built-in desktop backend</span>
            </div>
          </div>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Desktop access</span>
        <SettingRow label="Primary display" hint="Initial Computer Use is limited to one display.">
          <code className="computer-readonly-value">Built-in display · 3024 × 1964 px</code>
        </SettingRow>
        <SettingRow
          label="Screen capture"
          hint="Required to send screenshots to the selected provider."
        >
          <span className={`computer-inline-status is-${capturePermission}`}>
            <span className="settings-status-dot" />
            {capturePermission === "allowed" ? "Allowed" : "Permission needed"}
          </span>
        </SettingRow>
        <SettingRow
          label="Input control"
          hint="Required for mouse and keyboard actions after approval."
        >
          <span className={`computer-inline-status is-${inputPermission}`}>
            <span className="settings-status-dot" />
            {inputPermission === "allowed" ? "Allowed" : "Permission needed"}
          </span>
        </SettingRow>
        <SettingRow
          label="Current control owner"
          hint="The user and agent never control desktop input simultaneously."
        >
          <code className="computer-readonly-value">
            {controlOwner === "agent" ? "Agent · idle" : "User"}
          </code>
        </SettingRow>
        <div className="settings-actions">
          {controlOwner === "agent" ? (
            <Button size="sm" disabled={!enabled} onClick={takeControl}>
              Take control
            </Button>
          ) : (
            <Button variant="primary" size="sm" disabled={!enabled} onClick={returnControl}>
              Return control to agent
            </Button>
          )}
        </div>
        <div className="settings-actions">
          <Button
            size="sm"
            disabled={capturePermission === "allowed"}
            onClick={requestCapturePermission}
          >
            {capturePermission === "allowed" ? "Screen capture allowed" : "Request screen capture"}
          </Button>
          <Button
            size="sm"
            disabled={inputPermission === "allowed"}
            onClick={requestInputPermission}
          >
            {inputPermission === "allowed" ? "Input control allowed" : "Request input control"}
          </Button>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Safety and limits</span>
        <SettingRow
          label="Input approval"
          hint="Full Control does not bypass Computer Use approval."
        >
          <code className="computer-readonly-value">Once per input batch</code>
        </SettingRow>
        <SettingRow
          label="Execution mode"
          hint="Computer Use is denied in scripts and other non-interactive runs."
        >
          <code className="computer-readonly-value">Interactive only</code>
        </SettingRow>
        <SettingRow
          label="Emergency stop"
          hint="Releases held keys and mouse buttons and invalidates prior coordinates."
        >
          <Button variant="danger" size="sm" disabled={!enabled} onClick={emergencyStop}>
            Stop Computer Use
          </Button>
        </SettingRow>
        <SettingRow
          label="Temporary screenshots"
          hint="Screenshot bytes are retained only for the active turn."
        >
          <Button size="sm" disabled>
            Clear temporary screenshots
          </Button>
        </SettingRow>
      </div>
      <div className="computer-authority-note">
        <Icon name="lock" size={15} />
        <div>
          <strong>Computer Use controls the real desktop, not a sandbox</strong>
          <span>
            On-screen content is untrusted and cannot grant authority. Secure desktops and locked
            sessions are unavailable, and actions in external applications cannot be undone by
            SunCode.
          </span>
        </div>
      </div>
    </div>
  );
}
