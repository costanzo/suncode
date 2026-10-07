import { useState } from "react";
import { useWebStore } from "../store";

export function Pairing({ onBack, onPaired }: { onBack: () => void; onPaired: () => void }) {
  const pairFromUrl = useWebStore((state) => state.pairFromUrl);
  const pairingError = useWebStore((state) => state.pairingError);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  return (
    <main className="settings-page">
      <header>
        <button className="back-button" type="button" onClick={onBack}>
          ‹ Settings
        </button>
        <span className="eyebrow">SECURE DEVICE LINK</span>
      </header>
      <div className="pairing-content">
        <span className="eyebrow">PAIR A DESKTOP</span>
        <h1>Pair this browser</h1>
        <p>
          Paste the one-time pairing URL from SunCode Desktop. The encryption key remains in this
          browser and is never sent to the relay.
        </p>
        <label>
          Pairing URL
          <textarea
            value={value}
            onChange={(event) => setValue(event.target.value)}
            placeholder="https://relay…?hostId=…&code=…&k=…&e2e=1"
          />
        </label>
        {pairingError && <div className="error-note">{pairingError}</div>}
        <button
          className="primary-button"
          type="button"
          disabled={busy || !value.trim()}
          onClick={async () => {
            setBusy(true);
            // A failure is rendered from `pairingError` above.
            const paired = await pairFromUrl(value);
            setBusy(false);
            if (paired) onPaired();
          }}
        >
          {busy ? "Pairing…" : "Confirm pairing"}
        </button>
      </div>
    </main>
  );
}
