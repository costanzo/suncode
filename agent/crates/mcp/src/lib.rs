//! MCP client adapter. No rmcp type crosses this crate's public boundary.
//!
//! - `config`: caller-facing connection and TLS settings.
//! - `connection`: the `Connection` trait and `connect` entry point.
//! - `client`: the rmcp-backed `Connection` implementation.
//! - `transport`: stdio and Streamable HTTP transport startup.
//! - `tool`: tool definitions and bounded result normalization.
//! - `error`: MCP error construction and credential redaction.

mod client;
mod config;
mod connection;
mod error;
mod provider;
mod runtime;
mod spec;
mod tool;
mod transport;

pub use config::{ConnectionConfig, TlsConfig};
pub use connection::{connect, Connected, Connection};
pub use runtime::{is_mcp_tool, McpLoadProgress, McpRuntime, McpRuntimeState, McpRuntimeStatus};
pub use spec::{ServerSource, ServerSpec, ServerTransport, WorkingDirectory};
pub use tool::{tool_result_value, ToolDefinition, ToolResult};
