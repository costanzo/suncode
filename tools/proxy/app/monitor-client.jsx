"use client";

import { useEffect, useMemo, useRef, useState } from "react";

function Icon({ name, size = 16 }) {
  const paths = {
    activity: <path d="M2 9h3l2-5 4 10 2-5h5" />,
    arrow: <path d="m6 9 5 5 5-5" />,
    chevron: <path d="m6 9 5 5 5-5" />,
    close: <path d="m5 5 10 10m0-10L5 15" />,
    copy: (
      <>
        <rect x="5" y="5" width="8" height="9" rx="1" />
        <path d="M8 5V3h6a1 1 0 0 1 1 1v8h-2" />
      </>
    ),
    filter: (
      <>
        <path d="M3 4h14M5 8h10M8 12h4" />
        <path d="M5 16h10" />
      </>
    ),
    pause: (
      <>
        <path d="M6 4v12M12 4v12" />
      </>
    ),
    play: <path d="m6 4 10 6-10 6z" />,
    search: (
      <>
        <circle cx="8" cy="8" r="5" />
        <path d="m12 12 4 4" />
      </>
    ),
    trash: (
      <>
        <path d="M4 5h12M7 5V3h6v2M6 7v8h8V7" />
        <path d="M8 9v4M12 9v4" />
      </>
    ),
  };
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 18 18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {paths[name]}
    </svg>
  );
}

function formatDuration(value) {
  if (value == null) return "—";
  return value < 1000
    ? `${Math.round(value)} ms`
    : `${(value / 1000).toFixed(2)} s`;
}

function formatBytes(value) {
  if (!value) return "0 B";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function methodClass(method) {
  return `method method-${String(method).toLowerCase()}`;
}

function statusClass(status) {
  if (!status) return "status-pending";
  if (status >= 500) return "status-danger";
  if (status >= 400) return "status-warning";
  return "status-success";
}

function HeaderTable({ title, headers = {} }) {
  return (
    <section className="detail-section">
      <div className="section-heading">
        <h3>{title}</h3>
        <span>{Object.keys(headers).length} headers</span>
      </div>
      <div className="header-table">
        {Object.entries(headers).map(([key, value]) => (
          <div className="header-row" key={key}>
            <code>{key}</code>
            <span>
              {Array.isArray(value) ? value.join(", ") : String(value)}
            </span>
          </div>
        ))}
        {!Object.keys(headers).length && (
          <div className="subtle-empty">No headers captured.</div>
        )}
      </div>
    </section>
  );
}

function Payload({ title, value, truncated, bytes }) {
  return (
    <section className="detail-section">
      <div className="section-heading">
        <h3>{title}</h3>
        <span>
          {formatBytes(bytes)}
          {truncated ? " · truncated" : ""}
        </span>
      </div>
      <pre className="payload">{value || "No body captured."}</pre>
    </section>
  );
}

export default function ProxyToolClient() {
  const [requests, setRequests] = useState([]);
  const [selectedId, setSelectedId] = useState(null);
  const [selectedDetail, setSelectedDetail] = useState(null);
  const [detailTab, setDetailTab] = useState("request");
  const [query, setQuery] = useState("");
  const [method, setMethod] = useState("all");
  const [state, setState] = useState("all");
  const [paused, setPaused] = useState(false);
  const [theme, setTheme] = useState("dark");
  const [connected, setConnected] = useState(false);
  const selectedIdRef = useRef(null);

  useEffect(() => {
    selectedIdRef.current = selectedId;
  }, [selectedId]);

  useEffect(() => {
    const syncRequests = () =>
      fetch("/api/requests", { cache: "no-store" })
        .then((response) => response.json())
        .then((payload) => {
          if (!paused) setRequests(payload.requests || []);
        })
        .catch(() => {});
    syncRequests();
    const poll = window.setInterval(syncRequests, 2000);
    if (typeof EventSource === "undefined") {
      return () => window.clearInterval(poll);
    }
    const source = new EventSource("/api/events");
    source.addEventListener("open", () => setConnected(true));
    source.addEventListener("error", () => setConnected(false));
    source.addEventListener("snapshot", (event) => {
      if (!paused) setRequests(JSON.parse(event.data).requests || []);
    });
    source.addEventListener("update", (event) => {
      if (paused) return;
      const { record } = JSON.parse(event.data);
      setRequests((current) => {
        const next = current.filter((item) => item.id !== record.id);
        return [record, ...next].slice(0, 500);
      });
      if (record.id === selectedIdRef.current) {
        fetch(`/api/requests/${encodeURIComponent(record.id)}`, {
          cache: "no-store",
        })
          .then((response) => (response.ok ? response.json() : null))
          .then((detail) => {
            if (detail && selectedIdRef.current === record.id) setSelectedDetail(detail);
          })
          .catch(() => {});
      }
    });
    source.addEventListener("clear", () => {
      if (!paused) {
        setRequests([]);
        selectedIdRef.current = null;
        setSelectedId(null);
        setSelectedDetail(null);
      }
    });
    return () => {
      window.clearInterval(poll);
      source.close();
    };
  }, [paused]);

  const filtered = useMemo(
    () =>
      requests.filter((request) => {
        const text =
          `${request.method} ${request.url} ${request.target}`.toLowerCase();
        return (
          (method === "all" || request.method === method) &&
          (state === "all" || request.state === state) &&
          (!query || text.includes(query.toLowerCase()))
        );
      }),
    [requests, query, method, state],
  );
  const selected =
    requests.find((request) => request.id === selectedId) || null;
  const activeCount = requests.filter(
    (request) =>
      request.state === "forwarding" ||
      request.state === "streaming" ||
      request.state === "tunnel",
  ).length;
  const streamCount = requests.filter((request) =>
    request.contentType?.includes("text/event-stream"),
  ).length;

  async function clearAll() {
    await fetch("/api/requests", { method: "DELETE" });
    setRequests([]);
    selectedIdRef.current = null;
    setSelectedId(null);
    setSelectedDetail(null);
    setDetailTab("request");
  }

  async function selectRequest(request) {
    if (selectedIdRef.current === request.id) return;
    selectedIdRef.current = request.id;
    setSelectedId(request.id);
    setSelectedDetail(null);
    setDetailTab("request");
    try {
      const response = await fetch(
        `/api/requests/${encodeURIComponent(request.id)}`,
        { cache: "no-store" },
      );
      if (response.ok) {
        const detail = await response.json();
        if (selectedIdRef.current === request.id) setSelectedDetail(detail);
      }
    } catch {
      // The list remains usable when a just-finished request is evicted.
    }
  }

  return (
    <main className={`proxy-shell theme-${theme}`}>
      <header className="topbar">
        <div className="brand-lockup">
          <span className="brand-mark">
            <Icon name="activity" size={18} />
          </span>
          <div>
            <strong>Proxy Tool</strong>
            <span>local request inspector</span>
          </div>
        </div>
        <div className="topbar-actions">
          <span className={`live-status ${connected ? "is-live" : ""}`}>
            <i />
            {connected ? "Monitor connected" : "Connecting"}
          </span>
          <button
            className="icon-button"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            aria-label="Toggle theme"
          >
            {theme === "dark" ? "Light" : "Dark"}
          </button>
          <button className="quiet-button" onClick={() => setPaused(!paused)}>
            <Icon name={paused ? "play" : "pause"} size={14} />
            {paused ? "Resume" : "Pause"}
          </button>
          <button className="quiet-button danger-action" onClick={clearAll}>
            <Icon name="trash" size={14} />
            Clear
          </button>
        </div>
      </header>

      <section className="status-strip" aria-label="Proxy status">
        <div className="endpoint-card">
          <span className="eyebrow">Proxy listener</span>
          <code>http://127.0.0.1:8080</code>
          <span className="endpoint-note">
            HTTP + HTTPS MITM · local CA required
          </span>
        </div>
        <div className="metric">
          <span>Captured</span>
          <strong>{requests.length}</strong>
        </div>
        <div className="metric">
          <span>Active</span>
          <strong>{activeCount}</strong>
        </div>
        <div className="metric">
          <span>SSE streams</span>
          <strong>{streamCount}</strong>
        </div>
      </section>

      <section className="toolbar" aria-label="Request filters">
        <div className="search-field">
          <Icon name="search" size={15} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search URL or target"
            aria-label="Search URL or target"
          />
        </div>
        <label>
          <span>Method</span>
          <select
            value={method}
            onChange={(event) => setMethod(event.target.value)}
          >
            <option value="all">All methods</option>
            {["GET", "POST", "PUT", "PATCH", "DELETE", "CONNECT"].map(
              (item) => (
                <option key={item} value={item}>
                  {item}
                </option>
              ),
            )}
          </select>
        </label>
        <label>
          <span>State</span>
          <select
            value={state}
            onChange={(event) => setState(event.target.value)}
          >
            <option value="all">All states</option>
            <option value="forwarding">Forwarding</option>
            <option value="streaming">Streaming</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="tunnel">Tunnel</option>
          </select>
        </label>
        <span className="result-count">
          <Icon name="filter" size={14} />
          {filtered.length} shown
        </span>
      </section>

      <section className="workspace-grid">
        <aside className="request-pane">
          <div className="pane-heading">
            <div>
              <span className="eyebrow">Live capture</span>
              <h1>Requests</h1>
            </div>
            <span className="pane-count">{filtered.length}</span>
          </div>
          {filtered.length ? (
            <div className="request-list">
              {filtered.map((request) => (
                <button
                  type="button"
                  className={`request-row ${selected?.id === request.id ? "is-selected" : ""}`}
                  key={request.id}
                  onClick={() => selectRequest(request)}
                >
                  <div className="request-row-top">
                    <span className={methodClass(request.method)}>
                      {request.method}
                    </span>
                    <span
                      className={`status-chip ${statusClass(request.status)}`}
                    >
                      {request.status || "…"}
                    </span>
                    <span className={`state-dot state-${request.state}`} />
                  </div>
                  <strong>{request.url || request.target}</strong>
                  <div className="request-row-meta">
                    <span>{formatDuration(request.durationMs)}</span>
                    <span>{formatBytes(request.responseBytes)}</span>
                    <span>
                      {request.contentType?.split(";")[0] || "unknown"}
                    </span>
                  </div>
                </button>
              ))}
            </div>
          ) : (
            <div className="empty-list">
              <div className="empty-icon">
                <Icon name="activity" size={22} />
              </div>
              <h2>Waiting for traffic</h2>
              <p>Send a request through the local proxy to see it here.</p>
              <code>HTTP_PROXY=http://127.0.0.1:8080</code>
            </div>
          )}
        </aside>
        <article className="detail-pane">
          {selectedDetail ? (
            <>
              <div className="detail-header">
                <div>
                  <div className="detail-title-line">
                    <span className={methodClass(selectedDetail.method)}>
                      {selectedDetail.method}
                    </span>
                    <span
                      className={`status-chip ${statusClass(selectedDetail.status)}`}
                    >
                      {selectedDetail.status || selectedDetail.state}
                    </span>
                    <span className="detail-timing">
                      {formatDuration(selectedDetail.durationMs)}
                    </span>
                  </div>
                  <h2>{selectedDetail.target || selectedDetail.url}</h2>
                  <p>
                    {selectedDetail.id} · started{" "}
                    {new Date(selectedDetail.startedAt).toLocaleTimeString()}
                  </p>
                </div>
                <button
                  className="icon-button"
                  aria-label="Copy target"
                  onClick={() =>
                    navigator.clipboard?.writeText(
                      selectedDetail.target || selectedDetail.url,
                    )
                  }
                >
                  <Icon name="copy" size={16} />
                </button>
              </div>
              <div className="detail-summary">
                <div>
                  <span>Request bytes</span>
                  <strong>{formatBytes(selectedDetail.requestBytes)}</strong>
                </div>
                <div>
                  <span>Response bytes</span>
                  <strong>{formatBytes(selectedDetail.responseBytes)}</strong>
                </div>
                <div>
                  <span>Content type</span>
                  <strong>{selectedDetail.contentType || "—"}</strong>
                </div>
                <div>
                  <span>Transport</span>
                  <strong>{selectedDetail.protocol}</strong>
                </div>
              </div>
              <div
                className="detail-tabs"
                role="tablist"
                aria-label="Request detail tabs"
              >
                <button
                  type="button"
                  role="tab"
                  aria-selected={detailTab === "request"}
                  className={detailTab === "request" ? "is-active" : ""}
                  onClick={() => setDetailTab("request")}
                >
                  Request
                </button>
                <button
                  type="button"
                  role="tab"
                  aria-selected={detailTab === "response"}
                  className={detailTab === "response" ? "is-active" : ""}
                  onClick={() => setDetailTab("response")}
                >
                  Response
                </button>
              </div>
              {detailTab === "request" ? (
                <div role="tabpanel" aria-label="Request details">
                  <HeaderTable
                    title="Request headers"
                    headers={selectedDetail.requestHeaders}
                  />
                  <Payload
                    title="Request body"
                    value={selectedDetail.requestBodyPreview}
                    truncated={selectedDetail.requestTruncated}
                    bytes={selectedDetail.requestBytes}
                  />
                </div>
              ) : (
                <div role="tabpanel" aria-label="Response details">
                  <HeaderTable
                    title="Response headers"
                    headers={selectedDetail.responseHeaders}
                  />
                  <Payload
                    title="Response body"
                    value={selectedDetail.responseBodyPreview}
                    truncated={selectedDetail.responseTruncated}
                    bytes={selectedDetail.responseBytes}
                  />
                  {selectedDetail.sseEvents?.length ? (
                    <section className="detail-section">
                      <div className="section-heading">
                        <h3>SSE events</h3>
                        <span>{selectedDetail.sseEvents.length} parsed</span>
                      </div>
                      <div className="sse-list">
                        {selectedDetail.sseEvents.map((event, index) => (
                          <div
                            className="sse-event"
                            key={`${event.receivedAt}-${index}`}
                          >
                            <span className="event-index">
                              {String(index + 1).padStart(2, "0")}
                            </span>
                            <div>
                              <strong>{event.event || "message"}</strong>
                              {event.id && <code>id: {event.id}</code>}
                              <pre>{event.data}</pre>
                            </div>
                          </div>
                        ))}
                      </div>
                    </section>
                  ) : null}
                </div>
              )}
            </>
          ) : (
            <div className="detail-empty">
              <div className="empty-icon">
                <Icon name="search" size={22} />
              </div>
              <h2>
                {selected ? "Loading request details" : "Select a request"}
              </h2>
              <p>
                {selected
                  ? "Loading headers and payloads…"
                  : "Choose a captured request to inspect its headers and payloads."}
              </p>
            </div>
          )}
        </article>
      </section>
      <footer className="footer-note">
        <span>
          <i className="footer-dot" />
          Capture stays in memory
        </span>
        <span>
          Payload previews capped at 256 KB · sensitive headers redacted
        </span>
      </footer>
    </main>
  );
}
