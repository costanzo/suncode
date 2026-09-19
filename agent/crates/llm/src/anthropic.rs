use crate::{
    ApiKeyResolver, BusinessError, Completion, CompletionFuture, CompletionRequest, ContentPart,
    LlmProvider, Message, ToolCall, Usage,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use suncode_common::{HttpProxyConfiguration, HttpProxyMode};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const ANTHROPIC_VERSION: &str = "2023-06-01";
const REQUEST_ID_HEADERS: &[&str] = &["request-id", "x-request-id"];
const DEFAULT_MAX_TOKENS: u64 = 4096;

#[derive(Clone)]
pub struct AnthropicProvider {
    provider_id: String,
    provider_label: String,
    endpoint: String,
    keys: Arc<dyn ApiKeyResolver>,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: Arc<RwLock<Option<PathBuf>>>,
    proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
}

impl AnthropicProvider {
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_network_configuration(
        provider_id: impl Into<String>,
        provider_label: impl Into<String>,
        endpoint: impl Into<String>,
        keys: Arc<dyn ApiKeyResolver>,
        verify_https_certificates: Arc<AtomicBool>,
        use_system_certificates: Arc<AtomicBool>,
        certificate_path: Arc<RwLock<Option<PathBuf>>>,
        proxy_configuration: Arc<RwLock<HttpProxyConfiguration>>,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            provider_label: provider_label.into(),
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            keys,
            verify_https_certificates,
            use_system_certificates,
            certificate_path,
            proxy_configuration,
        }
    }

    fn client(&self) -> Result<reqwest::Client, BusinessError> {
        let verify_certificates = self.verify_https_certificates.load(Ordering::SeqCst);
        let mut builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(!verify_certificates)
            .danger_accept_invalid_hostnames(!verify_certificates);
        if verify_certificates && !self.use_system_certificates.load(Ordering::SeqCst) {
            builder = builder.tls_built_in_root_certs(false);
            if let Some(path) = self
                .certificate_path
                .read()
                .ok()
                .and_then(|path| path.clone())
            {
                let bytes = std::fs::read(&path).map_err(|error| {
                    BusinessError::provider(
                        "certificate_unavailable",
                        format!("could not read certificate file: {error}"),
                        false,
                        None,
                    )
                })?;
                let certificate = reqwest::Certificate::from_pem(&bytes)
                    .or_else(|_| reqwest::Certificate::from_der(&bytes))
                    .map_err(|error| {
                        BusinessError::provider(
                            "certificate_invalid",
                            format!("invalid certificate file: {error}"),
                            false,
                            None,
                        )
                    })?;
                builder = builder.add_root_certificate(certificate);
            }
        }
        let proxy = self
            .proxy_configuration
            .read()
            .map(|configuration| configuration.clone())
            .unwrap_or_default();
        builder = apply_proxy(builder, &proxy).map_err(|message| {
            BusinessError::provider("provider_client_unavailable", message, false, None)
        })?;
        builder.build().map_err(|error| {
            BusinessError::provider(
                "provider_client_unavailable",
                error.to_string(),
                false,
                None,
            )
        })
    }

    async fn complete_inner(
        &self,
        request: CompletionRequest<'_>,
        cancellation: &CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> Result<Completion, BusinessError> {
        let key = self.keys.api_key(&self.provider_id).ok_or_else(|| {
            BusinessError::new(
                "provider_unconfigured",
                format!("{} API key is not configured", self.provider_label),
            )
        })?;
        let (system, messages) = anthropic_messages(request.messages)?;
        let tools = anthropic_tools(&request);
        let mut body = json!({
            "model": request.wire_model,
            "max_tokens": request.max_output_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            "messages": messages,
        });
        if !system.is_empty() {
            body["system"] = Value::String(system);
        }
        if !tools.is_empty() {
            body["tools"] = Value::Array(tools);
        }
        if let Some(effort) = request.reasoning_effort {
            body["output_config"] = json!({"effort": effort});
        }

        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = self.client()?.post(format!("{}/messages", self.endpoint))
                .header("x-api-key", key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .json(&body)
                .send() => value.map_err(|error| BusinessError::provider(
                    "transient",
                    format!("{} request failed: {error}", self.provider_label),
                    true,
                    None,
                ))?,
        };
        let provider_request_id = REQUEST_ID_HEADERS.iter().find_map(|name| {
            response
                .headers()
                .get(*name)
                .and_then(|value| value.to_str().ok())
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        });
        if !response.status().is_success() {
            let status = response.status();
            let message = response
                .json::<Value>()
                .await
                .ok()
                .and_then(|value| {
                    value
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| {
                    format!(
                        "{} request failed with status {status}",
                        self.provider_label
                    )
                });
            let code = if status.as_u16() == 401 {
                "authentication"
            } else if status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error() {
                "transient"
            } else {
                "invalid_request"
            };
            return Err(BusinessError::provider(
                code,
                message,
                status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error(),
                provider_request_id,
            ));
        }
        let value = tokio::select! {
            _ = cancellation.cancelled() => return Err(cancelled()),
            value = response.json::<Value>() => value.map_err(|error| BusinessError::provider(
                "provider_protocol",
                format!("{} response was invalid: {error}", self.provider_label),
                false,
                provider_request_id.clone(),
            ))?,
        };
        parse_completion(value, provider_request_id, &deltas)
    }
}

impl LlmProvider for AnthropicProvider {
    fn complete<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        cancellation: &'a CancellationToken,
        deltas: mpsc::UnboundedSender<String>,
    ) -> CompletionFuture<'a> {
        Box::pin(self.complete_inner(request, cancellation, deltas))
    }
}

fn anthropic_tools(request: &CompletionRequest<'_>) -> Vec<Value> {
    let mut tools = request
        .tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.parameters,
            })
        })
        .collect::<Vec<_>>();
    tools.extend(request.client_toolsets.iter().map(|toolset| {
        let mut entry = toolset
            .configuration
            .as_object()
            .cloned()
            .unwrap_or_default();
        entry.insert("type".into(), Value::String(toolset.type_name.clone()));
        Value::Object(entry)
    }));
    tools
}

fn anthropic_messages(messages: &[Message]) -> Result<(String, Vec<Value>), BusinessError> {
    let mut system = Vec::new();
    let toolsets = messages
        .iter()
        .flat_map(|message| message.tool_calls.iter())
        .filter_map(|call| {
            call.toolset_name
                .as_ref()
                .map(|name| (call.call_id.clone(), name.clone()))
        })
        .collect::<HashMap<_, _>>();
    let mut wire = Vec::new();
    for message in messages {
        if message.role == "system" {
            if message.content.iter().any(|part| part.kind != "text") {
                return Err(provider_protocol("Anthropic system messages must be text"));
            }
            system.push(message.text_content());
            continue;
        }
        let (role, blocks) = if message.role == "tool" {
            let tool_use_id = message
                .tool_call_id
                .as_deref()
                .ok_or_else(|| provider_protocol("tool result is missing its tool use ID"))?;
            let mut result = Map::from_iter([
                ("type".into(), Value::String("tool_result".into())),
                ("tool_use_id".into(), Value::String(tool_use_id.into())),
                (
                    "content".into(),
                    Value::Array(content_blocks(&message.content)?),
                ),
            ]);
            if let Some(toolset_name) = toolsets.get(tool_use_id) {
                result.insert("toolset_name".into(), Value::String(toolset_name.clone()));
            }
            ("user", vec![Value::Object(result)])
        } else {
            let role = match message.role.as_str() {
                "user" => "user",
                "assistant" => "assistant",
                _ => return Err(provider_protocol("Anthropic message role is unsupported")),
            };
            let mut blocks = content_blocks(&message.content)?;
            if role == "assistant" {
                blocks.extend(message.tool_calls.iter().map(|call| {
                    let mut block = json!({
                        "type": "tool_use",
                        "id": call.call_id,
                        "name": call.name,
                        "input": call.arguments,
                    });
                    if let Some(toolset_name) = &call.toolset_name {
                        block["toolset_name"] = Value::String(toolset_name.clone());
                    }
                    block
                }));
            }
            (role, blocks)
        };
        push_message(&mut wire, role, blocks);
    }
    Ok((system.join("\n\n"), wire))
}

fn push_message(messages: &mut Vec<Value>, role: &str, mut blocks: Vec<Value>) {
    if let Some(last) = messages.last_mut() {
        if last.get("role").and_then(Value::as_str) == Some(role) {
            if let Some(content) = last.get_mut("content").and_then(Value::as_array_mut) {
                content.append(&mut blocks);
                return;
            }
        }
    }
    messages.push(json!({"role": role, "content": blocks}));
}

fn content_blocks(parts: &[ContentPart]) -> Result<Vec<Value>, BusinessError> {
    parts
        .iter()
        .map(|part| match part.kind.as_str() {
            "text" => Ok(json!({"type": "text", "text": part.text})),
            "image_url" => {
                let (media_type, data) = parse_image_data_url(&part.text)?;
                Ok(json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": data,
                    }
                }))
            }
            _ => Err(provider_protocol("Anthropic content part is unsupported")),
        })
        .collect()
}

fn parse_image_data_url(value: &str) -> Result<(&str, &str), BusinessError> {
    let value = value
        .strip_prefix("data:")
        .ok_or_else(|| provider_protocol("Anthropic image content must be a data URL"))?;
    let (metadata, data) = value
        .split_once(',')
        .ok_or_else(|| provider_protocol("Anthropic image data URL is invalid"))?;
    let media_type = metadata
        .strip_suffix(";base64")
        .ok_or_else(|| provider_protocol("Anthropic image data URL must use base64"))?;
    if !matches!(
        media_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    ) {
        return Err(provider_protocol(
            "Anthropic image media type is unsupported",
        ));
    }
    STANDARD
        .decode(data)
        .map_err(|_| provider_protocol("Anthropic image data is invalid base64"))?;
    Ok((media_type, data))
}

fn parse_completion(
    value: Value,
    provider_request_id: Option<String>,
    deltas: &mpsc::UnboundedSender<String>,
) -> Result<Completion, BusinessError> {
    let content = value
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| provider_protocol("Anthropic response content is missing"))?;
    let mut text = Vec::new();
    let mut tool_calls = Vec::new();
    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                let value = block
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| provider_protocol("Anthropic text block is invalid"))?;
                text.push(value.to_string());
                let _ = deltas.send(value.to_string());
            }
            Some("tool_use") => tool_calls.push(ToolCall {
                call_id: required_string(block, "id")?,
                name: required_string(block, "name")?,
                arguments: block.get("input").cloned().unwrap_or_else(|| json!({})),
                toolset_name: block
                    .get("toolset_name")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            }),
            Some(_) => {}
            None => return Err(provider_protocol("Anthropic content block type is missing")),
        }
    }
    let input_tokens = value
        .pointer("/usage/input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = value
        .pointer("/usage/output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    Ok(Completion {
        text: text.join("\n"),
        tool_calls,
        finish_reason: value
            .get("stop_reason")
            .and_then(Value::as_str)
            .unwrap_or("end_turn")
            .to_string(),
        usage: Some(Usage {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
            cache_read_tokens: value
                .pointer("/usage/cache_read_input_tokens")
                .and_then(Value::as_u64),
            cache_miss_tokens: None,
            cache_write_tokens: value
                .pointer("/usage/cache_creation_input_tokens")
                .and_then(Value::as_u64),
            reasoning_tokens: None,
        }),
        provider_request_id,
        provider_response_id: value.get("id").and_then(Value::as_str).map(str::to_owned),
    })
}

fn required_string(value: &Value, key: &str) -> Result<String, BusinessError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| provider_protocol("Anthropic tool use block is invalid"))
}

fn apply_proxy(
    builder: reqwest::ClientBuilder,
    configuration: &HttpProxyConfiguration,
) -> Result<reqwest::ClientBuilder, String> {
    match configuration.mode {
        HttpProxyMode::NoProxy => Ok(builder.no_proxy()),
        HttpProxyMode::System => Ok(builder),
        HttpProxyMode::Custom => {
            let mut proxy = reqwest::Proxy::all(&configuration.url)
                .map_err(|error| format!("proxy configuration is invalid: {error}"))?;
            if !configuration.username.is_empty() {
                proxy = proxy.basic_auth(&configuration.username, &configuration.password);
            }
            proxy = proxy.no_proxy(reqwest::NoProxy::from_string(
                &configuration.no_proxy_value(),
            ));
            Ok(builder.no_proxy().proxy(proxy))
        }
    }
}

fn provider_protocol(message: &'static str) -> BusinessError {
    BusinessError::provider("provider_protocol", message, false, None)
}

fn cancelled() -> BusinessError {
    BusinessError::new("cancelled", "Turn was cancelled")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_computer_toolset_identity_and_image_results() {
        let assistant = Message {
            role: "assistant".into(),
            content: Vec::new(),
            tool_calls: vec![ToolCall {
                call_id: "toolu_1".into(),
                name: "screenshot".into(),
                arguments: json!({}),
                toolset_name: Some("computer".into()),
            }],
            tool_call_id: None,
        };
        let tool = Message {
            role: "tool".into(),
            content: vec![ContentPart {
                kind: "image_url".into(),
                text: "data:image/png;base64,iVBORw0KGgo=".into(),
            }],
            tool_calls: Vec::new(),
            tool_call_id: Some("toolu_1".into()),
        };
        let (_, messages) = anthropic_messages(&[assistant, tool]).unwrap();
        assert_eq!(messages[0]["content"][0]["toolset_name"], "computer");
        assert_eq!(messages[1]["content"][0]["toolset_name"], "computer");

        let (sender, _receiver) = mpsc::unbounded_channel();
        let completion = parse_completion(
            json!({
                "id":"msg_1",
                "content":[{
                    "type":"tool_use",
                    "id":"toolu_2",
                    "name":"left_click",
                    "toolset_name":"computer",
                    "input":{"coordinate":[10,20]}
                }],
                "stop_reason":"tool_use",
                "usage":{"input_tokens":10,"output_tokens":5}
            }),
            None,
            &sender,
        )
        .unwrap();
        assert_eq!(
            completion.tool_calls[0].toolset_name.as_deref(),
            Some("computer")
        );
    }
}
