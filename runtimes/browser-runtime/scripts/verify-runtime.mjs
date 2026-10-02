import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const runtime = path.resolve(process.argv[2] || ".");
const manifest = JSON.parse(await fs.readFile(path.join(runtime, "runtime-manifest.json"), "utf8"));
const lock = JSON.parse(await fs.readFile(path.join(runtime, "runtime-lock.json"), "utf8"));
if (manifest.protocolVersion !== lock.protocolVersion
    || manifest.schemaVersion !== 1
    || manifest.target !== targetName()
    || manifest.workerVersion !== lock.worker.version
    || manifest.nodeVersion !== lock.node.version
    || manifest.playwrightVersion !== lock.playwright.version
    || manifest.chromiumVersion !== lock.chromium.version
    || manifest.chromiumRevision !== lock.chromium.revision
    || manifest.ffmpegRevision !== lock.ffmpeg.revision
    || manifest.nodeArchive !== lock.node.targets[manifest.target]?.archive
    || manifest.nodeArchiveSha256 !== lock.node.targets[manifest.target]?.sha256) {
  throw new Error("Runtime manifest does not match runtime lock");
}
const nodePath = manifest.target === "win32-x64"
  ? path.join(runtime, "node", "node.exe")
  : path.join(runtime, "node", "bin", "node");
await fs.access(nodePath);
await fs.access(path.join(runtime, "worker", "index.mjs"));
await fs.access(path.join(runtime, "browsers"));
await fs.access(chromiumExecutable(runtime, manifest.target, manifest.chromiumRevision));
await fs.access(path.join(runtime, "browsers", `ffmpeg-${manifest.ffmpegRevision}`));
for (const [directory, expected] of [
  [path.join(runtime, "node"), manifest.nodeTreeSha256],
  [path.join(runtime, "worker"), manifest.workerTreeSha256],
  [path.join(runtime, "browsers"), manifest.browserTreeSha256],
]) {
  if (await treeHash(directory) !== expected) throw new Error(`Runtime tree integrity failed: ${path.basename(directory)}`);
}
const digest = createHash("sha256").update(await fs.readFile(nodePath)).digest("hex");
process.stdout.write(JSON.stringify({ ok: true, target: manifest.target, nodeExecutableSha256: digest }) + "\n");

function targetName() {
  if (process.platform === "darwin" && process.arch === "arm64") return "darwin-arm64";
  if (process.platform === "win32" && process.arch === "x64") return "win32-x64";
  if (process.platform === "linux" && process.arch === "x64") return "linux-x64";
  return `${process.platform}-${process.arch}`;
}

function chromiumExecutable(runtimeRoot, target, revision) {
  const directory = path.join(runtimeRoot, "browsers", `chromium-${revision}`);
  if (target === "darwin-arm64") return path.join(directory, "chrome-mac", "Chromium.app", "Contents", "MacOS", "Chromium");
  if (target === "win32-x64") return path.join(directory, "chrome-win", "chrome.exe");
  if (target === "linux-x64") return path.join(directory, "chrome-linux", "chrome");
  throw new Error(`Unsupported browser runtime target: ${target}`);
}

async function treeHash(rootPath) {
  const hash = createHash("sha256");
  const visit = async (current, relative = "") => {
    const entries = await fs.readdir(current, { withFileTypes: true });
    entries.sort((left, right) => left.name < right.name ? -1 : left.name > right.name ? 1 : 0);
    for (const entry of entries) {
      const nextRelative = path.posix.join(relative, entry.name);
      const next = path.join(current, entry.name);
      if (entry.isDirectory()) await visit(next, nextRelative);
      else if (entry.isFile()) {
        hash.update("file\0");
        hash.update(nextRelative);
        hash.update("\0");
        hash.update(await fs.readFile(next));
      } else if (entry.isSymbolicLink()) {
        const targetValue = await fs.readlink(next);
        const resolvedTarget = path.resolve(path.dirname(next), targetValue);
        if (path.isAbsolute(targetValue) || !isWithin(rootPath, resolvedTarget)) {
          throw new Error(`Runtime symlink escapes its tree: ${nextRelative}`);
        }
        hash.update("link\0");
        hash.update(nextRelative);
        hash.update("\0");
        hash.update(targetValue.replaceAll("\\", "/"));
      }
    }
  };
  await visit(rootPath);
  return hash.digest("hex");
}

function isWithin(rootPath, candidate) {
  const relative = path.relative(path.resolve(rootPath), candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}
