import http from "node:http";
import { URL } from "node:url";
import next from "next";
import { Proxy as MitmProxy } from "http-mitm-proxy";
import {
  appendRequestChunk,
  appendResponseChunk,
  captureEvents,
  clearCaptures,
  createCapture,
  finishCapture,
  getCapture,
  getCaptureLimit,
  listCaptures,
  sanitizeHeaders,
  updateCapture,
} from "./lib/capture-store.mjs";

const uiPort = Number.parseInt(process.env.PORT || "3000", 10);
const proxyPort = Number.parseInt(process.env.PROXY_PORT || "8080", 10);
const dev = process.argv.includes("--dev") || process.env.NODE_ENV !== "production";
const nextApp = next({ dev, dir: new URL(".", import.meta.url).pathname });

function setCors(res) {
  res.setHeader("access-control-allow-origin", "*");
  res.setHeader("access-control-allow-headers", "content-type, authorization, x-api-key");
  res.setHeader("access-control-allow-methods", "GET,POST,OPTIONS");
}

function json(res, status, payload) {
  setCors(res);
  res.writeHead(status, { "content-type": "application/json; charset=utf-8" });
  res.end(JSON.stringify(payload));
}

async function handleUiRequest(req, res, handleNext) {
  const pathname = new URL(req.url, "http://localhost").pathname;
  if (pathname === "/api/requests" && req.method === "GET") {
    json(res, 200, { requests: listCaptures(), captureLimit: getCaptureLimit() });
    return;
  }
  if (pathname.startsWith("/api/requests/") && req.method === "GET") {
    const record = getCapture(pathname.slice("/api/requests/".length));
    if (!record) return json(res, 404, { error: "Request not found." });
    json(res, 200, record);
    return;
  }
  if (pathname === "/api/requests" && req.method === "DELETE") {
    clearCaptures();
    json(res, 200, { ok: true });
    return;
  }
  if (pathname === "/api/events" && req.method === "GET") {
    setCors(res);
    res.writeHead(200, {
      "cache-control": "no-cache, no-transform",
      "content-type": "text/event-stream; charset=utf-8",
      connection: "keep-alive",
    });
    res.write(`event: snapshot\ndata: ${JSON.stringify({ requests: listCaptures() })}\n\n`);
    const send = (payload) => res.write(`event: update\ndata: ${JSON.stringify(payload)}\n\n`);
    const clear = () => res.write("event: clear\ndata: {}\n\n");
    captureEvents.on("record", send);
    captureEvents.on("clear", clear);
    const heartbeat = setInterval(() => res.write(": ping\n\n"), 15_000);
    req.on("close", () => {
      clearInterval(heartbeat);
      captureEvents.off("record", send);
      captureEvents.off("clear", clear);
    });
    return;
  }
  if (pathname === "/api/health" && req.method === "GET") {
    json(res, 200, { ok: true, proxyPort, uiPort });
    return;
  }
  await handleNext(req, res);
}

await nextApp.prepare();
const handleNext = nextApp.getRequestHandler();
const uiServer = http.createServer((req, res) => {
  handleUiRequest(req, res, handleNext).catch((error) => {
    if (!res.headersSent) json(res, 500, { error: "Monitor server failed." });
    else res.destroy(error);
  });
});
const mitmProxy = new MitmProxy();
const mitmCaptures = new WeakMap();
mitmProxy.use(MitmProxy.gunzip);

mitmProxy.onError((context, error) => {
  const capture = context && mitmCaptures.get(context);
  if (capture) finishCapture(capture.id, { state: "failed", error: error.message });
  else console.error(`Proxy error: ${error.message}`);
});

mitmProxy.onRequest((context, callback) => {
  const request = context.clientToProxyRequest;
  const host = request.headers.host || "unknown-host";
  const protocol = context.isSSL ? "https" : "http";
  const target = `${protocol}://${host}${request.url || "/"}`;
  const capture = createCapture({
    method: request.method,
    url: request.url,
    target,
    protocol,
    requestHeaders: request.headers,
  });
  mitmCaptures.set(context, capture);
  context.onRequestData((ctx, chunk, next) => {
    appendRequestChunk(capture.id, chunk);
    next(null, chunk);
  });
  context.onRequestEnd((ctx, next) => {
    updateCapture(capture.id, { requestBodyComplete: true });
    next();
  });
  callback();
});

mitmProxy.onResponse((context, callback) => {
  const capture = mitmCaptures.get(context);
  if (!capture) return callback();
  const response = context.serverToProxyResponse;
  updateCapture(capture.id, {
    status: response.statusCode || null,
    responseHeaders: sanitizeHeaders(response.headers),
    contentType: String(response.headers["content-type"] || ""),
    state: "streaming",
  });
  context.onResponseData((ctx, chunk, next) => {
    appendResponseChunk(capture.id, chunk);
    next(null, chunk);
  });
  context.onResponseEnd((ctx, next) => {
    finishCapture(capture.id);
    next();
  });
  callback();
});

uiServer.listen(uiPort, "127.0.0.1", () => {
  console.log(`Proxy Tool UI listening on http://127.0.0.1:${uiPort}`);
});
mitmProxy.listen({ port: proxyPort, host: "127.0.0.1", sslCaDir: new URL(".ca", import.meta.url).pathname }, () => {
  console.log(`Proxy listener ready on http://127.0.0.1:${proxyPort}`);
  console.log(`MITM CA directory: ${new URL(".ca", import.meta.url).pathname}`);
});
