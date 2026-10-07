import type { PairingPayload } from "./types";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function parsePairingUrl(value: string): PairingPayload {
  const url = new URL(value);
  const endpoint = `${url.protocol}//${url.host}${url.pathname}`.replace(/\/$/, "");
  const hostId = url.searchParams.get("hostId");
  const code = url.searchParams.get("code");
  const key = url.searchParams.get("k") ?? undefined;
  const e2e = url.searchParams.get("e2e") !== "0";
  if (!/^https?:$/.test(url.protocol) || !hostId || !code || (e2e && !key)) {
    throw new Error(
      "Invalid pairing URL: endpoint, hostId, code, and encryption key are required.",
    );
  }
  return { endpoint, hostId, code, key, e2e };
}

function bytesFromBase64Url(value: string): Uint8Array {
  const normalized = value
    .replace(/-/g, "+")
    .replace(/_/g, "/")
    .padEnd(Math.ceil(value.length / 4) * 4, "=");
  const binary = atob(normalized);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function base64UrlFromBytes(bytes: ArrayBuffer): string {
  // Chunk to stay below the engine's maximum argument count for String.fromCharCode.
  const view = new Uint8Array(bytes);
  let binary = "";
  for (let offset = 0; offset < view.length; offset += 0x8000) {
    binary += String.fromCharCode(...view.subarray(offset, offset + 0x8000));
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

export async function importPairingKey(key: string): Promise<CryptoKey> {
  const bytes = bytesFromBase64Url(key);
  return crypto.subtle.importKey("raw", bytes.buffer as ArrayBuffer, { name: "AES-GCM" }, false, [
    "encrypt",
    "decrypt",
  ]);
}

export async function encryptPayload(value: unknown, key: CryptoKey): Promise<string> {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    encoder.encode(JSON.stringify(value)),
  );
  const joined = new Uint8Array(iv.byteLength + ciphertext.byteLength);
  joined.set(iv, 0);
  joined.set(new Uint8Array(ciphertext), iv.byteLength);
  return `e2e-v1:${base64UrlFromBytes(joined.buffer)}`;
}

export async function decryptPayload(value: string, key: CryptoKey): Promise<unknown> {
  if (!value.startsWith("e2e-v1:")) throw new Error("Unsupported encrypted payload version.");
  const bytes = bytesFromBase64Url(value.slice(7));
  const plaintext = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: bytes.slice(0, 12) },
    key,
    bytes.slice(12),
  );
  return JSON.parse(decoder.decode(plaintext));
}
