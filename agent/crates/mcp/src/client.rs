//! rmcp-backed `Connection` implementation.

use crate::{
    connection::Connection,
    error::{mcp_error, mcp_service_error},
    tool::{ToolDefinition, ToolResult},
};
use futures_util::future::BoxFuture;
use rmcp::{
    model::{CallToolRequestParams, ClientInfo, Implementation, PaginatedRequestParams},
    service::{NotificationContext, RunningService},
    ClientHandler, RoleClient,
};
use serde_json::Value;
use std::time::Duration;
use suncode_common::BusinessError;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const MAX_TOOLS_PER_SERVER: usize = 128;
const CLOSE_TIMEOUT: Duration = Duration::from_secs(3);

pub(crate) type ClientService = RunningService<RoleClient, Handler>;

#[derive(Clone)]
pub(crate) struct Handler {
    tool_list_changes: mpsc::UnboundedSender<()>,
}

impl Handler {
    pub(crate) fn new(tool_list_changes: mpsc::UnboundedSender<()>) -> Self {
        Self { tool_list_changes }
    }
}

impl ClientHandler for Handler {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("SunCode", env!("CARGO_PKG_VERSION")),
        )
    }

    async fn on_tool_list_changed(&self, _context: NotificationContext<RoleClient>) {
        let _ = self.tool_list_changes.send(());
    }
}

pub(crate) struct RmcpConnection {
    service: Mutex<ClientService>,
    request_timeout: Duration,
    secret_values: Vec<String>,
}

impl RmcpConnection {
    pub(crate) fn new(
        service: ClientService,
        request_timeout: Duration,
        secret_values: Vec<String>,
    ) -> Self {
        Self {
            service: Mutex::new(service),
            request_timeout,
            secret_values,
        }
    }
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
                            format!("MCP server exposes more than {MAX_TOOLS_PER_SERVER} tools"),
                        ));
                    }
                    tools.push(ToolDefinition::from_rmcp(tool)?);
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
            ToolResult::from_rmcp(result)
        })
    }

    fn close(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            let mut service = self.service.lock().await;
            let _ = service.close_with_timeout(CLOSE_TIMEOUT).await;
        })
    }

    fn is_closed(&self) -> bool {
        self.service
            .try_lock()
            .map(|service| service.is_closed())
            .unwrap_or(false)
    }
}
