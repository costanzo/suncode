import { useShallow } from "zustand/react/shallow";
import { selectHost, useWebStore } from "../store";
import { Connection } from "./Connection";
import { Icon } from "./Icon";

export function Settings({ onBack, onPair }: { onBack: () => void; onPair: () => void }) {
  const { host, paired, hasKey, encryptionEnabled, unpair, setEncryptionEnabled } = useWebStore(
    useShallow((state) => ({
      host: selectHost(state),
      paired: state.credential !== null,
      hasKey: Boolean(state.credential?.e2eKey),
      encryptionEnabled: Boolean(state.credential?.encryptionEnabled),
      unpair: state.unpair,
      setEncryptionEnabled: state.setEncryptionEnabled,
    })),
  );
  return (
    <main className="settings-page">
      <header>
        <button className="back-button" type="button" onClick={onBack}>
          ‹ Sessions
        </button>
        <Connection state={host?.connectionState ?? "offline"} />
      </header>
      <div className="settings-content">
        <span className="eyebrow">BROWSER SETTINGS</span>
        <h1>Settings</h1>
        <p>Manage the paired Host, browser-side encryption, and connection recovery.</p>
        <div className="settings-list">
          <div>
            <span>
              <Icon name="lock" />
            </span>
            <strong>
              Encrypt outgoing requests
              <small>Send request bodies as AES-256-GCM payloads when enabled</small>
            </strong>
            <label className="settings-toggle">
              <input
                type="checkbox"
                checked={encryptionEnabled && hasKey}
                disabled={!hasKey}
                onChange={(event) => setEncryptionEnabled(event.target.checked)}
              />
              <span>{hasKey ? (encryptionEnabled ? "On" : "Off") : "Unavailable"}</span>
            </label>
          </div>
          <div>
            <span>
              <Icon name="monitor" />
            </span>
            <strong>
              {host?.displayName ?? "No Desktop paired"}
              <small>{host ? "Remote Server host projection" : "Pair a Desktop to start"}</small>
            </strong>
            <b>{host ? host.connectionState : "Not paired"}</b>
          </div>
          <div>
            <span>
              <Icon name="plus" />
            </span>
            <strong>
              Pair another Desktop<small>Use a one-time Desktop pairing URL</small>
            </strong>
            <button className="quiet-button" type="button" onClick={onPair}>
              Pair
            </button>
          </div>
          <div>
            <span>
              <Icon name="panel" />
            </span>
            <strong>
              Session stream recovery<small>Reconnect with Last-Event-ID after cursor expiry</small>
            </strong>
            <code>automatic</code>
          </div>
        </div>
        {paired && (
          <div className="security-note">
            Access credentials and the browser E2E key are kept in this browser session. The key is
            never sent to the relay.
          </div>
        )}
        <button className="danger-button" type="button" onClick={() => void unpair()}>
          Unpair this browser
        </button>
      </div>
    </main>
  );
}
