import { useState } from "react";
import { Button } from "../../../../components/universal/button/index.js";
import { ConfirmationDialog } from "../../../../components/universal/modal/ConfirmationDialog.jsx";
import { Icon } from "../../../../shared/Icon.jsx";

export const keychainNames = {
  macos: "macOS Keychain",
  windows: "Windows Credential Manager",
  linux: "Linux Secret Service",
};

const issues = {
  locked: ["System keychain is locked", "Unlock your system keychain, then try again."],
  denied: [
    "Keychain access denied",
    "Allow SunCode to access your system keychain, then try again.",
  ],
  unavailable: [
    "System keychain unavailable",
    "Check that your user session has a running, unlocked credential service, then try again.",
  ],
  missing: ["Saved key not found", "Enter a replacement key to use this provider."],
  save_failed: ["API key could not be saved", "Your saved key is unchanged. Try saving again."],
  remove_failed: [
    "API key could not be removed",
    "Your saved key is still configured. Try removing it again.",
  ],
};

function maskKey(value) {
  const characters = Array.from(value.trim());
  if (characters.length <= 12) return "••••••••";
  return `${characters.slice(0, 4).join("")}••••••••${characters.slice(-4).join("")}`;
}

// Review-only state. Secret values stay in this component and are never persisted.
export function ProviderCredential({ provider, credential, onChange, platform, issue, onRetry }) {
  const [draft, setDraft] = useState("");
  const [confirmRemoval, setConfirmRemoval] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const configured = Boolean(credential.mask);
  const dirty = Boolean(draft.trim());
  const busy = issue === "saving";
  const visibleIssue = error || issue;
  const issueCopy = issues[visibleIssue];
  const ready = configured && !["locked", "denied", "unavailable", "missing"].includes(issue);
  const storeName = keychainNames[platform];

  const save = (event) => {
    event.preventDefault();
    if (!dirty || busy) return;
    if (["locked", "denied", "unavailable", "save_failed"].includes(issue)) {
      setError(issue);
      return;
    }
    onChange({ mask: maskKey(draft) });
    setDraft("");
    setError("");
    setMessage("API key saved.");
    if (issue === "missing") onRetry();
  };
  const remove = () => {
    setConfirmRemoval(false);
    if (["locked", "denied", "unavailable", "remove_failed"].includes(issue)) {
      setError(issue);
      return;
    }
    onChange({ mask: "" });
    setDraft("");
    setError("");
    setMessage("API key removed. Add a key to use this provider again.");
    if (issue === "missing") onRetry();
  };
  const cancel = () => {
    setDraft("");
    setMessage("");
    setError("");
  };

  return (
    <form className="settings-panel-section provider-credential" onSubmit={save} aria-busy={busy}>
      <span className="settings-section-label">API key</span>
      <div className={`settings-credential-status ${ready ? "is-configured" : ""}`}>
        <span className="settings-status-dot" />
        <div>
          <strong>
            {configured
              ? ready
                ? "API key saved"
                : "Saved key needs attention"
              : "No API key saved"}
          </strong>
          {configured && <code aria-label="Saved API key, masked">{credential.mask}</code>}
          <small>
            {configured ? `Saved in ${storeName}` : `API keys are saved in ${storeName}.`}
          </small>
        </div>
      </div>
      {issueCopy && (
        <div className="credential-feedback is-error" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>{issueCopy[0]}</strong>
            <span>{issueCopy[1]}</span>
          </div>
          {visibleIssue !== "missing" && (
            <Button
              type="button"
              size="sm"
              disabled={busy}
              onClick={() => {
                setError("");
                onRetry();
              }}
            >
              Try again
            </Button>
          )}
        </div>
      )}
      <label className="credential-key-field">
        <span>{configured ? "Replace API key" : "API key"}</span>
        <input
          className="field settings-key mono"
          type="password"
          value={draft}
          disabled={busy}
          onChange={(event) => {
            setDraft(event.target.value);
            setMessage("");
          }}
          placeholder={configured ? "Leave empty to keep the saved key" : provider.placeholder}
          aria-label={`${provider.label} API key`}
          autoComplete="new-password"
          spellCheck={false}
        />
      </label>
      <p className="credential-help">Your operating system may ask you to allow access.</p>
      <div className="settings-actions credential-actions">
        <Button type="submit" variant="primary" size="sm" disabled={busy || !dirty}>
          {busy ? "Saving…" : configured ? "Save changes" : "Save key"}
        </Button>
        {draft && (
          <Button type="button" size="sm" disabled={busy} onClick={cancel}>
            Cancel
          </Button>
        )}
        <Button
          type="button"
          variant="danger"
          size="sm"
          disabled={!configured || busy}
          onClick={() => setConfirmRemoval(true)}
        >
          Remove key
        </Button>
      </div>
      <p className="credential-save-message" role="status">
        {busy ? "Saving the API key. Please wait." : message}
      </p>
      <ConfirmationDialog
        open={confirmRemoval}
        title={`Remove ${provider.label} API key?`}
        description="This provider will need a new API key before you can send another message."
        confirmLabel="Remove key"
        onConfirm={remove}
        onCancel={() => setConfirmRemoval(false)}
      />
    </form>
  );
}
