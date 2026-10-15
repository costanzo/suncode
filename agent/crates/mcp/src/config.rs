use std::{collections::BTreeMap, path::PathBuf, time::Duration};
use suncode_common::HttpProxyConfiguration;

#[derive(Clone)]
pub struct TlsConfig {
    pub verify_certificates: bool,
    pub use_system_certificates: bool,
    pub certificate_path: Option<PathBuf>,
    pub proxy: HttpProxyConfiguration,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_certificates: true,
            use_system_certificates: true,
            certificate_path: None,
            proxy: HttpProxyConfiguration::default(),
        }
    }
}

#[derive(Clone)]
pub enum ConnectionConfig {
    Stdio {
        command: String,
        arguments: Vec<String>,
        working_directory: PathBuf,
        environment: BTreeMap<String, String>,
        startup_timeout: Duration,
        request_timeout: Duration,
    },
    StreamableHttp {
        url: String,
        headers: BTreeMap<String, String>,
        startup_timeout: Duration,
        request_timeout: Duration,
        tls: TlsConfig,
    },
}

impl ConnectionConfig {
    pub(crate) fn startup_timeout(&self) -> Duration {
        match self {
            Self::Stdio {
                startup_timeout, ..
            }
            | Self::StreamableHttp {
                startup_timeout, ..
            } => *startup_timeout,
        }
    }

    pub(crate) fn request_timeout(&self) -> Duration {
        match self {
            Self::Stdio {
                request_timeout, ..
            }
            | Self::StreamableHttp {
                request_timeout, ..
            } => *request_timeout,
        }
    }

    /// Configured environment and header values, redacted from any error text.
    pub(crate) fn secret_values(&self) -> Vec<String> {
        let values = match self {
            Self::Stdio { environment, .. } => environment.values(),
            Self::StreamableHttp { headers, .. } => headers.values(),
        };
        values.filter(|value| !value.is_empty()).cloned().collect()
    }
}
