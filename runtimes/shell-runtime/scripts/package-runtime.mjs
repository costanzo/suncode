import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const lock = JSON.parse(await fs.readFile(path.join(root, "runtime-lock.json"), "utf8"));
if (process.platform !== "win32" || process.arch !== "x64") throw new Error("PowerShell runtime packaging supports Windows x64 only");
const output = path.resolve(process.argv[2] || path.join(root, "..", "..", "artifacts", "runtimes", "powershell", "win-x64"));
const temporary = await fs.mkdtemp(path.join(os.tmpdir(), "suncode-powershell-package-"));
const archive = path.join(temporary, lock.archive);
try {
  const response = await fetch(lock.releaseUrl);
  if (!response.ok) throw new Error(`Download failed (${response.status})`);
  await fs.writeFile(archive, Buffer.from(await response.arrayBuffer()));
  const extracted = path.join(temporary, "extracted");
  await fs.mkdir(extracted);
  await run("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", "Expand-Archive", "-LiteralPath", archive, "-DestinationPath", extracted, "-Force"]);
  await fs.rm(output, { recursive: true, force: true });
  await fs.mkdir(output, { recursive: true });
  await fs.cp(extracted, output, { recursive: true, verbatimSymlinks: true });
  await fs.access(path.join(output, "pwsh.exe"));
  await fs.writeFile(path.join(output, "runtime-manifest.json"), `${JSON.stringify({ schemaVersion: 1, product: lock.product, version: lock.version, target: lock.target, archive: lock.archive, archiveSha256: sha256(await fs.readFile(archive)) }, null, 2)}\n`);
  process.stdout.write(`${output}\n`);
} finally { await fs.rm(temporary, { recursive: true, force: true }); }
function sha256(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
function run(command, args) { return new Promise((resolve, reject) => { const child = spawn(command, args, { stdio: "inherit" }); child.on("error", reject); child.on("exit", code => code === 0 ? resolve() : reject(new Error(`${command} exited with ${code}`))); }); }
