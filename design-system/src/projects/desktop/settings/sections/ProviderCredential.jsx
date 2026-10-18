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
  missing: [
    "Saved key not found",
    "The system keychain entry is missing. Enter a replacement key to use this provider.",
  ],
  save_failed: [
    "API key could not be saved",
    "Your saved key and storage method are unchanged. Try saving again.",
  ],
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

// Review-only state: secrets never leave this component and are never persisted.
export function ProviderCredential({ provider, credential, onChange, platform, issue, onRetry }) {
  const [mode, setMode] = useState(credential.mode);
  const [draft, setDraft] = useState("");
  const [confirmation, setConfirmation] = useState(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const configured = Boolean(credential.mask);
  const dirty = mode !== credential.mode || Boolean(draft.trim());
  const busy = issue === "saving";
  const accessProblem = ["locked", "denied", "unavailable"].includes(issue);
  const needsOldKey = configured && !draft.trim() && mode !== credential.mode;
  const blocked =
    (mode === "keychain" && accessProblem) ||
    (needsOldKey && credential.mode === "keychain" && (accessProblem || issue === "missing"));
  const visibleIssue =
    error || (mode === "keychain" || credential.mode === "keychain" ? issue : "none");
  const issueCopy = issues[visibleIssue];
  const ready =
    configured &&
    (credential.mode === "plaintext" ||
      !["locked", "denied", "unavailable", "missing"].includes(issue));
  const destination = mode === "keychain" ? keychainNames[platform] : "the local database";

  const commit = () => {
    setConfirmation(null);
    if (issue === "save_failed") {
      setError("save_failed");
      return;
    }
    onChange({ mode, mask: draft.trim() ? maskKey(draft) : credential.mask });
    setDraft("");
    setError("");
    setMessage(configured ? "API key settings saved." : "API key saved.");
  };
  const save = (event) => {
    event.preventDefault();
    if (!dirty || busy || blocked || (!configured && !draft.trim())) return;
    if (mode === "plaintext") setConfirmation("plaintext");
    else commit();
  };
  const remove = () => {
    setConfirmation(null);
    if (issue === "remove_failed") {
      setError("remove_failed");
      return;
    }
    onChange({ ...credential, mask: "" });
    setDraft("");
    setMode(credential.mode);
    setError("");
    setMessage("API key removed. Add a key to use this provider again.");
  };
  const cancel = () => {
    setMode(credential.mode);
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
            {configured
              ? `Saved in ${credential.mode === "keychain" ? keychainNames[platform] : "the local database · Plaintext"}`
              : "Add a key to use this provider. System keychain is the default."}
          </small>
        </div>
      </div>

      <fieldset className="credential-storage-options" disabled={busy}>
        <legend>Save this provider’s key</legend>
        <label className={mode === "keychain" ? "is-selected" : ""}>
          <input
            type="radio"
            name={`storage-${provider.label}`}
            value="keychain"
            checked={mode === "keychain"}
            onChange={() => {
              setMode("keychain");
              setMessage("");
            }}
          />
          <span>
            <strong>
              System keychain <small>Recommended</small>
            </strong>
            <span>Protected by {keychainNames[platform]}. The key stays outside the database.</span>
          </span>
          <Icon name="lock" size={16} />
        </label>
        <label className={mode === "plaintext" ? "is-selected" : ""}>
          <input
            type="radio"
            name={`storage-${provider.label}`}
            value="plaintext"
            checked={mode === "plaintext"}
            onChange={() => {
              setMode("plaintext");
              setMessage("");
            }}
          />
          <span>
            <strong>Plaintext</strong>
            <span>Saved in SunCode’s local database.</span>
          </span>
          <Icon name="file-config" size={16} />
        </label>
      </fieldset>

      {mode === "plaintext" && (
        <div className="settings-warning" role="note">
          <Icon name="warning" size={16} />
          <div>
            <strong>The complete key will be readable</strong>
            <span>
              Anyone with access to the database or its backups can read this key. You’ll confirm
              before saving.
            </span>
          </div>
        </div>
      )}
      {issueCopy && (
        <div className="credential-feedback is-error" role="alert">
          <Icon name="warning" size={16} />
          <div>
            <strong>{issueCopy[0]}</strong>
            <span>{issueCopy[1]}</span>
            {configured && (
              <span>The saved mask stays visible. Storage never switches automatically.</span>
            )}
          </div>
          {visibleIssue !== "missing" && (
            <Button
              type="button"
              size="sm"
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
      <p className="credential-help">
        {dirty
          ? `Pending change · Save to ${destination}.`
          : "Storage applies only to this provider. Saved keys are shown masked."}
        {mode === "keychain" && " Your operating system may ask you to allow access."}
      </p>
      <div className="settings-actions credential-actions">
        <Button
          type="submit"
          variant="primary"
          size="sm"
          disabled={busy || blocked || !dirty || (!configured && !draft.trim())}
        >
          {busy ? "Saving…" : configured ? "Save changes" : "Save key"}
        </Button>
        {dirty && (
          <Button type="button" size="sm" disabled={busy} onClick={cancel}>
            Cancel
          </Button>
        )}
        <Button
          type="button"
          variant="danger"
          size="sm"
          disabled={!configured || busy || (credential.mode === "keychain" && accessProblem)}
          onClick={() => setConfirmation("remove")}
        >
          Remove key
        </Button>
      </div>
      <p className="credential-save-message" role="status">
        {busy ? "Saving the API key. Please wait." : message}
      </p>
      <ConfirmationDialog
        open={confirmation === "plaintext"}
        title="Save API key as plaintext?"
        description={`The complete ${provider.label} API key will be readable in SunCode’s local database and its backups.`}
        confirmLabel="Save as plaintext"
        onConfirm={commit}
        onCancel={() => setConfirmation(null)}
      >
        <p>This changes only {provider.label}. System keychain storage is recommended.</p>
      </ConfirmationDialog>
      <ConfirmationDialog
        open={confirmation === "remove"}
        title={`Remove ${provider.label} API key?`}
        description="This provider will need a new API key before you can send another message."
        confirmLabel="Remove key"
        onConfirm={remove}
        onCancel={() => setConfirmation(null)}
      />
    </form>
  );
}
