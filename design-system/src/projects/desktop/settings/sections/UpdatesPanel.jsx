import { useCallback, useEffect, useState } from "react";
import { Button } from "../../../../components/universal/button/index.js";
import { ConfirmationDialog } from "../../../../components/universal/modal/ConfirmationDialog.jsx";
import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";
import { updateComponents, updateMessages, updateScenarios } from "../data/updates.js";

export function UpdatesPanel({ scenario = "sdk" }) {
  const preset = updateScenarios.find((item) => item.value === scenario) ?? updateScenarios[4];
  const [phase, setPhase] = useState(preset.phase);
  const [kind] = useState(preset.kind ?? "sdk");
  const [progress, setProgress] = useState(46);
  const [playing, setPlaying] = useState(false);
  const [automaticChecks, setAutomaticChecks] = useState(true);
  const [confirmRestart, setConfirmRestart] = useState(false);
  const [installed, setInstalled] = useState(false);
  const [repaired, setRepaired] = useState(false);
  const closeRestart = useCallback(() => setConfirmRestart(false), []);
  const running = ["checking", "downloading", "verifying", "installing"].includes(phase);
  const ready = ["ready", "busy"].includes(phase);
  const hasPlan = ![
    "idle",
    "checking",
    "current",
    "check-failed",
    "installed",
    "installing",
    "restored",
    "unconfigured",
  ].includes(phase);
  const targets = updateComponents.filter((item) => kind === "paired" || item.id === kind);
  const size = targets.reduce((sum, item) => sum + item.size, 0);
  const summary = kind === "paired" ? "Desktop and Agent SDK" : targets[0].label;
  const error = phase.endsWith("failed");
  const warning = ["busy", "restored", "missing"].includes(phase);
  const [title, description] =
    phase === "available"
      ? [
          kind === "paired"
            ? "An update for SunCode is available"
            : `An update for ${summary} is available`,
          kind === "paired"
            ? "These versions need to be installed together."
            : "Download now and restart when you’re ready.",
        ]
      : phase === "installing"
        ? ["Restarting SunCode…", "Applying the downloaded update."]
        : updateMessages[phase];
  const icon =
    error || warning
      ? "warning"
      : ["current", "installed", "ready"].includes(phase)
        ? "check"
        : "refresh";

  // Only explicit specimen actions advance time; selected review states remain inspectable.
  useEffect(() => {
    if (!playing) return undefined;
    let timer;
    if (phase === "checking")
      timer = setTimeout(() => {
        setPhase(installed ? "current" : "available");
        setPlaying(false);
      }, 1000);
    if (phase === "downloading")
      timer = setTimeout(() => {
        if (progress >= 100) setPhase("verifying");
        else setProgress((value) => Math.min(100, value + 8));
      }, 320);
    if (phase === "verifying")
      timer = setTimeout(() => {
        setPhase("ready");
        setPlaying(false);
      }, 1100);
    if (phase === "installing")
      timer = setTimeout(() => {
        setInstalled(true);
        setRepaired(true);
        setPhase("installed");
        setPlaying(false);
      }, 1100);
    return () => clearTimeout(timer);
  }, [phase, playing, progress, installed]);

  const check = () => {
    setPhase("checking");
    setPlaying(true);
  };
  const download = () => {
    setProgress(0);
    setPhase("downloading");
    setPlaying(true);
  };
  const cancel = () => {
    setPlaying(false);
    setPhase(scenario === "missing" && !repaired ? "missing" : "available");
  };
  const showInstalled = installed || phase === "installed";

  return (
    <div className="settings-panel-content updates-content">
      <div className="settings-panel-heading updates-heading">
        <h2>Updates</h2>
        <p>Keep SunCode up to date.</p>
      </div>

      <div
        className="updates-summary"
        data-tone={error ? "danger" : warning ? "warning" : "neutral"}
      >
        <Icon name={icon} size={18} className={running ? "updates-working-icon" : ""} />
        <div role="status" aria-live="polite" aria-atomic="true">
          <h3>{title}</h3>
          <p>{description}</p>
        </div>
      </div>

      <div className="updates-components" aria-label="Component versions">
        {updateComponents.map((component) => {
          const affected = hasPlan && targets.includes(component);
          const upgraded = showInstalled && targets.includes(component);
          const unavailable = scenario === "missing" && component.id === "sdk" && !repaired;
          const rowStatus = unavailable
            ? "Unavailable"
            : affected && error
              ? "Update not installed"
              : affected
                ? ready
                  ? "Ready to install"
                  : phase === "downloading"
                    ? "Downloading"
                    : phase === "verifying"
                      ? "Verifying"
                      : "Update available"
                : ["current", "installed"].includes(phase) || hasPlan
                  ? "Up to date"
                  : "Installed";
          return (
            <div className="updates-component" key={component.id}>
              <strong>{component.label}</strong>
              <div className="updates-version">
                {unavailable ? (
                  <span>Unavailable</span>
                ) : (
                  <code>v{upgraded ? component.target : component.installed}</code>
                )}
                {affected && (
                  <>
                    <Icon name="arrow" size={13} />
                    <code>v{component.target}</code>
                  </>
                )}
              </div>
              <span className="updates-component-status">{rowStatus}</span>
            </div>
          );
        })}
      </div>

      {["downloading", "verifying"].includes(phase) && (
        <div className="updates-progress">
          <progress
            max="100"
            value={phase === "downloading" ? progress : undefined}
            aria-label={
              phase === "downloading" ? `Downloading ${summary}` : "Verifying downloaded update"
            }
          />
          <span>
            {phase === "downloading"
              ? `${Math.round((size * progress) / 100)} of ${size} MB · ${progress}%`
              : "Checking download integrity"}
          </span>
        </div>
      )}

      <div className="updates-actions">
        {ready ? (
          <Button
            variant="primary"
            size="sm"
            disabled={phase === "busy"}
            aria-describedby={phase === "busy" ? "updates-busy-reason" : undefined}
            onClick={() => setConfirmRestart(true)}
          >
            Restart SunCode
          </Button>
        ) : phase === "downloading" || phase === "verifying" ? (
          <Button size="sm" onClick={cancel}>
            {phase === "verifying" ? "Cancel update" : "Cancel download"}
          </Button>
        ) : hasPlan ? (
          <Button variant="primary" size="sm" onClick={download}>
            {phase === "missing"
              ? "Repair Agent SDK"
              : error
                ? "Download again"
                : "Download update"}
          </Button>
        ) : (
          <Button size="sm" disabled={running || phase === "unconfigured"} onClick={check}>
            {phase === "checking"
              ? "Checking…"
              : phase === "installing"
                ? "Restarting…"
                : phase === "check-failed"
                  ? "Try again"
                  : "Check for updates"}
          </Button>
        )}
        {hasPlan && !ready && !running && (
          <span>
            {size} MB · {kind === "paired" ? "Both components" : `${summary} only`}
          </span>
        )}
        {ready && <span>Downloaded · {summary}</span>}
        {phase === "available" && (
          <Button variant="quiet" size="sm" onClick={check} className="updates-recheck">
            Check again
          </Button>
        )}
      </div>

      {phase === "busy" && (
        <p id="updates-busy-reason" className="updates-busy-reason">
          1 turn and 1 terminal are still running. Your download is saved.
        </p>
      )}
      {phase === "missing" && (
        <p className="updates-footnote">
          Settings and update repair are available while Agent SDK is unavailable.
        </p>
      )}

      {hasPlan && phase !== "missing" && (
        <details className="updates-release-notes">
          <summary>What’s new</summary>
          {targets.map((component) => (
            <div key={component.id}>
              <strong>
                {component.label} <code>v{component.target}</code>
              </strong>
              <p>
                {component.id === "desktop"
                  ? "Settings layout and keyboard navigation improvements."
                  : "Model compatibility and agent reliability improvements."}
              </p>
            </div>
          ))}
        </details>
      )}

      <div className="updates-preferences">
        <SettingRow
          label="Automatically check for updates"
          hint="Check at startup and periodically while SunCode is open."
        >
          <label className="settings-switch updates-switch">
            <input
              type="checkbox"
              checked={automaticChecks}
              onChange={(event) => setAutomaticChecks(event.target.checked)}
              aria-label="Automatically check for updates"
            />
            <span className="settings-switch-track">
              <span />
            </span>
          </label>
        </SettingRow>
        <p>Downloads start when you choose. Updates take effect after a restart.</p>
      </div>

      <ConfirmationDialog
        open={confirmRestart}
        title="Restart SunCode to update?"
        description={`The downloaded ${summary} update will be installed. Save your work in every SunCode window before continuing.`}
        confirmLabel="Restart and install"
        cancelLabel="Later"
        confirmVariant="primary"
        onCancel={closeRestart}
        onConfirm={() => {
          setConfirmRestart(false);
          setPhase("installing");
          setPlaying(true);
        }}
      />
    </div>
  );
}
