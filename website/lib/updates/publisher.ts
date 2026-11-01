import { createHash, createPrivateKey, createPublicKey, sign } from "node:crypto";
import { createReadStream, constants } from "node:fs";
import { copyFile, mkdir, open, readFile, realpath, rename, rm, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { canonicalJson, compareVersions, httpsUrl, maximumCatalogBytes, requireValue, signatureMessage, targets, validateCatalog, verifyEnvelope, versionParts } from "./protocol.ts";
import type { Catalog, Envelope, Package, PublicKey, Release } from "./protocol.ts";

export const websiteRoot = fileURLToPath(new URL("../../", import.meta.url));
type InputRelease = Omit<Release, "package"> & {
  archive: string; minimumOsVersion?: string; minimumGlibcVersion?: string;
  requiredSdkAbi?: number; abi?: number;
};
type ReleaseInput = {
  target: string; revision: number; publishedAt?: string; expiresAt?: string;
  components: { desktop: InputRelease; sdk: InputRelease };
  compatiblePairs: Catalog["compatiblePairs"];
};
type Distribution = { catalogOrigin: string; downloadOrigins: string[]; publicKeys: PublicKey[] };
export type PublishOptions = { releaseFile: string; privateKeyFile: string; keyId: string; distributionFile: string; root?: string; now?: number };

async function hashFile(file: string) {
  const digest = createHash("sha256");
  for await (const chunk of createReadStream(file)) digest.update(chunk);
  return digest.digest("hex");
}

async function jsonFile<T>(file: string): Promise<T> {
  requireValue((await stat(file)).size <= maximumCatalogBytes, "Publication JSON exceeds 1 MiB");
  return JSON.parse(await readFile(file, "utf8"));
}

function within(directory: string, file: string) {
  const relative = path.relative(directory, file);
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
}

// Offline only. The website runtime never imports this module or receives a private key.
export async function publishRelease(options: PublishOptions) {
  await mkdir(path.resolve(options.root ?? websiteRoot), { recursive: true });
  const root = await realpath(path.resolve(options.root ?? websiteRoot));
  const privatePath = await realpath(options.privateKeyFile);
  const repositoryRoot = await realpath(path.join(websiteRoot, ".."));
  requireValue(!within(repositoryRoot, privatePath) && !within(root, privatePath), "Keep the signing private key outside the repository and website");
  requireValue(/^[A-Za-z0-9._-]{1,64}$/.test(options.keyId), "Invalid signing key ID");
  const distribution = await jsonFile<Distribution>(options.distributionFile);
  const origin = httpsUrl(distribution.catalogOrigin).origin;
  requireValue(new URL(distribution.catalogOrigin).pathname === "/" && !new URL(distribution.catalogOrigin).search, "Catalog origin must contain only an HTTPS authority");
  const origins = distribution.downloadOrigins.map(value => {
    const url = httpsUrl(value);
    requireValue(url.pathname === "/" && !url.search, "Download origins must contain only an HTTPS authority");
    return url.origin;
  });
  requireValue(origins.includes(origin), "Allow the website origin for hosted archives");
  const key = distribution.publicKeys.find(key => key.id === options.keyId && !key.revoked);
  requireValue(key, "Signing key ID is not active in the Desktop distribution");
  const privateKey = createPrivateKey(await readFile(privatePath));
  requireValue(privateKey.asymmetricKeyType === "ed25519", "An Ed25519 private key is required");
  const jwk = createPublicKey(privateKey).export({ format: "jwk" });
  requireValue(jwk.x === key.publicKey, "Private key does not match the pinned Desktop public key");

  const input = await jsonFile<ReleaseInput>(options.releaseFile);
  requireValue(targets.includes(input.target), "Choose a supported target");
  const now = options.now ?? Date.now();
  const publishedAt = input.publishedAt ?? new Date(now).toISOString();
  const expiresAt = input.expiresAt ?? new Date(Date.parse(publishedAt) + 7 * 86400000).toISOString();
  const archives: { source: string; destination: string; identity: Package }[] = [];
  async function component(id: "desktop" | "sdk") {
    const release = input.components?.[id];
    requireValue(release && typeof release.archive === "string", `A real ${id} archive is required`);
    versionParts(release.version);
    const source = path.resolve(path.dirname(options.releaseFile), release.archive);
    const info = await stat(source);
    requireValue(info.isFile() && info.size > 0 && info.size <= 8 * 1024 ** 3, "Archive is missing, empty, or exceeds 8 GiB");
    const format = source.endsWith(".tar.gz") ? "tar.gz" : source.endsWith(".zip") ? "zip" : null;
    requireValue(format, "Component archives must be ZIP or tar.gz");
    const relative = `updates/releases/${input.target}/${id}/${release.version}.${format}`;
    const identity: Package = { url: `${origin}/${relative}`, format, sizeBytes: info.size, sha256: await hashFile(source) };
    if (release.minimumOsVersion !== undefined) identity.minimumOsVersion = release.minimumOsVersion;
    if (release.minimumGlibcVersion !== undefined) identity.minimumGlibcVersion = release.minimumGlibcVersion;
    archives.push({ source, destination: path.join(root, "public", relative), identity });
    return { version: release.version, publishedAt: release.publishedAt, package: identity, releaseNotes: release.releaseNotes };
  }
  const desktop = await component("desktop"), sdk = await component("sdk");
  const signed: Catalog = {
    schemaVersion: 1, revision: input.revision, channel: "stable", target: input.target, publishedAt, expiresAt,
    components: { desktop: { ...desktop, requiredSdkAbi: input.components.desktop.requiredSdkAbi! }, sdk: { ...sdk, abi: input.components.sdk.abi! } },
    compatiblePairs: input.compatiblePairs,
  };
  validateCatalog(signed, origins, now);
  const envelope: Envelope = { signed, signatures: [{ keyId: options.keyId, algorithm: "Ed25519", value: sign(null, signatureMessage(signed), privateKey).toString("base64url") }] };
  verifyEnvelope(envelope, distribution.publicKeys);
  const bytes = JSON.stringify(envelope, null, 2) + "\n";
  requireValue(Buffer.byteLength(bytes) <= maximumCatalogBytes, "Catalog exceeds 1 MiB");

  const catalogPath = path.join(root, "updates", "v1", "stable", input.target, "latest.json");
  await mkdir(path.dirname(catalogPath), { recursive: true });
  const lock = await open(catalogPath + ".publish.lock", "wx", 0o600);
  try {
    let previous: Envelope | undefined;
    try { previous = await jsonFile<Envelope>(catalogPath); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error; }
    if (previous?.signed) {
      verifyEnvelope(previous, distribution.publicKeys);
      validateCatalog(previous.signed, origins, now, false);
      requireValue(previous.signed.target === signed.target && previous.signed.channel === signed.channel, "Previous catalog context differs");
      requireValue(signed.revision > previous.signed.revision, "Catalog revision must increase, including expiry renewal");
      for (const id of ["desktop", "sdk"] as const) {
        const old = previous.signed.components[id], next = signed.components[id];
        requireValue(compareVersions(next.version, old.version) >= 0, "Component downgrades are not supported");
        if (next.version === old.version) requireValue(canonicalJson(old) === canonicalJson(next), "An existing component release is immutable; reuse its descriptor or increase its version");
      }
    }
    // Upload immutable bytes before making the catalog visible. Existing files
    // must match; publication never silently replaces a version's archive.
    for (const archive of archives) {
      await mkdir(path.dirname(archive.destination), { recursive: true });
      let copied = false;
      try {
        try { await copyFile(archive.source, archive.destination, constants.COPYFILE_EXCL); copied = true; }
        catch (error) { if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error; }
        requireValue((await stat(archive.destination)).size === archive.identity.sizeBytes && await hashFile(archive.destination) === archive.identity.sha256, "Archive changed during publication or conflicts with an immutable release");
      } catch (error) { if (copied) await rm(archive.destination); throw error; }
    }
    const temporary = catalogPath + ".tmp";
    try {
      const output = await open(temporary, "wx", 0o644);
      try { await output.writeFile(bytes); await output.sync(); } finally { await output.close(); }
      await rename(temporary, catalogPath);
    } finally { await rm(temporary, { force: true }); }
    return { catalogPath, catalogUrl: `${origin}/updates/v1/stable/${input.target}/latest.json`, revision: signed.revision, desktop: desktop.version, sdk: sdk.version };
  } finally { await lock.close(); await rm(catalogPath + ".publish.lock", { force: true }); }
}
