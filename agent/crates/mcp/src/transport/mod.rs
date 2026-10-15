//! Transport startup. Each transport produces an rmcp transport that
//! `handshake` turns into a running client service.

pub(crate) mod http;
pub(crate) mod stdio;

use crate::{
    client::{ClientService, Handler},
    error::{mcp_error, redact_business_error},
};
use rmcp::{service::ServiceExt, transport::IntoTransport, RoleClient};
use std::time::Duration;
use suncode_common::BusinessError;

pub(crate) async fn handshake<T, E, A>(
    handler: Handler,
    transport: T,
    startup_timeout: Duration,
    secret_values: &[String],
) -> Result<ClientService, BusinessError>
where
    T: IntoTransport<RoleClient, E, A>,
    E: std::error::Error + Send + Sync + 'static,
{
    tokio::time::timeout(startup_timeout, handler.serve(transport))
        .await
        .map_err(|_| mcp_error("mcp_startup_timeout", "MCP server startup timed out"))?
        .map_err(|error| {
            redact_business_error(
                mcp_error("mcp_handshake_failed", error.to_string()),
                secret_values,
            )
        })
}
