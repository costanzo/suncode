//! Discovery of the CEF page target that a session attaches to.

use crate::cdp::error::{browser_error, CDP_NO_PAGE, CDP_PROTOCOL_ERROR};
use crate::cdp::launch::{CdpLaunch, PageTarget};
use chromiumoxide::{Browser, Page};
use std::time::Duration;
use suncode_common::BusinessError;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Polls CEF until a page matching `launch.page` appears or the startup
/// timeout elapses. CEF may expose the page shortly after the endpoint is up.
pub(crate) async fn discover_page(
    browser: &Browser,
    launch: &CdpLaunch,
) -> Result<Page, BusinessError> {
    let target = &launch.page;
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
            let id = page.target_id().as_ref().to_owned();
            let url = page.url().await.ok().flatten().unwrap_or_default();
            candidates.push((page, Candidate { id, url }));
        }
        log_candidates(&candidates, target);
        let summaries: Vec<&Candidate> =
            candidates.iter().map(|(_, candidate)| candidate).collect();
        if let Some(index) = select_page(&summaries, target) {
            return Ok(candidates.swap_remove(index).0);
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    suncode_common::logging::warn(
        "browser.cdp",
        format!(
            "page_discovery_failed reason=target_not_found by_id={} by_url={} last_count={last_page_count}",
            target.target_id.is_some(),
            target.initial_url.is_some()
        ),
    );
    Err(browser_error(
        CDP_NO_PAGE,
        "CEF did not expose the requested browser page target",
    ))
}

struct Candidate {
    id: String,
    url: String,
}

/// Prefers the previously attached target, then the page still showing the
/// initial URL; with no target, the first page.
fn select_page(candidates: &[&Candidate], target: &PageTarget) -> Option<usize> {
    if target.target_id.is_none() && target.initial_url.is_none() {
        return (!candidates.is_empty()).then_some(0);
    }
    let by_id = target
        .target_id
        .as_deref()
        .and_then(|id| candidates.iter().position(|candidate| candidate.id == id));
    by_id.or_else(|| {
        let url = target.initial_url.as_deref()?;
        candidates.iter().position(|candidate| candidate.url == url)
    })
}

fn log_candidates(candidates: &[(Page, Candidate)], target: &PageTarget) {
    // URLs are logged by length only; they may contain user data.
    let summary: Vec<_> = candidates
        .iter()
        .map(|(_, candidate)| {
            (
                candidate.id.as_str(),
                candidate.url.len(),
                target.target_id.as_deref() == Some(candidate.id.as_str()),
                target.initial_url.as_deref() == Some(candidate.url.as_str()),
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
    use super::*;

    fn candidates() -> Vec<Candidate> {
        vec![
            Candidate {
                id: "a".into(),
                url: "about:blank".into(),
            },
            Candidate {
                id: "b".into(),
                url: "https://example.test/".into(),
            },
            Candidate {
                id: "c".into(),
                url: "data:text/html,c".into(),
            },
        ]
    }

    fn select(target: PageTarget) -> Option<usize> {
        let candidates = candidates();
        let refs: Vec<&Candidate> = candidates.iter().collect();
        select_page(&refs, &target)
    }

    #[test]
    fn selects_initial_url() {
        assert_eq!(
            select(PageTarget {
                initial_url: Some("data:text/html,c".into()),
                target_id: None
            }),
            Some(2)
        );
    }

    #[test]
    fn known_target_id_survives_navigation() {
        let target = PageTarget {
            initial_url: Some("data:text/html,b".into()),
            target_id: Some("b".into()),
        };
        assert_eq!(select(target), Some(1));
    }

    #[test]
    fn falls_back_to_initial_url_when_target_is_gone() {
        let target = PageTarget {
            initial_url: Some("data:text/html,c".into()),
            target_id: Some("gone".into()),
        };
        assert_eq!(select(target), Some(2));
    }

    #[test]
    fn missing_target_selects_nothing() {
        assert_eq!(
            select(PageTarget {
                initial_url: Some("data:x".into()),
                target_id: None
            }),
            None
        );
    }

    #[test]
    fn untargeted_selects_first_page() {
        assert_eq!(select(PageTarget::default()), Some(0));
        assert_eq!(select_page(&[], &PageTarget::default()), None);
    }
}
