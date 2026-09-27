import { EventEmitter } from "node:events";

const REDACTED_HEADERS = new Set([
  "authorization",
  "cookie",
  "proxy-authorization",
  "set-cookie",
  "x-api-key",
  "x-auth-token",
  "x-goog-api-key",
]);

const captureLimit = Number.parseInt(process.env.CAPTURE_LIMIT_BYTES || "262144", 10);
const maxRecords = Number.parseInt(process.env.MAX_CAPTURED_REQUESTS || "500", 10);

export const captureEvents = new EventEmitter();
const records = new Map();
let sequence = 0;

function safeNumber(value, fallback = 0) {
  return Number.isFinite(value) ? value : fallback;
}

export function sanitizeHeaders(headers = {}) {
  return Object.fromEntries(
    Object.entries(headers).map(([name, value]) => [
      name.toLowerCase(),
      REDACTED_HEADERS.has(name.toLowerCase()) ? "[redacted]" : value,
    ]),
  );
}

function decodePreview(buffer, contentType = "") {
  if (!buffer?.length) return "";
  const text = buffer.toString("utf8");
  const looksBinary = buffer.subarray(0, Math.min(buffer.length, 512)).includes(0);
  if (looksBinary && !contentType.includes("json") && !contentType.startsWith("text/")) {
    return `[binary payload · ${buffer.length.toLocaleString()} bytes]\n${buffer
      .subarray(0, 96)
      .toString("base64")}…`;
  }
  if (contentType.includes("json")) {
    try {
      return JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      return text;
    }
  }
  return text;
}

function boundedAppend(target, chunk) {
  if (target.bytes >= captureLimit) return;
  const remaining = Math.max(0, captureLimit - target.bytes);
  const slice = chunk.subarray(0, remaining);
  target.chunks.push(slice);
  target.bytes += slice.length;
  target.truncated ||= slice.length < chunk.length;
}

function recordForPublic(record) {
  const { requestBuffer, responseBuffer, sseBuffer, sseCurrent, ...publicRecord } = record;
  return {
    ...publicRecord,
    requestHeaders: sanitizeHeaders(record.requestHeaders),
    responseHeaders: sanitizeHeaders(record.responseHeaders),
  };
}

function recordForSummary(record) {
  return {
    id: record.id,
    method: record.method,
    url: record.url,
    target: record.target,
    protocol: record.protocol,
    status: record.status,
    state: record.state,
    startedAt: record.startedAt,
    finishedAt: record.finishedAt,
    durationMs: record.durationMs,
    requestBytes: record.requestBytes,
    responseBytes: record.responseBytes,
    requestTruncated: record.requestTruncated,
    responseTruncated: record.responseTruncated,
    contentType: record.contentType,
    error: record.error,
  };
}

function publish(type, record) {
  captureEvents.emit("record", { type, record: recordForSummary(record) });
}

function trimRecords() {
  while (records.size > maxRecords) {
    const oldestId = records.keys().next().value;
    records.delete(oldestId);
  }
}

export function createCapture(input) {
  const now = new Date().toISOString();
  const record = {
    id: `req-${Date.now().toString(36)}-${(++sequence).toString(36)}`,
    method: input.method || "GET",
    url: input.url || "",
    target: input.target || "",
    protocol: input.protocol || "http",
    status: null,
    state: "forwarding",
    startedAt: now,
    finishedAt: null,
    durationMs: null,
    requestHeaders: input.requestHeaders || {},
    responseHeaders: {},
    requestBodyPreview: "",
    responseBodyPreview: "",
    requestBytes: 0,
    responseBytes: 0,
    requestTruncated: false,
    responseTruncated: false,
    contentType: "",
    error: null,
    sseEvents: [],
    sseCurrent: null,
    requestBuffer: { chunks: [], bytes: 0, truncated: false },
    responseBuffer: { chunks: [], bytes: 0, truncated: false },
    sseBuffer: "",
  };
  records.set(record.id, record);
  trimRecords();
  publish("created", record);
  return record;
}

export function appendRequestChunk(id, chunk) {
  const record = records.get(id);
  if (!record) return;
  boundedAppend(record.requestBuffer, chunk);
  record.requestBytes += chunk.length;
  record.requestTruncated = record.requestBuffer.truncated;
  record.requestBodyPreview = decodePreview(
    Buffer.concat(record.requestBuffer.chunks),
    record.requestHeaders["content-type"] || "",
  );
  publish("updated", record);
}

export function appendResponseChunk(id, chunk) {
  const record = records.get(id);
  if (!record) return;
  boundedAppend(record.responseBuffer, chunk);
  record.responseBytes += chunk.length;
  record.responseTruncated = record.responseBuffer.truncated;
  record.responseBodyPreview = decodePreview(
    Buffer.concat(record.responseBuffer.chunks),
    record.contentType,
  );
  if (record.contentType.includes("text/event-stream")) parseSse(record, chunk);
  publish("updated", record);
}

function parseSse(record, chunk) {
  record.sseBuffer += chunk.toString("utf8");
  const lines = record.sseBuffer.split(/\r?\n/);
  record.sseBuffer = lines.pop() || "";
  let event = record.sseCurrent;
  for (const line of lines) {
    if (line === "") {
      if (event && (event.data || event.event || event.id)) {
        record.sseEvents.push({ ...event, receivedAt: new Date().toISOString() });
        record.sseEvents = record.sseEvents.slice(-200);
      }
      event = null;
      continue;
    }
    if (line.startsWith(":")) continue;
    const separator = line.indexOf(":");
    const field = separator === -1 ? line : line.slice(0, separator);
    const value = (separator === -1 ? "" : line.slice(separator + 1)).replace(/^ /, "");
    event ||= { event: "message", data: "", id: "" };
    if (field === "data") event.data = event.data ? `${event.data}\n${value}` : value;
    else if (field === "event") event.event = value;
    else if (field === "id") event.id = value;
  }
  record.sseCurrent = event;
}

function flushSse(record) {
  if (!record.sseBuffer.trim() && !record.sseCurrent) return;
  const buffered = record.sseBuffer;
  record.sseBuffer = "";
  parseSse(record, Buffer.from(`${buffered}\n\n`));
}

export function updateCapture(id, patch) {
  const record = records.get(id);
  if (!record) return null;
  Object.assign(record, patch);
  publish("updated", record);
  return record;
}

export function finishCapture(id, patch = {}) {
  const record = records.get(id);
  if (!record) return null;
  if (record.contentType.includes("text/event-stream")) flushSse(record);
  const finishedAt = new Date();
  const startedAt = new Date(record.startedAt);
  Object.assign(record, {
    ...patch,
    finishedAt: finishedAt.toISOString(),
    durationMs: Math.max(0, finishedAt.getTime() - startedAt.getTime()),
    state: patch.state || (record.error ? "failed" : "completed"),
  });
  publish("finished", record);
  return record;
}

export function listCaptures() {
  return [...records.values()].reverse().map(recordForSummary);
}

export function getCapture(id) {
  const record = records.get(id);
  return record ? recordForPublic(record) : null;
}

export function clearCaptures() {
  records.clear();
  captureEvents.emit("clear");
}

export function getCaptureLimit() {
  return safeNumber(captureLimit, 262144);
}
