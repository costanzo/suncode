import { useState } from "react";
import { Icon } from "../../shared/Icon.jsx";
import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";

const requests = [
  { method: "POST", status: 200, path: "/v1/chat/completions", type: "application/json", duration: "1.24 s", size: "18.4 KB", state: "completed" },
  { method: "GET", status: 200, path: "/v1/models", type: "application/json", duration: "86 ms", size: "4.2 KB", state: "completed" },
  { method: "POST", status: 200, path: "/v1/responses", type: "text/event-stream", duration: "streaming", size: "62.8 KB", state: "streaming" },
  { method: "CONNECT", status: 200, path: "api.anthropic.com:443", type: "TLS tunnel", duration: "3.08 s", size: "—", state: "tunnel" },
];

function RequestRow({ request, selected }) {
  return (
    <div className={`proxy-request-row${selected ? " is-selected" : ""}`}>
      <div className="proxy-row-line"><code className={`proxy-method is-${request.method.toLowerCase()}`}>{request.method}</code><span className={`proxy-status is-${request.status >= 400 ? "warning" : "success"}`}>{request.status}</span><i className={`proxy-state-dot is-${request.state}`} /></div>
      <strong>{request.path}</strong>
      <div className="proxy-row-meta"><span>{request.duration}</span><span>{request.size}</span><span>{request.type}</span></div>
    </div>
  );
}

function HeaderRows({ response = false }) {
  const headers = response ? [["content-type", "text/event-stream"], ["cache-control", "no-cache"], ["x-request-id", "req_91f2…"]] : [["content-type", "application/json"], ["authorization", "[redacted]"], ["x-client-version", "0.1.0"]];
  return <div className="proxy-header-table">{headers.map(([key, value]) => <div key={key}><code>{key}</code><span>{value}</span></div>)}</div>;
}

function ProxyWorkspace({ state = "default" }) {
  const empty = state === "empty";
  const failed = state === "failed";
  const [detailTab, setDetailTab] = useState("request");
  return (
    <div className={`proxy-workspace-specimen is-${state}`}>
      <div className="proxy-specimen-topbar"><div className="proxy-specimen-brand"><span><Icon name="activity" size={15} /></span><strong>Proxy Tool</strong><small>local request inspector</small></div><div className="proxy-specimen-actions"><span className="proxy-live-pill"><i />Monitor connected</span><button aria-label="Pause capture"><Icon name="pause" size={13} /></button><button aria-label="Clear captures"><Icon name="close" size={13} /></button></div></div>
      <div className="proxy-endpoint-strip"><div><small>PROXY LISTENER</small><code>http://127.0.0.1:8080</code><span>HTTP + HTTPS MITM · local CA required</span></div><div><small>CAPTURED</small><strong>{empty ? "0" : "4"}</strong></div><div><small>ACTIVE</small><strong>{state === "streaming" ? "1" : "0"}</strong></div><div><small>SSE STREAMS</small><strong>{state === "streaming" ? "1" : "0"}</strong></div></div>
      <div className="proxy-filter-bar"><span><Icon name="search" size={13} /> Search URL or target</span><b>All methods</b><b>All states</b><em>{empty ? "0 shown" : "4 shown"}</em></div>
      <div className="proxy-specimen-body"><div className="proxy-request-column"><div className="proxy-column-heading"><div><small>LIVE CAPTURE</small><strong>Requests</strong></div><code>{empty ? "0" : "4"}</code></div>{empty ? <div className="proxy-specimen-empty"><Icon name="activity" size={20} /><strong>Waiting for traffic</strong><span>Send a request through the local proxy to see it here.</span></div> : requests.map((request, index) => <RequestRow key={request.path} request={{ ...request, status: failed && index === 0 ? 502 : request.status }} selected={index === 0} />)}</div><div className="proxy-detail-column">{empty ? <div className="proxy-specimen-empty"><Icon name="search" size={20} /><strong>Select a request</strong><span>Choose a captured request to inspect its headers and payloads.</span></div> : <><div className="proxy-detail-heading"><div><div><code className="proxy-method is-post">POST</code><span className={`proxy-status is-${failed ? "warning" : "success"}`}>{failed ? "502" : "200"}</span><small>{failed ? "Upstream failed" : "1.24 s"}</small></div><strong>https://api.openai.com/v1/chat/completions</strong><span>req-mh7p8 · started 10:42:18</span></div><button aria-label="Copy target"><Icon name="copy" size={14} /></button></div><div className="proxy-detail-metrics"><div><small>REQUEST BYTES</small><code>1.8 KB</code></div><div><small>RESPONSE BYTES</small><code>{failed ? "0 B" : "18.4 KB"}</code></div><div><small>CONTENT TYPE</small><code>{failed ? "—" : "application/json"}</code></div><div><small>TRANSPORT</small><code>https</code></div></div><div className="proxy-detail-tabs"><button className={detailTab === "request" ? "is-active" : ""} onClick={() => setDetailTab("request")}>Request</button><button className={detailTab === "response" ? "is-active" : ""} onClick={() => setDetailTab("response")}>Response</button></div><div className="proxy-detail-sections">{detailTab === "request" ? <><div><h4>Request headers <small>3 headers</small></h4><HeaderRows /></div><div><h4>Request body <small>1.8 KB</small></h4><pre>{`{\n  "model": "gpt-5",\n  "stream": true,\n  "messages": [...]\n}`}</pre></div></> : <><div><h4>Response headers <small>3 headers</small></h4><HeaderRows response /></div>{state === "streaming" ? <div><h4>SSE events <small>3 parsed</small></h4><div className="proxy-sse-events"><div><code>01</code><strong>response.output_text.delta</strong><span>"The request is ready…"</span></div><div><code>02</code><strong>response.output_text.delta</strong><span>"Here is the implementation…"</span></div><div><code>03</code><strong>response.completed</strong><span>{`{ "status": "completed" }`}</span></div></div></div> : <div><h4>Response body <small>{failed ? "No body" : "18.4 KB"}</small></h4><pre>{failed ? "Upstream request failed before a response body arrived." : `{\n  "id": "chatcmpl_91f2",\n  "choices": [...]\n}`}</pre></div>}</>}</div></>}</div></div>
    </div>
  );
}

export function ProxyToolPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    { id: "default", title: "Request inspection", description: "A completed API request with redacted headers and bounded JSON payloads." },
    { id: "streaming", title: "SSE stream", description: "An active event stream keeps raw response flow and parsed events visible together." },
    { id: "empty", title: "Waiting for traffic", description: "The first-run state explains the explicit proxy address and keeps the monitor calm." },
    { id: "failed", title: "Upstream failure", description: "A failed request makes the status, error context, and missing body explicit." },
  ];
  return <><PageHeader title="Proxy Tool" description="Standalone local proxy and request inspector for API development traffic." status="Review reference" tone="implemented" /><Section id="proxy-workspace" title="Monitoring workspace" description="The monitor keeps the request timeline and selected payload detail in one dense, reviewable surface."><div className="proxy-guide-grid">{states.map((item) => <div className="proxy-guide-state" key={item.id}><div className="proxy-guide-copy"><strong>{item.title}</strong><span>{item.description}</span><button onClick={() => setOpenGuide(openGuide === item.id ? null : item.id)}>{openGuide === item.id ? "Hide notes" : "View notes"}<Icon name="chevron-right" size={13} /></button></div><ProxyWorkspace state={item.id} />{openGuide === item.id && <div className="proxy-guide-notes"><span>Use the left list to select one request; the detail column preserves headers, body, timing, and transport state.</span><span>Credential headers remain visibly redacted, while content type and byte limits stay available for diagnosis.</span></div>}</div>)}</div></Section></>;
}
