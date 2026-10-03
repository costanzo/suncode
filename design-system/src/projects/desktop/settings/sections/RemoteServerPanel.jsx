import { useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";
import { Button } from "../../../../components/universal/button/index.js";
import { SettingRow } from "../components/SettingRow.jsx";

function PairingCodeGrid({ seed }) {
  const size = 29;
  const finderAt = (x, y, startX, startY) => {
    const dx = x - startX;
    const dy = y - startY;
    if (dx < 0 || dy < 0 || dx > 6 || dy > 6) return null;
    return (
      dx === 0 || dy === 0 || dx === 6 || dy === 6 || (dx >= 2 && dx <= 4 && dy >= 2 && dy <= 4)
    );
  };
  const cells = Array.from({ length: size * size }, (_, index) => {
    const x = index % size;
    const y = Math.floor(index / size);
    const finder =
      finderAt(x, y, 0, 0) ?? finderAt(x, y, size - 7, 0) ?? finderAt(x, y, 0, size - 7);
    const value = (x * 17 + y * 31 + seed.charCodeAt((x + y) % seed.length)) % 11 < 5;
    return <span key={index} className={(finder ?? value) ? "is-dark" : ""} />;
  });

  return (
    <div className="remote-pairing-code-grid" role="img" aria-label="Pairing QR code preview">
      {cells}
    </div>
  );
}

export function RemoteServerPanel({ onSave }) {
  const [serverUrl, setServerUrl] = useState("https://relay.suncode.dev");
  const [pairingCode, setPairingCode] = useState("");
  const [e2eEnabled, setE2eEnabled] = useState(true);
  const [connected, setConnected] = useState(false);
  const [showQr, setShowQr] = useState(false);
  const [error, setError] = useState("");
  const [pairingGeneration, setPairingGeneration] = useState(1);

  const connect = () => {
    setError("");
    if (!/^https?:\/\//i.test(serverUrl.trim())) {
      setError("Enter a valid server URL beginning with https:// or http://.");
      return;
    }
    if (pairingCode.trim().length < 6) {
      setError("Enter the pairing code provided by your Remote Server.");
      return;
    }
    setConnected(true);
    setShowQr(false);
    onSave("Connected to Remote Server.");
  };

  const disconnect = () => {
    setConnected(false);
    setShowQr(false);
    setPairingCode("");
    onSave("Remote Server disconnected.");
  };

  const rotatePairingCode = () => setPairingGeneration((generation) => generation + 1);
  const qrSeed = `${serverUrl}:${pairingGeneration}`;

  return (
    <div className="settings-panel-content remote-server-content">
      <div className="settings-panel-heading">
        <h2>Remote Server</h2>
        <p>
          Connect this desktop to a Remote Server, then pair SunCode Mobile to access its projects
          and sessions.
        </p>
      </div>

      <section className="settings-panel-section" aria-label="Remote Server connection">
        <span className="settings-section-label">Server connection</span>
        <SettingRow
          label="Server URL"
          hint="Use the HTTPS address supplied by your Remote Server administrator."
        >
          <input
            className="field mono remote-server-field"
            type="url"
            value={serverUrl}
            onChange={(event) => setServerUrl(event.target.value)}
            aria-label="Remote Server URL"
            placeholder="https://relay.example.com"
            spellCheck="false"
            disabled={connected}
          />
        </SettingRow>
        {!connected && (
          <SettingRow
            label="Pairing code"
            hint="Enter the code issued by the Remote Server to connect this desktop."
          >
            <input
              className="field mono remote-server-field"
              value={pairingCode}
              onChange={(event) => setPairingCode(event.target.value)}
              aria-label="Remote Server pairing code"
              placeholder="Enter pairing code"
              autoComplete="off"
              spellCheck="false"
            />
          </SettingRow>
        )}
        <SettingRow
          label="End-to-end encryption"
          hint="Encrypt every Remote Server request body. Routing fields remain visible so Desktop can dispatch the request."
        >
          <label className="settings-switch">
            <input
              type="checkbox"
              checked={e2eEnabled}
              onChange={(event) => setE2eEnabled(event.target.checked)}
              aria-label="Enable end-to-end encryption"
            />
            <span className="settings-switch-track"><span /></span>
            <b>{e2eEnabled ? "On" : "Off"}</b>
          </label>
        </SettingRow>

        {error && (
          <div className="remote-server-error" role="alert">
            <Icon name="close" size={14} />
            <span>{error}</span>
          </div>
        )}

        {connected ? (
          <div className="remote-server-connection-state" role="status">
            <span className="settings-status-dot is-configured" />
            <span>
              <strong>Connected</strong>
              <small>Desktop is available to paired mobile devices.</small>
            </span>
            <Button variant="quiet" size="sm" onClick={disconnect}>
              Disconnect
            </Button>
          </div>
        ) : (
          <div className="settings-actions remote-server-actions">
            <Button variant="primary" size="sm" icon="link" onClick={connect}>
              Connect
            </Button>
            <span className="settings-save-status">
              Pairing code is stored with this connection.
            </span>
          </div>
        )}
      </section>

      {connected && (
        <>
          <div className="settings-divider" />
          <section className="settings-panel-section" aria-label="Mobile pairing">
            <span className="settings-section-label">Mobile pairing</span>
            {!showQr ? (
              <div className="remote-mobile-pairing-row">
                <div>
                  <strong>Pair SunCode Mobile</strong>
                  <span>Generate a one-time QR code that connects a phone to this desktop.</span>
                </div>
                <Button variant="neutral" size="sm" icon="qr-code" onClick={() => setShowQr(true)}>
                  Show QR code
                </Button>
              </div>
            ) : (
              <div className="remote-pairing-preview">
                <div className="remote-pairing-visual">
                  <PairingCodeGrid seed={qrSeed} />
                </div>
                <div className="remote-pairing-copy">
                  <strong>Scan with SunCode Mobile</strong>
                  <span>One-time pairing code</span>
                  <code>SC-{String(pairingGeneration).padStart(2, "0")}7K-4M2P</code>
                  <small>This code expires in 10 minutes.</small>
                  <div className="remote-pairing-actions">
                    <Button
                      variant="quiet"
                      size="sm"
                      icon="refresh"
                      onClick={rotatePairingCode}
                      aria-label="Generate a new pairing code"
                    />
                    <Button variant="quiet" size="sm" onClick={() => setShowQr(false)}>
                      Close
                    </Button>
                  </div>
                </div>
              </div>
            )}
          </section>
        </>
      )}
    </div>
  );
}
