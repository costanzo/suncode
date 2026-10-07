import type { CredentialState } from "./types";

const STORAGE_KEY = "suncode.web.credentials.v1";
/** Pre-refactor location of the endpoint; migrated into the credential on read. */
const LEGACY_ENDPOINT_KEY = `${STORAGE_KEY}.endpoint`;

/** The serializable credential. The imported CryptoKey is rebuilt from `e2eKeyRaw`. */
export type PersistedCredential = Omit<CredentialState, "e2eKey">;

export function saveCredential(credential: CredentialState | null): void {
  if (!credential) {
    clearCredential();
    return;
  }
  const { e2eKey, ...serializable } = credential;
  void e2eKey;
  sessionStorage.setItem(STORAGE_KEY, JSON.stringify(serializable));
  sessionStorage.removeItem(LEGACY_ENDPOINT_KEY);
}

/**
 * Reads the saved credential. A credential written before the endpoint moved inside it is
 * completed from the legacy key and rewritten. Returns null when nothing usable is saved.
 */
export function loadCredential(): PersistedCredential | null {
  let saved: Partial<PersistedCredential> | null;
  try {
    const value = sessionStorage.getItem(STORAGE_KEY);
    saved = value ? (JSON.parse(value) as Partial<PersistedCredential>) : null;
  } catch {
    return null;
  }
  if (!saved || typeof saved !== "object") return null;
  if (!saved.endpoint) {
    const legacyEndpoint = sessionStorage.getItem(LEGACY_ENDPOINT_KEY);
    if (!legacyEndpoint) return { ...saved, endpoint: "" } as PersistedCredential;
    saved = { ...saved, endpoint: legacyEndpoint };
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(saved));
    sessionStorage.removeItem(LEGACY_ENDPOINT_KEY);
  }
  return saved as PersistedCredential;
}

export function clearCredential(): void {
  sessionStorage.removeItem(STORAGE_KEY);
  sessionStorage.removeItem(LEGACY_ENDPOINT_KEY);
}
