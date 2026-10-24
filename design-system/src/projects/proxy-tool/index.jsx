import { useState } from 'react';
import { Icon } from '../../shared/Icon.jsx';
import { PageHeader } from '../../shared/PagePrimitives.jsx';
import { endpoints, getCapture, scenarios } from './data.js';
import { Inspector } from './Inspector.jsx';

function Empty({ filtered, onReset }) {
  return <div className="llm-empty"><Icon name={filtered ? 'search' : 'activity'} size={24} /><h3>{filtered ? 'No matching calls' : 'Waiting for traffic'}</h3><p>{filtered ? 'Try a different endpoint, model, or request ID.' : 'Send a provider request through the local proxy to inspect it here.'}</p>{filtered ? <button type="button" className="proxy-action" onClick={onReset}>Reset filters</button> : <code>HTTP_PROXY=http://127.0.0.1:8080</code>}</div>;
}
function ProxyWorkspace({ scenario }) {
  const [selected, setSelected] = useState('responses');
  const [query, setQuery] = useState('');
  const [protocol, setProtocol] = useState('all');
  const [paused, setPaused] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const capture = getCapture(selected, scenario);
  const empty = cleared || scenario === 'empty';
  const filtered = empty ? [] : endpoints.filter((endpoint) => (protocol === 'all' || endpoint.id === protocol) && `${endpoint.label} ${endpoint.path} ${endpoint.host} ${endpoint.model} req_demo_${endpoint.id}_01`.toLowerCase().includes(query.toLowerCase()));
  const reset = () => { setQuery(''); setProtocol('all'); };
  return <div className="proxy-workspace-specimen">
    <header className="proxy-specimen-topbar"><div className="proxy-specimen-brand"><Icon name="activity" size={20} /><strong>Proxy Tool</strong><span className="llm-local-address">127.0.0.1:8080</span></div><div className="proxy-specimen-actions"><span className={`proxy-live-pill ${paused ? 'is-paused' : ''}`}><i />{paused ? 'Updates paused' : 'Monitor connected'}</span><button type="button" className="proxy-action" aria-pressed={paused} onClick={() => setPaused(!paused)}><Icon name={paused ? 'play' : 'pause'} size={14} />{paused ? 'Resume' : 'Pause'}</button><button type="button" className="proxy-action" disabled={empty} onClick={() => setConfirmClear(!confirmClear)}><Icon name="close" size={14} />Clear</button></div></header>
    {confirmClear && <div className="llm-clear-confirm"><span>Clear the captured calls from this preview?</span><button type="button" className="proxy-action is-danger" onClick={() => { setCleared(true); setConfirmClear(false); }}>Clear calls</button><button type="button" className="proxy-action" onClick={() => setConfirmClear(false)}>Keep calls</button></div>}
    <div className="proxy-filter-bar"><label className="llm-search"><Icon name="search" size={16} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search endpoint, model, or request ID" aria-label="Search captured calls" /></label><select value={protocol} onChange={(event) => setProtocol(event.target.value)} aria-label="Filter endpoint"><option value="all">All endpoints</option>{endpoints.map((endpoint) => <option key={endpoint.id} value={endpoint.id}>{endpoint.label}</option>)}</select><span className="llm-filter-count">{filtered.length} calls</span></div>
    <div className="proxy-specimen-body"><aside className="proxy-request-column" aria-label="Captured provider calls"><div className="proxy-column-heading"><strong>Captured calls</strong><span>Newest first</span></div>{filtered.length ? filtered.map((endpoint, index) => {
      const item = getCapture(endpoint.id, scenario);
      return <button type="button" className={`proxy-request-row ${selected === endpoint.id ? 'is-selected' : ''}`} aria-pressed={selected === endpoint.id} key={endpoint.id} onClick={() => setSelected(endpoint.id)}><div className="llm-row-heading"><strong>{endpoint.label}</strong><span className={scenario === 'failed' ? 'is-danger' : ''}>{item.status}</span></div><code>{endpoint.path}</code><div className="llm-row-model"><span>{endpoint.model}</span><span>{endpoint.mode}</span></div><div className="proxy-row-meta"><span>14:32:0{8 - index * 2}</span><span>{item.duration}</span><span>{item.state}</span></div></button>;
    }) : <Empty filtered={!empty} onReset={reset} />}</aside>{empty ? <div className="llm-detail-empty"><Empty />{cleared && <button type="button" className="proxy-action" onClick={() => { setCleared(false); reset(); }}>Restore example calls</button>}</div> : <Inspector key={`${selected}-${scenario}`} capture={capture} scenario={scenario} />}</div>
    <footer className="proxy-specimen-footer"><span><Icon name="lock" size={12} />Credential headers redacted</span><span>Illustrative captures · local review only</span></footer>
  </div>;
}
export function ProxyToolPage() {
  const [scenario, setScenario] = useState('complete');
  return <><PageHeader title="Proxy Tool" description="Read the conversation. Inspect the wire." status="Design reference" tone="implemented" /><div className="llm-review-toolbar"><div><strong>LLM request inspector</strong><p>Three provider protocols, one readable debugging surface.</p></div><label>Preview state<select value={scenario} onChange={(event) => setScenario(event.target.value)}>{scenarios.map(([id, label]) => <option key={id} value={id}>{label}</option>)}</select></label></div><ProxyWorkspace key={scenario} scenario={scenario} /><details className="llm-spec-notes"><summary>Protocol & interaction notes</summary><ul><li>Responses: input items, flat function tools, call_id correlation, streamed response events.</li><li>Chat Completions: messages, nested function tools, tool_call_id correlation, delta chunks and final usage.</li><li>Messages: system outside messages, tool_use / tool_result blocks, x-api-key and anthropic-version; this adapter receives JSON.</li><li>Readable is the default. Raw preserves the captured payload; Headers and Events expose transport details. Truncated and malformed responses remain inspectable without inventing a completion.</li><li>Pause freezes monitor updates; it does not stop proxy forwarding. This specimen uses fixed illustrative captures and performs no provider requests.</li></ul></details></>;
}
