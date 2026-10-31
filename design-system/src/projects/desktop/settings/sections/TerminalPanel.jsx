import { useEffect, useRef, useState } from "react";
import { Button } from "../../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import {
  existingPowerShell,
  installFailures,
  managedPowerShell,
  shellPlatforms,
} from "../data/shells.js";

function sameShell(a, b) {
  return a.mode === b.mode && (a.mode !== "custom" || a.path.trim() === b.path.trim());
}

function probeFixture(path, platform) {
  if (!path.trim()) return { state: "empty", message: "Choose a shell executable." };
  if (/missing|not-found/i.test(path))
    return { state: "error", message: "Executable not found. Choose an existing shell." };
  if (/timeout/i.test(path))
    return { state: "error", message: "The shell did not respond. Check the path and try again." };
  const absolute = platform === "windows" ? /^[a-z]:[\\/]/i.test(path) : path.startsWith("/");
  if (!absolute) return { state: "error", message: "Enter an absolute executable path." };
  const name = path.trim().split(/[\\/]/).pop();
  const versions = {
    "pwsh.exe": path === existingPowerShell.path ? existingPowerShell.version : "PowerShell 7.6.0",
    "powershell.exe": "Windows PowerShell 5.1",
    sh: "POSIX sh",
    bash: "GNU bash 5.2",
    zsh: "zsh 5.9",
    pwsh: "PowerShell 7.6.0",
  };
  const version = versions[name];
  if (
    !version ||
    (platform === "windows" && !name.endsWith(".exe")) ||
    (platform !== "windows" && name.endsWith(".exe"))
  ) {
    return {
      state: "error",
      message: "Unsupported shell. Choose PowerShell, sh, bash, or zsh for this platform.",
    };
  }
  return { state: "available", message: `${version} · Available` };
}

function ShellEditor({
  kind,
  platform,
  installed,
  managed,
  appliedShell,
  onApply,
  disabled,
  scenario,
}) {
  const label = kind === "agent" ? "Agent shell" : "Terminal shell";
  const system = shellPlatforms[platform][kind];
  const [draft, setDraft] = useState(() =>
    scenario === "missing" && kind === "agent"
      ? { mode: "custom", path: platform === "windows" ? "C:\\missing\\pwsh.exe" : "/missing/bash" }
      : scenario === "timeout" && kind === "agent"
        ? {
            mode: "custom",
            path: platform === "windows" ? "C:\\timeout\\pwsh.exe" : "/timeout/bash",
          }
        : scenario === "unsupported" && kind === "agent"
          ? {
              mode: "custom",
              path: platform === "windows" ? "C:\\Tools\\unknown.exe" : "/usr/local/bin/unknown",
            }
          : appliedShell,
  );
  const [probe, setProbe] = useState({ state: "checking", message: "Checking shell…" });
  const [attempt, setAttempt] = useState(0);
  const [saved, setSaved] = useState(false);
  const lastApplied = useRef(appliedShell);
  const path =
    draft.mode === "system" ? system.path : draft.mode === "managed" ? managed.path : draft.path;
  const dirty = !sameShell(draft, appliedShell);

  useEffect(() => {
    if (lastApplied.current !== appliedShell) {
      lastApplied.current = appliedShell;
      setDraft(appliedShell);
    }
  }, [appliedShell]);

  useEffect(() => {
    setSaved(false);
    if (draft.mode === "custom" && !path.trim()) {
      setProbe({ state: "empty", message: "Choose a shell executable." });
      return;
    }
    setProbe({ state: "checking", message: "Checking shell…" });
    const timer = setTimeout(
      () =>
        setProbe(
          draft.mode === "custom"
            ? probeFixture(path, platform)
            : {
                state: "available",
                message: `${draft.mode === "system" ? system.version : managed.version} · Available`,
              },
        ),
      700,
    );
    return () => clearTimeout(timer);
  }, [draft.mode, path, platform, system.version, managed.version, attempt]);

  const options = [
    { value: "system", label: "System default" },
    ...(installed ? [{ value: "managed", label: "PowerShell 7" }] : []),
    { value: "custom", label: "Custom path…" },
  ];
  const apply = (event) => {
    event.preventDefault();
    if (probe.state !== "available" || !dirty || disabled) return;
    onApply({ ...draft, path: path.trim() });
    setSaved(true);
  };

  return (
    <form className="shell-editor" onSubmit={apply} aria-label={`${label} configuration`}>
      <fieldset disabled={disabled}>
        <div className="shell-editor-heading">
          <div>
            <h3>{label}</h3>
            <p>
              {kind === "agent"
                ? "Applies to the next turn."
                : "Applies to new or restarted terminal tabs."}
            </p>
          </div>
          <SingleDropdown
            options={options}
            value={draft.mode}
            ariaLabel={label}
            className="settings-dropdown"
            onChange={(mode) => setDraft({ mode, path: draft.path })}
          />
        </div>
        {draft.mode === "custom" ? (
          <div className="shell-path-input">
            <input
              className="field mono"
              aria-label={`${label} executable path`}
              aria-describedby={`${kind}-shell-result`}
              aria-invalid={probe.state === "error"}
              value={draft.path}
              placeholder={shellPlatforms[platform].example}
              spellCheck="false"
              autoComplete="off"
              onChange={(event) => setDraft({ mode: "custom", path: event.target.value })}
            />
            <Button
              type="button"
              variant="quiet"
              size="sm"
              aria-label={`Choose ${label.toLowerCase()} executable`}
              title="Choose executable (preview)"
              icon="folder"
              onClick={() => setDraft({ mode: "custom", path: shellPlatforms[platform].example })}
            />
          </div>
        ) : (
          <code className="shell-resolved-path" title={path}>
            {path}
          </code>
        )}
        <div className="shell-editor-footer">
          <span
            id={`${kind}-shell-result`}
            className={`shell-probe is-${probe.state}`}
            role="status"
            aria-live="polite"
          >
            <Icon
              name={
                probe.state === "available"
                  ? "check"
                  : probe.state === "error"
                    ? "warning"
                    : "refresh"
              }
              size={13}
            />
            {saved ? "Saved" : probe.message}
          </span>
          <div className="shell-editor-actions">
            <Button
              type="button"
              size="sm"
              variant="quiet"
              disabled={probe.state === "checking" || probe.state === "empty"}
              aria-label={`Check ${label.toLowerCase()}`}
              onClick={() => setAttempt((value) => value + 1)}
            >
              Check
            </Button>
            <Button type="submit" size="sm" disabled={!dirty || probe.state !== "available"}>
              Save
            </Button>
          </div>
        </div>
      </fieldset>
    </form>
  );
}

const installStages = [
  { state: "downloading", label: "Downloading PowerShell 7…", progress: 35 },
  { state: "verifying", label: "Verifying download…", progress: 70 },
  { state: "extracting", label: "Installing PowerShell 7…", progress: 85 },
  { state: "checking", label: "Checking PowerShell 7…", progress: 95 },
];

export function TerminalPanel({ platform, architecture, scenario }) {
  const managed = managedPowerShell(architecture);
  const existing = platform === "windows" && scenario.startsWith("existing");
  const discovering = platform === "windows" && scenario === "discovering";
  const initialShell =
    scenario === "existing-selected"
      ? { mode: "custom", path: existingPowerShell.path }
      : { mode: "system", path: "" };
  const [installed, setInstalled] = useState(scenario === "installed");
  const [applied, setApplied] = useState({
    agent: initialShell,
    terminal: initialShell,
  });
  const [installState, setInstallState] = useState(
    installFailures[scenario] ? scenario : scenario === "installed" ? "installed" : "idle",
  );
  const [useForBoth, setUseForBoth] = useState(true);
  const [installTargets, setInstallTargets] = useState(false);
  const [notice, setNotice] = useState("");
  const stageIndex = installStages.findIndex((stage) => stage.state === installState);
  const busy = stageIndex !== -1;
  const stage = installStages[stageIndex];

  useEffect(() => {
    if (!busy) return;
    const timer = setTimeout(() => {
      if (stageIndex < installStages.length - 1)
        setInstallState(installStages[stageIndex + 1].state);
      else {
        setInstalled(true);
        setInstallState("installed");
        if (installTargets) {
          setApplied({
            agent: { mode: "managed", path: managed.path },
            terminal: { mode: "managed", path: managed.path },
          });
          setNotice(
            "PowerShell 7 is ready for both shells. Applies to the next turn and new or restarted terminal tabs.",
          );
        } else
          setNotice("PowerShell 7 is installed. Choose it independently for either shell above.");
      }
    }, 1400);
    return () => clearTimeout(timer);
  }, [busy, stageIndex, installTargets, managed.path]);

  const startInstall = () => {
    setNotice("");
    setInstallTargets(useForBoth);
    setInstallState("downloading");
  };
  const useInstalled = () => {
    setApplied({
      agent: { mode: "managed", path: managed.path },
      terminal: { mode: "managed", path: managed.path },
    });
    setNotice(
      "PowerShell 7 selected for both shells. Applies to the next turn and new or restarted terminal tabs.",
    );
  };
  const bothManaged = applied.agent.mode === "managed" && applied.terminal.mode === "managed";
  const bothExisting = [applied.agent, applied.terminal].every(
    (shell) => shell.mode === "custom" && shell.path === existingPowerShell.path,
  );
  const useExisting = () => {
    const shell = { mode: "custom", path: existingPowerShell.path };
    setApplied({ agent: shell, terminal: shell });
    setNotice(
      "PowerShell 7 selected for both shells. Applies to the next turn and new or restarted terminal tabs.",
    );
  };

  return (
    <div className="settings-panel-content terminal-settings-content">
      <div className="settings-panel-heading">
        <h2>Terminal</h2>
      </div>
      {["agent", "terminal"].map((kind) => (
        <ShellEditor
          key={kind}
          kind={kind}
          platform={platform}
          installed={installed}
          managed={managed}
          appliedShell={applied[kind]}
          onApply={(shell) => setApplied((current) => ({ ...current, [kind]: shell }))}
          disabled={busy && installTargets}
          scenario={scenario}
        />
      ))}
      {platform === "windows" && (
        <section
          className="shell-install"
          aria-labelledby="powershell-install-title"
          aria-busy={busy}
        >
          <div className="shell-install-heading">
            <h3 id="powershell-install-title">PowerShell 7</h3>
            <span>{architecture === "arm64" ? "Windows ARM64" : "Windows x64"}</span>
          </div>
          <p>For the best experience, use PowerShell 7.</p>
          {discovering && (
            <span className="shell-probe" role="status">
              Checking existing installations…
            </span>
          )}
          {existing && (
            <>
              <span className="shell-probe is-available">
                <Icon name="check" size={13} />
                Existing installation found
              </span>
              <p>{existingPowerShell.version}</p>
              <code className="shell-resolved-path">{existingPowerShell.path}</code>
              <Button
                type="button"
                variant="primary"
                size="sm"
                disabled={bothExisting}
                onClick={useExisting}
              >
                {bothExisting
                  ? "Selected for both shells"
                  : "Use existing installation for both shells"}
              </Button>
            </>
          )}
          {installed ? (
            <>
              <span className="shell-probe is-available">
                <Icon name="check" size={13} />
                {managed.version} · Installed
              </span>
              <code className="shell-resolved-path">{managed.path}</code>
              <Button type="button" size="sm" disabled={bothManaged} onClick={useInstalled}>
                {bothManaged ? "Selected for both shells" : "Use for both shells"}
              </Button>
            </>
          ) : (
            !existing &&
            !discovering && (
              <>
                {!busy && (
                  <p className="shell-install-location">
                    Installs to <code>%LOCALAPPDATA%\SunCode\runtimes\powershell</code>
                  </p>
                )}
                <label className="shell-install-choice">
                  <input
                    type="checkbox"
                    checked={useForBoth}
                    disabled={busy}
                    onChange={(event) => setUseForBoth(event.target.checked)}
                  />
                  Use for Agent and Terminal after installation
                </label>
                {busy && (
                  <div className="shell-install-progress" role="status" aria-live="polite">
                    <span>{stage.label}</span>
                    <progress max="100" value={stage.progress} aria-label={stage.label} />
                  </div>
                )}
                {installFailures[installState] && (
                  <p className="shell-install-error" role="alert">
                    <Icon name="warning" size={14} />
                    {installFailures[installState]}
                  </p>
                )}
                <div className="shell-install-actions">
                  {busy ? (
                    <Button
                      type="button"
                      size="sm"
                      onClick={() => {
                        setInstallState("cancelled");
                        setNotice("Installation cancelled. Shell settings were kept.");
                      }}
                    >
                      Cancel installation
                    </Button>
                  ) : (
                    <Button type="button" variant="primary" size="sm" onClick={startInstall}>
                      {installFailures[installState]
                        ? "Retry download and install"
                        : "Download and install"}
                    </Button>
                  )}
                  {!busy && (
                    <span>Official Microsoft release · No administrator access needed</span>
                  )}
                </div>
              </>
            )
          )}
          {notice && (
            <p className="shell-install-notice" role="status">
              {notice}
            </p>
          )}
        </section>
      )}
    </div>
  );
}
