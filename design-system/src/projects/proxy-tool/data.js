export const systemPrompt = 'You are SunCode, a coding agent. Inspect the project before proposing changes. Use tools to read files and preserve the user’s work.';
export const question = 'Find the LLM provider routes and explain how tool results are sent back to the model.';
export const sampleAnswer = 'The provider layer has three wire adapters. Responses sends a function_call_output item; Chat Completions uses a tool message; Messages wraps the result in a user message with a tool_result block.';
export const toolCallId = 'call_read_01';
export const toolArguments = { path: 'agent/crates/llm/src/normalize.rs' };
export const toolResult = 'Tool results preserve the original call ID. Image-bearing results are encoded as multimodal content.';
export const toolSchema = { type: 'object', properties: { path: { type: 'string', description: 'Project-relative file path' } }, required: ['path'], additionalProperties: false };
export const endpoints = [
  { id: 'responses', label: 'Responses', path: '/v1/responses', host: 'api.openai.com', model: 'gpt-5', mode: 'SSE', input: 'input', limit: 'max_output_tokens', effort: 'reasoning.effort' },
  { id: 'chat', label: 'Chat Completions', path: '/v1/chat/completions', host: 'api.hunyuan.cloud.tencent.com', model: 'hy4-preview', mode: 'SSE', input: 'messages', limit: 'max_tokens' },
  { id: 'messages', label: 'Messages', path: '/v1/messages', host: 'api.anthropic.com', model: 'claude-example', mode: 'JSON', input: 'messages', limit: 'max_tokens', effort: 'output_config.effort' },
];
export const scenarios = [['complete', 'Completed'], ['streaming', 'In progress'], ['failed', 'Provider error'], ['truncated', 'Truncated capture'], ['malformed', 'Unparseable body'], ['loading', 'Loading detail'], ['empty', 'No traffic']];
const assistant = 'I’ll read the transcript serializer.';
const call = { id: toolCallId, type: 'function', function: { name: 'read', arguments: JSON.stringify(toolArguments) } };
const requests = {
  responses: {
    model: 'gpt-5', input: [{ role: 'system', content: systemPrompt }, { role: 'user', content: question }, { role: 'assistant', content: [{ type: 'output_text', text: assistant }] }, { type: 'function_call', call_id: toolCallId, name: 'read', arguments: JSON.stringify(toolArguments) }, { type: 'function_call_output', call_id: toolCallId, output: toolResult }],
    tools: [{ type: 'function', name: 'read', description: 'Read a project file', parameters: toolSchema, strict: false }], stream: true, store: false, reasoning: { effort: 'high' }, max_output_tokens: 4096,
  },
  chat: {
    model: 'hy4-preview', messages: [{ role: 'system', content: systemPrompt }, { role: 'user', content: question }, { role: 'assistant', content: assistant, tool_calls: [call] }, { role: 'tool', tool_call_id: toolCallId, content: toolResult }],
    tools: [{ type: 'function', function: { name: 'read', description: 'Read a project file', parameters: toolSchema } }], stream: true, stream_options: { include_usage: true }, max_tokens: 4096,
  },
  messages: {
    model: 'claude-example', max_tokens: 4096, system: systemPrompt, messages: [{ role: 'user', content: [{ type: 'text', text: question }] }, { role: 'assistant', content: [{ type: 'text', text: assistant }, { type: 'tool_use', id: toolCallId, name: 'read', input: toolArguments }] }, { role: 'user', content: [{ type: 'tool_result', tool_use_id: toolCallId, content: [{ type: 'text', text: toolResult }] }] }],
    tools: [{ name: 'read', description: 'Read a project file', input_schema: toolSchema }], output_config: { effort: 'high' },
  },
};
const usage = { input_tokens: 1842, output_tokens: 216, total_tokens: 2058, input_tokens_details: { cached_tokens: 1024 }, output_tokens_details: { reasoning_tokens: 64 } };
const response = { id: 'resp_demo_01', object: 'response', status: 'completed', output: [{ id: 'msg_demo_01', type: 'message', role: 'assistant', content: [{ type: 'output_text', text: sampleAnswer, annotations: [] }] }], usage };
const message = { id: 'msg_demo_01', type: 'message', role: 'assistant', model: 'claude-example', content: [{ type: 'text', text: sampleAnswer }], stop_reason: 'end_turn', stop_sequence: null, usage: { input_tokens: 818, output_tokens: 216, cache_read_input_tokens: 1024, cache_creation_input_tokens: 0 } };
function eventsFor(id, pending) {
  if (id === 'messages') return [];
  const chunks = id === 'responses' ? [
    { type: 'response.created', response: { id: response.id, status: 'in_progress' } },
    { type: 'response.output_text.delta', item_id: 'msg_demo_01', output_index: 0, content_index: 0, delta: sampleAnswer.slice(0, 98) },
    ...(!pending ? [{ type: 'response.output_text.delta', item_id: 'msg_demo_01', output_index: 0, content_index: 0, delta: sampleAnswer.slice(98) }, { type: 'response.completed', response }] : []),
  ] : [
    { id: 'chatcmpl_demo_01', object: 'chat.completion.chunk', choices: [{ index: 0, delta: { role: 'assistant', content: '' }, finish_reason: null }] },
    { id: 'chatcmpl_demo_01', object: 'chat.completion.chunk', choices: [{ index: 0, delta: { content: sampleAnswer.slice(0, 98) }, finish_reason: null }] },
    ...(!pending ? [{ id: 'chatcmpl_demo_01', object: 'chat.completion.chunk', choices: [{ index: 0, delta: { content: sampleAnswer.slice(98) }, finish_reason: null }] }, { id: 'chatcmpl_demo_01', object: 'chat.completion.chunk', choices: [{ index: 0, delta: {}, finish_reason: 'stop' }] }, { id: 'chatcmpl_demo_01', object: 'chat.completion.chunk', choices: [], usage: { prompt_tokens: 1842, completion_tokens: 216, total_tokens: 2058, prompt_tokens_details: { cached_tokens: 1024 } } }, '[DONE]'] : []),
  ];
  return chunks.map((value, index) => ({ name: typeof value === 'string' ? value : value.type || (value.usage ? 'usage' : 'chat.completion.chunk'), value, at: index ? `${(0.34 + index * 0.22).toFixed(2)} s` : '342 ms' }));
}
export function getCapture(id, scenario = 'complete') {
  const endpoint = endpoints.find((item) => item.id === id);
  const failed = scenario === 'failed';
  const pending = scenario === 'streaming';
  const events = failed ? [] : eventsFor(id, pending);
  const error = { type: 'error', error: { type: 'rate_limit_error', message: 'Too many requests. Try again after 30 seconds.' } };
  const raw = failed ? JSON.stringify(error, null, 2) : endpoint.mode === 'SSE' ? events.map(({ name, value }) => `${id === 'responses' ? `event: ${name}\n` : ''}data: ${typeof value === 'string' ? value : JSON.stringify(value)}\n\n`).join('') : pending ? '' : JSON.stringify(message, null, 2);
  return { ...endpoint, id: `${id}-demo-01`, endpointId: id, requestBytes: id === 'responses' ? '2.8 KB' : id === 'chat' ? '2.1 KB' : '2.4 KB', responseBytes: id === 'responses' ? '18.4 KB' : id === 'chat' ? '12.7 KB' : '8.1 KB', status: failed ? 429 : 200,
    state: failed ? 'Failed' : pending ? (endpoint.mode === 'SSE' ? 'Streaming' : 'Receiving') : 'Completed', duration: pending ? 'In progress' : '1.46 s', started: '14:32:08', requestId: `req_demo_${id}_01`,
    request: requests[id], events, rawResponse: scenario === 'malformed' ? '{ "error": upstream returned invalid JSON' : scenario === 'truncated' ? raw.slice(0, 420) : raw,
    requestHeaders: id === 'messages' ? { 'content-type': 'application/json', 'x-api-key': '[redacted]', 'anthropic-version': '2023-06-01' } : { 'content-type': 'application/json', authorization: '[redacted]' },
    responseHeaders: { 'content-type': failed || endpoint.mode === 'JSON' ? 'application/json' : 'text/event-stream', [id === 'messages' ? 'request-id' : 'x-request-id']: `req_demo_${id}_01`, ...(failed ? { 'retry-after': '30' } : {}) },
  };
}
