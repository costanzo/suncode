//! Desired MCP server configuration, supplied by the host.
//!
//! The runtime does not read persistence directly. The host implements
//! [`ServerSource`] over its own store and converts records into these types.

use crate::config::TlsConfig;
use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use suncode_common::BusinessError;

/// One configured MCP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSpec {
    pub server_id: String,
    pub display_name: String,
    /// Prefix in exposed tool names: `mcp__<prefix>__<remote name>`.
    pub tool_prefix: String,
    pub enabled: bool,
    /// Changes whenever the configuration changes; stale connections whose
    /// revision no longer matches are discarded.
    pub revision: u64,
    pub transport: ServerTransport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerTransport {
    Stdio {
        command: String,
        arguments: Vec<String>,
        working_directory: WorkingDirectory,
        environment: BTreeMap<String, String>,
        startup_timeout: Duration,
        request_timeout: Duration,
    },
    StreamableHttp {
        url: String,
        headers: BTreeMap<String, String>,
        startup_timeout: Duration,
        request_timeout: Duration,
    },
}

impl ServerTransport {
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::StreamableHttp { .. })
    }
}

/// Where a local server process starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingDirectory {
    /// The opened project root.
    Project,
    /// The host's application data directory.
    ApplicationData,
}

/// Supplies desired server configuration and outbound network settings.
pub trait ServerSource: Send + Sync {
    fn servers(&self) -> Result<Vec<ServerSpec>, BusinessError>;
    fn server(&self, server_id: &str) -> Result<Option<ServerSpec>, BusinessError>;
    /// TLS and proxy settings for remote servers, read at connection time.
    fn network(&self) -> Result<TlsConfig, BusinessError>;
    /// Working directory for [`WorkingDirectory::ApplicationData`].
    fn application_data(&self) -> PathBuf;
}
