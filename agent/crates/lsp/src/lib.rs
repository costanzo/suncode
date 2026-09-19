//! Bounded local-stdio Language Server Protocol client.
//!
//! The crate owns protocol framing and process lifecycle only. Project authority,
//! persistence, model tool semantics, and client DTOs remain outside this boundary.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, Weak,
    },
    time::Duration,
};
use suncode_common::BusinessError;
use tokio::{
    io::{
        AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
        BufReader,
    },
    process::{Child, Command},
    sync::{oneshot, Mutex, RwLock},
};
use tokio_util::sync::CancellationToken;
use url::Url;

const MAX_MESSAGE_BYTES: usize = 4 * 1024 * 1024;
const MAX_DIAGNOSTICS_PER_DOCUMENT: usize = 1_000;
const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub command: String,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub initialization_options: Value,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionEncoding {
    Utf8,
    Utf16,
    Utf32,
}

impl PositionEncoding {
    pub fn as_lsp_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Utf16 => "utf-16",
            Self::Utf32 => "utf-32",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub position_encoding: PositionEncoding,
    pub pull_diagnostics: bool,
    pub definition: bool,
    pub references: bool,
    pub hover: bool,
    pub document_symbols: bool,
}

#[derive(Debug, Clone)]
struct OpenDocument {
    version: i32,
    language_id: String,
    text: String,
}

struct Inner {
    writer: Mutex<Box<dyn AsyncWrite + Unpin + Send>>,
    child: Mutex<Option<Child>>,
    pending: Mutex<HashMap<i64, oneshot::Sender<Result<Value, BusinessError>>>>,
    diagnostics: RwLock<HashMap<String, Value>>,
    documents: Mutex<HashMap<String, OpenDocument>>,
    capabilities: RwLock<ServerCapabilities>,
    next_id: AtomicI64,
    closed: AtomicBool,
    request_timeout: Duration,
    secret_values: Vec<String>,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

pub async fn connect(config: ConnectionConfig, root: &Path) -> Result<Client, BusinessError> {
    validate_connection_config(&config)?;
    let mut process = Command::new(&config.command);
    process
        .args(&config.arguments)
        .current_dir(&config.working_directory)
        .kill_on_drop(true)
        .env_clear()
        .envs(default_environment(
            &config.working_directory,
            &config.environment,
        ))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = process.spawn().map_err(|error| {
        lsp_error(
            "lsp_process_start_failed",
            redact(&error.to_string(), config.environment.values()),
        )
    })?;
    let stdin = child.stdin.take().ok_or_else(|| {
        lsp_error(
            "lsp_process_start_failed",
            "Language server stdin is unavailable",
        )
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        lsp_error(
            "lsp_process_start_failed",
            "Language server stdout is unavailable",
        )
    })?;
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(drain_stderr(stderr));
    }
    let client = Client::from_io(
        stdout,
        stdin,
        Some(child),
        config.request_timeout,
        config.environment.values().cloned().collect(),
    );
    let root_uri = Url::from_directory_path(root).map_err(|_| {
        lsp_error(
            "lsp_root_invalid",
            "Project root could not be converted to a file URI",
        )
    })?;
    let initialize = client.initialize(root_uri, config.initialization_options);
    match tokio::time::timeout(config.startup_timeout, initialize).await {
        Ok(Ok(())) => Ok(client),
        Ok(Err(error)) => {
            client.force_close().await;
            Err(error)
        }
        Err(_) => {
            client.force_close().await;
            Err(lsp_error(
                "lsp_startup_timeout",
                "Language server initialization timed out",
            ))
        }
    }
}

impl Client {
    fn from_io<R, W>(
        reader: R,
        writer: W,
        child: Option<Child>,
        request_timeout: Duration,
        secret_values: Vec<String>,
    ) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let inner = Arc::new(Inner {
            writer: Mutex::new(Box::new(writer)),
            child: Mutex::new(child),
            pending: Mutex::new(HashMap::new()),
            diagnostics: RwLock::new(HashMap::new()),
            documents: Mutex::new(HashMap::new()),
            capabilities: RwLock::new(ServerCapabilities {
                position_encoding: PositionEncoding::Utf16,
                pull_diagnostics: false,
                definition: false,
                references: false,
                hover: false,
                document_symbols: false,
            }),
            next_id: AtomicI64::new(1),
            closed: AtomicBool::new(false),
            request_timeout,
            secret_values,
        });
        tokio::spawn(read_loop(BufReader::new(reader), Arc::downgrade(&inner)));
        Self { inner }
    }

    async fn initialize(
        &self,
        root_uri: Url,
        initialization_options: Value,
    ) -> Result<(), BusinessError> {
        let root_uri = root_uri.to_string();
        let result = self
            .request(
                "initialize",
                json!({
                    "processId": null,
                    "clientInfo": {"name":"SunCode","version":env!("CARGO_PKG_VERSION")},
                    "rootUri": root_uri,
                    "workspaceFolders": [{"uri":root_uri,"name":"project"}],
                    "initializationOptions": initialization_options,
                    "capabilities": {
                        "general": {"positionEncodings":["utf-8","utf-16","utf-32"]},
                        "workspace": {"applyEdit":false,"workspaceFolders":true,"configuration":false},
                        "textDocument": {
                            "synchronization": {"dynamicRegistration":false,"didSave":false},
                            "publishDiagnostics": {"relatedInformation":true,"versionSupport":true},
                            "diagnostic": {"dynamicRegistration":false,"relatedDocumentSupport":false},
                            "definition": {"dynamicRegistration":false,"linkSupport":false},
                            "references": {"dynamicRegistration":false},
                            "hover": {"dynamicRegistration":false,"contentFormat":["markdown","plaintext"]},
                            "documentSymbol": {"dynamicRegistration":false,"hierarchicalDocumentSymbolSupport":true}
                        }
                    },
                    "trace":"off"
                }),
                CancellationToken::new(),
            )
            .await?;
        let capabilities = result
            .get("capabilities")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let position_encoding = match capabilities.get("positionEncoding").and_then(Value::as_str) {
            Some("utf-8") => PositionEncoding::Utf8,
            Some("utf-32") => PositionEncoding::Utf32,
            _ => PositionEncoding::Utf16,
        };
        let pull_diagnostics = capability_enabled(capabilities.get("diagnosticProvider"));
        let definition = capability_enabled(capabilities.get("definitionProvider"));
        let references = capability_enabled(capabilities.get("referencesProvider"));
        let hover = capability_enabled(capabilities.get("hoverProvider"));
        let document_symbols = capability_enabled(capabilities.get("documentSymbolProvider"));
        *self.inner.capabilities.write().await = ServerCapabilities {
            position_encoding,
            pull_diagnostics,
            definition,
            references,
            hover,
            document_symbols,
        };
        self.notify("initialized", json!({})).await
    }

    pub async fn capabilities(&self) -> ServerCapabilities {
        self.inner.capabilities.read().await.clone()
    }

    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::SeqCst)
    }

    pub async fn sync_document(
        &self,
        uri: &str,
        language_id: &str,
        text: &str,
    ) -> Result<i32, BusinessError> {
        if text.len() > MAX_DOCUMENT_BYTES {
            return Err(lsp_error(
                "lsp_document_too_large",
                "Language server document exceeds 4 MiB",
            ));
        }
        let (version, method, params) = {
            let mut documents = self.inner.documents.lock().await;
            if let Some(document) = documents.get_mut(uri) {
                if document.text == text && document.language_id == language_id {
                    return Ok(document.version);
                }
                document.version = document.version.saturating_add(1);
                document.language_id = language_id.to_string();
                document.text = text.to_string();
                (
                    document.version,
                    "textDocument/didChange",
                    json!({
                        "textDocument":{"uri":uri,"version":document.version},
                        "contentChanges":[{"text":text}]
                    }),
                )
            } else {
                let document = OpenDocument {
                    version: 1,
                    language_id: language_id.to_string(),
                    text: text.to_string(),
                };
                documents.insert(uri.to_string(), document);
                (
                    1,
                    "textDocument/didOpen",
                    json!({
                        "textDocument":{"uri":uri,"languageId":language_id,"version":1,"text":text}
                    }),
                )
            }
        };
        self.notify(method, params).await?;
        Ok(version)
    }

    pub async fn diagnostics(
        &self,
        uri: &str,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        if self.capabilities().await.pull_diagnostics {
            let value = self
                .request(
                    "textDocument/diagnostic",
                    json!({"textDocument":{"uri":uri}}),
                    cancellation,
                )
                .await?;
            let items = value
                .get("items")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            return Ok(json!({"items":bound_array(items, MAX_DIAGNOSTICS_PER_DOCUMENT)}));
        }
        tokio::select! {
            _ = cancellation.cancelled() => Err(lsp_error("cancelled", "Language server request was cancelled")),
            _ = tokio::time::sleep(Duration::from_millis(150)) => {
                Ok(self.inner.diagnostics.read().await.get(uri).cloned().unwrap_or_else(|| json!({"items":[]})))
            }
        }
    }

    pub async fn definition(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        self.request(
            "textDocument/definition",
            text_position_params(uri, line, character),
            cancellation,
        )
        .await
    }

    pub async fn references(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        self.request(
            "textDocument/references",
            json!({
                "textDocument":{"uri":uri},
                "position":{"line":line,"character":character},
                "context":{"includeDeclaration":true}
            }),
            cancellation,
        )
        .await
    }

    pub async fn hover(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        self.request(
            "textDocument/hover",
            text_position_params(uri, line, character),
            cancellation,
        )
        .await
    }

    pub async fn document_symbols(
        &self,
        uri: &str,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        self.request(
            "textDocument/documentSymbol",
            json!({"textDocument":{"uri":uri}}),
            cancellation,
        )
        .await
    }

    async fn request(
        &self,
        method: &str,
        params: Value,
        cancellation: CancellationToken,
    ) -> Result<Value, BusinessError> {
        if self.is_closed() {
            return Err(lsp_error(
                "lsp_connection_closed",
                "Language server connection is closed",
            ));
        }
        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = oneshot::channel();
        self.inner.pending.lock().await.insert(id, sender);
        if let Err(error) = self
            .write_value(&json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
            .await
        {
            self.inner.pending.lock().await.remove(&id);
            return Err(error);
        }
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                self.inner.pending.lock().await.remove(&id);
                let _ = self.notify("$/cancelRequest", json!({"id":id})).await;
                Err(lsp_error("cancelled", "Language server request was cancelled"))
            }
            result = tokio::time::timeout(self.inner.request_timeout, receiver) => {
                match result {
                    Ok(Ok(result)) => result,
                    Ok(Err(_)) => Err(lsp_error("lsp_connection_closed", "Language server connection closed before responding")),
                    Err(_) => {
                        self.inner.pending.lock().await.remove(&id);
                        let _ = self.notify("$/cancelRequest", json!({"id":id})).await;
                        Err(lsp_error("lsp_request_timeout", "Language server request timed out"))
                    }
                }
            }
        }
    }

    async fn notify(&self, method: &str, params: Value) -> Result<(), BusinessError> {
        self.write_value(&json!({"jsonrpc":"2.0","method":method,"params":params}))
            .await
    }

    async fn write_value(&self, value: &Value) -> Result<(), BusinessError> {
        let body = serde_json::to_vec(value).map_err(|error| {
            lsp_error(
                "lsp_serialization_failed",
                redact(&error.to_string(), self.inner.secret_values.iter()),
            )
        })?;
        if body.len() > MAX_MESSAGE_BYTES {
            return Err(lsp_error(
                "lsp_message_too_large",
                "Language server message exceeds 4 MiB",
            ));
        }
        let mut writer = self.inner.writer.lock().await;
        writer
            .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
            .await
            .map_err(|error| self.io_error("Language server write failed", error))?;
        writer
            .write_all(&body)
            .await
            .map_err(|error| self.io_error("Language server write failed", error))?;
        writer
            .flush()
            .await
            .map_err(|error| self.io_error("Language server write failed", error))
    }

    pub async fn close(&self) {
        if self.is_closed() {
            return;
        }
        let documents = std::mem::take(&mut *self.inner.documents.lock().await);
        for uri in documents.keys() {
            let _ = self
                .notify("textDocument/didClose", json!({"textDocument":{"uri":uri}}))
                .await;
        }
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            self.request("shutdown", json!(null), CancellationToken::new()),
        )
        .await;
        let _ = self.notify("exit", json!(null)).await;
        self.force_close().await;
    }

    async fn force_close(&self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        if let Some(mut child) = self.inner.child.lock().await.take() {
            let _ = child.start_kill();
            let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
        }
        fail_pending(
            &self.inner,
            lsp_error("lsp_connection_closed", "Language server connection closed"),
        )
        .await;
    }

    fn io_error(&self, prefix: &str, error: std::io::Error) -> BusinessError {
        lsp_error(
            "lsp_io_failed",
            format!(
                "{prefix}: {}",
                redact(&error.to_string(), self.inner.secret_values.iter())
            ),
        )
    }
}

async fn read_loop<R>(mut reader: R, inner: Weak<Inner>)
where
    R: AsyncBufRead + Unpin,
{
    loop {
        let Some(inner) = inner.upgrade() else {
            return;
        };
        match read_value(&mut reader).await {
            Ok(Some(value)) => handle_incoming(&inner, value).await,
            Ok(None) => {
                inner.closed.store(true, Ordering::SeqCst);
                fail_pending(
                    &inner,
                    lsp_error("lsp_connection_closed", "Language server closed stdout"),
                )
                .await;
                return;
            }
            Err(error) => {
                inner.closed.store(true, Ordering::SeqCst);
                fail_pending(&inner, error).await;
                return;
            }
        }
    }
}

async fn handle_incoming(inner: &Arc<Inner>, value: Value) {
    if value.get("method").is_some() && value.get("id").is_some() {
        handle_server_request(inner, value).await;
        return;
    }
    if let Some(id) = value.get("id").and_then(Value::as_i64) {
        if let Some(sender) = inner.pending.lock().await.remove(&id) {
            let result = if let Some(error) = value.get("error") {
                Err(lsp_error(
                    "lsp_request_failed",
                    redact(&error.to_string(), inner.secret_values.iter()),
                ))
            } else {
                Ok(value.get("result").cloned().unwrap_or(Value::Null))
            };
            let _ = sender.send(result);
        }
        return;
    }
    if value.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics") {
        if let Some(uri) = value
            .pointer("/params/uri")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            let mut params = value.get("params").cloned().unwrap_or_else(|| json!({}));
            if let Some(items) = params.get_mut("diagnostics").and_then(Value::as_array_mut) {
                items.truncate(MAX_DIAGNOSTICS_PER_DOCUMENT);
            }
            if let Some(object) = params.as_object_mut() {
                if let Some(items) = object.remove("diagnostics") {
                    object.insert("items".into(), items);
                }
            }
            inner.diagnostics.write().await.insert(uri, params);
        }
    }
}

async fn handle_server_request(inner: &Arc<Inner>, value: Value) {
    let id = value.get("id").cloned().unwrap_or(Value::Null);
    let method = value
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let response = match method {
        "workspace/applyEdit" => json!({
            "jsonrpc":"2.0",
            "id":id,
            "result":{"applied":false,"failureReason":"SunCode does not accept language-server edits"}
        }),
        "workspace/configuration" => {
            let count = value
                .pointer("/params/items")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            json!({"jsonrpc":"2.0","id":id,"result":vec![Value::Null;count]})
        }
        "client/registerCapability"
        | "client/unregisterCapability"
        | "window/workDoneProgress/create" => {
            json!({"jsonrpc":"2.0","id":id,"result":Value::Null})
        }
        _ => json!({
            "jsonrpc":"2.0",
            "id":id,
            "error":{"code":-32601,"message":"Method not supported by SunCode"}
        }),
    };
    let _ = write_value(inner, &response).await;
}

async fn write_value(inner: &Arc<Inner>, value: &Value) -> Result<(), BusinessError> {
    let body = serde_json::to_vec(value)?;
    if body.len() > MAX_MESSAGE_BYTES {
        return Err(lsp_error(
            "lsp_message_too_large",
            "Language server message exceeds 4 MiB",
        ));
    }
    let mut writer = inner.writer.lock().await;
    writer
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

async fn read_value<R>(reader: &mut R) -> Result<Option<Value>, BusinessError>
where
    R: AsyncBufRead + Unpin,
{
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line).await.map_err(|error| {
            lsp_error(
                "lsp_io_failed",
                format!("Language server read failed: {error}"),
            )
        })?;
        if read == 0 {
            return Ok(None);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some(value) = line
            .trim()
            .strip_prefix("Content-Length:")
            .or_else(|| line.trim().strip_prefix("content-length:"))
        {
            content_length = Some(value.trim().parse::<usize>().map_err(|_| {
                lsp_error(
                    "lsp_framing_invalid",
                    "Language server Content-Length is invalid",
                )
            })?);
        }
    }
    let content_length = content_length.ok_or_else(|| {
        lsp_error(
            "lsp_framing_invalid",
            "Language server message omitted Content-Length",
        )
    })?;
    if content_length > MAX_MESSAGE_BYTES {
        return Err(lsp_error(
            "lsp_message_too_large",
            "Language server message exceeds 4 MiB",
        ));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).await.map_err(|error| {
        lsp_error(
            "lsp_io_failed",
            format!("Language server body failed: {error}"),
        )
    })?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| lsp_error("lsp_json_invalid", error.to_string()))
}

async fn fail_pending(inner: &Arc<Inner>, error: BusinessError) {
    let pending = std::mem::take(&mut *inner.pending.lock().await);
    for (_, sender) in pending {
        let _ = sender.send(Err(error.clone()));
    }
}

fn text_position_params(uri: &str, line: u32, character: u32) -> Value {
    json!({
        "textDocument":{"uri":uri},
        "position":{"line":line,"character":character}
    })
}

fn capability_enabled(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::Object(_)) => true,
        _ => false,
    }
}

fn bound_array(mut values: Vec<Value>, limit: usize) -> Vec<Value> {
    values.truncate(limit);
    values
}

fn validate_connection_config(config: &ConnectionConfig) -> Result<(), BusinessError> {
    if config.command.trim().is_empty() || config.command.chars().count() > 2_048 {
        return Err(BusinessError::invalid(
            "Language server command is required and must not exceed 2048 characters",
        ));
    }
    if config.arguments.len() > 128
        || config
            .arguments
            .iter()
            .any(|value| value.chars().count() > 8_192)
    {
        return Err(BusinessError::invalid(
            "Language server arguments exceed the supported bounds",
        ));
    }
    if !config.initialization_options.is_null() && !config.initialization_options.is_object() {
        return Err(BusinessError::invalid(
            "Language server initialization options must be a JSON object",
        ));
    }
    if !config.working_directory.is_dir() {
        return Err(BusinessError::invalid(
            "Language server working directory is unavailable",
        ));
    }
    if config.startup_timeout.is_zero() || config.request_timeout.is_zero() {
        return Err(BusinessError::invalid(
            "Language server timeouts must be greater than zero",
        ));
    }
    Ok(())
}

fn lsp_error(code: &str, message: impl Into<String>) -> BusinessError {
    BusinessError::new(code, message).with_retryable(matches!(
        code,
        "lsp_process_start_failed"
            | "lsp_startup_timeout"
            | "lsp_request_timeout"
            | "lsp_connection_closed"
            | "lsp_io_failed"
    ))
}

fn redact<'a>(message: &str, values: impl IntoIterator<Item = &'a String>) -> String {
    let mut redacted = message.replace(['\r', '\n'], " ");
    for value in values {
        if value.len() >= 4 {
            redacted = redacted.replace(value, "[redacted]");
        }
    }
    redacted.chars().take(1_024).collect()
}

async fn drain_stderr<R>(mut stderr: R)
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        match stderr.read(&mut buffer).await {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
    }
}

fn default_environment(
    working_directory: &Path,
    configured: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
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
            .or_insert(system_root.clone());
        values.entry("WINDIR".into()).or_insert(system_root.clone());
        values
            .entry("ComSpec".into())
            .or_insert_with(|| format!(r"{system_root}\System32\cmd.exe"));
        values
            .entry("PATHEXT".into())
            .or_insert_with(|| ".COM;.EXE;.BAT;.CMD".into());
        let temp = host("TEMP")
            .or_else(|| host("TMP"))
            .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
        values.entry("TEMP".into()).or_insert(temp.clone());
        values.entry("TMP".into()).or_insert(temp);
        values.entry("PATH".into()).or_insert_with(|| {
            host("Path")
                .or_else(|| host("PATH"))
                .unwrap_or_else(|| format!(r"{system_root}\System32;{system_root}"))
        });
    }

    #[cfg(not(windows))]
    {
        for key in [
            "HOME",
            "USER",
            "LOGNAME",
            "SHELL",
            "SSH_AUTH_SOCK",
            "LANG",
            "LC_ALL",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "NVM_BIN",
            "PNPM_HOME",
            "VIRTUAL_ENV",
            "CONDA_PREFIX",
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
        if let Some(existing) = host("PATH") {
            paths.push(existing);
        }
        paths.dedup();
        values.insert("PATH".into(), paths.join(":"));
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
        values.insert(
            "PWD".into(),
            working_directory.to_string_lossy().into_owned(),
        );
    }

    for (key, value) in configured {
        values.insert(key.clone(), value.clone());
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, split, AsyncWriteExt};

    async fn read_test_message<R>(reader: &mut BufReader<R>) -> Value
    where
        R: AsyncRead + Unpin,
    {
        read_value(reader).await.unwrap().unwrap()
    }

    async fn write_test_message<W>(writer: &mut W, value: Value)
    where
        W: AsyncWrite + Unpin,
    {
        let body = serde_json::to_vec(&value).unwrap();
        writer
            .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
            .await
            .unwrap();
        writer.write_all(&body).await.unwrap();
        writer.flush().await.unwrap();
    }

    #[tokio::test]
    async fn initializes_syncs_and_requests_semantics() {
        let (client_stream, server_stream) = duplex(128 * 1024);
        let (client_reader, client_writer) = split(client_stream);
        let (server_reader, mut server_writer) = split(server_stream);
        let server = tokio::spawn(async move {
            let mut reader = BufReader::new(server_reader);
            let initialize = read_test_message(&mut reader).await;
            assert_eq!(initialize["method"], "initialize");
            write_test_message(
                &mut server_writer,
                json!({
                    "jsonrpc":"2.0",
                    "id":initialize["id"],
                    "result":{"capabilities":{
                        "positionEncoding":"utf-8",
                        "diagnosticProvider":true,
                        "definitionProvider":true,
                        "referencesProvider":true,
                        "hoverProvider":true,
                        "documentSymbolProvider":true
                    }}
                }),
            )
            .await;
            assert_eq!(
                read_test_message(&mut reader).await["method"],
                "initialized"
            );
            assert_eq!(
                read_test_message(&mut reader).await["method"],
                "textDocument/didOpen"
            );
            let diagnostic = read_test_message(&mut reader).await;
            assert_eq!(diagnostic["method"], "textDocument/diagnostic");
            write_test_message(
                &mut server_writer,
                json!({"jsonrpc":"2.0","id":diagnostic["id"],"result":{"kind":"full","items":[{"message":"problem"}]}}),
            )
            .await;
            let definition = read_test_message(&mut reader).await;
            assert_eq!(definition["method"], "textDocument/definition");
            write_test_message(
                &mut server_writer,
                json!({"jsonrpc":"2.0","id":definition["id"],"result":{"uri":"file:///tmp/lib.rs","range":{"start":{"line":1,"character":2},"end":{"line":1,"character":4}}}}),
            )
            .await;
        });
        let client = Client::from_io(
            client_reader,
            client_writer,
            None,
            Duration::from_secs(1),
            Vec::new(),
        );
        client
            .initialize(Url::parse("file:///tmp/project/").unwrap(), json!({}))
            .await
            .unwrap();
        assert_eq!(
            client.capabilities().await.position_encoding,
            PositionEncoding::Utf8
        );
        client
            .sync_document("file:///tmp/project/src/lib.rs", "rust", "fn main() {}")
            .await
            .unwrap();
        let diagnostics = client
            .diagnostics("file:///tmp/project/src/lib.rs", CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(diagnostics["items"][0]["message"], "problem");
        let definition = client
            .definition(
                "file:///tmp/project/src/lib.rs",
                0,
                3,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(definition["uri"], "file:///tmp/lib.rs");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn refuses_server_requested_edits() {
        let (client_stream, server_stream) = duplex(32 * 1024);
        let (client_reader, client_writer) = split(client_stream);
        let (server_reader, mut server_writer) = split(server_stream);
        let client = Client::from_io(
            client_reader,
            client_writer,
            None,
            Duration::from_secs(1),
            Vec::new(),
        );
        let server = tokio::spawn(async move {
            let mut reader = BufReader::new(server_reader);
            write_test_message(
                &mut server_writer,
                json!({"jsonrpc":"2.0","id":99,"method":"workspace/applyEdit","params":{"edit":{}}}),
            )
            .await;
            let response = read_test_message(&mut reader).await;
            assert_eq!(response["id"], 99);
            assert_eq!(response["result"]["applied"], false);
        });
        server.await.unwrap();
        tokio::task::yield_now().await;
        assert!(client.is_closed());
    }

    #[tokio::test]
    async fn rejects_oversized_framing() {
        let data = format!("Content-Length: {}\r\n\r\n", MAX_MESSAGE_BYTES + 1);
        let mut reader = BufReader::new(data.as_bytes());
        let error = read_value(&mut reader).await.unwrap_err();
        assert_eq!(error.code, "lsp_message_too_large");
    }

    #[tokio::test]
    async fn missing_executable_has_a_stable_error_code() {
        let root = tempfile::tempdir().unwrap();
        let result = connect(
            ConnectionConfig {
                command: "definitely-not-a-real-suncode-language-server".into(),
                arguments: Vec::new(),
                working_directory: root.path().to_path_buf(),
                environment: BTreeMap::new(),
                initialization_options: json!({}),
                startup_timeout: Duration::from_secs(1),
                request_timeout: Duration::from_secs(1),
            },
            root.path(),
        )
        .await;
        let error = match result {
            Ok(_) => panic!("missing language server unexpectedly started"),
            Err(error) => error,
        };
        assert_eq!(error.code, "lsp_process_start_failed");
    }
}
