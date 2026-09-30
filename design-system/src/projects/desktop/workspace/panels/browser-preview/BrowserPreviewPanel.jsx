import { useState } from "react";
import { Icon } from "../../../../../shared/Icon.jsx";

export function BrowserPreviewPanel() {
  const [running, setRunning] = useState(false);
  const [address, setAddress] = useState("http://127.0.0.1:5173/");
  const [reloadCount, setReloadCount] = useState(0);

  return (
    <section className="workspace-preview workspace-panel" aria-label="Browser preview">
      <header className="workspace-preview-toolbar">
        <strong>Browser preview</strong>
        <span className={`workspace-preview-state ${running ? "is-ready" : ""}`}>
          <i aria-hidden="true" />{running ? "Live" : "Stopped"}
        </span>
        <input
          aria-label="Local preview URL"
          value={address}
          onChange={(event) => setAddress(event.target.value)}
          spellCheck="false"
        />
        <button type="button" aria-label="Reload preview" title="Reload preview" disabled={!running} onClick={() => setReloadCount((count) => count + 1)}>
          <Icon name="refresh" size={15} />
        </button>
        <button type="button" className="workspace-preview-run" onClick={() => setRunning((value) => !value)}>
          {running ? "Stop" : "Start preview"}
        </button>
      </header>
      {running ? (
        <div className="workspace-preview-viewport" key={reloadCount}>
          <div className="workspace-preview-demo">
            <span>DESIGN SPECIMEN · LOCAL PAGE</span>
            <h2>Changes appear here as the project updates.</h2>
            <p>The embedded Chromium viewport stays beside the conversation while the development server sends live updates.</p>
            <div className="workspace-preview-demo-controls">
              <button type="button">Sample action</button>
              <span>localhost</span>
            </div>
          </div>
        </div>
      ) : (
        <div className="workspace-preview-empty">
          <Icon name="server" size={24} />
          <h2>Preview is stopped</h2>
          <p>Start the project’s development server to see its local page here while you chat.</p>
          <button type="button" onClick={() => setRunning(true)}>Start preview</button>
        </div>
      )}
    </section>
  );
}
