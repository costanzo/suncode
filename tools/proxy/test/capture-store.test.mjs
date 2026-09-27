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
