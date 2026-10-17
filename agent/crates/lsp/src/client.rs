//! Language-server process lifecycle, request/response correlation, and the
//! incoming-message read loop.

use crate::{
    capabilities::{capability_enabled, PositionEncoding, ServerCapabilities},
    config::{validate_connection_config, ConnectionConfig},
    environment::default_environment,
    error::{lsp_error, redact},
    framing::{self, read_value},
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, Weak,
    },
    time::Duration,
};
use suncode_common::BusinessError;
use tokio::{
    io::{AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWrite, BufReader},
    process::{Child, Command},
    sync::{oneshot, Mutex, RwLock},
};
use tokio_util::sync::CancellationToken;
use url::Url;

const MAX_DIAGNOSTICS_PER_DOCUMENT: usize = 1_000;
const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

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
    configuration: Value,
    secret_values: Vec<String>,
    /// Cancelled when `Inner` is dropped so the read loop can stop waiting.
    released: CancellationToken,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.released.cancel();
    }
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
        config.configuration.clone(),
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
        configuration: Value,
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
            configuration,
            secret_values,
            released: CancellationToken::new(),
        });
        tokio::spawn(read_loop(
            BufReader::new(reader),
            Arc::downgrade(&inner),
            inner.released.clone(),
        ));
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
                        "workspace": {"applyEdit":false,"workspaceFolders":true,"configuration":true},
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
        self.notify("initialized", json!({})).await?;
        if self
            .inner
            .configuration
            .as_object()
            .is_some_and(|settings| !settings.is_empty())
        {
            self.notify(
                "workspace/didChangeConfiguration",
                json!({"settings":self.inner.configuration.clone()}),
            )
            .await?;
        }
        Ok(())
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
        self.inner.write_value(value).await
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
}

impl Inner {
    /// Serialises writers and bounds each write by the request timeout so a
    /// server that stops reading stdin cannot block the client forever.
    async fn write_value(&self, value: &Value) -> Result<(), BusinessError> {
        let mut writer = self.writer.lock().await;
        framing::write_value(
            &mut **writer,
            value,
            self.request_timeout,
            &self.secret_values,
        )
        .await
    }
}

/// Owns only the reader while waiting for input. The strong reference to
/// `Inner` is taken per message, so dropping every `Client` drops `Inner` (and
/// its kill-on-drop `Child`) even while the server is idle; the next message
/// or end of stream then ends the loop.
async fn read_loop<R>(mut reader: R, inner: Weak<Inner>, released: CancellationToken)
where
    R: AsyncBufRead + Unpin,
{
    loop {
        let outcome = tokio::select! {
            outcome = read_value(&mut reader) => outcome,
            () = released.cancelled() => return,
        };
        let Some(inner) = inner.upgrade() else {
            return;
        };
        match outcome {
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
            let result = value
                .pointer("/params/items")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| {
                            item.get("section")
                                .and_then(Value::as_str)
                                .map(|section| configuration_section(&inner.configuration, section))
                                .unwrap_or_else(|| inner.configuration.clone())
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            json!({"jsonrpc":"2.0","id":id,"result":result})
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
    let _ = inner.write_value(&response).await;
}

fn configuration_section(configuration: &Value, section: &str) -> Value {
    if section.trim().is_empty() {
        return configuration.clone();
    }
    section
        .split('.')
        .try_fold(configuration, |value, part| value.get(part))
        .cloned()
        .unwrap_or(Value::Null)
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

fn bound_array(mut values: Vec<Value>, limit: usize) -> Vec<Value> {
    values.truncate(limit);
    values
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
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
            json!({}),
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
    async fn advertises_and_serves_workspace_configuration() {
        let (client_stream, server_stream) = duplex(64 * 1024);
        let (client_reader, client_writer) = split(client_stream);
        let (server_reader, mut server_writer) = split(server_stream);
        let configuration = json!({
            "vtsls":{"autoUseWorkspaceTsdk":true},
            "typescript":{"tsserver":{"maxTsServerMemory":4096}}
        });
        let client = Client::from_io(
            client_reader,
            client_writer,
            None,
            Duration::from_secs(1),
            configuration.clone(),
            Vec::new(),
        );
        let server = tokio::spawn(async move {
            let mut reader = BufReader::new(server_reader);
            let initialize = read_test_message(&mut reader).await;
            assert_eq!(
                initialize["params"]["capabilities"]["workspace"]["configuration"],
                true
            );
            write_test_message(
                &mut server_writer,
                json!({"jsonrpc":"2.0","id":initialize["id"],"result":{"capabilities":{}}}),
            )
            .await;
            assert_eq!(
                read_test_message(&mut reader).await["method"],
                "initialized"
            );
            let changed = read_test_message(&mut reader).await;
            assert_eq!(changed["method"], "workspace/didChangeConfiguration");
            assert_eq!(changed["params"]["settings"], configuration);
            write_test_message(
                &mut server_writer,
                json!({
                    "jsonrpc":"2.0",
                    "id":42,
                    "method":"workspace/configuration",
                    "params":{"items":[
                        {"section":"vtsls"},
                        {"section":"typescript.tsserver"},
                        {"section":"missing"},
                        {}
                    ]}
                }),
            )
            .await;
            let response = read_test_message(&mut reader).await;
            assert_eq!(response["id"], 42);
            assert_eq!(response["result"][0]["autoUseWorkspaceTsdk"], true);
            assert_eq!(response["result"][1]["maxTsServerMemory"], 4096);
            assert!(response["result"][2].is_null());
            assert_eq!(response["result"][3], configuration);
        });
        client
            .initialize(Url::parse("file:///tmp/project/").unwrap(), json!({}))
            .await
            .unwrap();
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
            json!({}),
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
    async fn missing_executable_has_a_stable_error_code() {
        let root = tempfile::tempdir().unwrap();
        let result = connect(
            ConnectionConfig {
                command: "definitely-not-a-real-suncode-language-server".into(),
                arguments: Vec::new(),
                working_directory: root.path().to_path_buf(),
                environment: BTreeMap::new(),
                initialization_options: json!({}),
                configuration: json!({}),
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

    #[tokio::test]
    async fn dropping_the_last_client_releases_an_idle_connection() {
        let (client_stream, server_stream) = duplex(1024);
        let (client_reader, client_writer) = split(client_stream);
        let client = Client::from_io(
            client_reader,
            client_writer,
            None,
            Duration::from_secs(1),
            json!({}),
            Vec::new(),
        );
        let weak = Arc::downgrade(&client.inner);
        drop(client);
        assert!(weak.upgrade().is_none());
        // The idle read loop must also drop its reader half; once both halves are
        // gone the server side observes end of stream instead of hanging.
        let mut server_stream = server_stream;
        let mut buffer = [0_u8; 1];
        let read = tokio::time::timeout(Duration::from_secs(1), server_stream.read(&mut buffer))
            .await
            .expect("idle read loop kept the connection alive")
            .unwrap();
        assert_eq!(read, 0);
    }
}
