import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { clearCredential, loadCredential, saveCredential } from "./persistence";
import type { CredentialState } from "./types";

const KEY = "suncode.web.credentials.v1";
const LEGACY_ENDPOINT_KEY = `${KEY}.endpoint`;

class MemoryStorage {
  private values = new Map<string, string>();
  getItem(key: string) {
    return this.values.get(key) ?? null;
  }
  setItem(key: string, value: string) {
    this.values.set(key, value);
  }
  removeItem(key: string) {
    this.values.delete(key);
  }
}

let storage: MemoryStorage;
beforeEach(() => {
  storage = new MemoryStorage();
  vi.stubGlobal("sessionStorage", storage);
});
afterEach(() => vi.unstubAllGlobals());

const credential: CredentialState = {
  endpoint: "https://relay.example",
  host: { id: "h1", displayName: "Desk", connectionState: "connected" },
  accessToken: "a",
  refreshToken: "r",
  accessTokenExpiresAt: "2026-01-01T00:00:00.000Z",
  e2eKeyRaw: "raw",
  encryptionEnabled: true,
};
const { endpoint, ...legacyCredential } = credential;

describe("persistence", () => {
  it("saves the endpoint inside the credential and never the CryptoKey", () => {
    saveCredential({ ...credential, e2eKey: {} as CryptoKey });
    const stored = JSON.parse(storage.getItem(KEY) ?? "{}") as Record<string, unknown>;
    expect(stored.endpoint).toBe(endpoint);
    expect("e2eKey" in stored).toBe(false);
    expect(loadCredential()).toEqual(credential);
  });

  it("migrates the legacy endpoint key on read", () => {
    storage.setItem(KEY, JSON.stringify(legacyCredential));
    storage.setItem(LEGACY_ENDPOINT_KEY, endpoint);
    expect(loadCredential()).toEqual(credential);
    expect(storage.getItem(LEGACY_ENDPOINT_KEY)).toBeNull();
    expect(JSON.parse(storage.getItem(KEY) ?? "{}").endpoint).toBe(endpoint);
  });

  it("returns an empty endpoint when neither location has one", () => {
    storage.setItem(KEY, JSON.stringify(legacyCredential));
    expect(loadCredential()?.endpoint).toBe("");
  });

  it("returns null for missing or corrupt data", () => {
    expect(loadCredential()).toBeNull();
    storage.setItem(KEY, "{not json");
    expect(loadCredential()).toBeNull();
  });

  it("clears both the current and legacy keys", () => {
    storage.setItem(KEY, "{}");
    storage.setItem(LEGACY_ENDPOINT_KEY, "x");
    clearCredential();
    expect(storage.getItem(KEY)).toBeNull();
    expect(storage.getItem(LEGACY_ENDPOINT_KEY)).toBeNull();
  });
});
