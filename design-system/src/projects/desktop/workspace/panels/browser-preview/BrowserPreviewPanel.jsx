import { useState } from "react";
import { Icon } from "../../../../../shared/Icon.jsx";

export function BrowserPreviewPanel() {
  const [running, setRunning] = useState(true);
  const [address, setAddress] = useState("http://127.0.0.1:5173/");
  const [reloadCount, setReloadCount] = useState(0);
  const [activeTab, setActiveTab] = useState("project");

  return (
    <section className="workspace-browser workspace-panel" aria-label="Built-in browser">
      {running ? (
        <>
          <div className="workspace-browser-tabs" role="tablist" aria-label="Browser tabs">
            <button
              type="button"
              role="tab"
              aria-selected={activeTab === "project"}
              className={activeTab === "project" ? "is-active" : ""}
              onClick={() => setActiveTab("project")}
            >
              <span className="workspace-browser-tab-dot" />
              Project preview
              <Icon name="close" size={11} />
            </button>
            <button
              type="button"
              className="workspace-browser-new-tab"
              aria-label="New browser tab"
            >
              <Icon name="plus" size={13} />
            </button>
          </div>
          <div className="workspace-browser-controls">
            <button
              type="button"
              className="workspace-browser-nav"
              aria-label="Back"
              title="Back"
              disabled
            >
              <Icon name="arrow" size={14} className="is-back" />
            </button>
            <button
              type="button"
              className="workspace-browser-nav"
              aria-label="Forward"
              title="Forward"
              disabled
            >
              <Icon name="arrow" size={14} />
            </button>
            <button
              type="button"
              className="workspace-browser-nav"
              aria-label="Reload page"
              title="Reload page"
              onClick={() => setReloadCount((count) => count + 1)}
            >
              <Icon name="refresh" size={14} />
            </button>
            <input
              aria-label="Browser address"
              value={address}
              onChange={(event) => setAddress(event.target.value)}
              spellCheck="false"
            />
            <button type="button" className="workspace-browser-go">
              Go
            </button>
            <button
              type="button"
              className="workspace-browser-stop"
              onClick={() => setRunning(false)}
            >
              Disconnect
            </button>
          </div>
          <div className="workspace-preview-viewport" key={reloadCount}>
            <div className="workspace-preview-demo">
              <span>LOCAL PAGE · LIVE FROM CHROMIUM</span>
              <h2>Changes appear here as the project updates.</h2>
              <p>
                The built-in browser stays beside the conversation for Browser Use tasks and
                user-controlled review.
              </p>
              <div className="workspace-preview-demo-controls">
                <button type="button">Sample action</button>
                <span>{address.replace(/^https?:\/\//, "")}</span>
              </div>
            </div>
          </div>
        </>
      ) : (
        <div className="workspace-browser-empty">
          <Icon name="server" size={24} />
          <h2>Browser is offline</h2>
          <p>
            Connect the embedded browser to inspect the current page and let Browser Use work in it.
          </p>
          <button type="button" onClick={() => setRunning(true)}>
            Connect browser
          </button>
        </div>
      )}
    </section>
  );
}
