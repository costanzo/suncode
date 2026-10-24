import test from "node:test";
import assert from "node:assert/strict";
import { endpointFor, normalizeCapture, parseSse, readResponse } from "../lib/provider-payload.mjs";

const stream = (events) => events.map((value) => `data: ${typeof value === "string" ? value : JSON.stringify(value)}\n\n`).join("");
const captureFor = (path, events) => normalizeCapture({ method: "POST", target: `https://custom.test/base/${path}`, contentType: "text/event-stream", responseBodyPreview: stream(events) });

test("endpoint recognition accepts custom base paths and rejects other HTTP operations", () => {
  for (const [path, id] of [["responses", "responses"], ["chat/completions", "chat"], ["messages", "messages"]]) {
    assert.equal(endpointFor({ method: "POST", target: `https://custom.test/arbitrary/prefix/${path}?key=value` }).id, id);
    assert.equal(endpointFor({ method: "GET", target: `https://custom.test/${path}` }), null);
  }
  assert.equal(endpointFor({ method: "POST", target: "https://custom.test/responses/extra" }), null);
});

test("provider payload parser reconstructs a Responses text stream", () => {
  const body = [
    'event: response.output_text.delta\ndata: {"type":"response.output_text.delta","delta":"Hello"}',
    'event: response.completed\ndata: {"type":"response.completed","response":{"id":"resp_1","status":"completed","usage":{"input_tokens":4,"output_tokens":2}}}',
    "data: [DONE]",
    "",
  ].join("\n\n");
  const capture = normalizeCapture({ method: "POST", target: "https://gateway.test/v1/responses", contentType: "text/event-stream", responseBodyPreview: body, requestBodyPreview: '{"model":"gpt-test"}', sseEvents: [] });
  const result = readResponse(capture);
  assert.equal(capture.provider.id, "responses");
  assert.equal(result.items[0].content[0].text, "Hello");
  assert.equal(result.identity, "resp_1");
  assert.equal(result.usage.output_tokens, 2);
  assert.equal(result.terminal, true);
});

test("provider payload parser joins Chat Completions tool-call deltas", () => {
  const firstChunk = {
    id: "chat_1",
    choices: [{
      index: 0,
      delta: { tool_calls: [{ index: 0, id: "call_1", function: { name: "read", arguments: '{"path":' } }] },
    }],
  };
  const secondChunk = {
    id: "chat_1",
    choices: [{
      index: 0,
      delta: { tool_calls: [{ index: 0, function: { arguments: '"README.md"}' } }] },
      finish_reason: "tool_calls",
    }],
  };
  const body = [
    `data: ${JSON.stringify(firstChunk)}`,
    `data: ${JSON.stringify(secondChunk)}`,
    "data: [DONE]",
    "",
  ].join("\n\n");
  const capture = normalizeCapture({ method: "POST", target: "https://gateway.test/v1/chat/completions", contentType: "text/event-stream", responseBodyPreview: body, requestBodyPreview: '{"model":"chat-test"}', sseEvents: [] });
  const result = readResponse(capture);
  assert.equal(result.items[0].tool_calls[0].id, "call_1");
  assert.equal(result.items[0].tool_calls[0].function.name, "read");
  assert.equal(result.items[0].tool_calls[0].function.arguments, '{"path":"README.md"}');
  assert.equal(result.terminal, true);
});

test("SSE parser keeps malformed and unfinished frames visible", () => {
  const result = parseSse('data: {oops}\n\ndata: {"type":"response.created"}\n\ndata: {"type":"response.output_text.delta"}');
  assert.equal(result.events[0].malformed, true);
  assert.equal(result.events[1].malformed, false);
  assert.equal(result.pending, true);
});

test("Responses tool-only output survives done snapshots without duplicate arguments", () => {
  const item = { type: "function_call", id: "fc1", call_id: "c1", name: "read", arguments: "" };
  const capture = captureFor("responses", [
    { type: "response.output_item.added", output_index: 0, item },
    { type: "response.function_call_arguments.delta", output_index: 0, delta: '{"path":' },
    { type: "response.function_call_arguments.delta", output_index: 0, delta: '"a.rs"}' },
    { type: "response.function_call_arguments.done", output_index: 0, arguments: '{"path":"a.rs"}' },
    { type: "response.output_item.done", output_index: 0, item: { ...item, arguments: '{"path":"a.rs"}' } },
    { type: "response.completed", response: { id: "r1", status: "completed", output: [{ ...item, arguments: '{"path":"a.rs"}' }] } },
  ]);
  const result = readResponse(capture);
  assert.equal(result.items.length, 1);
  assert.equal(result.items[0].arguments, '{"path":"a.rs"}');
  assert.equal(result.items[0].call_id, "c1");
  assert.equal(result.usage, undefined);
});

test("long captured streams keep the first delta beyond the rolling 200 event tail", () => {
  const events = Array.from({ length: 220 }, () => ({ type: "response.output_text.delta", delta: "x" }));
  const capture = captureFor("responses", events);
  capture.sseEvents = [];
  const result = readResponse(capture);
  assert.equal(result.items[0].content[0].text.length, 220);
  assert.equal(result.terminal, false);
  assert.equal(result.usage, null);
});

test("Chat usage-only chunks and unfinished streams do not fabricate completion", () => {
  const capture = captureFor("chat/completions", [
    { id: "ch1", choices: [{ index: 0, delta: { content: "Hi" }, finish_reason: "stop" }] },
    { id: "ch1", choices: [], usage: { prompt_tokens: 12 } },
  ]);
  const result = readResponse(capture);
  assert.deepEqual(result.usage, { prompt_tokens: 12 });
  assert.equal(result.terminal, false);
  assert.equal(result.items[0].content, "Hi");
});

test("Anthropic buffered JSON preserves tool_use blocks and reported cache usage", () => {
  const response = { id: "m1", role: "assistant", type: "message", stop_reason: "tool_use", content: [{ type: "tool_use", id: "t1", name: "read", input: { path: "a.rs" } }], usage: { input_tokens: 2, output_tokens: 3, cache_read_input_tokens: 4 } };
  const capture = normalizeCapture({ method: "POST", target: "https://custom.test/v1/messages", contentType: "application/json", responseBodyPreview: JSON.stringify(response) });
  assert.deepEqual(readResponse(capture).items[0].content, response.content);
  assert.equal(readResponse(capture).usage.cache_read_input_tokens, 4);
  assert.equal(readResponse(capture).terminal, true);
});

test("stream errors and malformed JSON retain a visible failure", () => {
  const capture = captureFor("responses", [{ type: "error", message: "Rate limited" }]);
  assert.equal(readResponse(capture).error.message, "Rate limited");
  assert.equal(readResponse(normalizeCapture({ method: "POST", target: "https://custom.test/messages", contentType: "application/json", responseBodyPreview: "{bad" })).malformed, true);
});
