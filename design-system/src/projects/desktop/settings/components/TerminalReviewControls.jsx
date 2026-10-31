import { shellReviewStates } from "../data/shells.js";

export function TerminalReviewControls({
  platform,
  setPlatform,
  architecture,
  setArchitecture,
  scenario,
  setScenario,
  windowSize,
  setWindowSize,
}) {
  return (
    <div className="shell-review-controls" aria-label="Terminal specimen controls">
      <div>
        <strong>Terminal · Design preview</strong>
        <span>Simulated detection and installation. No system changes.</span>
      </div>
      <label>
        Platform
        <select
          className="field"
          value={platform}
          onChange={(event) => setPlatform(event.target.value)}
        >
          <option value="windows">Windows</option>
          <option value="macos">macOS</option>
          <option value="linux">Linux</option>
        </select>
      </label>
      {platform === "windows" && (
        <label>
          Architecture
          <select
            className="field"
            value={architecture}
            onChange={(event) => setArchitecture(event.target.value)}
          >
            <option value="x64">x64</option>
            <option value="arm64">ARM64</option>
          </select>
        </label>
      )}
      <label>
        Review state
        <select
          className="field"
          value={scenario}
          onChange={(event) => setScenario(event.target.value)}
        >
          {shellReviewStates
            .filter(
              (state) =>
                platform === "windows" ||
                ["default", "missing", "timeout", "unsupported"].includes(state.value),
            )
            .map((state) => (
              <option key={state.value} value={state.value}>
                {state.label}
              </option>
            ))}
        </select>
      </label>
      <label>
        Window size
        <select
          className="field"
          value={windowSize}
          onChange={(event) => setWindowSize(event.target.value)}
        >
          <option value="default">900 × 672</option>
          <option value="minimum">720 × 552</option>
        </select>
      </label>
    </div>
  );
}
