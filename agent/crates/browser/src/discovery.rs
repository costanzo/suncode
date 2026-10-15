//! Discovery of the CEF page target that a session attaches to.

use crate::error::{browser_error, CDP_NO_PAGE, CDP_PROTOCOL_ERROR};
use crate::launch::CdpLaunch;
use chromiumoxide::{Browser, Page};
use std::time::Duration;
use suncode_common::BusinessError;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Polls CEF until a page matching `launch.target_url` appears or the startup
/// timeout elapses. CEF may expose the page shortly after the endpoint is up.
pub(crate) async fn discover_page(
    browser: &Browser,
    launch: &CdpLaunch,
) -> Result<Page, BusinessError> {
    let target_url = launch.target_url.as_deref();
    let deadline = tokio::time::Instant::now() + launch.startup_timeout;
    let mut last_page_count = 0usize;
    while tokio::time::Instant::now() < deadline {
        let pages = browser.pages().await.map_err(|error| {
            browser_error(
                CDP_PROTOCOL_ERROR,
                format!("CEF pages could not be discovered: {error}"),
            )
        })?;
        last_page_count = pages.len();
        let mut candidates = Vec::with_capacity(pages.len());
        for page in pages {
            let url = page.url().await.ok().flatten().unwrap_or_default();
            candidates.push((page, url));
        }
        log_candidates(&candidates, target_url);
        let urls: Vec<&str> = candidates.iter().map(|(_, url)| url.as_str()).collect();
        if let Some(index) = select_page(&urls, target_url) {
            return Ok(candidates.swap_remove(index).0);
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    suncode_common::logging::warn(
        "browser.cdp",
        format!(
            "page_discovery_failed reason=target_not_found expected={target_url:?} last_count={last_page_count}"
        ),
    );
    Err(browser_error(
        CDP_NO_PAGE,
        "CEF did not expose the requested browser page target",
    ))
}

/// Picks the page whose URL equals `target_url`, or the first page when no
/// target is requested.
fn select_page(urls: &[&str], target_url: Option<&str>) -> Option<usize> {
    match target_url {
        Some(target_url) => urls.iter().position(|url| *url == target_url),
        None => (!urls.is_empty()).then_some(0),
    }
}

fn log_candidates(candidates: &[(Page, String)], target_url: Option<&str>) {
    // URLs are logged by length only; they may contain user data.
    let summary: Vec<_> = candidates
        .iter()
        .map(|(page, url)| {
            (
                page.target_id().as_ref().to_owned(),
                url.len(),
                target_url == Some(url.as_str()),
            )
        })
        .collect();
    suncode_common::logging::debug(
        "browser.cdp",
        format!(
            "page_discovery count={} targets={summary:?}",
            candidates.len()
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::select_page;

    #[test]
    fn selects_exact_target_url() {
        let urls = ["about:blank", "data:text/html,a"];
        assert_eq!(select_page(&urls, Some("data:text/html,a")), Some(1));
    }

    #[test]
    fn missing_target_url_selects_nothing() {
        assert_eq!(select_page(&["about:blank"], Some("data:x")), None);
    }

    #[test]
    fn untargeted_selects_first_page() {
        assert_eq!(select_page(&["a", "b"], None), Some(0));
        assert_eq!(select_page(&[], None), None);
    }
}
