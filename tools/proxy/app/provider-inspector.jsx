"use client";

import { useEffect, useMemo, useState } from "react";

import { array, isObject, providerSummary, normalizeCapture, readResponse } from "../lib/provider-payload.mjs";

function Icon({ name, size = 16 }) {
  const paths = {
    activity: <path d="M2 9h3l2-5 4 10 2-5h5" />,
    check: <path d="m4 9 3 3 7-7" />,
    close: <path d="m5 5 10 10m0-10L5 15" />,
    copy: <><rect x="5" y="5" width="8" height="9" rx="1" /><path d="M8 5V3h6a1 1 0 0 1 1 1v8h-2" /></>,
    file: <><path d="M5 2h6l3 3v11H5z" /><path d="M11 2v4h4" /></>,
    lock: <><rect x="4" y="8" width="10" height="8" rx="1" /><path d="M6 8V5a3 3 0 0 1 6 0v3" /></>,
    pause: <><path d="M6 4v12M12 4v12" /></>,
    play: <path d="m6 4 10 6-10 6z" />,
    search: <><circle cx="8" cy="8" r="5" /><path d="m12 12 4 4" /></>,
    tool: <><path d="m4 4 10 10M11 4a3 3 0 0 0 3 3M7 14a3 3 0 0 0-3-3" /><path d="M5 5 3 3m10 12 2-2" /></>,
    warning: <><path d="m9 2 7 14H2z" /><path d="M9 6v4m0 3v.1" /></>,
  };
  return <svg aria-hidden="true" width={size} height={size} viewBox="0 0 18 18" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">{paths[name]}</svg>;
}

function formatBytes(value) {
  if (!value) return "0 B";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function formatDuration(value) {
  if (value == null) return "In progress";
  return value < 1000 ? `${Math.round(value)} ms` : `${(value / 1000).toFixed(2)} s`;
}

function formatTime(value) {
  if (!value) return "--:--:--";
  try { return new Date(value).toLocaleTimeString([], { hour12: false }); } catch { return "--:--:--"; }
}

function statusTone(status) {
  if (!status) return "pending";
  if (status >= 500) return "danger";
  if (status >= 400) return "warning";
  return "success";
}

export const getProviderSummary = providerSummary;

function CopyButton({ value, label = "Copy" }) {
  const [message, setMessage] = useState("");
  useEffect(() => { setMessage(""); }, [value]);
  async function copy() {
    try { await navigator.clipboard.writeText(value || ""); setMessage("Copied"); } catch { setMessage("Copy unavailable"); }
  }
  return <button type="button" className="proxy-action" onClick={copy}><Icon name={message === "Copied" ? "check" : "copy"} size={14} /><span aria-live="polite">{message || label}</span></button>;
}

function Code({ value }) {
  const text = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  const tokens = String(text || "").split(/("(?:\\.|[^"\\])*"\s*:|"(?:\\.|[^"\\])*"|\b(?:true|false|null|-?\d+(?:\.\d+)?)\b)/g);
  return <pre className="llm-code" tabIndex={0}><code>{tokens.map((token, index) => <span key={index} className={token.endsWith(":") ? "is-key" : token.startsWith('"') ? "is-string" : /^(true|false|null|-?\d+(?:\.\d+)?)$/.test(token) ? "is-value" : undefined}>{token}</span>)}</code></pre>;
}

function Field({ name, children }) {
  return <div className="llm-field"><dt>{name}</dt><dd>{children}</dd></div>;
}

const callAnchor = (scope, id) => `${scope}-call-${encodeURIComponent(id || "unknown")}`;

function CallLink({ id, scope }) {
  if (!id) return <span className="llm-source">Unlinked tool result</span>;
  return <a className="llm-call-link" href={`#${callAnchor(scope, id)}`} onClick={(event) => {
    event.preventDefault();
    const target = document.getElementById(callAnchor(scope, id));
    if (target) { target.open = true; target.querySelector("summary")?.focus(); target.scrollIntoView({ block: "nearest" }); }
  }}>{id}</a>;
}

function ToolCall({ name, id, args, source, scope }) {
  let parsed = args;
  if (typeof args === "string") { try { parsed = JSON.parse(args); } catch { /* Preserve partial or malformed arguments. */ } }
  return <details className="llm-tool-call" id={scope ? callAnchor(scope, id) : undefined} open><summary><Icon name="tool" size={14} /><strong>{name || "Unnamed tool"}</strong><code>{id || "No call ID"}</code></summary><div className="llm-tool-content"><span className="llm-source">{source}</span><Code value={parsed ?? "No arguments captured."} /></div></details>;
}

function Content({ content, source, scope }) {
  if (typeof content === "string") return <p>{content}</p>;
  if (!Array.isArray(content)) return content == null ? null : <Code value={content} />;
  return content.map((block, index) => {
    if (!isObject(block)) return <Code key={index} value={block} />;
    const path = `${source}[${index}]`;
    if (["text", "input_text", "output_text"].includes(block.type)) return typeof block.text === "string" ? <p key={index}>{block.text}</p> : <Code key={index} value={block} />;
    if (block.type === "tool_use") return <ToolCall key={index} name={block.name} id={block.id} args={block.input} source={path} scope={scope} />;
    if (block.type === "tool_result") return <div key={index} className="llm-tool-result"><div><Icon name={block.is_error ? "warning" : "check"} size={14} /><strong>{block.is_error ? "Tool error" : "Tool result"}</strong><CallLink id={block.tool_use_id} scope={scope} /></div><span className="llm-source">{path}</span><Content content={block.content} source={`${path}.content`} scope={scope} /></div>;
    if (["input_image", "image", "image_url"].includes(block.type)) return <div key={index} className="llm-attachment"><Icon name="file" size={16} /><span>Image attachment · inspect Raw for source data</span></div>;
    return <Code key={index} value={block} />;
  });
}

function RequestContent({ capture }) {
  const request = capture.request;
  if (!capture.provider) return <><p className="llm-muted">Generic capture · original request body</p><Code value={capture.requestBodyPreview || "No request body captured."} /></>;
  if (!request) return <><div className="llm-notice"><Icon name="warning" size={16} /><div><strong>{capture.requestTruncated ? "Request capture is incomplete" : capture.requestBodyComplete ? "Request body is not a JSON object" : "Waiting for the request body"}</strong><p>Inspect Raw for the available payload.</p></div></div><Code value={capture.requestBodyPreview || "No request body captured."} /></>;
  const inputKey = capture.provider.input;
  const input = request[inputKey];
  const items = typeof input === "string" ? [{ role: "user", content: input }] : array(input);
  const instructions = [];
  if (request.instructions != null) instructions.push(["instructions", request.instructions]);
  if (request.system != null) instructions.push(["system", request.system]);
  items.forEach((item, index) => { if (["system", "developer"].includes(item?.role)) instructions.push([`${inputKey}[${index}].content`, item.content]); });
  const settings = Object.entries(request).filter(([key]) => !["input", "messages", "system", "instructions", "tools"].includes(key));
  const scope = `${capture.id}-request`;
  return <>
    <div className="llm-section-heading"><h3>Generation settings</h3><span>Sent to the provider</span></div>
    <dl className="llm-fields">{settings.map(([key, value]) => <Field key={key} name={key}>{isObject(value) || Array.isArray(value) ? <Code value={value} /> : <code>{String(value)}</code>}</Field>)}</dl>
    <div className="llm-section-heading"><h3>Conversation</h3><span>{inputKey} · {items.length} wire items</span></div>
    {instructions.length > 0 && <details className="llm-disclosure"><summary><strong>System instructions</strong><span>{instructions.length} source{instructions.length === 1 ? "" : "s"}</span></summary>{instructions.map(([source, content]) => <div key={source}><code className="llm-source">{source}</code><Content content={content} source={source} scope={scope} /></div>)}</details>}
    <div className="llm-transcript">{items.map((item, index) => {
      const source = typeof input === "string" ? inputKey : `${inputKey}[${index}]`;
      if (!isObject(item)) return <Code key={source} value={item} />;
      if (["system", "developer"].includes(item.role)) return null;
      if (item.type === "function_call") return <ToolCall key={source} name={item.name} id={item.call_id} args={item.arguments} source={source} scope={scope} />;
      const tool = item.type === "function_call_output" || item.role === "tool";
      if (!item.role && !tool) return <Code key={source} value={item} />;
      return <article className="llm-message" key={source}><header><strong>{tool ? "Tool result" : item.role === "assistant" ? "Assistant" : item.role === "user" ? "User" : item.role}</strong><code>{source}</code></header>{tool && <CallLink id={item.call_id || item.tool_call_id} scope={scope} />}<Content content={item.content ?? item.output} source={`${source}.${item.type === "function_call_output" ? "output" : "content"}`} scope={scope} />{array(item.tool_calls).map((call, callIndex) => <ToolCall key={callIndex} name={call?.function?.name} id={call?.id} args={call?.function?.arguments} source={`${source}.tool_calls[${callIndex}]`} scope={scope} />)}</article>;
    })}</div>
    {input != null && !Array.isArray(input) && typeof input !== "string" && <Code value={input} />}
    {array(request.tools).length > 0 && <><div className="llm-section-heading"><h3>Available tools</h3><span>{request.tools.length} tools</span></div>{request.tools.map((tool, index) => {
      if (!isObject(tool)) return <Code key={index} value={tool} />;
      const definition = tool.function || tool;
      return <details className="llm-disclosure" key={index}><summary><Icon name="tool" size={14} /><strong>{definition.name || tool.type || "Unnamed tool"}</strong><span>{tool.input_schema ? "input_schema" : "parameters"}</span></summary>{typeof definition.description === "string" && <p>{definition.description}</p>}<Code value={definition.parameters || definition.input_schema || definition} /></details>;
    })}</>}
  </>;
}

function ResponseContent({ capture }) {
  const result = readResponse(capture);
  const pending = ["streaming", "forwarding"].includes(capture.state);
  const incomplete = !result.terminal || capture.responseTruncated || result.malformed || capture.pendingFrame;
  if (capture.error || capture.status >= 400 || result.error) {
    const requestId = capture.responseHeaders?.["x-request-id"] || capture.responseHeaders?.["request-id"];
    const retry = capture.responseHeaders?.["retry-after"];
    return <div className="llm-error"><Icon name="warning" size={20} /><div><h3>{capture.status === 429 ? "Rate limit exceeded" : "Provider request failed"}</h3><p>{capture.error || `The provider returned HTTP ${capture.status || "unknown"}.`}</p>{retry && <p>Retry-After: <code>{retry}</code></p>}{requestId && <code>{requestId}</code>}<Code value={result.error || capture.response || capture.responseBodyPreview || "No response body captured."} /></div></div>;
  }
  if (!capture.provider) return <><p className="llm-muted">Generic capture · original response body</p><Code value={capture.responseBodyPreview || "No body captured."} /></>;
  return <>
    {incomplete && <div className="llm-notice"><Icon name="warning" size={16} /><div><strong>{capture.responseTruncated ? "Capture is incomplete" : result.malformed ? "Response could not be parsed completely" : pending ? "Response in progress" : "Completion is unconfirmed"}</strong><p>{capture.responseTruncated ? "Only the retained body prefix is available. Completion and usage cannot be confirmed." : pending ? "The provider has not finished sending its response." : "No complete terminal response was captured. Inspect Raw and Events for the available payload."}</p></div></div>}
    <div className="llm-section-heading"><h3>Assistant output</h3><span>{incomplete ? "Partial capture" : result.reason}</span></div>
    {result.items.length ? <div className="llm-transcript">{result.items.map((item, index) => item.type === "function_call" ? <ToolCall key={index} name={item.name} id={item.call_id} args={item.arguments} source={`output[${index}]`} /> : <article className="llm-output" key={index}>{item.content != null && <Content content={item.content} source={`output[${index}]`} />}{array(item.tool_calls).map((call, callIndex) => <ToolCall key={callIndex} name={call.function?.name} id={call.id} args={call.function?.arguments} source={`choices[${item.choice ?? index}].tool_calls[${callIndex}]`} />)}{!item.content && !item.tool_calls?.length && <Code value={item} />}</article>)}</div> : pending ? <div className="llm-skeleton" role="status" aria-label="Waiting for response"><i /><i /><i /></div> : <p className="llm-muted">No assistant output could be reconstructed. Inspect Raw for the captured payload.</p>}
    {isObject(result.usage) && !capture.responseTruncated ? <><div className="llm-section-heading"><h3>Token usage</h3><span>Reported by the provider</span></div><dl className="llm-fields">{usageFields(result.usage).map(([name, value]) => <Field key={name} name={name}><code>{value.toLocaleString()}</code></Field>)}</dl></> : <p className="llm-muted">Token usage is unavailable for this capture.</p>}
    <details className="llm-disclosure"><summary><strong>Response identity</strong><code>{result.identity || "Unavailable"}</code></summary><dl className="llm-fields"><Field name="request ID"><code>{capture.responseHeaders?.["x-request-id"] || capture.responseHeaders?.["request-id"] || "Unavailable"}</code></Field><Field name="finish reason"><code>{result.reason || "Unconfirmed"}</code></Field></dl></details>
    {result.malformed && <Code value={capture.responseBodyPreview} />}
  </>;
}

function usageFields(usage, prefix = "") {
  return Object.entries(usage).flatMap(([key, value]) => typeof value === "number" ? [[`${prefix}${key}`, value]] : isObject(value) ? usageFields(value, `${prefix}${key}.`) : []);
}

function Headers({ capture, side }) {
  const headers = side === "request" ? capture.requestHeaders : capture.responseHeaders;
  return <><div className="llm-section-heading"><h3>{side === "request" ? "Request" : "Response"} headers</h3><span>Credential values redacted</span></div><dl className="llm-headers">{Object.entries(headers || {}).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{Array.isArray(value) ? value.join(", ") : String(value)}</dd></div>)}{!Object.keys(headers || {}).length && <p className="llm-muted">No headers captured.</p>}</dl></>;
}

function Events({ capture }) {
  return <><div className="llm-section-heading"><h3>Stream events</h3><span>{capture.events.length} complete events in retained body</span></div><div className="llm-events">{capture.events.map((event, index) => <details key={index}><summary><code className="llm-event-index">{String(index + 1).padStart(2, "0")}</code><strong>{event.name}</strong><code>{event.receivedAt ? formatTime(event.receivedAt) : "Time not retained"}</code></summary>{event.id && <code className="llm-source">id: {event.id}</code>}<Code value={event.value ?? event.data} /></details>)}{!capture.events.length && <p className="llm-muted">No complete SSE event has been captured yet.</p>}{capture.pendingFrame && <p className="llm-muted">An unfinished event remains available in Raw.</p>}</div></>;
}

export function ProviderInspector({ detail }) {
  const capture = useMemo(() => normalizeCapture(detail), [detail]);
  const [side, setSide] = useState(capture.status >= 400 ? "response" : "request");
  const [view, setView] = useState("readable");
  const body = side === "request" ? detail.requestBodyRawPreview || detail.requestBodyPreview : detail.responseBodyRawPreview || detail.responseBodyPreview;
  const views = ["readable", "raw", "headers", ...(side === "response" && capture.mode === "SSE" ? ["events"] : [])];
  const target = detail.target || detail.url || "";
  return <article className="proxy-detail-column"><header className="llm-detail-heading"><div><div className="llm-detail-title"><h2>{capture.label}</h2><span className={`llm-status is-${statusTone(capture.status)}`}>{capture.status || "Pending"} · {capture.state}</span></div><code>{detail.method || "REQUEST"} {target}</code><p><code>{capture.model || "Model unavailable"}</code><span>{formatDuration(detail.durationMs)}</span><span>{capture.mode === "SSE" ? "Server-sent events" : capture.mode === "JSON" ? "JSON response" : "HTTP response"}</span></p></div><CopyButton value={target} label="Copy URL" /></header><div className="llm-view-toolbar"><div className="llm-side-switch" aria-label="Payload direction">{["request", "response"].map((item) => <button type="button" aria-pressed={side === item} key={item} onClick={() => { setSide(item); setView("readable"); }}>{item === "request" ? "Request" : "Response"}</button>)}</div><div className="llm-view-switch" aria-label="Payload view">{views.map((item) => <button type="button" aria-pressed={view === item} key={item} onClick={() => setView(item)}>{item === "readable" ? "Readable" : item === "raw" ? "Raw" : item === "headers" ? "Headers" : `Events (${capture.events.length})`}</button>)}</div></div><div className="llm-detail-body">{view === "headers" ? <Headers capture={capture} side={side} /> : view === "raw" ? <><div className="llm-section-heading"><h3>{side === "request" ? "Request body" : capture.mode === "SSE" ? "Raw event stream" : "Response body"}</h3><CopyButton value={body} label="Copy body" /></div><Code value={body || "No body captured."} /></> : view === "events" ? <Events capture={capture} /> : side === "request" ? <RequestContent capture={capture} /> : <ResponseContent capture={capture} />}</div></article>;
}

export function ProviderEmpty({ filtered, onReset, hasCapture, listener }) {
  return <div className="llm-empty"><Icon name={filtered ? "search" : "activity"} size={24} /><h3>{filtered ? "No matching calls" : hasCapture ? "Select a captured call" : "Waiting for traffic"}</h3><p>{filtered ? "Try a different endpoint, model, or request ID." : hasCapture ? "Choose a call to inspect its provider payload." : "Send a provider request through the local proxy to inspect it here."}</p>{filtered ? <button type="button" className="proxy-action" onClick={onReset}>Reset filters</button> : !hasCapture && listener && <code>HTTP_PROXY=http://{listener}</code>}</div>;
}

export { formatBytes, formatDuration, formatTime, Icon, statusTone };
