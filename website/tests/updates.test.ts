import assert from "node:assert/strict";
import { createHash, generateKeyPairSync } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { GET, HEAD } from "../app/updates/v1/[channel]/[target]/latest.json/route.ts";
import { catalogResponse } from "../lib/updates/http.ts";
import { canonicalJson, targets, validateCatalog, verifyEnvelope } from "../lib/updates/protocol.ts";
import type { Envelope } from "../lib/updates/protocol.ts";
import { publishRelease, websiteRoot } from "../lib/updates/publisher.ts";

const now = Date.parse("2026-11-01T03:00:00Z");
const target = "osx-arm64";
const url = `https://suncode.ai/updates/v1/stable/${target}/latest.json`;

async function fixture() {
  const directory = await mkdtemp(path.join(os.tmpdir(), "suncode-publisher-"));
  const root = path.join(directory, "website");
  const { privateKey, publicKey } = generateKeyPairSync("ed25519");
  const keys = [{ id: "fixture", publicKey: publicKey.export({ format: "jwk" }).x! }];
  const privateKeyFile = path.join(directory, "private.pem");
  await writeFile(privateKeyFile, privateKey.export({ format: "pem", type: "pkcs8" }), { mode: 0o600 });
  const distributionFile = path.join(directory, "distribution.json");
  await writeFile(distributionFile, JSON.stringify({ catalogOrigin: "https://suncode.ai", downloadOrigins: ["https://suncode.ai"], publicKeys: keys }));
  const archive = path.join(directory, "component.zip");
  // A valid empty ZIP is sufficient for metadata tests. These fixtures never
  // attest to platform layout/signing or become published application packages.
  await writeFile(archive, Buffer.from("504b0506000000000000000000000000000000000000", "hex"));
  const release = {
    target, revision: 1,
    components: {
      desktop: { version: "0.1.0", requiredSdkAbi: 27, publishedAt: "2026-11-01T02:00:00Z", archive, releaseNotes: { "en-US": "Initial release", "zh-CN": "首次发布" } },
      sdk: { version: "0.1.0", abi: 27, publishedAt: "2026-11-01T02:00:00Z", archive, releaseNotes: { "en-US": "Initial SDK" } },
    },
    compatiblePairs: [{ desktop: "0.1.0", sdk: "0.1.0" }],
  };
  const releaseFile = path.join(directory, "release.json");
  const save = () => writeFile(releaseFile, JSON.stringify(release));
  await save();
  const options = { releaseFile, privateKeyFile, keyId: "fixture", distributionFile, root, now };
  return { directory, root, keys, archive, release, save, options, clean: () => rm(directory, { recursive: true, force: true }) };
}

test("JCS preserves complete signed data, UTF-16 key ordering and JavaScript numbers", () => {
  assert.equal(canonicalJson({ "2": 2, "10": 10, z: -0, a: [1e30, 1e-7, 0.000001] }), '{"10":10,"2":2,"a":[1e+30,1e-7,0.000001],"z":0}');
  assert.equal(canonicalJson({ "\ue000": 2, "😀": 1, a: "é/\n\u2028" }), '{"a":"é/\\n\u2028","😀":1,"\ue000":2}');
  assert.throws(() => canonicalJson({ value: "\ud800" }), /surrogate/);
  assert.throws(() => canonicalJson(Infinity), /number/);
});

for (const platform of targets) test(`${platform} route serves its signed catalog with HTTP 200`, async () => {
  const bytes = await readFile(path.join(websiteRoot, "updates/v1/stable", platform, "latest.json"), "utf8");
  const catalog = JSON.parse(bytes);
  const distribution = JSON.parse(await readFile(path.join(websiteRoot, "../apps/desktop-avalonia/Updates/update-distribution.json"), "utf8"));
  verifyEnvelope(catalog, distribution.publicKeys);
  validateCatalog(catalog.signed, distribution.downloadOrigins, Date.now(), false);
  assert.deepEqual(Object.keys(catalog).sort(), ["signatures", "signed"]);
  const context = { params: Promise.resolve({ channel: "stable", target: platform }) };
  const response = await GET(new Request(`https://suncode.ai/updates/v1/stable/${platform}/latest.json`), context);
  assert.equal(response.status, 200);
  assert.equal(catalog.signed.target, platform); assert.equal(catalog.signed.channel, "stable");
  assert.equal(response.headers.get("content-type"), "application/json");
  assert.equal(await response.text(), bytes);
  const conditional = await GET(new Request(url, { headers: { "If-None-Match": response.headers.get("etag")! } }), context);
  assert.equal(conditional.status, 304); assert.equal(await conditional.text(), "");
  const head = await HEAD(new Request(url, { method: "HEAD" }), context);
  assert.equal(head.status, 200); assert.equal(await head.text(), "");
});

test("invalid channels and traversal targets cannot select filesystem paths", async () => {
  for (const [channel, target] of [["beta", "osx-arm64"], ["stable", "../osx-arm64"], ["..", "linux-x64"]]) {
    const response = await catalogResponse(new Request(url), channel, target);
    assert.equal(response.status, 400); assert.equal((await response.json()).code, "invalid_request");
  }
});

test("publication signs real archive identities and HTTP supports ETag, 304 and HEAD", async () => {
  const f = await fixture();
  try {
    const result = await publishRelease(f.options);
    assert.equal(result.catalogUrl, url);
    const bytes = await readFile(result.catalogPath);
    const envelope: Envelope = JSON.parse(bytes.toString());
    verifyEnvelope(envelope, f.keys);
    const invalidDate = structuredClone(envelope.signed);
    invalidDate.publishedAt = "2026-02-30T02:00:00Z";
    assert.throws(() => validateCatalog(invalidDate, ["https://suncode.ai"], now), /UTC publication timestamp/);
    const invalidPrerequisite = { ...envelope.signed, components: {
      ...envelope.signed.components, sdk: { ...envelope.signed.components.sdk, package: { ...envelope.signed.components.sdk.package, minimumOsVersion: 13 } },
    } };
    assert.throws(() => validateCatalog(invalidPrerequisite as unknown as Envelope["signed"], ["https://suncode.ai"], now), /minimum OS/);
    const malformedExtra = structuredClone(envelope);
    malformedExtra.signatures.push({ keyId: "extra", algorithm: "Ed25519", value: "bad" });
    assert.throws(() => verifyEnvelope(malformedExtra, f.keys), /encoding/);
    assert.equal(envelope.signed.components.desktop.version, "0.1.0");
    const pkg = envelope.signed.components.sdk.package;
    assert.equal(pkg.sizeBytes, (await stat(f.archive)).size);
    assert.equal(pkg.sha256, createHash("sha256").update(await readFile(f.archive)).digest("hex"));
    assert.equal(pkg.url, "https://suncode.ai/updates/releases/osx-arm64/sdk/0.1.0.zip");
    assert.deepEqual(await readFile(path.join(f.root, "public", new URL(pkg.url).pathname)), await readFile(f.archive));
    const get = await catalogResponse(new Request(url), "stable", target, path.join(f.root, "updates"));
    assert.equal(get.status, 200); assert.equal(await get.text(), bytes.toString());
    assert.equal(get.headers.get("cache-control"), "public, max-age=300, must-revalidate");
    assert.equal(get.headers.get("content-type"), "application/json");
    for (const value of [get.headers.get("etag")!, `"other", W/${get.headers.get("etag")}`, "*"]) {
      const cached = await catalogResponse(new Request(url, { headers: { "If-None-Match": value } }), "stable", target, path.join(f.root, "updates"));
      assert.equal(cached.status, 304); assert.equal(await cached.text(), "");
      assert.equal(cached.headers.get("etag"), get.headers.get("etag"));
    }
    const head = await catalogResponse(new Request(url, { method: "HEAD" }), "stable", target, path.join(f.root, "updates"));
    assert.equal(head.status, 200); assert.equal(await head.text(), "");
    envelope.signed.components.sdk.releaseNotes["en-US"] = "tampered";
    assert.throws(() => verifyEnvelope(envelope, f.keys), /not trusted/);
  } finally { await f.clean(); }
});

test("renewal requires a larger revision and preserves immutable component descriptors", async () => {
  const f = await fixture();
  try {
    const first = await publishRelease(f.options);
    const before = await readFile(first.catalogPath, "utf8");
    await assert.rejects(publishRelease(f.options), /revision must increase/);
    assert.equal(await readFile(first.catalogPath, "utf8"), before);
    f.release.revision = 2; await f.save();
    const second = await publishRelease({ ...f.options, now: now + 86400000 });
    const next = JSON.parse(await readFile(second.catalogPath, "utf8"));
    assert.equal(next.signed.revision, 2);
    assert.deepEqual(next.signed.components, JSON.parse(before).signed.components);
    assert.notEqual(next.signed.expiresAt, JSON.parse(before).signed.expiresAt);
    await writeFile(f.archive, "changed archive");
    f.release.revision = 3; await f.save();
    await assert.rejects(publishRelease({ ...f.options, now: now + 86400000 }), /immutable/);
    assert.equal(JSON.parse(await readFile(second.catalogPath, "utf8")).signed.revision, 2);
  } finally { await f.clean(); }
});

test("missing archives and untrusted signing keys cannot publish a catalog", async () => {
  const f = await fixture();
  try {
    await assert.rejects(publishRelease({ ...f.options, keyId: "unknown" }), /not active/);
    const other = generateKeyPairSync("ed25519");
    const otherFile = path.join(f.directory, "other.pem");
    await writeFile(otherFile, other.privateKey.export({ format: "pem", type: "pkcs8" }));
    await assert.rejects(publishRelease({ ...f.options, privateKeyFile: otherFile }), /does not match/);
    await rm(f.archive);
    await assert.rejects(publishRelease(f.options), /ENOENT/);
    await assert.rejects(stat(path.join(f.root, "updates/v1/stable", target, "latest.json")), /ENOENT/);
  } finally { await f.clean(); }
});

test("an SDK-only release preserves the existing Desktop descriptor and archive", async () => {
  const f = await fixture();
  try {
    const first = await publishRelease(f.options);
    const previous: Envelope = JSON.parse(await readFile(first.catalogPath, "utf8"));
    const sdkArchive = path.join(f.directory, "sdk.zip");
    const zip = await readFile(f.archive); zip.writeUInt16LE(3, 20);
    await writeFile(sdkArchive, Buffer.concat([zip, Buffer.from("sdk")]));
    f.release.revision = 2;
    f.release.components.sdk.version = "0.1.1"; f.release.components.sdk.archive = sdkArchive;
    f.release.compatiblePairs.push({ desktop: "0.1.0", sdk: "0.1.1" }); await f.save();
    const second = await publishRelease(f.options);
    const next: Envelope = JSON.parse(await readFile(second.catalogPath, "utf8"));
    verifyEnvelope(next, f.keys);
    assert.deepEqual(next.signed.components.desktop, previous.signed.components.desktop);
    assert.equal(next.signed.components.sdk.version, "0.1.1");
    assert.notEqual(next.signed.components.sdk.package.sha256, previous.signed.components.sdk.package.sha256);
  } finally { await f.clean(); }
});

test("ABI mismatch, missing tested pair and malformed versions fail before publication", async () => {
  const f = await fixture();
  try {
    f.release.components.sdk.abi = 28; await f.save();
    await assert.rejects(publishRelease(f.options), /ABI/);
    f.release.components.sdk.abi = 27; f.release.compatiblePairs = []; await f.save();
    await assert.rejects(publishRelease(f.options), /pairs/);
    f.release.compatiblePairs = [{ desktop: "0.1.0", sdk: "0.1.0" }];
    f.release.components.sdk.version = "00.1.0"; await f.save();
    await assert.rejects(publishRelease(f.options), /version/);
  } finally { await f.clean(); }
});

test("publication refuses expired metadata and packages already stored with different bytes", async () => {
  const f = await fixture();
  try {
    const descriptor = { ...f.release, publishedAt: "2026-10-01T00:00:00Z", expiresAt: "2026-10-08T00:00:00Z" };
    await writeFile(f.options.releaseFile, JSON.stringify(descriptor));
    await assert.rejects(publishRelease(f.options), /later than the catalog|expired/);
    await f.save();
    const archive = path.join(f.root, "public/updates/releases", target, "desktop/0.1.0.zip");
    await mkdir(path.dirname(archive), { recursive: true }); await writeFile(archive, "different bytes");
    await assert.rejects(publishRelease(f.options), /immutable release/);
    assert.equal(await readFile(archive, "utf8"), "different bytes");
    await assert.rejects(stat(path.join(f.root, "updates/v1/stable", target, "latest.json")), /ENOENT/);
  } finally { await f.clean(); }
});

test("downgrades cannot replace the current publication", async () => {
  const f = await fixture();
  try {
    const result = await publishRelease(f.options);
    const previous = await readFile(result.catalogPath, "utf8");
    f.release.revision = 2;
    f.release.components.desktop.version = "0.0.9";
    f.release.components.sdk.version = "0.0.9";
    f.release.compatiblePairs = [{ desktop: "0.0.9", sdk: "0.0.9" }]; await f.save();
    await assert.rejects(publishRelease(f.options), /downgrades/);
    assert.equal(await readFile(result.catalogPath, "utf8"), previous);
  } finally { await f.clean(); }
});

test("private keys inside the website are refused and concurrent publication leaves one valid catalog", async () => {
  const f = await fixture();
  try {
    const inside = path.join(f.root, "private.pem"); await mkdir(f.root, { recursive: true });
    await writeFile(inside, await readFile(f.options.privateKeyFile));
    await assert.rejects(publishRelease({ ...f.options, privateKeyFile: inside }), /outside/);
    await rm(inside);
    const results = await Promise.allSettled([publishRelease(f.options), publishRelease(f.options)]);
    assert.equal(results.filter(result => result.status === "fulfilled").length, 1);
    const catalog = path.join(f.root, "updates/v1/stable", target, "latest.json");
    verifyEnvelope(JSON.parse(await readFile(catalog, "utf8")), f.keys);
    await assert.rejects(stat(catalog + ".publish.lock"), /ENOENT/);
  } finally { await f.clean(); }
});

test("corrupt or absent deployed metadata is an error, never an up-to-date catalog", async () => {
  const f = await fixture();
  try {
    const directory = path.join(f.root, "updates");
    assert.equal((await catalogResponse(new Request(url), "stable", target, directory)).status, 404);
    const catalog = path.join(directory, "v1/stable", target, "latest.json");
    await mkdir(path.dirname(catalog), { recursive: true }); await writeFile(catalog, "{broken");
    const response = await catalogResponse(new Request(url), "stable", target, directory);
    assert.equal(response.status, 503); assert.equal(response.headers.get("cache-control"), "no-store");
  } finally { await f.clean(); }
});
