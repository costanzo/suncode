import { useEffect, useState } from "react";
import { Button } from "../../../../components/universal/button/index.js";
import { ConfirmationDialog } from "../../../../components/universal/modal/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";
import {
  browserExampleHash,
  browserPhaseLabels,
  browserPhaseMessages,
  browserScenarios,
  browserSources,
  browserTargets,
  browserVersion,
} from "../data/browser.js";

export function BrowserUsePanel({ onSave, scenario = "missing", platform = "macos" }) {
  const preset = browserScenarios.find((item) => item.value === scenario) ?? browserScenarios[0];
  const [enabled, setEnabled] = useState(preset.enabled ?? true);
  const [phase, setPhase] = useState(preset.phase);
  const [installed, setInstalled] = useState(preset.installed ?? false);
  const [installedVersion, setInstalledVersion] = useState(
    preset.phase === "incompatible" ? "120.2.7" : browserVersion,
  );
  const [runtime, setRuntime] = useState(preset.runtime ?? "not_started");
  const [source, setSource] = useState("official");
  const [progress, setProgress] = useState(42);
  const [playing, setPlaying] = useState(false);
  const [operation, setOperation] = useState("install");
  const [previousPhase, setPreviousPhase] = useState(
    ["downloading", "verifying", "installing"].includes(preset.phase) ? "missing" : preset.phase,
  );
  const [clearOpen, setClearOpen] = useState(false);
  const [restartOpen, setRestartOpen] = useState(false);
  const [uninstallOpen, setUninstallOpen] = useState(false);
  const [loaded, setLoaded] = useState(Boolean(preset.runtime) || preset.phase === "restart_required");
  const [profileCleared, setProfileCleared] = useState(false);
  const hasProject = scenario !== "no_project";
  const busy = ["downloading", "verifying", "installing", "uninstalling"].includes(phase);
  const uninstallPending = ["uninstall_restart_required", "uninstall_failed"].includes(phase);
  const restartRequired = ["restart_required", "uninstall_restart_required"].includes(phase);
  const broken = ["corrupt", "incompatible", "restart_required", "uninstall_restart_required", "uninstall_failed"].includes(phase);
  const usable = installed && !broken && phase !== "unsupported";
  const active = ["background", "user_controlled", "starting"].includes(runtime);
  const canStart = enabled && usable && !busy && hasProject;
  const error = [
    "download_failed",
    "hash_mismatch",
    "corrupt",
    "incompatible",
    "source_missing",
    "uninstall_failed",
  ].includes(phase);
  const target = browserTargets[platform];
  const runtimeLabels = {
    not_started: "Not started",
    starting: "Starting",
    background: "Running",
    user_controlled: "User controlled",
    failed: "Startup failed",
  };

  // Preset states stay still for review. Only explicit actions run the simulation.
  useEffect(() => {
    if (!playing) return undefined;
    const timer = setTimeout(
      () => {
        if (phase === "downloading") {
          if (progress >= 100) setPhase("verifying");
          else setProgress((value) => Math.min(100, value + 20));
        } else if (phase === "verifying") {
          if (operation === "verify") {
            setPhase(previousPhase);
            setPlaying(false);
          } else setPhase("installing");
        } else if (phase === "installing") {
          setInstalled(true);
          setInstalledVersion(browserVersion);
          setPhase("ready");
          setPlaying(false);
        } else if (phase === "uninstalling") {
          setInstalled(false);
          setPhase("missing");
          setPlaying(false);
          onSave("Chromium uninstalled. Saved browser data is retained.");
        } else if (runtime === "starting") {
          setRuntime("background");
          setPlaying(false);
        }
      },
      phase === "downloading" ? 400 : 1000,
    );
    return () => clearTimeout(timer);
  }, [playing, phase, progress, runtime, operation, previousPhase, onSave]);

  const download = () => {
    setPreviousPhase(phase);
    setOperation("install");
    setProgress(0);
    setPhase("downloading");
    setPlaying(true);
  };
  const cancel = () => {
    setPlaying(false);
    setPhase(previousPhase);
    onSave(
      operation === "verify"
        ? "File verification canceled."
        : "Installation canceled. Temporary download files were removed.",
    );
  };
  const toggleEnabled = (next) => {
    setEnabled(next);
    if (!next) {
      setRuntime("not_started");
      if (!busy) setPlaying(false);
    }
    onSave(
      next
        ? "Browser Use enabled. Chromium is checked before every browser operation."
        : "Browser Use disabled. Project browser sessions were stopped.",
    );
  };
  const start = () => {
    setLoaded(true);
    setRuntime("starting");
    setPlaying(true);
  };
  const verify = () => {
    setPreviousPhase(phase);
    setOperation("verify");
    setPhase("verifying");
    setPlaying(true);
  };
  const components = [
    { label: "Target", value: target.label },
    { label: "Required version", value: browserVersion },
    { label: "NuGet package", value: target.package },
    { label: "Package SHA-256", value: browserExampleHash },
    { label: "Manifest", value: "SunCode trusted component manifest" },
    { label: "Installed version", value: installed ? installedVersion : "Not installed" },
    {
      label: "Component path",
      value: installed
        ? `<SunCode data>/browser/runtimes/${platform}/${installedVersion}`
        : "Not installed",
    },
    { label: "CDP endpoint", value: active ? "127.0.0.1:9222" : "Not started" },
  ];

  return (
    <div className="settings-panel-content settings-browser-content">
      <div className="settings-panel-heading">
        <h2>Browser use</h2>
        <p>Browse and inspect web pages with the built-in CEF browser.</p>
      </div>
      <div className="settings-panel-section">
        <SettingRow
          label="Enable Browser Use"
          hint="Chromium starts when a browser operation needs it. Sensitive or consequential actions still require approval."
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
      </div>
      <div className="settings-divider" />
      <section
        className="settings-panel-section browser-install"
        aria-labelledby="browser-component-heading"
        aria-busy={busy}
      >
        <div className="browser-install-heading">
          <h3 id="browser-component-heading">Chromium</h3>
          <span
            className="browser-install-status"
            data-tone={
              error
                ? "danger"
                : busy || restartRequired
                  ? "warning"
                  : usable
                    ? "success"
                    : "muted"
            }
            role="status"
            aria-live="polite"
          >
            <span className="settings-status-dot" />
            {operation === "verify" && phase === "verifying"
              ? "Verifying installed files"
              : browserPhaseLabels[phase]}
          </span>
        </div>
        <p>Download the compatible component for Browser Use and the built-in browser.</p>
        <div className="browser-install-identity">
          <code>
            {target.label} · {browserVersion}
          </code>
          {!installed && <span>Installed on demand</span>}
        </div>
        <SettingRow label="Download source">
          <select
            className="field browser-source"
            aria-label="Chromium download source"
            disabled={busy || phase === "unsupported" || restartRequired || uninstallPending}
            value={source}
            onChange={(event) => setSource(event.target.value)}
          >
            {Object.entries(browserSources).map(([id, item]) => (
              <option key={id} value={id}>
                {item.label}
              </option>
            ))}
          </select>
        </SettingRow>
        <p className="browser-source-url">
          <code>{browserSources[source].url}</code>
        </p>
        <div
          className="browser-install-message"
          data-tone={error ? "danger" : "neutral"}
          role={error ? "alert" : "status"}
          aria-live="polite"
          aria-atomic="true"
        >
          {error && <Icon name="warning" size={15} />}
          <div>
            <p>
              {operation === "verify" && phase === "verifying"
                ? "Checking installed files against the installation’s file manifest."
                : browserPhaseMessages[phase]}
            </p>
            {installed &&
              ["hash_mismatch", "download_failed", "source_missing"].includes(phase) && (
                <p>The previously installed component is still available.</p>
              )}
          </div>
        </div>
        {busy && (
          <div className="browser-install-progress">
            <progress
              max="100"
              value={phase === "downloading" ? progress : undefined}
              aria-label={browserPhaseLabels[phase]}
            />
            <span>
              {phase === "downloading"
                ? `${progress}% · ${Math.round((180 * progress) / 100)} of 180 MB`
                : operation === "verify"
                  ? "Verifying installed files"
                  : browserPhaseLabels[phase]}
            </span>
          </div>
        )}
        <div className="settings-actions browser-install-actions">
          {busy && phase !== "uninstalling" ? (
            <Button size="sm" onClick={cancel}>
              {operation === "verify" ? "Cancel verification" : "Cancel installation"}
            </Button>
          ) : restartRequired ? (
            <Button size="sm" variant="primary" onClick={() => setRestartOpen(true)}>
              Restart SunCode
            </Button>
          ) : (
            !busy && !uninstallPending && phase !== "unsupported" && (
              <>
                <Button
                  size="sm"
                  variant={usable ? "neutral" : "primary"}
                  disabled={active}
                  onClick={download}
                >
                  {phase === "incompatible"
                    ? "Install compatible version"
                    : phase === "corrupt"
                      ? "Repair Chromium"
                      : error
                        ? "Retry download"
                        : installed
                          ? "Repair Chromium"
                          : "Download Chromium"}
                </Button>
                {active && <span>Stop the browser before repairing or uninstalling Chromium.</span>}
              </>
            )
          )}
          {installed && !busy && phase !== "uninstall_restart_required" && (
            <Button size="sm" variant="danger" disabled={active} onClick={() => setUninstallOpen(true)}>
              {phase === "uninstall_failed" ? "Retry uninstall" : "Uninstall Chromium"}
            </Button>
          )}
        </div>
        <p className="browser-check-note">
          Downloads are verified with SHA-256 before installation. Every browser operation checks
          component availability and compatibility.
        </p>
      </section>
      <div className="settings-divider" />
      <section className="settings-panel-section" aria-labelledby="project-browser-heading">
        <h3 id="project-browser-heading" className="browser-section-heading">
          Current project browser
        </h3>
        {!hasProject ? (
          <p className="browser-project-message">
            Open a project to manage its browser and saved site data.
          </p>
        ) : (
          <>
            <div className="browser-project-status">
              <strong>suncode</strong>
              <span className={`browser-runtime-state is-${runtime}`}>
                <span className="settings-status-dot" />
                {runtimeLabels[runtime]}
              </span>
            </div>
            <p className="browser-project-message">
              {!enabled
                ? "Enable Browser Use to start a project browser."
                : !usable
                  ? "Install a compatible Chromium component to start this project’s browser."
                  : runtime === "user_controlled"
                    ? "Browser tools are paused while you control Chromium. Returning control invalidates previous page references."
                    : runtime === "background"
                      ? "The agent controls this project’s page."
                      : runtime === "failed"
                        ? "Chromium could not start. Retry or check the component in Advanced diagnostics."
                        : "The browser starts on the first browser operation."}
            </p>
            <div className="settings-actions browser-runtime-actions">
              {runtime === "user_controlled" ? (
                <Button size="sm" variant="primary" onClick={() => setRuntime("background")}>
                  Return control to agent
                </Button>
              ) : runtime === "background" ? (
                <Button size="sm" onClick={() => setRuntime("user_controlled")}>
                  Show browser and take control
                </Button>
              ) : (
                <Button size="sm" disabled={!canStart || runtime === "starting"} onClick={start}>
                  {runtime === "starting"
                    ? "Starting…"
                    : runtime === "failed"
                      ? "Retry browser"
                      : "Start browser"}
                </Button>
              )}
              <Button
                size="sm"
                disabled={!active}
                onClick={() => {
                  setRuntime("not_started");
                  if (!busy) setPlaying(false);
                }}
              >
                Stop
              </Button>
              <Button
                size="sm"
                variant="danger"
                disabled={active || busy || profileCleared}
                onClick={() => setClearOpen(true)}
              >
                Clear browser data
              </Button>
            </div>
          </>
        )}
      </section>
      <details className="browser-diagnostics">
        <summary>Advanced diagnostics</summary>
        <div className="browser-diagnostics-content">
          <div className="settings-actions">
            <Button
              size="sm"
              disabled={
                !installed ||
                busy ||
                active ||
                phase === "restart_required" ||
                uninstallPending ||
                phase === "unsupported"
              }
              onClick={verify}
            >
              Verify installed files
            </Button>
          </div>
          <div className="browser-component-list" aria-label="Browser component identity">
            {components.map(({ label, value }) => (
              <div className="browser-component-row" key={label}>
                <span>{label}</span>
                <code title={value}>{value}</code>
                <button
                  type="button"
                  className="btn btn-icon btn-quiet"
                  aria-label={`Copy ${label}`}
                  title={`Copy ${label}`}
                  onClick={() => navigator.clipboard?.writeText(value)}
                >
                  <Icon name="copy" size={14} />
                </button>
              </div>
            ))}
          </div>
          <p className="browser-check-note">
            Both download sources use the same expected package SHA-256 from SunCode’s trusted
            manifest. Installed files are checked against the installation’s file manifest.
          </p>
          {hasProject && (
            <>
              <SettingRow label="Profile path">
                <code
                  className="browser-readonly-value"
                  title="<SunCode data>/cef/browser--project"
                >
                  &lt;SunCode data&gt;/cef/browser--project
                </code>
              </SettingRow>
              <SettingRow label="Profile usage">
                <code className="browser-readonly-value">
                  {profileCleared ? "0 MB" : "38.4 MB"} · {active ? 1 : 0} active pages
                </code>
              </SettingRow>
            </>
          )}
        </div>
      </details>
      <p className="browser-check-note browser-authority-copy">
        Websites and saved browser data are outside filesystem undo. Page content is untrusted.
        Sensitive or consequential web actions still require approval.
      </p>
      <ConfirmationDialog
        open={clearOpen}
        title="Clear browser data?"
        description="Remove saved logins, cookies, and site data for project suncode."
        confirmLabel="Clear browser data"
        onCancel={() => setClearOpen(false)}
        onConfirm={() => {
          setProfileCleared(true);
          setClearOpen(false);
          onSave("Browser data cleared for project suncode.");
        }}
      />
      <ConfirmationDialog
        open={uninstallOpen}
        title="Uninstall Chromium?"
        description="Remove the downloaded Chromium runtime and temporary packages. Saved browser data is retained. Download Chromium again to use browser features. A loaded runtime is removed after restarting SunCode."
        confirmLabel="Uninstall Chromium"
        onCancel={() => setUninstallOpen(false)}
        onConfirm={() => {
          setUninstallOpen(false);
          setRuntime("not_started");
          if (loaded) {
            setPhase("uninstall_restart_required");
          } else {
            setPhase("uninstalling");
            setPlaying(true);
          }
        }}
      />
      <ConfirmationDialog
        open={restartOpen}
        title="Restart SunCode?"
        description={uninstallPending
          ? "Restart to finish removing Chromium. Active turns must finish or be stopped first."
          : "Restart to activate the verified Chromium component. Active turns must finish or be stopped first."}
        confirmLabel="Restart SunCode"
        onCancel={() => setRestartOpen(false)}
        onConfirm={() => {
          setRestartOpen(false);
          setPhase(uninstallPending ? "missing" : "ready");
          setInstalled(!uninstallPending);
          setLoaded(false);
          setRuntime("not_started");
          onSave(uninstallPending ? "Chromium uninstalled. Saved browser data is retained." : "Design preview restarted. Chromium is ready.");
        }}
      />
    </div>
  );
}
