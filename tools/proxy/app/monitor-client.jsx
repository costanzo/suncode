"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import {
  ProviderEmpty,
  ProviderInspector,
  getProviderSummary,
  formatBytes,
  formatDuration,
  formatTime,
  Icon,
  statusTone,
} from "./provider-inspector.jsx";

const endpointOptions = [
  ["all", "All endpoints"],
  ["responses", "Responses"],
  ["chat", "Chat Completions"],
  ["messages", "Messages"],
];

function connectionLabel(connected, paused) {
  if (paused) return "Updates paused";
  return connected ? "Monitor connected" : "Connecting";
}

export default function ProxyToolClient() {
  const [requests, setRequests] = useState([]);
  const [selectedId, setSelectedId] = useState(null);
  const [selectedDetail, setSelectedDetail] = useState(null);
  const [query, setQuery] = useState("");
  const [endpoint, setEndpoint] = useState("all");
  const [state, setState] = useState("all");
  const [paused, setPaused] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const [theme, setTheme] = useState("dark");
  const [connected, setConnected] = useState(false);
  const [listener, setListener] = useState(null);
  const [captureLimit, setCaptureLimit] = useState(null);
  const [detailError, setDetailError] = useState("");
  const [retryVersion, setRetryVersion] = useState(0);
  const [clearError, setClearError] = useState("");
  const [clearing, setClearing] = useState(false);
  const [loadingOlder, setLoadingOlder] = useState(false);
  const [total, setTotal] = useState(0);
  const [activeCount, setActiveCount] = useState(0);
  const [streamCount, setStreamCount] = useState(0);
  const syncCursor = useRef(null);
  const beforeCursor = useRef(null);

  useEffect(() => {
    const controller = new AbortController();
    fetch("/api/health", { signal: controller.signal, cache: "no-store" })
      .then((response) => response.json()).then((health) => setListener(`127.0.0.1:${health.proxyPort}`)).catch(() => {});
    return () => controller.abort();
  }, []);

  useEffect(() => {
    if (paused) return;
    const controller = new AbortController();
    let syncing = false, refreshTimer;
    syncCursor.current = null;
    beforeCursor.current = null;
    setRequests([]);
    const params = new URLSearchParams({ limit: "50" });
    if (query) params.set("q", query);
    if (endpoint !== "all") params.set("endpoint", endpoint);
    if (state !== "all") params.set("state", state);
    const syncRequests = async () => {
      if (syncing || controller.signal.aborted) return;
      syncing = true;
      try {
        const cursor = syncCursor.current;
        const requestParams = new URLSearchParams(params);
        if (cursor) requestParams.set("since", cursor);
        const response = await fetch(`/api/requests?${requestParams}`, { signal: controller.signal, cache: "no-store" });
        if (!response.ok) throw new Error("Capture list unavailable");
        const payload = await response.json();
        if (controller.signal.aborted) return;
        if (payload.reset || !cursor) {
          setRequests(payload.requests || []);
          beforeCursor.current = payload.nextBefore;
        } else {
          setRequests((current) => {
            const updates = new Map((payload.requests || []).map((item) => [item.id, item]));
            const removed = new Set(payload.removed || []);
            return [...current.filter((item) => !removed.has(item.id)).map((item) => updates.get(item.id) || item), ...[...updates.values()].filter((item) => !current.some((existing) => existing.id === item.id))].sort((a, b) => new Date(b.startedAt) - new Date(a.startedAt));
          });
        }
        syncCursor.current = payload.cursor;
        setTotal(payload.total ?? 0);
        setActiveCount(payload.activeCount ?? 0);
        setStreamCount(payload.streamCount ?? 0);
        setCaptureLimit(payload.captureLimit);
        setConnected(true);
      } catch { if (!controller.signal.aborted) setConnected(false); }
      finally { syncing = false; }
    };
    const scheduleRefresh = () => {
      if (!refreshTimer) refreshTimer = window.setTimeout(() => { refreshTimer = null; syncRequests(); }, 120);
    };
    syncRequests();
    const poll = window.setInterval(syncRequests, 2000);
    const source = typeof EventSource === "undefined" ? null : new EventSource("/api/events");
    source?.addEventListener("open", () => setConnected(true));
    source?.addEventListener("error", () => setConnected(false));
    source?.addEventListener("snapshot", scheduleRefresh);
    source?.addEventListener("update", scheduleRefresh);
    source?.addEventListener("clear", () => { syncCursor.current = null; beforeCursor.current = null; setRequests([]); setTotal(0); setActiveCount(0); setStreamCount(0); setSelectedId(null); setSelectedDetail(null); });
    return () => { controller.abort(); window.clearInterval(poll); window.clearTimeout(refreshTimer); source?.close(); };
  }, [paused, query, endpoint, state]);

  useEffect(() => {
    if (!selectedId || paused) return;
    const controller = new AbortController();
    let timer;
    async function load() {
      let shouldPoll = false;
      try {
        const response = await fetch(`/api/requests/${encodeURIComponent(selectedId)}`, { signal: controller.signal, cache: "no-store" });
        if (!response.ok) throw new Error(response.status === 404 ? "This capture is no longer in memory. Select another call." : "Could not load this capture. Retry or select another call.");
        const detail = await response.json();
        shouldPoll = ["forwarding", "streaming"].includes(detail.state);
        if (!controller.signal.aborted) { setSelectedDetail(detail); setDetailError(""); }
      } catch (error) { if (!controller.signal.aborted) setDetailError(error.message); }
      finally { if (!controller.signal.aborted && !paused && shouldPoll) timer = window.setTimeout(load, 800); }
    }
    load();
    return () => { controller.abort(); window.clearTimeout(timer); };
  }, [selectedId, paused, retryVersion]);

  const filtered = useMemo(() => requests.filter((request) => {
    const meta = getProviderSummary(request);
    const text = `${meta.label} ${meta.path} ${meta.model} ${request.method} ${request.url || ""} ${request.target || ""} ${request.id} ${request.providerRequestId || ""}`.toLowerCase();
    return (endpoint === "all" || meta.provider?.id === endpoint) && (state === "all" || request.state === state) && (!query || text.includes(query.toLowerCase()));
  }), [requests, query, endpoint, state]);

  const selected = requests.find((request) => request.id === selectedId) || null;

  function selectRequest(request) {
    if (selectedId === request.id) { setRetryVersion((value) => value + 1); return; }
    setSelectedId(request.id);
    setSelectedDetail(null);
    setDetailError("");
  }

  useEffect(() => {
    if (!selectedId && filtered[0]) selectRequest(filtered[0]);
  }, [filtered, selectedId]);

  async function clearAll() {
    setClearing(true);
    setClearError("");
    try {
      const response = await fetch("/api/requests", { method: "DELETE" });
      if (!response.ok) throw new Error();
      setRequests([]);
      setSelectedId(null);
      setSelectedDetail(null);
      setDetailError("");
      setConfirmClear(false);
    } catch { setClearError("Could not clear captures. Check the monitor connection and retry."); }
    finally { setClearing(false); }
  }

  async function loadOlder() {
    if (loadingOlder || !beforeCursor.current) return;
    setLoadingOlder(true);
    const params = new URLSearchParams({ limit: "50", before: beforeCursor.current });
    if (query) params.set("q", query);
    if (endpoint !== "all") params.set("endpoint", endpoint);
    if (state !== "all") params.set("state", state);
    try {
      const response = await fetch(`/api/requests?${params}`, { cache: "no-store" });
      if (!response.ok) throw new Error();
      const payload = await response.json();
      setRequests((current) => {
        const existing = new Set(current.map((item) => item.id));
        return [...current, ...(payload.requests || []).filter((item) => !existing.has(item.id))];
      });
      beforeCursor.current = payload.nextBefore;
    } catch { setConnected(false); }
    finally { setLoadingOlder(false); }
  }

  const resetFilters = () => { setQuery(""); setEndpoint("all"); setState("all"); };

  return <main className={`proxy-shell theme-${theme}`}>
    <header className="proxy-specimen-topbar">
      <div className="proxy-specimen-brand"><Icon name="activity" size={20} /><strong>Proxy Tool</strong><span className="llm-local-address">{listener || "Listener unavailable"}</span></div>
      <div className="proxy-specimen-actions"><span className={`proxy-live-pill ${paused ? "is-paused" : connected ? "" : "is-connecting"}`}><i />{connectionLabel(connected, paused)}</span><button type="button" className="proxy-action" aria-pressed={paused} onClick={() => setPaused(!paused)}><Icon name={paused ? "play" : "pause"} size={14} />{paused ? "Resume" : "Pause"}</button><button type="button" className="proxy-action" aria-label="Toggle theme" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>{theme === "dark" ? "Light" : "Dark"}</button><button type="button" className="proxy-action" disabled={!requests.length || clearing} onClick={() => setConfirmClear(!confirmClear)}><Icon name="close" size={14} />Clear</button></div>
    </header>
    {confirmClear && <div className="llm-clear-confirm" role="alert"><span>Clear all captured calls?</span><button type="button" className="proxy-action is-danger" disabled={clearing} onClick={clearAll}>{clearing ? "Clearing…" : "Clear calls"}</button><button type="button" className="proxy-action" disabled={clearing} onClick={() => setConfirmClear(false)}>Keep calls</button>{clearError && <p role="alert">{clearError}</p>}</div>}
    <div className="proxy-filter-bar"><label className="llm-search"><Icon name="search" size={16} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search endpoint, model, or request ID" aria-label="Search captured calls" /></label><select value={endpoint} onChange={(event) => setEndpoint(event.target.value)} aria-label="Filter endpoint">{endpointOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select><select value={state} onChange={(event) => setState(event.target.value)} aria-label="Filter state"><option value="all">All states</option><option value="forwarding">Forwarding</option><option value="streaming">Streaming</option><option value="completed">Completed</option><option value="failed">Failed</option><option value="tunnel">Tunnel</option></select><span className="llm-filter-count">{total || filtered.length} calls · {activeCount} active · {streamCount} SSE</span></div>
    <section className="proxy-specimen-body">
      <aside className="proxy-request-column" aria-label="Captured provider calls"><div className="proxy-column-heading"><strong>Captured calls</strong><span>Newest first</span></div>{filtered.length ? <div className="proxy-request-list">{filtered.map((request) => { const meta = getProviderSummary(request); return <button type="button" className={`proxy-request-row ${selected?.id === request.id ? "is-selected" : ""}`} aria-pressed={selected?.id === request.id} key={request.id} onClick={() => selectRequest(request)}><div className="llm-row-heading"><strong>{meta.label}</strong><span className={`is-${statusTone(request.status)}`}>{request.status || "Pending"}</span></div><code>{meta.path}</code><div className="llm-row-model"><span>{meta.model || "Model unavailable"}</span><span>{meta.mode}</span></div><div className="proxy-row-meta"><span>{formatTime(request.startedAt)}</span><span>{formatDuration(request.durationMs)}</span><span>{request.state}</span></div></button>; })}{beforeCursor.current && <button type="button" className="proxy-load-more" onClick={loadOlder} disabled={loadingOlder}>{loadingOlder ? "Loading earlier calls…" : "Load earlier calls"}</button>}</div> : <ProviderEmpty filtered={Boolean(query || endpoint !== "all" || state !== "all")} onReset={resetFilters} hasCapture={requests.length > 0} listener={listener} />}</aside>
      {detailError ? <div className="llm-detail-empty" role="alert"><h3>Capture unavailable</h3><p>{detailError}</p><button className="proxy-action" onClick={() => setRetryVersion((value) => value + 1)}>Retry</button></div> : selectedDetail ? <ProviderInspector key={selectedDetail.id} detail={selectedDetail} /> : <div className="llm-detail-empty">{selectedId ? <div className="llm-loading" role="status"><span>Loading captured payload…</span><div className="llm-skeleton"><i /><i /><i /></div></div> : <ProviderEmpty hasCapture={requests.length > 0} filtered={false} listener={listener} />}</div>}
    </section>
    <footer className="proxy-specimen-footer"><span><Icon name="lock" size={12} />Credential headers redacted</span><span>Capture stays in memory · previews capped at {captureLimit == null ? "configured limit" : formatBytes(captureLimit)}</span></footer>
  </main>;
}
