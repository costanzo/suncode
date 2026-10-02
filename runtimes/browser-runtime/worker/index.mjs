import { chromium } from "playwright";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const PROTOCOL_VERSION = 1;
const WORKER_VERSION = "0.1.0";
const MAX_FRAME_BYTES = 8 * 1024 * 1024;
const MAX_SNAPSHOT_ITEMS = 500;
const MAX_SNAPSHOT_TEXT = 128 * 1024;

let inputBuffer = Buffer.alloc(0);
let context;
let controlOwner = "agent";
let nextPageId = 1;
const pageStates = new Map();
const secretValues = new Set();

function targetName() {
  const platform = process.platform === "darwin" ? "darwin" : process.platform;
  const arch = process.arch === "arm64" ? "arm64" : "x64";
  return `${platform}-${arch}`;
}

function hello() {
  return {
    protocolVersion: PROTOCOL_VERSION,
    workerVersion: WORKER_VERSION,
    nodeVersion: process.versions.node,
    playwrightVersion: "1.55.0",
    chromiumVersion: "140.0.7339.16",
    chromiumRevision: "1187",
    target: targetName(),
  };
}

function writeFrame(value) {
  const payload = Buffer.from(JSON.stringify(value));
  if (payload.length > MAX_FRAME_BYTES) throw new Error("response exceeds protocol frame limit");
  const header = Buffer.alloc(4);
  header.writeUInt32BE(payload.length);
  process.stdout.write(header);
  process.stdout.write(payload);
}

function redactedErrorText(error) {
  let message = error instanceof Error ? error.message : String(error);
  for (const secret of secretValues) {
    if (secret) message = message.replaceAll(secret, "[redacted]");
  }
  for (const marker of ["Authorization", "Bearer ", "token=", "key="]) {
    const index = message.toLowerCase().indexOf(marker.toLowerCase());
    if (index >= 0) {
      message = `${message.slice(0, index)}[redacted]`;
      break;
    }
  }
  return message.replace(/[\r\n]+/g, " ").slice(0, 500);
}

function browserFailure(code, message) {
  const error = new Error(message);
  error.browserCode = code;
  return error;
}

function safeFailure(error) {
  if (typeof error?.browserCode === "string" && error.browserCode.startsWith("browser_")) {
    return { code: error.browserCode, message: error.message.slice(0, 200) };
  }
  const redacted = redactedErrorText(error).toLowerCase();
  if (redacted.includes("timeout") || redacted.includes("timed out")) {
    return { code: "browser_action_timeout", message: "Browser operation timed out" };
  }
  if (redacted.includes("target page, context or browser has been closed")) {
    return { code: "browser_runtime_closed", message: "Browser page or runtime closed during the operation" };
  }
  return { code: "browser_worker_error", message: "Browser operation failed" };
}

function pageState(page) {
  let state = pageStates.get(page);
  if (!state) {
    state = { id: `page_${nextPageId++}`, revision: 0, refs: new Map() };
    pageStates.set(page, state);
    page.on("close", () => pageStates.delete(page));
    page.on("framenavigated", (frame) => {
      if (frame === page.mainFrame()) {
        state.revision += 1;
        state.refs.clear();
      }
    });
  }
  return state;
}

function requireContext() {
  if (!context) throw browserFailure("browser_runtime_not_started", "Browser runtime is not started");
  return context;
}

function requireAgentControl() {
  if (controlOwner !== "agent") throw browserFailure("browser_user_controlled", "Browser is controlled by the user");
}

function findPage(pageId) {
  const current = requireContext();
  const pages = current.pages();
  if (!pageId) return pages.at(-1) ?? null;
  return pages.find((page) => pageState(page).id === pageId) ?? null;
}

async function ensureDirectories(...values) {
  for (const value of values) await fs.mkdir(value, { recursive: true });
}

async function start(params) {
  if (context) return runtimeState();
  const profilePath = path.resolve(params.profilePath);
  const downloadsPath = path.resolve(params.downloadsPath);
  if (params.proxy?.username) secretValues.add(params.proxy.username);
  if (params.proxy?.password) secretValues.add(params.proxy.password);
  await ensureDirectories(profilePath, downloadsPath);
  const launchArgs = params.headless === true ? [] : ["--start-minimized"];
  if (params.proxy?.mode === "no_proxy") launchArgs.push("--no-proxy-server");
  const playwrightProxy = params.proxy?.mode === "custom"
    ? {
        server: params.proxy.server,
        username: params.proxy.username || undefined,
        password: params.proxy.password || undefined,
        bypass: params.proxy.bypass || undefined,
      }
    : undefined;
  context = await chromium.launchPersistentContext(profilePath, {
    headless: params.headless === true,
    channel: "chromium",
    acceptDownloads: false,
    downloadsPath,
    noDefaultViewport: true,
    args: launchArgs,
    proxy: playwrightProxy,
  });
  context.on("page", (page) => pageState(page));
  for (const page of context.pages()) pageState(page);
  if (!context.pages().length) pageState(await context.newPage());
  controlOwner = "agent";
  return runtimeState();
}

async function runtimeState() {
  return {
    started: Boolean(context),
    controlOwner,
    pages: context
      ? await Promise.all(context.pages().map(async (page) => ({
          pageId: pageState(page).id,
          title: await page.title().catch(() => ""),
          url: page.url(),
          revision: pageState(page).revision,
        })))
      : [],
  };
}

async function resolveTarget(page, target) {
  if (!target || typeof target !== "object") throw browserFailure("browser_invalid_target", "Browser target is required");
  if (target.kind === "ref") {
    const state = pageState(page);
    if (target.revision !== state.revision) throw browserFailure("browser_reference_stale", "Browser element reference is stale");
    const saved = state.refs.get(target.ref);
    if (!saved) throw browserFailure("browser_reference_stale", "Browser element reference is stale");
    const locator = page.locator(saved.selector).nth(saved.index);
    if ((await locator.count()) !== 1) throw browserFailure("browser_reference_stale", "Browser element reference is stale");
    const fingerprint = await locator.evaluate((element) => ({
      tag: element.tagName.toLowerCase(),
      text: (
        element.getAttribute("aria-label") ||
        element.getAttribute("placeholder") ||
        element.textContent ||
        element.getAttribute("name") ||
        ""
      ).trim().replace(/\s+/g, " ").slice(0, 200),
    }));
    if (fingerprint.tag !== saved.tag || fingerprint.text !== saved.text) {
      throw browserFailure("browser_reference_stale", "Browser element reference is stale");
    }
    return locator;
  }
  if (target.kind === "role") {
    return page.getByRole(target.role, { name: target.name, exact: target.exact === true });
  }
  if (target.kind === "label") return page.getByLabel(target.value, { exact: target.exact === true });
  if (target.kind === "placeholder") {
    return page.getByPlaceholder(target.value, { exact: target.exact === true });
  }
  if (target.kind === "test_id") return page.getByTestId(target.value);
  if (target.kind === "text") return page.getByText(target.value, { exact: target.exact === true });
  throw browserFailure("browser_invalid_target", "Browser target kind is unsupported");
}

async function snapshot(params) {
  const page = findPage(params.pageId);
  if (!page) throw browserFailure("browser_page_unavailable", "Browser page is unavailable");
  const state = pageState(page);
  state.revision += 1;
  state.refs.clear();
  const selector = "a,button,input,textarea,select,[role],[tabindex]";
  const locator = page.locator(selector);
  const count = Math.min(await locator.count(), MAX_SNAPSHOT_ITEMS);
  const lines = [];
  for (let index = 0; index < count; index += 1) {
    const item = locator.nth(index);
    if (!(await item.isVisible().catch(() => false))) continue;
    const details = await item.evaluate((element) => {
      const tag = element.tagName.toLowerCase();
      const role =
        element.getAttribute("role") ||
        ({ a: "link", button: "button", input: "textbox", textarea: "textbox", select: "combobox" }[tag] ?? tag);
      const name = (
        element.getAttribute("aria-label") ||
        element.getAttribute("placeholder") ||
        element.textContent ||
        element.getAttribute("name") ||
        ""
      )
        .trim()
        .replace(/\s+/g, " ")
        .slice(0, 200);
      const value = "value" in element ? String(element.value ?? "").slice(0, 200) : "";
      return { tag, role, name, value, disabled: Boolean(element.disabled) };
    });
    const ref = `e${state.refs.size + 1}`;
    state.refs.set(ref, {
      selector,
      index,
      tag: details.tag,
      text: details.name,
    });
    const value = details.value ? ` value=${JSON.stringify(details.value)}` : "";
    lines.push(`[${ref}] ${details.role} ${JSON.stringify(details.name)}${value}${details.disabled ? " disabled" : ""}`);
    if (lines.join("\n").length >= MAX_SNAPSHOT_TEXT) break;
  }
  return {
    pageId: state.id,
    revision: state.revision,
    title: await page.title(),
    url: page.url(),
    text: lines.join("\n").slice(0, MAX_SNAPSHOT_TEXT),
    truncated: count === MAX_SNAPSHOT_ITEMS || lines.join("\n").length >= MAX_SNAPSHOT_TEXT,
  };
}

async function dispatch(method, params = {}) {
  if (method === "probe") return hello();
  if (method === "start") return start(params);
  if (method === "state") return runtimeState();
  if (method === "take_control") {
    const page = findPage(params.pageId);
    if (page) await page.bringToFront();
    controlOwner = "user";
    return runtimeState();
  }
  if (method === "return_control") {
    controlOwner = "agent";
    for (const state of pageStates.values()) {
      state.revision += 1;
      state.refs.clear();
    }
    return runtimeState();
  }
  if (method === "stop") {
    if (context) await context.close();
    context = undefined;
    pageStates.clear();
    return runtimeState();
  }
  if (method === "shutdown") {
    if (context) await context.close();
    process.exitCode = 0;
    return { stopped: true };
  }
  requireAgentControl();
  if (method === "open") {
    const page = await requireContext().newPage();
    pageState(page);
    if (params.url) await page.goto(params.url, { waitUntil: "domcontentloaded", timeout: params.timeoutMs });
    return snapshot({ pageId: pageState(page).id });
  }
  const page = findPage(params.pageId);
  if (!page) throw browserFailure("browser_page_unavailable", "Browser page is unavailable");
  if (method === "snapshot") return snapshot(params);
  if (method === "navigate") {
    await page.goto(params.url, { waitUntil: "domcontentloaded", timeout: params.timeoutMs });
    return snapshot({ pageId: pageState(page).id });
  }
  if (method === "click") await (await resolveTarget(page, params.target)).click({ timeout: params.timeoutMs });
  else if (method === "fill") await (await resolveTarget(page, params.target)).fill(params.value, { timeout: params.timeoutMs });
  else if (method === "press") await (await resolveTarget(page, params.target)).press(params.key, { timeout: params.timeoutMs });
  else if (method === "screenshot") {
    const bytes = await page.screenshot({ fullPage: params.fullPage === true, type: "png" });
    if (bytes.length > 5 * 1024 * 1024) throw browserFailure("browser_screenshot_too_large", "Browser screenshot exceeds 5 MiB");
    return { pageId: pageState(page).id, mimeType: "image/png", base64: bytes.toString("base64") };
  } else if (method === "close_page") {
    await page.close();
    return runtimeState();
  } else throw browserFailure("browser_protocol_error", "Browser worker method is unsupported");
  const state = pageState(page);
  state.revision += 1;
  state.refs.clear();
  return snapshot({ pageId: state.id });
}

async function handle(request) {
  try {
    const result = await dispatch(request.method, request.params);
    writeFrame({ id: request.id, ok: true, result });
  } catch (error) {
    writeFrame({ id: request.id, ok: false, error: safeFailure(error) });
  }
}

process.stdin.on("data", (chunk) => {
  inputBuffer = Buffer.concat([inputBuffer, chunk]);
  while (inputBuffer.length >= 4) {
    const length = inputBuffer.readUInt32BE(0);
    if (length > MAX_FRAME_BYTES) {
      process.stderr.write("browser worker frame exceeds limit\n");
      process.exit(2);
    }
    if (inputBuffer.length < 4 + length) break;
    const payload = inputBuffer.subarray(4, 4 + length);
    inputBuffer = inputBuffer.subarray(4 + length);
    let request;
    try {
      request = JSON.parse(payload.toString("utf8"));
    } catch {
      writeFrame({ id: null, ok: false, error: { code: "invalid_json", message: "request is not valid JSON" } });
      continue;
    }
    void handle(request);
  }
});

process.stdin.on("end", async () => {
  if (context) await context.close().catch(() => {});
});

writeFrame({ id: null, ok: true, event: "hello", result: hello() });
