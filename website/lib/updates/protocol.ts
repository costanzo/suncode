import { createPublicKey, verify } from "node:crypto";

export const targets = ["win-x64", "win-arm64", "osx-x64", "osx-arm64", "linux-x64", "linux-arm64"];
export const maximumCatalogBytes = 1024 * 1024;
export const signaturePrefix = Buffer.from("SunCode Update Catalog v1\n", "utf8");
export type Package = { url: string; format: "zip" | "tar.gz"; sizeBytes: number; sha256: string; minimumOsVersion?: string; minimumGlibcVersion?: string };
export type Release = { version: string; publishedAt: string; package: Package; releaseNotes: Record<string, string> };
export type Catalog = {
  schemaVersion: 1; revision: number; channel: "stable"; target: string; publishedAt: string; expiresAt: string;
  components: { desktop: Release & { requiredSdkAbi: number }; sdk: Release & { abi: number } };
  compatiblePairs: { desktop: string; sdk: string }[];
};
export type PublicKey = { id: string; publicKey: string; revoked?: boolean };
export type Envelope = { signed: Catalog; signatures: { keyId: string; algorithm: "Ed25519"; value: string }[] };

export function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function validString(value: string) {
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(++i);
      requireValue(next >= 0xdc00 && next <= 0xdfff, "Unpaired Unicode surrogate");
    } else requireValue(code < 0xdc00 || code > 0xdfff, "Unpaired Unicode surrogate");
  }
  return JSON.stringify(value);
}

// JCS uses JavaScript number serialization and UTF-16 key ordering. Build the
// object text directly: JSON.stringify(object) reorders integer-looking keys.
export function canonicalJson(value: unknown, depth = 0): string {
  requireValue(depth <= 48, "Catalog JSON is too deep");
  if (value === null) return "null";
  if (typeof value === "string") return validString(value);
  if (typeof value === "boolean") return JSON.stringify(value);
  if (typeof value === "number") {
    requireValue(Number.isFinite(value), "Invalid JSON number");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(item => canonicalJson(item, depth + 1)).join(",")}]`;
  requireValue(typeof value === "object", "Unsupported JSON value");
  return `{${Object.keys(value).sort().map(key => `${validString(key)}:${canonicalJson((value as Record<string, unknown>)[key], depth + 1)}`).join(",")}}`;
}

export function versionParts(value: string) {
  requireValue(typeof value === "string" && /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.test(value), "Invalid stable product version");
  const parts = value.split(".").map(Number);
  requireValue(parts.every(part => part <= 2147483647), "Version part exceeds the supported size");
  return parts;
}

export function compareVersions(left: string, right: string) {
  const a = versionParts(left), b = versionParts(right);
  for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] - b[i];
  return 0;
}

export function httpsUrl(text: string) {
  requireValue(typeof text === "string" && text.length <= 2048, "Invalid update URL");
  const url = new URL(text);
  requireValue(url.protocol === "https:" && !url.username && !url.password && !url.hash, "Updates require HTTPS URLs without credentials or fragments");
  return url;
}

function timestamp(value: string) {
  requireValue(typeof value === "string" && value.length <= 64 && /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(\.\d+)?Z$/.test(value) && Number.isFinite(Date.parse(value)), "Invalid UTC publication timestamp");
  const date = new Date(value);
  // Date.parse normalizes dates such as February 30; the native client rejects them.
  requireValue(date.getUTCFullYear() >= 1 && date.toISOString().slice(0, 19) === value.slice(0, 19), "Invalid UTC publication timestamp");
  return date.getTime();
}

export function validateCatalog(catalog: Catalog, origins: string[], now: number, checkFreshness = true) {
  requireValue(catalog?.schemaVersion === 1 && catalog.channel === "stable" && targets.includes(catalog.target), "Unsupported catalog context");
  requireValue(Number.isSafeInteger(catalog.revision) && catalog.revision >= 1, "Invalid catalog revision");
  const published = timestamp(catalog.publishedAt), expires = timestamp(catalog.expiresAt);
  requireValue(expires > published && expires <= published + 7 * 86400000, "Catalog expiry must be within seven days of publication");
  if (checkFreshness) requireValue(published <= now + 300000 && expires > now, "Catalog is future-dated or expired");
  const desktop = catalog.components?.desktop, sdk = catalog.components?.sdk;
  requireValue(desktop && sdk && Number.isInteger(desktop.requiredSdkAbi) && desktop.requiredSdkAbi > 0
    && desktop.requiredSdkAbi <= 2147483647 && desktop.requiredSdkAbi === sdk.abi, "Desktop and SDK ABI must match");
  for (const release of [desktop, sdk]) {
    versionParts(release.version);
    requireValue(timestamp(release.publishedAt) <= published, "Release publication is later than the catalog");
    const pkg = release.package;
    requireValue(pkg && ["zip", "tar.gz"].includes(pkg.format) && Number.isSafeInteger(pkg.sizeBytes) && pkg.sizeBytes > 0
      && pkg.sizeBytes <= 8 * 1024 ** 3 && /^[a-f0-9]{64}$/.test(pkg.sha256), "Invalid archive identity");
    requireValue(origins.includes(httpsUrl(pkg.url).origin), "Archive download origin is not allowed");
    if (pkg.minimumOsVersion !== undefined) requireValue(typeof pkg.minimumOsVersion === "string" && /^[0-9]+(\.[0-9]+){0,3}$/.test(pkg.minimumOsVersion), "Invalid minimum OS version");
    if (pkg.minimumGlibcVersion !== undefined) requireValue(typeof pkg.minimumGlibcVersion === "string" && catalog.target.startsWith("linux-") && /^[0-9]+\.[0-9]+$/.test(pkg.minimumGlibcVersion), "Invalid glibc prerequisite");
    const notes = release.releaseNotes;
    requireValue(notes && typeof notes === "object" && !Array.isArray(notes) && Object.hasOwn(notes, "en-US")
      && Object.keys(notes).length <= 16, "English release notes are required");
    for (const [locale, note] of Object.entries(notes)) requireValue(/^[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*$/.test(locale)
      && typeof note === "string" && note.length >= 1 && note.length <= 16000, "Invalid localized release notes");
  }
  requireValue(Array.isArray(catalog.compatiblePairs) && catalog.compatiblePairs.length >= 1 && catalog.compatiblePairs.length <= 256, "Exact tested version pairs are required");
  const pairs = new Set<string>();
  for (const pair of catalog.compatiblePairs) {
    requireValue(pair && Object.keys(pair).length === 2 && Object.hasOwn(pair, "desktop") && Object.hasOwn(pair, "sdk"), "Invalid compatible pair");
    versionParts(pair.desktop); versionParts(pair.sdk);
    const key = `${pair.desktop}/${pair.sdk}`;
    requireValue(!pairs.has(key), "Duplicate compatible pair"); pairs.add(key);
  }
  requireValue(pairs.has(`${desktop.version}/${sdk.version}`), "The recommended component pair must be certified");
}

export function signatureMessage(signed: Catalog) {
  return Buffer.concat([signaturePrefix, Buffer.from(canonicalJson(signed), "utf8")]);
}

export function verifyEnvelope(envelope: Envelope, keys: PublicKey[]) {
  requireValue(envelope && Array.isArray(envelope.signatures) && envelope.signatures.length >= 1 && envelope.signatures.length <= 8, "Invalid signature envelope");
  const message = signatureMessage(envelope.signed);
  const results = envelope.signatures.map(signature => {
    requireValue(signature.algorithm === "Ed25519" && /^[A-Za-z0-9._-]{1,64}$/.test(signature.keyId)
      && typeof signature.value === "string" && /^[A-Za-z0-9_-]+$/.test(signature.value), "Invalid signature encoding");
    const bytes = Buffer.from(signature.value, "base64url");
    requireValue(bytes.length === 64 && bytes.toString("base64url") === signature.value, "Invalid signature encoding");
    const key = keys.find(key => key.id === signature.keyId && !key.revoked);
    if (!key) return false;
    const raw = Buffer.from(key.publicKey, "base64url");
    requireValue(raw.length === 32 && raw.toString("base64url") === key.publicKey, "Invalid public key");
    const publicKey = createPublicKey({ key: Buffer.concat([Buffer.from("302a300506032b6570032100", "hex"), raw]), format: "der", type: "spki" });
    return verify(null, message, publicKey, bytes);
  });
  requireValue(results.some(Boolean), "Catalog signature is not trusted");
}
