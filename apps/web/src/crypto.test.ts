import { describe, expect, it } from "vitest";
import { decryptPayload, encryptPayload, importPairingKey, parsePairingUrl } from "./crypto";

// 32 bytes of 0x01, base64url-encoded without padding.
const KEY = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE";

describe("payload encryption", () => {
  it("round-trips JSON values", async () => {
    const key = await importPairingKey(KEY);
    const value = { text: "héllo ✓", nested: [1, { ok: true }] };
    const sealed = await encryptPayload(value, key);
    expect(sealed.startsWith("e2e-v1:")).toBe(true);
    expect(sealed).not.toMatch(/[+/=]/);
    expect(await decryptPayload(sealed, key)).toEqual(value);
  });

  it("round-trips payloads larger than one encoding chunk", async () => {
    const key = await importPairingKey(KEY);
    const value = "x".repeat(100_000);
    expect(await decryptPayload(await encryptPayload(value, key), key)).toBe(value);
  });

  it("uses a fresh IV for every payload", async () => {
    const key = await importPairingKey(KEY);
    expect(await encryptPayload("same", key)).not.toBe(await encryptPayload("same", key));
  });

  it("rejects unknown versions and tampered ciphertext", async () => {
    const key = await importPairingKey(KEY);
    await expect(decryptPayload("e2e-v2:abc", key)).rejects.toThrow(/Unsupported/);
    const sealed = await encryptPayload({ a: 1 }, key);
    // Flip a character inside the ciphertext body (past the 7-char prefix and 16-char IV).
    const index = 30;
    const swapped = sealed[index] === "A" ? "B" : "A";
    const tampered = `${sealed.slice(0, index)}${swapped}${sealed.slice(index + 1)}`;
    await expect(decryptPayload(tampered, key)).rejects.toThrow();
  });
});

describe("parsePairingUrl", () => {
  it("parses endpoint, host, code, and key", () => {
    expect(
      parsePairingUrl(`https://relay.example/prefix/?hostId=h1&code=c1&k=${KEY}&e2e=1`),
    ).toEqual({
      endpoint: "https://relay.example/prefix",
      hostId: "h1",
      code: "c1",
      key: KEY,
      e2e: true,
    });
  });

  it("allows a missing key only when E2E is disabled", () => {
    expect(parsePairingUrl("https://relay.example?hostId=h1&code=c1&e2e=0").e2e).toBe(false);
    expect(() => parsePairingUrl("https://relay.example?hostId=h1&code=c1")).toThrow(
      /Invalid pairing URL/,
    );
  });
});
