mod client;
mod config;
mod controller;
mod crypto;
mod events;
mod sse;
mod worker;

use serde_json::Value;
use std::{future::Future, pin::Pin};
use suncode_common::{BusinessError, HttpProxyConfiguration};
use tokio::sync::mpsc;

pub use config::{RemoteServerConfiguration, RemoteServerStatus};
pub use controller::RemoteController;
pub use events::should_upload_event;

pub type RemoteResult<T> = Result<T, BusinessError>;

#[derive(Clone)]
pub struct RemoteNetworkConfiguration {
    pub verify_https_certificates: bool,
    pub use_system_certificates: bool,
    pub certificate_path: Option<std::path::PathBuf>,
    pub proxy: HttpProxyConfiguration,
}

#[derive(Debug, Clone)]
pub struct RemoteSessionEvent {
    pub session_id: String,
    pub occurred_at: String,
    pub event_type: String,
    pub payload: Value,
}

/// Agent-owned operations used by the transport worker. The transport crate
/// never reaches into SDK DTOs or persistence semantics.
pub trait RemoteHost: Send + Sync {
    fn network_configuration(&self) -> RemoteNetworkConfiguration;
    fn snapshot(&self) -> RemoteResult<Value>;
    fn subscribe_session_events(
        &self,
        session_id: &str,
    ) -> RemoteResult<mpsc::Receiver<RemoteResult<RemoteSessionEvent>>>;
    fn dispatch<'a>(
        &'a self,
        request_id: &'a str,
        operation: &'a str,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = RemoteResult<Value>> + Send + 'a>>;
}

mod logging {
    pub fn debug(_scope: &str, _message: impl std::fmt::Display) {}
    pub fn warn(_scope: &str, _message: impl std::fmt::Display) {}
    pub fn error(_scope: &str, _message: impl std::fmt::Display) {}
    pub fn write_business_error(
        _scope: &str,
        _operation: &str,
        _error: &suncode_common::BusinessError,
        _context: impl std::fmt::Display,
    ) {
    }
}
