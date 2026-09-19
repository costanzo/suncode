import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const directory = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(directory, "..");
const lock = JSON.parse(await fs.readFile(path.join(root, "runtime-lock.json"), "utf8"));
const workerPackage = JSON.parse(await fs.readFile(path.join(root, "worker", "package.json"), "utf8"));
const workerPackageLock = JSON.parse(await fs.readFile(path.join(root, "worker", "package-lock.json"), "utf8"));
const lockedPlaywright = workerPackageLock.packages?.["node_modules/playwright"];
if (workerPackage.version !== lock.worker.version
    || workerPackage.engines?.node !== lock.node.version
    || workerPackage.dependencies?.playwright !== lock.playwright.version
    || lockedPlaywright?.version !== lock.playwright.version
    || lockedPlaywright?.integrity !== lock.playwright.packageIntegrity) {
  throw new Error("Browser worker package metadata does not match runtime-lock.json");
}
const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}
const target = args.get("--target") || hostTarget();
const output = path.resolve(args.get("--output") || path.join(root, "..", "artifacts", "browser-runtime", target));
const reuseBrowsers = args.get("--reuse-browsers") === "true";
if (target !== hostTarget()) {
  throw new Error(`Browser runtime packaging must run on its target host: requested ${target}, current ${hostTarget()}`);
}
const nodeTarget = lock.node.targets[target];
if (!nodeTarget) throw new Error(`Unsupported browser runtime target: ${target}`);

const temporary = await fs.mkdtemp(path.join(os.tmpdir(), "suncode-browser-package-"));
const stagedOutput = path.join(path.dirname(output), `.${path.basename(output)}.staging-${process.pid}`);
const previousOutput = path.join(path.dirname(output), `.${path.basename(output)}.previous-${process.pid}`);
let completed = false;
try {
  const cachedBrowsers = path.join(temporary, "cached-browsers");
  if (reuseBrowsers) {
    await copyTree(path.join(output, "browsers"), cachedBrowsers, target).catch(() => {});
  }
  await fs.rm(stagedOutput, { recursive: true, force: true });
  await fs.rm(previousOutput, { recursive: true, force: true });
  await fs.mkdir(stagedOutput, { recursive: true });
  const archivePath = path.join(temporary, nodeTarget.archive);
  const archiveUrl = `https://nodejs.org/dist/v${lock.node.version}/${nodeTarget.archive}`;
  await download(archiveUrl, archivePath);
  await verifyFile(archivePath, nodeTarget.sha256);
  const nodeOutput = path.join(stagedOutput, "node");
  await fs.mkdir(nodeOutput, { recursive: true });
  if (target === "win32-x64") {
    await run("powershell", [
      "-NoProfile",
      "-Command",
      `Expand-Archive -LiteralPath '${escapePowerShell(archivePath)}' -DestinationPath '${escapePowerShell(temporary)}' -Force`,
    ]);
    const extracted = path.join(temporary, `node-v${lock.node.version}-win-x64`);
    await fs.cp(extracted, nodeOutput, { recursive: true, verbatimSymlinks: true });
  } else {
    await run("tar", ["-xJf", archivePath, "--strip-components=1", "-C", nodeOutput]);
  }
  await pruneNodeDistribution(nodeOutput, target);

  const workerOutput = path.join(stagedOutput, "worker");
  await fs.mkdir(workerOutput, { recursive: true });
  for (const file of ["index.mjs", "package.json", "package-lock.json"]) {
    await fs.copyFile(path.join(root, "worker", file), path.join(workerOutput, file));
  }
  await run("npm", ["ci", "--omit=dev", "--omit=optional", "--ignore-scripts"], { cwd: workerOutput });

  const licensesOutput = path.join(stagedOutput, "licenses");
  await fs.mkdir(licensesOutput, { recursive: true });
  await fs.copyFile(path.join(nodeOutput, "LICENSE"), path.join(licensesOutput, "Node.js-LICENSE"));
  for (const [source, destination] of [
    ["playwright/LICENSE", "Playwright-LICENSE"],
    ["playwright/NOTICE", "Playwright-NOTICE"],
    ["playwright/ThirdPartyNotices.txt", "Playwright-ThirdPartyNotices.txt"],
    ["playwright-core/LICENSE", "Playwright-Core-LICENSE"],
    ["playwright-core/NOTICE", "Playwright-Core-NOTICE"],
    ["playwright-core/ThirdPartyNotices.txt", "Playwright-Core-ThirdPartyNotices.txt"],
  ]) {
    await fs.copyFile(path.join(workerOutput, "node_modules", source), path.join(licensesOutput, destination));
  }

  const browsersPath = path.join(stagedOutput, "browsers");
  if (reuseBrowsers && await exists(cachedBrowsers)) {
    await copyTree(cachedBrowsers, browsersPath, target);
  } else {
    await fs.mkdir(browsersPath, { recursive: true });
  }
  const nodeExecutable = target === "win32-x64"
    ? path.join(nodeOutput, "node.exe")
    : path.join(nodeOutput, "bin", "node");
  if (!reuseBrowsers || !await exists(path.join(browsersPath, `chromium-${lock.chromium.revision}`))) {
    await run(
      nodeExecutable,
      [path.join(workerOutput, "node_modules", "playwright", "cli.js"), "install", "--no-shell", "chromium"],
      { env: { ...process.env, PLAYWRIGHT_BROWSERS_PATH: browsersPath } },
    );
  }
  await fs.access(chromiumExecutable(browsersPath, target, lock.chromium.revision));
  await fs.access(path.join(browsersPath, `ffmpeg-${lock.ffmpeg.revision}`));

  await fs.copyFile(path.join(root, "runtime-lock.json"), path.join(stagedOutput, "runtime-lock.json"));
  const manifest = {
    schemaVersion: 1,
    target,
    protocolVersion: lock.protocolVersion,
    workerVersion: lock.worker.version,
    nodeVersion: lock.node.version,
    playwrightVersion: lock.playwright.version,
    chromiumVersion: lock.chromium.version,
    chromiumRevision: lock.chromium.revision,
    ffmpegRevision: lock.ffmpeg.revision,
    nodeArchive: nodeTarget.archive,
    nodeArchiveSha256: nodeTarget.sha256,
    nodeTreeSha256: await treeHash(nodeOutput),
    workerTreeSha256: await treeHash(workerOutput),
    browserTreeSha256: await treeHash(browsersPath),
  };
  await fs.writeFile(path.join(stagedOutput, "runtime-manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  const hadPreviousOutput = await exists(output);
  if (hadPreviousOutput) await fs.rename(output, previousOutput);
  try {
    await fs.rename(stagedOutput, output);
  } catch (error) {
    if (hadPreviousOutput && !await exists(output)) {
      await fs.rename(previousOutput, output).catch(() => {});
    }
    throw error;
  }
  if (hadPreviousOutput) await fs.rm(previousOutput, { recursive: true, force: true });
  completed = true;
  process.stdout.write(`${output}\n`);
} finally {
  if (!completed) await fs.rm(stagedOutput, { recursive: true, force: true });
  await fs.rm(temporary, { recursive: true, force: true });
}

function hostTarget() {
  if (process.platform === "darwin" && process.arch === "arm64") return "darwin-arm64";
  if (process.platform === "win32" && process.arch === "x64") return "win32-x64";
  if (process.platform === "linux" && process.arch === "x64") return "linux-x64";
  return `${process.platform}-${process.arch}`;
}

function chromiumExecutable(browsersPath, targetName, revision) {
  const directory = path.join(browsersPath, `chromium-${revision}`);
  if (targetName === "darwin-arm64") return path.join(directory, "chrome-mac", "Chromium.app", "Contents", "MacOS", "Chromium");
  if (targetName === "win32-x64") return path.join(directory, "chrome-win", "chrome.exe");
  if (targetName === "linux-x64") return path.join(directory, "chrome-linux", "chrome");
  throw new Error(`Unsupported browser runtime target: ${targetName}`);
}

async function download(url, destination) {
  let lastError;
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    try {
      const response = await fetch(url);
      if (!response.ok || !response.body) throw new Error(`Download failed: ${url} (${response.status})`);
      const bytes = Buffer.from(await response.arrayBuffer());
      await fs.writeFile(destination, bytes);
      return;
    } catch (error) {
      lastError = error;
      if (attempt < 3) await new Promise((resolve) => setTimeout(resolve, attempt * 1000));
    }
  }
  throw lastError;
}

async function verifyFile(file, expected) {
  const actual = createHash("sha256").update(await fs.readFile(file)).digest("hex");
  if (actual !== expected) throw new Error(`Checksum mismatch for ${path.basename(file)}`);
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

async function copyTree(source, destination, targetName) {
  if (targetName === "darwin-arm64") {
    await run("/usr/bin/ditto", [source, destination]);
    return;
  }
  if (targetName === "linux-x64") {
    await run("/bin/cp", ["-a", source, destination]);
    return;
  }
  await fs.cp(source, destination, { recursive: true, verbatimSymlinks: true });
}

function isWithin(rootPath, candidate) {
  const relative = path.relative(path.resolve(rootPath), candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function run(command, commandArgs, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, commandArgs, { stdio: "inherit", ...options });
    child.on("error", reject);
    child.on("exit", (code) => code === 0 ? resolve() : reject(new Error(`${command} exited with ${code}`)));
  });
}

async function exists(value) {
  try {
    await fs.access(value);
    return true;
  } catch {
    return false;
  }
}

async function pruneNodeDistribution(nodeOutput, target) {
  for (const value of [
    "include",
    "share",
    "lib/node_modules",
    "bin/npm",
    "bin/npx",
    "bin/corepack",
    "npm",
    "npm.cmd",
    "npx",
    "npx.cmd",
    "corepack",
    "corepack.cmd",
  ]) {
    await fs.rm(path.join(nodeOutput, value), { recursive: true, force: true });
  }
  if (target === "win32-x64") {
    for (const entry of await fs.readdir(nodeOutput)) {
      if (entry.endsWith(".pdb")) await fs.rm(path.join(nodeOutput, entry), { force: true });
    }
  }
}

function escapePowerShell(value) {
  return value.replaceAll("'", "''");
}
