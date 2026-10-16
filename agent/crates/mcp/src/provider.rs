//! [`McpRuntime`] as a tool provider. Every MCP tool is an external tool
//! whose approval is pinned to the connection generation the user saw.

use crate::runtime::{is_mcp_tool, McpRuntime};
use serde_json::{json, Value};
use suncode_common::tool::{
    ApprovalTarget, BoxFuture, Risk, ToolContext, ToolDefinition, ToolInvocation, ToolProvider,
};
use suncode_common::BusinessError;
use tokio_util::sync::CancellationToken;

impl ToolProvider for McpRuntime {
    fn owns(&self, name: &str) -> bool {
        is_mcp_tool(name)
    }

    fn catalog<'a>(&'a self, project_id: &'a str) -> BoxFuture<'a, Vec<ToolDefinition>> {
        Box::pin(McpRuntime::catalog(self, project_id))
    }

    /// Schema validation needs the live catalog, so it happens in
    /// [`ToolProvider::approval_target`] and again before the call.
    fn validate(&self, _name: &str, _arguments: &Value) -> Result<(), BusinessError> {
        Ok(())
    }

    fn risk(&self, _name: &str) -> Option<Risk> {
        Some(Risk::ExternalTool)
    }

    fn approval_target<'a>(
        &'a self,
        project_id: &'a str,
        name: &'a str,
        arguments: &'a Value,
    ) -> BoxFuture<'a, Result<ApprovalTarget, BusinessError>> {
        Box::pin(async move {
            let target = McpRuntime::approval_target(self, project_id, name, arguments)
                .await?
                .ok_or_else(tool_unavailable)?;
            Ok(ApprovalTarget {
                operation: target.label,
                arguments: json!({
                    "serverId": target.server_id,
                    "toolName": target.remote_name,
                    "arguments": arguments,
                }),
                // A bare generation number; snapshots taken before the generic
                // pin stored the same value.
                pin: Some(json!(target.generation)),
            })
        })
    }

    fn execute<'a>(
        &'a self,
        invocation: ToolInvocation<'a>,
        context: ToolContext<'a>,
    ) -> BoxFuture<'a, Result<Value, BusinessError>> {
        Box::pin(async move {
            let expected_generation = invocation.approval_pin.as_ref().and_then(Value::as_u64);
            // The connection API takes a Tokio token; bridge the call's
            // runtime-neutral cancellation into one.
            let token = CancellationToken::new();
            let call = self.call(
                context.project_id,
                invocation.name,
                invocation.arguments,
                token.clone(),
                expected_generation,
            );
            tokio::pin!(call);
            tokio::select! {
                result = &mut call => result,
                () = context.cancellation.cancelled() => {
                    token.cancel();
                    call.await
                }
            }
        })
    }
}

fn tool_unavailable() -> BusinessError {
    BusinessError::new(
        "mcp_tool_unavailable",
        "MCP tool is no longer available for this project",
    )
}
