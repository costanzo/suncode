//! The `webfetch` tool: bounded, origin-limited HTTP(S) text retrieval.
//!
//! - `client`: client construction, redirect origin policy, bounded reads.
//! - `convert`: HTML to Markdown or plain text.

mod client;
mod convert;

use crate::arguments::WebfetchArguments;
use crate::artifacts;
use crate::context::NetworkSettings;
use crate::error::fatal;
use encoding_rs::{Encoding, UTF_8};
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Url,
};
use serde_json::{json, Value};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use suncode_common::BusinessError;

/// Fallback user agent for a retry after a Cloudflare browser challenge.
const FALLBACK_USER_AGENT: &str = "suncode";
const BINARY_SNIFF_BYTES: usize = 4096;
const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;
const MAX_MODEL_BYTES: usize = 64 * 1024;
const DEFAULT_TIMEOUT_SECONDS: f64 = 30.0;
const MAX_TIMEOUT_SECONDS: f64 = 120.0;
const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Markdown,
    Html,
}

impl OutputFormat {
    fn parse(value: Option<&str>) -> Result<Self, BusinessError> {
        match value.unwrap_or("markdown") {
            "text" => Ok(Self::Text),
            "markdown" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            _ => Err(invalid("format must be text, markdown, or html")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Markdown => "markdown",
            Self::Html => "html",
        }
    }

    fn accept(self) -> &'static str {
        match self {
            Self::Markdown => {
                "text/markdown;q=1.0, text/x-markdown;q=0.9, text/plain;q=0.8, text/html;q=0.7, */*;q=0.1"
            }
            Self::Text => {
                "text/plain;q=1.0, text/markdown;q=0.9, text/html;q=0.8, */*;q=0.1"
            }
            Self::Html => {
                "text/html;q=1.0, application/xhtml+xml;q=0.9, text/plain;q=0.8, text/markdown;q=0.7, */*;q=0.1"
            }
        }
    }
}

pub(crate) fn execute(
    checkpoint_root: Option<&Path>,
    args: WebfetchArguments,
    cancellation: Option<&AtomicBool>,
    network: &NetworkSettings,
) -> Result<Value, BusinessError> {
    let url = args.url.trim();
    if url.is_empty() {
        return Err(invalid("url is required"));
    }
    let url = parse_http_url(url)?;
    let format = OutputFormat::parse(args.format.as_deref())?;
    let timeout = timeout(args.timeout)?;
    let deadline = Instant::now() + timeout;
    check_cancelled(cancellation)?;

    let client = client::build(&url, network)?;

    let mut response = client::send(
        &client,
        url.clone(),
        format,
        BROWSER_USER_AGENT,
        remaining(deadline)?,
    )?;
    if response.status().as_u16() == 403
        && response
            .headers()
            .get("cf-mitigated")
            .and_then(|value| value.to_str().ok())
            == Some("challenge")
    {
        response = client::send(
            &client,
            url.clone(),
            format,
            FALLBACK_USER_AGENT,
            remaining(deadline)?,
        )?;
    }
    if !response.status().is_success() {
        return Err(fetch_failure(
            "server returned an unsuccessful status",
            true,
        ));
    }
    if response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(fetch_failure("response exceeds the 5 MiB limit", false));
    }

    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if !is_textual_mime(&mime) {
        return Err(fetch_failure("response content type is not textual", false));
    }

    let body = client::read_bounded(&mut response, cancellation)?;
    if mime.is_empty() && looks_binary(&body) {
        return Err(fetch_failure("response content type is not textual", false));
    }
    let text = decode_body(&body, &content_type);
    let output = if is_html_mime(&mime) {
        convert::convert_html(&text, format)?
    } else {
        text
    };
    let (preview, truncated) = utf8_prefix(&output, MAX_MODEL_BYTES);
    let mut result = json!({
        "url": final_url,
        "content_type": content_type,
        "format": format.as_str(),
        "content": preview,
        "bytes": output.len(),
        "truncated": truncated
    });
    if truncated {
        let root = checkpoint_root.ok_or(fatal(
            "artifact_unavailable",
            "artifact storage is not configured",
        ))?;
        result["artifact_id"] = json!(artifacts::write_artifact(root, output.as_bytes())?);
    }
    Ok(result)
}

fn parse_http_url(value: &str) -> Result<Url, BusinessError> {
    let url = Url::parse(value).map_err(|_| invalid("url must be a valid HTTP or HTTPS URL"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid("url must use http or https"));
    }
    if url.host_str().is_none() {
        return Err(invalid("url must include a host"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(invalid("url must not contain embedded credentials"));
    }
    Ok(url)
}

fn decode_body(bytes: &[u8], content_type: &str) -> String {
    let bom_encoding = Encoding::for_bom(bytes).map(|(encoding, _)| encoding);
    let charset = content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.trim().split_once('=')?;
        name.eq_ignore_ascii_case("charset")
            .then(|| value.trim_matches([' ', '"', '\'']).as_bytes())
    });
    let encoding = bom_encoding
        .or_else(|| charset.and_then(Encoding::for_label))
        .unwrap_or(UTF_8);
    encoding.decode(bytes).0.into_owned()
}

fn timeout(value: Option<f64>) -> Result<Duration, BusinessError> {
    let seconds = value.unwrap_or(DEFAULT_TIMEOUT_SECONDS);
    if !seconds.is_finite() {
        return Err(invalid("timeout must be a number of seconds"));
    }
    if seconds <= 0.0 || seconds > MAX_TIMEOUT_SECONDS {
        return Err(invalid(
            "timeout must be greater than zero and no more than 120 seconds",
        ));
    }
    Ok(Duration::from_secs_f64(seconds))
}

fn remaining(deadline: Instant) -> Result<Duration, BusinessError> {
    let timeout = deadline.saturating_duration_since(Instant::now());
    if timeout.is_zero() {
        return Err(fetch_failure("request timed out", true));
    }
    Ok(timeout)
}

/// Textual media types. An empty type is accepted only when the body is
/// checked with `looks_binary`; images, including SVG, are always rejected.
fn is_textual_mime(mime: &str) -> bool {
    if mime.starts_with("image/") {
        return false;
    }
    mime.is_empty()
        || mime.starts_with("text/")
        || mime == "application/json"
        || mime.ends_with("+json")
        || mime == "application/xml"
        || mime.ends_with("+xml")
        || mime == "application/javascript"
        || mime == "application/x-javascript"
}

fn is_html_mime(mime: &str) -> bool {
    mime == "text/html" || mime == "application/xhtml+xml"
}

/// A NUL byte in the first block marks content as binary, matching the
/// heuristic used for grep.
fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(BINARY_SNIFF_BYTES).any(|byte| *byte == 0)
}

fn utf8_prefix(value: &str, max_bytes: usize) -> (&str, bool) {
    if value.len() <= max_bytes {
        return (value, false);
    }
    let mut boundary = max_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    (&value[..boundary], true)
}

fn check_cancelled(cancellation: Option<&AtomicBool>) -> Result<(), BusinessError> {
    if cancellation.is_some_and(|value| value.load(Ordering::Relaxed)) {
        return Err(fatal("cancelled", "web fetch was cancelled"));
    }
    Ok(())
}

fn map_request_error(error: reqwest::Error) -> BusinessError {
    if error.is_timeout() {
        fetch_failure("request timed out", true)
    } else {
        fetch_failure("request could not be completed", true)
    }
}

fn invalid(message: &'static str) -> BusinessError {
    BusinessError::invalid(message)
}

fn fetch_failure(message: &'static str, retryable: bool) -> BusinessError {
    BusinessError::new("webfetch_failed", message).with_retryable(retryable)
}

#[cfg(test)]
mod tests;
