import { useState } from 'react';
import { Icon } from '../../shared/Icon.jsx';

export function CopyButton({ value, label = 'Copy' }) {
  const [message, setMessage] = useState('');
  async function copy() {
    try { await navigator.clipboard.writeText(value); setMessage('Copied'); }
    catch { setMessage('Copy unavailable'); }
  }
  return <button type="button" className="proxy-action" onClick={copy}><Icon name={message === 'Copied' ? 'check' : 'copy'} size={14} /><span aria-live="polite">{message || label}</span></button>;
}
export function Code({ value }) {
  const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
  const tokens = text.split(/("(?:\\.|[^"\\])*"\s*:|"(?:\\.|[^"\\])*"|\b(?:true|false|null|\d+)\b)/g);
  return <pre className="llm-code" tabIndex={0}><code>{tokens.map((token, i) => <span key={i} className={token.endsWith(':') ? 'is-key' : token.startsWith('"') ? 'is-string' : /^(true|false|null|\d+)$/.test(token) ? 'is-value' : undefined}>{token}</span>)}</code></pre>;
}
function Field({ name, children }) {
  return <div className="llm-field"><dt>{name}</dt><dd>{children}</dd></div>;
}
function Content({ content, source }) {
  if (typeof content === 'string') return <p>{content}</p>;
  return (content || []).map((block, i) => {
    if (block.type === 'text' || block.type === 'input_text' || block.type === 'output_text') return <p key={i}>{block.text}</p>;
    if (block.type === 'tool_use') return <ToolCall key={i} name={block.name} id={block.id} args={block.input} source={`${source}.content[${i}]`} />;
    if (block.type === 'tool_result') return <div key={i} className="llm-tool-result"><div><Icon name="check" size={14} /><strong>Tool result</strong><code>{block.tool_use_id}</code></div><Content content={block.content} source={source} /></div>;
    if (block.type === 'input_image' || block.type === 'image' || block.type === 'image_url') return <div key={i} className="llm-attachment"><Icon name="file" size={16} /><span>Image attachment · data omitted</span></div>;
    return <Code key={i} value={block} />;
  });
}
function ToolCall({ name, id, args, source }) {
  let parsed = args;
  if (typeof args === 'string') { try { parsed = JSON.parse(args); } catch { /* Preserve malformed arguments for inspection. */ } }
  return <details className="llm-tool-call" open><summary><Icon name="tool" size={14} /><strong>{name}</strong><code>{id}</code></summary><div className="llm-tool-content"><span className="llm-source">{source}</span><Code value={parsed} /></div></details>;
}
function RequestContent({ capture }) {
  const request = capture.request;
  const items = request.input || request.messages;
  const system = request.system || items.filter((item) => item.role === 'system').map((item) => item.content).join('\n\n');
  const effort = request.reasoning?.effort || request.output_config?.effort;
  return <>
    <div className="llm-section-heading"><h3>Generation settings</h3><span>Sent to the provider</span></div>
    <dl className="llm-fields"><Field name="model"><code>{request.model}</code></Field><Field name={capture.limit}><code>{request[capture.limit].toLocaleString()}</code></Field><Field name="stream"><code>{request.stream === true ? 'true' : 'omitted · JSON response'}</code></Field>{effort && <Field name={capture.effort}><code>{effort}</code></Field>}{request.store === false && <Field name="store"><code>false</code></Field>}{request.stream_options && <Field name="include_usage"><code>true</code></Field>}</dl>
    <div className="llm-section-heading"><h3>Conversation</h3><span>{capture.input} · {items.length} wire items</span></div>
    <details className="llm-disclosure"><summary><strong>System instructions</strong><span>{request.system ? 'system' : `${capture.input}[0]`}</span></summary><p>{system}</p></details>
    <div className="llm-transcript">{items.map((item, index) => {
      if (item.role === 'system') return null;
      const source = `${capture.input}[${index}]`;
      if (item.type === 'function_call') return <ToolCall key={source} name={item.name} id={item.call_id} args={item.arguments} source={source} />;
      const tool = item.type === 'function_call_output' || item.role === 'tool';
      return <article className="llm-message" key={source}><header><strong>{tool ? 'Tool result' : item.role === 'assistant' ? 'Assistant' : 'User'}</strong><code>{source}</code></header>{tool && <a className="llm-call-link" href={`#${capture.id}-tool-call`} onClick={(event) => { event.preventDefault(); document.getElementById(`${capture.id}-tool-call`)?.scrollIntoView({ block: 'nearest' }); }}>{item.call_id || item.tool_call_id}</a>}<Content content={item.content || item.output} source={source} />{item.tool_calls?.map((call) => <ToolCall key={call.id} name={call.function.name} id={call.id} args={call.function.arguments} source={`${source}.tool_calls`} />)}</article>;
    })}</div>
    <div className="llm-section-heading"><h3>Available tools</h3><span>{request.tools.length} function</span></div>
    {request.tools.map((tool) => { const definition = tool.function || tool; return <details className="llm-disclosure" key={definition.name} id={`${capture.id}-tool-call`}><summary><Icon name="tool" size={14} /><strong>{definition.name}</strong><span>{tool.input_schema ? 'input_schema' : 'parameters'}</span></summary><p>{definition.description}</p><Code value={definition.parameters || definition.input_schema} /></details>; })}
  </>;
}
function ResponseContent({ capture, scenario }) {
  const pending = scenario === 'streaming';
  if (scenario === 'failed') return <div className="llm-error"><Icon name="warning" size={20} /><div><h3>Rate limit exceeded</h3><p>The provider returned HTTP 429. Retry after 30 seconds.</p><code>{capture.requestId}</code><Code value={JSON.parse(capture.rawResponse)} /></div></div>;
  if (scenario === 'truncated' || scenario === 'malformed') return <><div className="llm-notice"><Icon name="warning" size={16} /><div><strong>{scenario === 'truncated' ? 'Capture is incomplete' : 'Response could not be parsed'}</strong><p>{scenario === 'truncated' ? 'The body preview reached its capture limit. Only the captured prefix is available; completion and usage cannot be confirmed.' : 'The body is not valid JSON. Inspect the original payload and response headers.'}</p></div></div><Code value={capture.rawResponse} /></>;
  const chunks = capture.events.map((event) => event.value).filter((value) => typeof value !== 'string');
  const message = capture.mode === 'JSON' && !pending ? JSON.parse(capture.rawResponse) : null;
  const terminal = chunks.find((event) => event.type === 'response.completed')?.response;
  const text = message?.content.filter((part) => part.type === 'text').map((part) => part.text).join('\n') || chunks.map((event) => event.type === 'response.output_text.delta' ? event.delta : event.choices?.[0]?.delta?.content || '').join('');
  const usage = message?.usage || terminal?.usage || chunks.find((event) => event.usage)?.usage;
  const reason = message?.stop_reason || terminal?.status || chunks.find((event) => event.choices?.[0]?.finish_reason)?.choices[0].finish_reason;
  return <>
    <div className="llm-section-heading"><h3>Assistant output</h3><span>{pending ? capture.mode === 'SSE' ? 'Partial · stream open' : 'Waiting for JSON body' : reason}</span></div>
    <article className="llm-output">{text ? <p>{text}</p> : <div className="llm-skeleton" aria-label="Waiting for response"><i /><i /><i /></div>}</article>
    {usage ? <><div className="llm-section-heading"><h3>Token usage</h3><span>Reported by the provider</span></div><dl className="llm-fields"><Field name={capture.endpointId === 'chat' ? 'prompt_tokens' : 'input_tokens'}><code>{(usage.input_tokens ?? usage.prompt_tokens).toLocaleString()}</code></Field><Field name={capture.endpointId === 'chat' ? 'completion_tokens' : 'output_tokens'}><code>{(usage.output_tokens ?? usage.completion_tokens).toLocaleString()}</code></Field><Field name={capture.endpointId === 'messages' ? 'cache_read_input_tokens' : 'cached_tokens'}><code>{(usage.input_tokens_details?.cached_tokens ?? usage.prompt_tokens_details?.cached_tokens ?? usage.cache_read_input_tokens ?? 0).toLocaleString()}</code></Field>{usage.output_tokens_details && <Field name="reasoning_tokens"><code>{usage.output_tokens_details.reasoning_tokens}</code></Field>}</dl></> : <p className="llm-muted">Token usage is not available until the provider reports it.</p>}
    <details className="llm-disclosure"><summary><strong>Response identity</strong><code>{message?.id || terminal?.id || chunks[0]?.id || chunks[0]?.response?.id}</code></summary><dl className="llm-fields"><Field name="request ID"><code>{capture.requestId}</code></Field><Field name="finish reason"><code>{reason || 'Pending'}</code></Field></dl></details>
  </>;
}
function Events({ capture }) {
  return <><div className="llm-section-heading"><h3>Stream events</h3><span>{capture.events.length} captured</span></div><div className="llm-events">{capture.events.map((event, i) => <details key={i}><summary><code className="llm-event-index">{String(i + 1).padStart(2, '0')}</code><strong>{event.name}</strong><code>{event.at}</code></summary><Code value={event.value} /></details>)}</div></>;
}
function Headers({ capture, side }) {
  return <><div className="llm-section-heading"><h3>{side === 'request' ? 'Request' : 'Response'} headers</h3><span>Credential values redacted</span></div><dl className="llm-headers">{Object.entries(capture[`${side}Headers`]).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{value}</dd></div>)}</dl></>;
}
export function Inspector({ capture, scenario }) {
  const [side, setSide] = useState(scenario === 'failed' ? 'response' : 'request');
  const [view, setView] = useState('readable');
  const body = side === 'request' ? JSON.stringify(capture.request, null, 2) : capture.rawResponse;
  const views = ['readable', 'raw', 'headers', ...(side === 'response' && capture.mode === 'SSE' && !['failed', 'truncated', 'malformed'].includes(scenario) ? ['events'] : [])];
  return <article className="proxy-detail-column">
    <header className="llm-detail-heading"><div><div className="llm-detail-title"><h2>{capture.label}</h2><span className={`llm-status ${scenario === 'failed' ? 'is-danger' : ''}`}>{capture.status} · {capture.state}</span></div><code>POST https://{capture.host}{capture.path}</code><p><code>{capture.model}</code><span>{capture.duration}</span><span>{capture.mode === 'SSE' ? 'Server-sent events' : 'JSON response'}</span></p></div><CopyButton value={`https://${capture.host}${capture.path}`} label="Copy URL" /></header>
    <div className="llm-view-toolbar"><div className="llm-side-switch" aria-label="Payload direction">{['request', 'response'].map((item) => <button type="button" aria-pressed={side === item} key={item} onClick={() => { setSide(item); setView('readable'); }}>{item === 'request' ? 'Request' : 'Response'}</button>)}</div><div className="llm-view-switch" aria-label="Payload view">{views.map((item) => <button type="button" aria-pressed={view === item} key={item} onClick={() => setView(item)}>{item === 'readable' ? 'Readable' : item === 'raw' ? 'Raw' : item === 'headers' ? 'Headers' : `Events (${capture.events.length})`}</button>)}</div></div>
    <div className="llm-detail-body">{scenario === 'loading' ? <div className="llm-loading" role="status"><span>Loading captured payload…</span><div className="llm-skeleton"><i /><i /><i /></div></div> : view === 'headers' ? <Headers capture={capture} side={side} /> : view === 'raw' ? <><div className="llm-section-heading"><h3>{side === 'request' ? 'Request JSON' : capture.mode === 'SSE' && scenario !== 'failed' ? 'Raw event stream' : 'Response body'}</h3><CopyButton value={body} label="Copy body" /></div><Code value={body || 'No response body received yet.'} /></> : view === 'events' ? <Events capture={capture} /> : side === 'request' ? <RequestContent capture={capture} /> : <ResponseContent capture={capture} scenario={scenario} />}</div>
  </article>;
}
