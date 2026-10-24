// Inspection only: these functions never mutate captured payloads.
export const PROVIDERS = [
  { id: "responses", label: "Responses", path: "/responses", input: "input" },
  { id: "chat", label: "Chat Completions", path: "/chat/completions", input: "messages" },
  { id: "messages", label: "Messages", path: "/messages", input: "messages" },
];
export const array = (value) => Array.isArray(value) ? value : [];
export const isObject = (value) => value !== null && typeof value === "object" && !Array.isArray(value);
export function parseJson(value) { try { return JSON.parse(value); } catch { return null; } }
export function endpointFor(record) {
  if (record.method !== "POST") return null;
  try {
    const path = new URL(record.target || record.url, "http://proxy.local").pathname.replace(/\/+$/, "");
    return PROVIDERS.find((provider) => path.endsWith(provider.path)) || null;
  } catch { return null; }
}
export function providerSummary(record) {
  const provider = endpointFor(record);
  let path = record.url || record.target || "Unknown route";
  try { path = new URL(record.target || record.url, "http://proxy.local").pathname; } catch { /* Preserve the supplied route. */ }
  return { provider, path, label: provider?.label || `${record.method || "HTTP"} request`, model: record.model,
    mode: record.contentType?.includes("text/event-stream") ? "SSE" : record.contentType?.includes("json") ? "JSON" : "HTTP" };
}
// Use the retained body prefix instead of the capture store's rolling event
// tail, so the first text deltas never silently disappear from readable output.
export function parseSse(raw = "") {
  const frames = raw.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n\n");
  const tail = frames.pop();
  const events = [];
  for (const frame of frames) {
    let name = "message", id = "";
    const dataLines = [];
    for (const line of frame.split("\n")) {
      if (line.startsWith(":")) continue;
      const at = line.indexOf(":");
      const key = at < 0 ? line : line.slice(0, at);
      const value = (at < 0 ? "" : line.slice(at + 1)).replace(/^ /, "");
      if (key === "event") name = value;
      if (key === "id") id = value;
      if (key === "data") dataLines.push(value);
    }
    if (!dataLines.length) continue;
    const data = dataLines.join("\n"), value = parseJson(data);
    events.push({ name: name === "message" ? value?.type || value?.object || name : name, id, data, value,
      malformed: data !== "[DONE]" && !isObject(value) });
  }
  return { events, pending: Boolean(tail?.trim() && !tail.trim().startsWith(":")) };
}
export function normalizeCapture(detail) {
  const summary = providerSummary(detail), request = parseJson(detail.requestBodyPreview);
  const sse = summary.mode === "SSE" ? parseSse(detail.responseBodyPreview) : { events: [], pending: false };
  const timestamps = new Map(array(detail.sseEvents).map((event) => [event.sequence, event.receivedAt]));
  return { ...detail, ...summary, request: isObject(request) ? request : null, response: parseJson(detail.responseBodyPreview),
    events: sse.events.map((event, index) => ({ ...event, receivedAt: timestamps.get(index + 1) })),
    pendingFrame: sse.pending, model: typeof request?.model === "string" ? request.model : summary.model };
}
export function readResponse(capture) {
  const result = { items: [], usage: null, reason: null, identity: null, terminal: false, error: null, malformed: false };
  const protocol = capture.provider?.id;
  if (capture.mode !== "SSE") {
    const response = capture.response;
    if (!isObject(response)) return { ...result, malformed: true };
    result.error = response.error || (response.type === "error" ? response : null);
    result.identity = response.id;
    result.usage = response.usage;
    if (protocol === "responses") { result.items = array(response.output).filter(isObject); result.reason = response.status; }
    if (protocol === "chat") {
      result.items = array(response.choices).filter(isObject).map((choice) => ({ ...choice.message, choice: choice.index }));
      result.reason = array(response.choices).filter(isObject).map((choice) => choice.finish_reason).filter(Boolean).join(", ");
    }
    if (protocol === "messages") { result.items = [response]; result.reason = response.stop_reason; }
    result.terminal = Boolean(result.reason);
    return result;
  }
  const chunks = array(capture.events).map((event) => event.value).filter(isObject);
  result.malformed = array(capture.events).some((event) => event.malformed);
  result.error = chunks.find((event) => event.type === "error" || event.type === "response.failed" || event.error);
  if (protocol === "responses") {
    const items = new Map();
    const keyFor = (event) => event.output_index ?? event.item_id ?? 0;
    const get = (event) => {
      const key = keyFor(event);
      if (!items.has(key)) items.set(key, { type: "message", role: "assistant", content: [] });
      return items.get(key);
    };
    for (const event of chunks) {
      result.identity ||= event.response?.id;
      if (["response.completed", "response.incomplete", "response.failed"].includes(event.type)) {
        result.terminal = true;
        result.reason = event.response?.status || event.type;
        result.usage = event.response?.usage;
        if (Array.isArray(event.response?.output)) result.items = event.response.output.filter(isObject);
      } else if (["response.output_item.added", "response.output_item.done"].includes(event.type)) {
        items.set(keyFor(event), structuredClone(isObject(event.item) ? event.item : {}));
      } else if (["response.output_text.delta", "response.output_text.done"].includes(event.type)) {
        const item = get(event), index = event.content_index ?? 0;
        item.content = array(item.content);
        item.content[index] ||= { type: "output_text", text: "" };
        item.content[index].text = event.type.endsWith(".done") ? event.text : (item.content[index].text || "") + (event.delta || "");
      } else if (["response.function_call_arguments.delta", "response.function_call_arguments.done"].includes(event.type)) {
        const item = get(event);
        item.type = "function_call";
        item.arguments = event.type.endsWith(".done") ? event.arguments : (item.arguments || "") + (event.delta || "");
      }
    }
    if (!result.items.length) result.items = [...items.values()];
  } else if (protocol === "chat") {
    const choices = new Map();
    for (const event of chunks) {
      result.identity ||= event.id;
      if (isObject(event.usage)) result.usage = event.usage;
      for (const choice of array(event.choices).filter(isObject)) {
        const key = choice.index ?? 0;
        if (!choices.has(key)) choices.set(key, { role: "assistant", content: "", choice: key, calls: new Map() });
        const item = choices.get(key), delta = choice.delta || {};
        if (typeof delta.content === "string") item.content += delta.content;
        for (const call of array(delta.tool_calls).filter(isObject)) {
          const index = call.index ?? 0;
          if (!item.calls.has(index)) item.calls.set(index, { id: "", function: { name: "", arguments: "" } });
          const current = item.calls.get(index);
          if (call.id) current.id = call.id;
          current.function.name += call.function?.name || "";
          current.function.arguments += call.function?.arguments || "";
        }
        if (choice.finish_reason) { item.finish_reason = choice.finish_reason; result.reason = choice.finish_reason; }
      }
    }
    result.items = [...choices.values()].map(({ calls, ...item }) => ({ ...item, tool_calls: [...calls.values()] }));
    result.terminal = Boolean(result.reason && array(capture.events).some((event) => event.data === "[DONE]"));
  } else if (protocol === "messages") {
    const blocks = [];
    for (const event of chunks) {
      if (event.type === "message_start") { result.identity = event.message?.id; result.usage = event.message?.usage; }
      if (event.type === "content_block_start") blocks[event.index] = { ...event.content_block };
      if (event.type === "content_block_delta") {
        const block = blocks[event.index] ||= { type: "text", text: "" };
        if (event.delta?.text) block.text = (block.text || "") + event.delta.text;
        if (event.delta?.partial_json) block.arguments = (block.arguments || "") + event.delta.partial_json;
        if (block.arguments) block.input = parseJson(block.arguments) ?? block.arguments;
      }
      if (event.type === "message_delta") { result.reason = event.delta?.stop_reason; result.usage = { ...result.usage, ...event.usage }; }
      if (event.type === "message_stop") result.terminal = true;
    }
    result.items = [{ role: "assistant", content: blocks.filter(Boolean) }];
  }
  return result;
}
