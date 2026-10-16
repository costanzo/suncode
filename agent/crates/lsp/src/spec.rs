//! Desired language-server configuration, supplied by the host.
//!
//! The runtime does not read persistence directly. The host implements
//! [`ServerSource`] over its own store and converts records into these types.

use serde_json::Value;
use std::{collections::BTreeMap, time::Duration};
use suncode_common::BusinessError;

/// One configured language server.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerSpec {
    pub server_id: String,
    pub enabled: bool,
    /// Changes whenever the configuration changes; a running server whose
    /// revision no longer matches is restarted.
    pub revision: u64,
    pub command: String,
    pub arguments: Vec<String>,
    /// Language ids this server handles, compared case-insensitively.
    pub language_ids: Vec<String>,
    /// Files whose presence at the project root selects this server; empty
    /// means any project.
    pub root_markers: Vec<String>,
    pub initialization_options: Value,
    pub environment: BTreeMap<String, String>,
    pub startup_timeout: Duration,
    pub request_timeout: Duration,
}

/// Supplies desired language-server configuration.
pub trait ServerSource: Send + Sync {
    fn servers(&self) -> Result<Vec<ServerSpec>, BusinessError>;
    fn server(&self, server_id: &str) -> Result<Option<ServerSpec>, BusinessError>;
}
