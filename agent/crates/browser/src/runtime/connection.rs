//! CDP connection to the host-owned CEF page, starting CEF on demand.

use super::storage::project_directory_name;
use super::BrowserRuntime;
use crate::cdp::{CdpLaunch, CdpSession, PageTarget};
use std::sync::Arc;
use std::time::Duration;
use suncode_common::{logging, BusinessError};

/// Fast path for an already-running CEF.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
/// After asking the host to initialize CEF and create the page.
const HOST_START_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Host scope used when verifying CEF without a project page.
const VERIFY_SCOPE: &str = "verify";

impl BrowserRuntime {
    /// Attaches to the project's page, reusing its last known target so the
    /// page is found again after it navigates away from its initial URL.
    pub(super) async fn connect_project(
        &self,
        project_id: &str,
    ) -> Result<Arc<CdpSession>, BusinessError> {
        let page = PageTarget {
            initial_url: Some(browser_target_url(project_id)),
            target_id: self.known_target(project_id),
        };
        let session = self.connect(project_id, page).await?;
        self.remember_target(project_id, &session);
        Ok(session)
    }

    /// Attaches to any CEF page to prove the endpoint is reachable.
    pub(super) async fn connect_for_verification(&self) -> Result<Arc<CdpSession>, BusinessError> {
        self.connect(VERIFY_SCOPE, PageTarget::default()).await
    }

    /// Connects directly; if that fails, asks the host to start CEF for `scope`
    /// and retries with a longer timeout.
    async fn connect(
        &self,
        scope: &str,
        page: PageTarget,
    ) -> Result<Arc<CdpSession>, BusinessError> {
        logging::debug(
            "browser.cdp",
            format!("connect_begin port={}", self.inner.cdp_port),
        );
        let launch = |startup_timeout| {
            CdpSession::connect(CdpLaunch {
                port: self.inner.cdp_port,
                page: page.clone(),
                startup_timeout,
                request_timeout: REQUEST_TIMEOUT,
            })
        };
        let first_error = match launch(CONNECT_TIMEOUT).await {
            Ok(session) => return Ok(Arc::new(session)),
            Err(error) => error,
        };
        logging::write_business_error(
            "browser.cdp",
            "connect",
            &first_error,
            format!("phase=initial project={scope}"),
        );
        let Some(callback) = self
            .inner
            .host_callback
            .read()
            .ok()
            .and_then(|value| value.clone())
        else {
            return Err(first_error);
        };
        logging::info("browser.host", format!("start_requested project={scope}"));
        let callback_scope = scope.to_owned();
        tokio::task::spawn_blocking(move || callback(&callback_scope))
            .await
            .map_err(|error| {
                BusinessError::unavailable(format!("browser host callback failed: {error}"))
            })??;
        match launch(HOST_START_TIMEOUT).await {
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

    fn known_target(&self, project_id: &str) -> Option<String> {
        self.inner
            .page_targets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(project_id)
            .cloned()
    }

    fn remember_target(&self, project_id: &str, session: &CdpSession) {
        self.inner
            .page_targets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(project_id.to_owned(), session.target_id().to_owned());
    }
}

/// Initial URL the desktop host loads into a project's Browser Use page.
/// Must match `CefPreviewRuntime.BrowserTargetUrl` in the Avalonia client.
pub(super) fn browser_target_url(project_id: &str) -> String {
    format!(
        "data:text/html,%3Ctitle%3Esuncode-browser-use-{}%3C%2Ftitle%3E",
        project_directory_name(project_id)
    )
}
