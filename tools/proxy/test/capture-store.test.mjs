import test from "node:test";
import assert from "node:assert/strict";

test("capture-store redacts credential headers", async () => {
  const store = await import(`../lib/capture-store.mjs?test=${Date.now()}`);
  assert.deepEqual(store.sanitizeHeaders({ Authorization: "Bearer secret", "X-Trace": "ok" }), {
    authorization: "[redacted]",
    "x-trace": "ok",
  });
});

test("capture-store exposes bounded response previews and SSE events", async () => {
  const store = await import(`../lib/capture-store.mjs?test=${Date.now()}-sse`);
  const capture = store.createCapture({
    method: "GET",
    url: "http://example.test/events",
    target: "http://example.test/events",
    requestHeaders: {},
  });
  store.updateCapture(capture.id, { contentType: "text/event-stream" });
  store.appendResponseChunk(capture.id, Buffer.from("event: delta\ndata: hello\n\n"));
  store.appendResponseChunk(capture.id, Buffer.from("event: final\ndata: tail"));
  store.finishCapture(capture.id);
  const result = store.getCapture(capture.id);
  assert.equal(result.sseEvents[0].event, "delta");
  assert.equal(result.sseEvents[0].data, "hello");
  assert.equal(result.sseEvents[1].data, "tail");
  assert.match(result.responseBodyPreview, /data: hello/);
});

test("capture list returns summaries without payloads", async () => {
  const store = await import(`../lib/capture-store.mjs?test=${Date.now()}-summary`);
  const capture = store.createCapture({
    method: "POST",
    url: "http://example.test/inspect",
    target: "http://example.test/inspect",
    requestHeaders: { authorization: "Bearer secret", "content-type": "application/json" },
  });
  store.appendRequestChunk(capture.id, Buffer.from('{"hello":"world"}'));
  const summary = store.listCaptures()[0];
  assert.equal(summary.id, capture.id);
  assert.equal(summary.requestBodyPreview, undefined);
  assert.equal(summary.requestHeaders, undefined);
  const detail = store.getCapture(capture.id);
  assert.match(detail.requestBodyPreview, /"hello": "world"/);
  assert.equal(detail.requestHeaders.authorization, "[redacted]");
});

test("summary exposes only model and request identity while Raw preserves JSON whitespace", async () => {
  const store = await import(`../lib/capture-store.mjs?test=raw-${Date.now()}`);
  const capture = store.createCapture({ method: "POST", target: "https://custom.test/responses", requestHeaders: { "content-type": "application/json" } });
  const raw = '{ "model" : "test-model", "input" : "private prompt" }\n';
  store.appendRequestChunk(capture.id, Buffer.from(raw));
  store.updateCapture(capture.id, { responseHeaders: { "x-request-id": "provider-1" } });
  const detail = store.getCapture(capture.id), summary = store.listCaptures()[0];
  assert.equal(detail.requestBodyRawPreview, raw);
  assert.equal(summary.model, "test-model");
  assert.equal(summary.providerRequestId, "provider-1");
  assert.equal(summary.requestBodyRawPreview, undefined);
  assert.equal(detail.sseDecoder, undefined);
});

test("preview marks truncation when a later chunk exceeds an exact limit", async () => {
  const store = await import(`../lib/capture-store.mjs?test=limit-${Date.now()}`);
  const capture = store.createCapture({ method: "POST" });
  const limit = store.getCaptureLimit();
  store.appendRequestChunk(capture.id, Buffer.alloc(limit, 65));
  store.appendRequestChunk(capture.id, Buffer.from("extra"));
  assert.equal(store.getCapture(capture.id).requestTruncated, true);
  assert.equal(store.getCapture(capture.id).requestBodyRawPreview.length, limit);
  assert.equal(store.getCapture(capture.id).requestBytes, limit + 5);
});

test("SSE decoding preserves a Unicode character split across network chunks", async () => {
  const store = await import(`../lib/capture-store.mjs?test=utf8-${Date.now()}`);
  const capture = store.createCapture({ method: "POST" });
  store.updateCapture(capture.id, { contentType: "text/event-stream" });
  const raw = Buffer.from('data: {"delta":"你好"}\n\n');
  const split = raw.indexOf(Buffer.from("你")) + 1;
  store.appendResponseChunk(capture.id, raw.subarray(0, split));
  store.appendResponseChunk(capture.id, raw.subarray(split));
  assert.equal(store.getCapture(capture.id).sseEvents[0].data, '{"delta":"你好"}');
});
