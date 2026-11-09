import { browserScenarios } from "../data/browser.js";

export function BrowserReviewControls({
  scenario,
  setScenario,
  platform,
  setPlatform,
  windowSize,
  setWindowSize,
}) {
  return (
    <div className="update-review-controls" aria-label="Browser specimen controls">
      <div>
        <strong>Browser use · Design preview</strong>
        <span>
          Simulated installation. Package versions, sizes, paths, and hashes are illustrative.
        </span>
      </div>
      <label>
        Review state
        <select
          className="field"
          value={scenario}
          onChange={(event) => setScenario(event.target.value)}
        >
          {browserScenarios.map(({ value, label }) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </label>
      <label>
        Platform
        <select
          className="field"
          value={platform}
          onChange={(event) => setPlatform(event.target.value)}
        >
          <option value="macos">macOS</option>
          <option value="windows">Windows</option>
          <option value="linux">Linux</option>
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
