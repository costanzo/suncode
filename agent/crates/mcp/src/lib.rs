//! MCP client adapter. No rmcp type crosses this crate's public boundary.

use futures_util::future::BoxFuture;
use http::{HeaderName, HeaderValue};
use rmcp::{
    model::{
        CallToolRequestParams, ClientInfo, ContentBlock, Implementation, PaginatedRequestParams,
    },
    service::RunningService,
    transport::{
        streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
        TokioChildProcess,
    },
    ClientHandler, RoleClient, ServiceExt,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use suncode_common::BusinessError;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const MAX_TOOLS_PER_SERVER: usize = 128;
const MAX_RESULT_BYTES: usize = 1024 * 1024;
const MAX_SSE_EVENT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub verify_certificates: bool,
    pub use_system_certificates: bool,
    pub certificate_path: Option<PathBuf>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_certificates: true,
            use_system_certificates: true,
            certificate_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionConfig {
    Stdio {
        command: String,
        arguments: Vec<String>,
        working_directory: PathBuf,
        environment: BTreeMap<String, String>,
        startup_timeout: Duration,
        request_timeout: Duration,
    },
    StreamableHttp {
        url: String,
        headers: BTreeMap<String, String>,
        startup_timeout: Duration,
        request_timeout: Duration,
        tls: TlsConfig,
    },
}

impl ConnectionConfig {
    fn startup_timeout(&self) -> Duration {
        match self {
            Self::Stdio {
                startup_timeout, ..
            }
            | Self::StreamableHttp {
                startup_timeout, ..
            } => *startup_timeout,
        }
    }

    fn request_timeout(&self) -> Duration {
        match self {
            Self::Stdio {
                request_timeout, ..
            }
            | Self::StreamableHttp {
                request_timeout, ..
            } => *request_timeout,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub remote_name: String,
    pub title: Option<String>,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub content: Vec<String>,
    pub structured_content: Option<Value>,
    pub is_error: bool,
    pub truncated: bool,
}

pub trait Connection: Send + Sync {
    fn list_tools(&self) -> BoxFuture<'_, Result<Vec<ToolDefinition>, BusinessError>>;
    fn call_tool(
        &self,
        name: String,
        arguments: Value,
        cancellation: CancellationToken,
    ) -> BoxFuture<'_, Result<ToolResult, BusinessError>>;
    fn close(&self) -> BoxFuture<'_, ()>;
    fn is_closed(&self) -> bool;
}

pub struct Connected {
    pub connection: Arc<dyn Connection>,
    pub tool_list_changes: mpsc::UnboundedReceiver<()>,
}

#[derive(Clone)]
struct Handler {
    tool_list_changes: mpsc::UnboundedSender<()>,
}

impl ClientHandler for Handler {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("SunCode", env!("CARGO_PKG_VERSION")),
        )
    }

    async fn on_tool_list_changed(&self, _context: rmcp::service::NotificationContext<RoleClient>) {
        let _ = self.tool_list_changes.send(());
    }
}

struct RmcpConnection {
    service: Mutex<RunningService<RoleClient, Handler>>,
    request_timeout: Duration,
    secret_values: Vec<String>,
}

impl Connection for RmcpConnection {
    fn list_tools(&self) -> BoxFuture<'_, Result<Vec<ToolDefinition>, BusinessError>> {
        Box::pin(async move {
            let service = self.service.lock().await;
            let mut cursor = None;
            let mut tools = Vec::new();
            loop {
                let params = cursor
                    .take()
                    .map(|cursor| PaginatedRequestParams::default().with_cursor(Some(cursor)));
                let page = tokio::time::timeout(self.request_timeout, service.list_tools(params))
                    .await
                    .map_err(|_| mcp_error("mcp_request_timeout", "MCP tool discovery timed out"))?
                    .map_err(|error| {
                        mcp_service_error("MCP tool discovery failed", &error, &self.secret_values)
                    })?;
                for tool in page.tools {
                    if tools.len() >= MAX_TOOLS_PER_SERVER {
                        return Err(mcp_error(
                            "mcp_tool_limit_exceeded",
                            "MCP server exposes more than 128 tools",
                        ));
                    }
                    let input_schema = Value::Object((*tool.input_schema).clone());
                    if input_schema.get("type").and_then(Value::as_str) != Some("object") {
                        return Err(mcp_error(
                            "mcp_schema_invalid",
                            "MCP tool input schema must describe an object",
                        ));
                    }
                    tools.push(ToolDefinition {
                        remote_name: tool.name.into_owned(),
                        title: tool.title,
                        description: tool
                            .description
                            .map(|value| value.into_owned())
                            .unwrap_or_default(),
                        input_schema,
                    });
                }
                cursor = page.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }
            Ok(tools)
        })
    }

    fn call_tool(
        &self,
        name: String,
        arguments: Value,
        cancellation: CancellationToken,
    ) -> BoxFuture<'_, Result<ToolResult, BusinessError>> {
        Box::pin(async move {
            let arguments = arguments
                .as_object()
                .cloned()
                .ok_or_else(|| BusinessError::invalid("MCP tool arguments must be an object"))?;
            let service = self.service.lock().await;
            let request =
                service.call_tool(CallToolRequestParams::new(name).with_arguments(arguments));
            tokio::pin!(request);
            let result = tokio::select! {
                biased;
                _ = cancellation.cancelled() => {
                    return Err(mcp_error("cancelled", "MCP tool call was cancelled"));
                }
                result = tokio::time::timeout(self.request_timeout, &mut request) => {
                    result
                        .map_err(|_| mcp_error("mcp_request_timeout", "MCP tool call timed out"))?
                        .map_err(|error| {
                            mcp_service_error("MCP tool call failed", &error, &self.secret_values)
                        })?
                }
            };
            normalize_result(result)
        })
    }

    fn close(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            let mut service = self.service.lock().await;
            let _ = service.close_with_timeout(Duration::from_secs(3)).await;
        })
    }

    fn is_closed(&self) -> bool {
        self.service
            .try_lock()
            .map(|service| service.is_closed())
            .unwrap_or(false)
    }
}

pub async fn connect(config: ConnectionConfig) -> Result<Connected, BusinessError> {
    let startup_timeout = config.startup_timeout();
    let request_timeout = config.request_timeout();
    let secret_values = configured_secret_values(&config);
    let (changes_tx, changes_rx) = mpsc::unbounded_channel();
    let handler = Handler {
        tool_list_changes: changes_tx,
    };
    let service = match config {
        ConnectionConfig::Stdio {
            command,
            arguments,
            working_directory,
            environment,
            ..
        } => {
            let mut process = tokio::process::Command::new(command);
            process
                .args(arguments)
                .current_dir(&working_directory)
                .kill_on_drop(true)
                .env_clear()
                .envs(default_environment(&working_directory, &environment));
            let transport = TokioChildProcess::new(process)
                .map_err(|error| mcp_error("mcp_process_start_failed", error.to_string()))?;
            tokio::time::timeout(startup_timeout, handler.serve(transport))
                .await
                .map_err(|_| mcp_error("mcp_startup_timeout", "MCP server startup timed out"))?
                .map_err(|error| {
                    redact_business_error(
                        mcp_error("mcp_handshake_failed", error.to_string()),
                        &secret_values,
                    )
                })?
        }
        ConnectionConfig::StreamableHttp {
            url, headers, tls, ..
        } => {
            let headers = http_headers(headers)?;
            let client = http_client(&tls)?;
            let transport_config = StreamableHttpClientTransportConfig::with_uri(url)
                .max_concurrent_requests(1)
                .control_request_timeout(Duration::from_secs(5))
                .session_recovery_timeout(Duration::from_secs(5))
                .max_sse_event_size(MAX_SSE_EVENT_BYTES)
                .custom_headers(headers);
            let transport = StreamableHttpClientTransport::with_client(client, transport_config);
            tokio::time::timeout(startup_timeout, handler.serve(transport))
                .await
                .map_err(|_| mcp_error("mcp_startup_timeout", "MCP server startup timed out"))?
                .map_err(|error| {
                    redact_business_error(
                        mcp_error("mcp_handshake_failed", error.to_string()),
                        &secret_values,
                    )
                })?
        }
    };
    Ok(Connected {
        connection: Arc::new(RmcpConnection {
            service: Mutex::new(service),
            request_timeout,
            secret_values,
        }),
        tool_list_changes: changes_rx,
    })
}

fn default_environment(
    working_directory: &std::path::Path,
    configured: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    // GUI-launched hosts often have a sparse environment; rebuild only the
    // launch/runtime values MCP processes need instead of inheriting secrets.
    let mut values = BTreeMap::new();
    let host = |key: &str| std::env::var(key).ok().filter(|value| !value.is_empty());
    let insert_host = |values: &mut BTreeMap<String, String>, key: &str| {
        if let Some(value) = host(key) {
            values.insert(key.to_string(), value);
        }
    };

    #[cfg(windows)]
    {
        for key in [
            "SystemRoot",
            "WINDIR",
            "USERPROFILE",
            "HOMEDRIVE",
            "HOMEPATH",
            "APPDATA",
            "LOCALAPPDATA",
            "ComSpec",
            "PATHEXT",
            "USERNAME",
            "OS",
        ] {
            insert_host(&mut values, key);
        }

        let system_root = values
            .get("SystemRoot")
            .or_else(|| values.get("WINDIR"))
            .cloned()
            .unwrap_or_else(|| r"C:\Windows".to_string());
        values
            .entry("SystemRoot".into())
            .or_insert_with(|| system_root.clone());
        values
            .entry("WINDIR".into())
            .or_insert_with(|| system_root.clone());
        values
            .entry("OS".into())
            .or_insert_with(|| "Windows_NT".into());
        values
            .entry("ComSpec".into())
            .or_insert_with(|| format!("{system_root}\\System32\\cmd.exe"));
        values
            .entry("PATHEXT".into())
            .or_insert_with(|| ".COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC".into());

        if let Some(user_profile) = values.get("USERPROFILE").cloned() {
            values
                .entry("HOME".into())
                .or_insert_with(|| user_profile.clone());
            values
                .entry("APPDATA".into())
                .or_insert_with(|| format!("{user_profile}\\AppData\\Roaming"));
            values
                .entry("LOCALAPPDATA".into())
                .or_insert_with(|| format!("{user_profile}\\AppData\\Local"));
        }
        let temp = host("TEMP")
            .or_else(|| host("TMP"))
            .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
        values.entry("TEMP".into()).or_insert_with(|| temp.clone());
        values.entry("TMP".into()).or_insert(temp);

        let path = host("Path").or_else(|| host("PATH")).unwrap_or_else(|| {
            format!("{system_root}\\System32;{system_root};{system_root}\\System32\\Wbem")
        });
        values.entry("PATH".into()).or_insert(path);
    }

    #[cfg(not(windows))]
    {
        for key in [
            "HOME",
            "USER",
            "LOGNAME",
            "SHELL",
            "SSH_AUTH_SOCK",
            "GPG_AGENT_INFO",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "DBUS_SESSION_BUS_ADDRESS",
            "TERM_PROGRAM",
            "COLORTERM",
            "NVM_DIR",
            "NVM_BIN",
            "VOLTA_HOME",
            "PNPM_HOME",
            "ASDF_DATA_DIR",
            "MISE_DATA_DIR",
            "VIRTUAL_ENV",
            "CONDA_PREFIX",
            "LANG",
            "LC_ALL",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "__CF_USER_TEXT_ENCODING",
        ] {
            insert_host(&mut values, key);
        }

        let home = values.get("HOME").cloned();
        let mut paths = vec![
            "/opt/homebrew/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
            "/usr/sbin".to_string(),
            "/sbin".to_string(),
            "/snap/bin".to_string(),
        ];
        if let Some(home) = &home {
            paths.splice(
                0..0,
                [
                    format!("{home}/.local/bin"),
                    format!("{home}/.cargo/bin"),
                    format!("{home}/bin"),
                ],
            );
        }
        for key in ["NVM_BIN", "PNPM_HOME"] {
            if let Some(value) = values.get(key) {
                paths.insert(0, value.clone());
            }
        }
        for (key, suffix) in [
            ("VOLTA_HOME", "bin"),
            ("ASDF_DATA_DIR", "shims"),
            ("MISE_DATA_DIR", "shims"),
            ("VIRTUAL_ENV", "bin"),
            ("CONDA_PREFIX", "bin"),
        ] {
            if let Some(value) = values.get(key) {
                paths.insert(0, format!("{value}/{suffix}"));
            }
        }
        if let Some(existing) = host("PATH") {
            paths.push(existing);
        }
        let mut deduplicated_paths = Vec::with_capacity(paths.len());
        for path in paths {
            if !deduplicated_paths.iter().any(|value| value == &path) {
                deduplicated_paths.push(path);
            }
        }
        let path = deduplicated_paths.join(":");
        values.insert("PATH".into(), path);
        values
            .entry("TMPDIR".into())
            .or_insert_with(|| std::env::temp_dir().to_string_lossy().into_owned());
        values
            .entry("LANG".into())
            .or_insert_with(|| "C.UTF-8".into());
        values
            .entry("LC_ALL".into())
            .or_insert_with(|| "C.UTF-8".into());
        values
            .entry("SHELL".into())
            .or_insert_with(|| "/bin/sh".into());
        values.entry("TERM".into()).or_insert_with(|| "dumb".into());
        if let Some(home) = home {
            values
                .entry("XDG_CONFIG_HOME".into())
                .or_insert_with(|| format!("{home}/.config"));
            values
                .entry("XDG_DATA_HOME".into())
                .or_insert_with(|| format!("{home}/.local/share"));
            values
                .entry("XDG_CACHE_HOME".into())
                .or_insert_with(|| format!("{home}/.cache"));
        }
        values.insert(
            "PWD".into(),
            working_directory.to_string_lossy().into_owned(),
        );
    }

    for (key, value) in configured {
        #[cfg(windows)]
        {
            let existing = values
                .keys()
                .find(|existing| existing.eq_ignore_ascii_case(key))
                .cloned();
            if let Some(existing) = existing {
                values.remove(&existing);
            }
        }
        values.insert(key.clone(), value.clone());
    }
    values
}

fn configured_secret_values(config: &ConnectionConfig) -> Vec<String> {
    let values = match config {
        ConnectionConfig::Stdio { environment, .. } => environment.values(),
        ConnectionConfig::StreamableHttp { headers, .. } => headers.values(),
    };
    values.filter(|value| !value.is_empty()).cloned().collect()
}

fn http_headers(
    values: BTreeMap<String, String>,
) -> Result<HashMap<HeaderName, HeaderValue>, BusinessError> {
    values
        .into_iter()
        .map(|(name, value)| {
            let name = HeaderName::try_from(name)
                .map_err(|_| BusinessError::invalid("MCP HTTP header name is invalid"))?;
            let value = HeaderValue::try_from(value)
                .map_err(|_| BusinessError::invalid("MCP HTTP header value is invalid"))?;
            Ok((name, value))
        })
        .collect()
}

fn http_client(config: &TlsConfig) -> Result<reqwest::Client, BusinessError> {
    let mut builder = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(!config.verify_certificates);
    let custom_certificate = if let Some(path) = &config.certificate_path {
        let pem = std::fs::read(path)
            .map_err(|_| BusinessError::invalid("custom certificate file is unavailable"))?;
        Some(
            reqwest::Certificate::from_pem(&pem)
                .map_err(|_| BusinessError::invalid("custom certificate file is invalid"))?,
        )
    } else {
        None
    };
    if config.use_system_certificates {
        if let Some(certificate) = custom_certificate {
            builder = builder.tls_certs_merge([certificate]);
        }
    } else {
        builder = builder.tls_certs_only(custom_certificate.into_iter());
    }
    builder
        .build()
        .map_err(|error| mcp_error("mcp_http_client_failed", error.to_string()))
}

fn normalize_result(result: rmcp::model::CallToolResult) -> Result<ToolResult, BusinessError> {
    let mut content = Vec::new();
    for block in result.content {
        match block {
            ContentBlock::Text(text) => content.push(text.text),
            _ => {
                return Err(mcp_error(
                    "mcp_content_unsupported",
                    "MCP tool returned unsupported non-text content",
                ))
            }
        }
    }
    let mut normalized = ToolResult {
        content,
        structured_content: result.structured_content,
        is_error: result.is_error.unwrap_or(false),
        truncated: false,
    };
    let encoded = serde_json::to_vec(&normalized)
        .map_err(|_| mcp_error("mcp_result_invalid", "MCP result could not be encoded"))?;
    if encoded.len() > MAX_RESULT_BYTES {
        let mut remaining = MAX_RESULT_BYTES / 2;
        for value in &mut normalized.content {
            if value.len() > remaining {
                let mut boundary = remaining.min(value.len());
                while !value.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                value.truncate(boundary);
            }
            remaining = remaining.saturating_sub(value.len());
        }
        normalized.structured_content = normalized
            .structured_content
            .take()
            .map(|_| json!({"truncated": true}));
        normalized.truncated = true;
    }
    Ok(normalized)
}

fn mcp_service_error(
    message: &str,
    error: &impl std::fmt::Display,
    secret_values: &[String],
) -> BusinessError {
    redact_business_error(
        mcp_error("mcp_protocol_error", format!("{message}: {error}")),
        secret_values,
    )
}

fn mcp_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, redact_error(&message.into())).with_retryable(true)
}

fn redact_error(message: &str) -> String {
    let mut value = message.chars().take(500).collect::<String>();
    for marker in ["Authorization", "Bearer", "token=", "key="] {
        if let Some(index) = value
            .to_ascii_lowercase()
            .find(&marker.to_ascii_lowercase())
        {
            value.truncate(index);
            value.push_str("[redacted]");
            break;
        }
    }
    value
}

fn redact_business_error(mut error: BusinessError, secret_values: &[String]) -> BusinessError {
    for secret in secret_values {
        error.message = error.message.replace(secret, "[redacted]");
    }
    error
}

pub fn tool_result_value(result: ToolResult) -> Value {
    let mut value = Map::new();
    value.insert(
        "content".into(),
        Value::Array(result.content.into_iter().map(Value::String).collect()),
    );
    if let Some(structured) = result.structured_content {
        value.insert("structuredContent".into(), structured);
    }
    value.insert("isError".into(), Value::Bool(result.is_error));
    value.insert("truncated".into(), Value::Bool(result.truncated));
    Value::Object(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_environment_includes_platform_baseline_and_working_directory() {
        let working_directory = std::path::Path::new("/tmp/suncode-mcp-project");
        let values = default_environment(working_directory, &BTreeMap::new());
        assert!(values.get("PATH").is_some_and(|value| !value.is_empty()));
        assert!(
            values.get("TMPDIR").is_some_and(|value| !value.is_empty())
                || values.get("TEMP").is_some_and(|value| !value.is_empty())
        );
        #[cfg(not(windows))]
        {
            assert_eq!(values.get("LANG").map(String::as_str), Some("C.UTF-8"));
            assert_eq!(values.get("LC_ALL").map(String::as_str), Some("C.UTF-8"));
            assert_eq!(
                values.get("PWD").map(String::as_str),
                Some("/tmp/suncode-mcp-project")
            );
        }
        #[cfg(windows)]
        {
            assert!(values.contains_key("SystemRoot"));
            assert!(values.contains_key("ComSpec"));
        }
    }

    #[test]
    fn configured_environment_overrides_defaults_without_removing_baseline() {
        let configured = BTreeMap::from([
            ("PATH".into(), "/custom/bin".into()),
            ("MCP_TOKEN".into(), "secret".into()),
        ]);
        let values = default_environment(std::path::Path::new("/tmp/project"), &configured);
        assert_eq!(values.get("PATH").map(String::as_str), Some("/custom/bin"));
        assert_eq!(values.get("MCP_TOKEN").map(String::as_str), Some("secret"));
        assert!(values.contains_key("TMPDIR") || values.contains_key("TEMP"));
    }

    #[test]
    fn result_normalization_rejects_binary_content() {
        let result = rmcp::model::CallToolResult::success(vec![ContentBlock::image(
            "aGVsbG8=",
            "image/png",
        )]);
        assert_eq!(
            normalize_result(result).unwrap_err().code,
            "mcp_content_unsupported"
        );
    }

    #[test]
    fn redacts_likely_credentials_from_errors() {
        assert_eq!(
            redact_error("request failed Authorization: secret"),
            "request failed [redacted]"
        );
    }

    #[test]
    fn redacts_configured_values_from_protocol_errors() {
        let error = mcp_service_error(
            "request failed",
            &"remote echoed opaque-secret-value",
            &["opaque-secret-value".into()],
        );
        assert_eq!(error.message, "request failed: remote echoed [redacted]");
    }
}
