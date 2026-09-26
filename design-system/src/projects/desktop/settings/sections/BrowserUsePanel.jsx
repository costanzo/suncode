import { useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";

import { ConfirmationDialog } from "../../../../components/universal/modal/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";

const browserRuntimeComponents = [
  {
    label: "Target",
    value: "darwin-arm64",
    detail: "Bundled for this SunCode installation",
  },
  {
    label: "Node.js",
    value: "v24.11.1",
    detail: "/Applications/SunCode.app/Contents/Resources/browser-runtime/node/bin/node",
  },
  {
    label: "Playwright",
    value: "1.55.0",
    detail: "Exact production package",
  },
  {
    label: "Chromium",
    value: "140.0.7339.16 · revision 1187",
    detail:
      "/Applications/SunCode.app/Contents/Resources/browser-runtime/browsers/chromium-1187/Chromium.app",
  },
  {
    label: "Worker protocol",
    value: "1",
    detail: "Rust-managed framed stdio",
  },
];

export function BrowserUsePanel({ onSave }) {
  const [enabled, setEnabled] = useState(true);
  const [installationState, setInstallationState] = useState("ready");
  const [runtimeState, setRuntimeState] = useState("background");
  const [clearOpen, setClearOpen] = useState(false);

  const installationLabel = {
    disabled: "Disabled",
    ready: "Ready",
    verifying: "Verifying",
    invalid: "Integrity failed",
  }[enabled ? installationState : "disabled"];
  const runtimeLabel = {
    not_started: "Not started",
    starting: "Starting",
    background: "Running in background",
    user_controlled: "User controlled",
    failed: "Failed",
  }[enabled ? runtimeState : "not_started"];

  const toggleEnabled = (nextEnabled) => {
    setEnabled(nextEnabled);
    if (!nextEnabled) setRuntimeState("not_started");
    onSave(
      nextEnabled
        ? "Browser use enabled. Chromium starts only when a project needs it."
        : "Browser use disabled. Active browser runtimes were stopped.",
    );
  };
  const verifyRuntime = () => {
    setInstallationState("verifying");
    window.setTimeout(() => {
      setInstallationState("ready");
      onSave("Bundled browser runtime verified.");
    }, 700);
  };
  const takeControl = () => {
    setRuntimeState("user_controlled");
    onSave("Browser tools paused while you control Chromium.");
  };
  const returnControl = () => {
    setRuntimeState("background");
    onSave("Control returned. The agent must take a fresh page snapshot.");
  };
  const restartRuntime = () => {
    setRuntimeState("starting");
    window.setTimeout(() => {
      setRuntimeState("background");
      onSave("Project browser restarted with its persistent profile.");
    }, 700);
  };
  const stopRuntime = () => {
    setRuntimeState("not_started");
    onSave("Project browser stopped. Its profile was preserved.");
  };
  const clearProfile = () => {
    setRuntimeState("not_started");
    setClearOpen(false);
    onSave("Browser data cleared for project suncode.");
  };

  return (
    <div className="settings-panel-content settings-browser-content">
      <div className="settings-panel-heading">
        <h2>Browser use</h2>
        <p>
          Run the bundled Playwright Chromium for dynamic web inspection and interaction. Website
          changes and browser data are outside filesystem undo.
        </p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Availability</span>
        <SettingRow
          label="Enable Browser Use"
          hint="Makes browser tools available. Node.js and Chromium start lazily when a project needs them."
        >
          <label className="settings-switch">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(event) => toggleEnabled(event.target.checked)}
              aria-label="Enable Browser Use"
            />
            <span className="settings-switch-track">
              <span />
            </span>
            <b>{enabled ? "On" : "Off"}</b>
          </label>
        </SettingRow>
        <div className="browser-runtime-summary" aria-live="polite">
          <div className={`browser-runtime-state is-${enabled ? installationState : "disabled"}`}>
            <span className="settings-status-dot" />
            <div>
              <strong>{installationLabel}</strong>
              <span>Bundled installation</span>
            </div>
          </div>
          <div className={`browser-runtime-state is-${enabled ? runtimeState : "not_started"}`}>
            <span className="settings-status-dot" />
            <div>
              <strong>{runtimeLabel}</strong>
              <span>Project: suncode</span>
            </div>
          </div>
        </div>
        <div className="settings-actions">
          <Button
            size="sm"
            disabled={!enabled || installationState === "verifying"}
            onClick={verifyRuntime}
          >
            {installationState === "verifying" ? "Verifying…" : "Verify runtime"}
          </Button>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Bundled runtime</span>
        <div className="browser-component-list" aria-label="Bundled Browser Use components">
          {browserRuntimeComponents.map((component) => (
            <div className="browser-component-row" key={component.label}>
              <span>{component.label}</span>
              <div>
                <code>{component.value}</code>
                <small title={component.detail}>{component.detail}</small>
              </div>
              <button
                type="button"
                className="btn btn-icon btn-quiet"
                aria-label={`Copy ${component.label}`}
                title={`Copy ${component.label}`}
                onClick={() => navigator.clipboard?.writeText(component.detail)}
              >
                <Icon name="copy" size={14} />
              </button>
            </div>
          ))}
        </div>
        <div className="browser-integrity-note is-ready">
          <Icon name="lock" size={15} />
          <div>
            <strong>Integrity verified</strong>
            <span>
              Versions, packaged paths, and runtime trees match the application-protected manifest.
            </span>
          </div>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Current project browser</span>
        <SettingRow
          label="Profile scope"
          hint="Cookies and site data persist only for this project."
        >
          <code className="browser-readonly-value">Persistent per project</code>
        </SettingRow>
        <SettingRow label="Profile path" hint="Stored under the local SunCode data directory.">
          <code
            className="browser-readonly-value"
            title="~/.suncode/data/browser/profiles/prj_suncode"
          >
            ~/.suncode/data/browser/profiles/prj_suncode
          </code>
        </SettingRow>
        <SettingRow label="Profile usage" hint="Browser data is not part of the opened project.">
          <code className="browser-readonly-value">38.4 MB · 2 active pages</code>
        </SettingRow>
        <SettingRow
          label="Window control"
          hint="The full Chromium window is minimized in background mode. Wayland may require manual foreground selection."
        >
          <code className="browser-readonly-value">Full · macOS</code>
        </SettingRow>
        <div className="browser-control-note">
          <Icon name={runtimeState === "user_controlled" ? "unlock" : "lock"} size={15} />
          <div>
            <strong>
              {runtimeState === "user_controlled"
                ? "You control Chromium"
                : "Agent control is active"}
            </strong>
            <span>
              {runtimeState === "user_controlled"
                ? "Browser tools are paused. Returning control invalidates previous element references."
                : "Showing the browser transfers exclusive control to you and pauses browser tools."}
            </span>
          </div>
        </div>
        <div className="settings-actions browser-runtime-actions">
          {runtimeState === "user_controlled" ? (
            <Button variant="primary" size="sm" onClick={returnControl}>
              Return control to agent
            </Button>
          ) : (
            <Button
              size="sm"
              disabled={!enabled || runtimeState !== "background"}
              onClick={takeControl}
            >
              Show browser and take control
            </Button>
          )}
          <Button
            size="sm"
            disabled={!enabled || runtimeState === "not_started"}
            onClick={restartRuntime}
          >
            Restart
          </Button>
          <Button
            size="sm"
            disabled={!enabled || runtimeState === "not_started"}
            onClick={stopRuntime}
          >
            Stop
          </Button>
          <Button
            variant="danger"
            size="sm"
            disabled={runtimeState !== "not_started"}
            onClick={() => setClearOpen(true)}
          >
            Clear browser data
          </Button>
        </div>
      </div>
      <div className="browser-authority-note">
        <Icon name="lock" size={15} />
        <div>
          <strong>Browser access is not a sandbox or an authority grant</strong>
          <span>
            Page content is untrusted. Sensitive transmission and consequential web actions still
            require confirmation at the point of risk.
          </span>
        </div>
      </div>
      <ConfirmationDialog
        open={clearOpen}
        title="Clear browser data?"
        description="Saved logins, cookies, site storage, and browsing state for this project will be removed. Project files are unchanged."
        confirmLabel="Clear browser data"
        onCancel={() => setClearOpen(false)}
        onConfirm={clearProfile}
      >
        <div className="confirmation-dialog-target">
          <span>PROJECT BROWSER PROFILE</span>
          <strong>suncode</strong>
        </div>
      </ConfirmationDialog>
    </div>
  );
}
