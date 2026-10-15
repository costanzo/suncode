use crate::{
    client::{Handler, RmcpConnection},
    config::ConnectionConfig,
    tool::{ToolDefinition, ToolResult},
    transport,
};
use futures_util::future::BoxFuture;
use serde_json::Value;
use std::sync::Arc;
use suncode_common::BusinessError;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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

pub async fn connect(config: ConnectionConfig) -> Result<Connected, BusinessError> {
    let startup_timeout = config.startup_timeout();
    let request_timeout = config.request_timeout();
    let secret_values = config.secret_values();
    let (changes_tx, changes_rx) = mpsc::unbounded_channel();
    let handler = Handler::new(changes_tx);
    let service = match config {
        ConnectionConfig::Stdio {
            command,
            arguments,
            working_directory,
            environment,
            ..
        } => {
            let transport =
                transport::stdio::transport(command, arguments, &working_directory, &environment)?;
            transport::handshake(handler, transport, startup_timeout, &secret_values).await?
        }
        ConnectionConfig::StreamableHttp {
            url, headers, tls, ..
        } => {
            let transport = transport::http::transport(url, headers, &tls)?;
            transport::handshake(handler, transport, startup_timeout, &secret_values).await?
        }
    };
    Ok(Connected {
        connection: Arc::new(RmcpConnection::new(service, request_timeout, secret_values)),
        tool_list_changes: changes_rx,
    })
}
