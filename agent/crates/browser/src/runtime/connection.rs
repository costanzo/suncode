//! CDP connection to the host-owned CEF page, starting CEF on demand.

use super::storage::project_directory_name;
use super::BrowserRuntime;
use crate::cdp::{CdpLaunch, CdpSession};
use std::sync::Arc;
use std::time::Duration;
use suncode_common::{logging, BusinessError};

impl BrowserRuntime {
    pub(super) async fn connect_session(
        &self,
        scope: &str,
    ) -> Result<Arc<CdpSession>, BusinessError> {
        logging::debug(
            "browser.cdp",
            format!("connect_begin port={}", self.inner.cdp_port),
        );
        let target_url = (scope != "verify").then(|| browser_target_url(scope));
        let launch = |startup_timeout| {
            CdpSession::connect(CdpLaunch {
                port: self.inner.cdp_port,
                target_url: target_url.clone(),
                startup_timeout,
                request_timeout: Duration::from_secs(60),
            })
        };
        match launch(Duration::from_secs(2)).await {
            Ok(session) => Ok(Arc::new(session)),
            Err(first_error) => {
                logging::write_business_error(
                    "browser.cdp",
                    "connect",
                    &first_error,
                    format!("phase=initial project={scope}"),
                );
                let callback = self
                    .inner
                    .host_callback
                    .read()
                    .ok()
                    .and_then(|value| value.clone());
                let Some(callback) = callback else {
                    return Err(first_error);
                };
                logging::info("browser.host", format!("start_requested project={scope}"));
                let scope = scope.to_owned();
                let callback_scope = scope.clone();
                tokio::task::spawn_blocking(move || callback(&callback_scope))
                    .await
                    .map_err(|error| {
                        BusinessError::unavailable(format!("browser host callback failed: {error}"))
                    })??;
                match launch(Duration::from_secs(15)).await {
                    Ok(session) => {
                        logging::info("browser.cdp", "connected_after_host_start=true");
                        Ok(Arc::new(session))
                    }
                    Err(error) => {
                        logging::write_business_error(
                            "browser.cdp",
                            "connect_after_host_start",
                            &error,
                            format!("project={scope}"),
                        );
                        Err(error)
                    }
                }
            }
        }
    }
}

pub(super) fn browser_target_url(project_id: &str) -> String {
    format!(
        "data:text/html,%3Ctitle%3Esuncode-browser-use-{}%3C%2Ftitle%3E",
        project_directory_name(project_id)
    )
}
