use super::super::arguments::WebfetchArguments;
use super::super::{artifacts, BusinessError};
use encoding_rs::{Encoding, UTF_8};
use html2md_rs::{
    parser::safe_parse_html,
    structs::{Node, NodeType, ToMdConfig},
    to_md::to_md_with_config,
};
use reqwest::{
    blocking::{Client, Response},
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT},
    redirect::Policy,
    Url,
};
use serde_json::{json, Value};
use std::{
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

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

pub(super) fn execute(
    checkpoint_root: Option<&Path>,
    args: WebfetchArguments,
    cancellation: Option<&AtomicBool>,
    verify_https_certificates: bool,
    use_system_certificates: bool,
    certificate_path: Option<&Path>,
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

    let allowed_host = url.host_str().unwrap().to_string();
    let allowed_scheme = url.scheme().to_string();
    let allowed_port = url.port_or_known_default().unwrap();
    let mut client_builder = Client::builder()
        .danger_accept_invalid_certs(!verify_https_certificates)
        .danger_accept_invalid_hostnames(!verify_https_certificates)
        .tls_built_in_root_certs(use_system_certificates)
        .redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= 10 {
                return attempt.error("too many redirects");
            }
            let target = attempt.url();
            let same_host = target.host_str() == Some(allowed_host.as_str());
            let target_port = target.port_or_known_default();
            let same_endpoint =
                target.scheme() == allowed_scheme && target_port == Some(allowed_port);
            let standard_https_upgrade = allowed_scheme == "http"
                && allowed_port == 80
                && target.scheme() == "https"
                && target_port == Some(443);
            let has_credentials = !target.username().is_empty() || target.password().is_some();
            if same_host
                && (same_endpoint || standard_https_upgrade)
                && matches!(target.scheme(), "http" | "https")
                && !has_credentials
            {
                attempt.follow()
            } else {
                attempt.error("redirect target is outside the approved web origin")
            }
        }));
    if verify_https_certificates && !use_system_certificates {
        if let Some(path) = certificate_path {
            let bytes = std::fs::read(path).map_err(|_| {
                BusinessError::new("certificate_unavailable", "certificate file is unavailable")
                    .with_retryable(false)
            })?;
            let certificate = reqwest::Certificate::from_pem(&bytes)
                .or_else(|_| reqwest::Certificate::from_der(&bytes))
                .map_err(|_| {
                    BusinessError::new(
                        "certificate_invalid",
                        "certificate file is not a valid PEM or DER certificate",
                    )
                    .with_retryable(false)
                })?;
            client_builder = client_builder.add_root_certificate(certificate);
        }
    }
    let client = client_builder
        .build()
        .map_err(|_| fetch_failure("web client could not be created", false))?;

    let mut response = send(
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
        response = send(
            &client,
            url.clone(),
            format,
            "suncode",
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

    let body = read_bounded(&mut response, cancellation)?;
    let text = decode_body(&body, &content_type);
    let output = if content_type.to_ascii_lowercase().contains("text/html") {
        convert_html(&text, format)?
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
        let root = checkpoint_root.ok_or(
            BusinessError::new("artifact_unavailable", "artifact storage is not configured")
                .with_retryable(false),
        )?;
        result["artifact_id"] = json!(artifacts::write_artifact(root, output.as_bytes())?);
    }
    Ok(result)
}

fn send(
    client: &Client,
    url: Url,
    format: OutputFormat,
    user_agent: &str,
    timeout: Duration,
) -> Result<Response, BusinessError> {
    client
        .get(url)
        .header(USER_AGENT, user_agent)
        .header(ACCEPT, format.accept())
        .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
        .timeout(timeout)
        .send()
        .map_err(map_request_error)
}

fn read_bounded(
    response: &mut Response,
    cancellation: Option<&AtomicBool>,
) -> Result<Vec<u8>, BusinessError> {
    let mut output = Vec::new();
    let mut chunk = [0_u8; 16 * 1024];
    loop {
        check_cancelled(cancellation)?;
        let count = response
            .read(&mut chunk)
            .map_err(|_| fetch_failure("response body could not be read", true))?;
        if count == 0 {
            break;
        }
        if output.len() + count > MAX_RESPONSE_BYTES {
            return Err(fetch_failure("response exceeds the 5 MiB limit", false));
        }
        output.extend_from_slice(&chunk[..count]);
    }
    Ok(output)
}

fn convert_html(input: &str, format: OutputFormat) -> Result<String, BusinessError> {
    if format == OutputFormat::Html {
        return Ok(input.to_string());
    }
    let root = safe_parse_html(input.to_string())
        .map_err(|_| fetch_failure("HTML response could not be parsed", false))?;
    if format == OutputFormat::Markdown {
        let config = ToMdConfig {
            ignore_rendering: ignored_nodes(),
        };
        return Ok(to_md_with_config(root, &config).trim().to_string());
    }
    Ok(extract_text(&root))
}

fn ignored_nodes() -> Vec<NodeType> {
    vec![
        NodeType::Head,
        NodeType::Style,
        NodeType::Link,
        NodeType::Script,
        NodeType::Meta,
        NodeType::Title,
        NodeType::Unknown("noscript".into()),
        NodeType::Unknown("iframe".into()),
        NodeType::Unknown("object".into()),
        NodeType::Unknown("embed".into()),
    ]
}

fn extract_text(root: &Node) -> String {
    fn walk(node: &Node, output: &mut String) {
        let ignored = node
            .tag_name
            .as_ref()
            .is_some_and(|tag| ignored_nodes().contains(tag));
        if ignored {
            return;
        }
        if node.tag_name == Some(NodeType::Text) {
            if let Some(value) = node.value.as_deref() {
                let decoded = html_escape::decode_html_entities(value);
                append_words(output, &decoded);
            }
            return;
        }
        let block = node.tag_name.as_ref().is_some_and(|tag| {
            matches!(
                tag,
                NodeType::Body
                    | NodeType::Div
                    | NodeType::P
                    | NodeType::H1
                    | NodeType::H2
                    | NodeType::H3
                    | NodeType::H4
                    | NodeType::H5
                    | NodeType::H6
                    | NodeType::Li
                    | NodeType::Pre
                    | NodeType::Blockquote
                    | NodeType::Br
                    | NodeType::Hr
            )
        });
        if block {
            append_break(output);
        }
        for child in &node.children {
            walk(child, output);
        }
        if block {
            append_break(output);
        }
    }

    let mut output = String::new();
    walk(root, &mut output);
    output.trim().to_string()
}

fn append_words(output: &mut String, value: &str) {
    for word in value.split_whitespace() {
        if !output.is_empty() && !output.ends_with([' ', '\n']) {
            output.push(' ');
        }
        output.push_str(word);
    }
}

fn append_break(output: &mut String) {
    while output.ends_with(' ') {
        output.pop();
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
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

fn is_textual_mime(mime: &str) -> bool {
    mime.is_empty()
        || mime.starts_with("text/")
        || mime == "application/json"
        || mime.ends_with("+json")
        || mime == "application/xml"
        || mime.ends_with("+xml")
        || mime == "application/javascript"
        || mime == "application/x-javascript"
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
        return Err(
            BusinessError::new("cancelled", "web fetch was cancelled").with_retryable(false)
        );
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
mod tests {
    use super::super::super::arguments::WebfetchArguments;
    use super::*;
    use std::{
        fs,
        io::{Read, Write},
        net::TcpListener,
        path::PathBuf,
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn serve_once(
        content_type: &str,
        body: impl Into<Vec<u8>>,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let content_type = content_type.to_string();
        let body = body.into();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request);
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        });
        (format!("http://{address}/page"), handle)
    }

    fn temporary_checkpoints(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "suncode-webfetch-{name}-{}-{unique}",
            std::process::id()
        ));
        let checkpoints = root.join("checkpoints");
        fs::create_dir_all(&checkpoints).unwrap();
        checkpoints
    }

    fn redirect_once(location: String) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request);
            write!(
                stream,
                "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
        });
        (format!("http://{address}/redirect"), handle)
    }

    #[test]
    fn fetches_html_as_markdown_and_omits_script_content() {
        let (url, server) = serve_once(
            "text/html; charset=utf-8",
            "<html><body><h1>Hello</h1><script>secret()</script><p>World</p></body></html>",
        );
        let result = execute(
            None,
            WebfetchArguments {
                url,
                format: None,
                timeout: None,
            },
            None,
            true,
            true,
            None,
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(result["format"], "markdown");
        assert!(result["content"].as_str().unwrap().contains("# Hello"));
        assert!(result["content"].as_str().unwrap().contains("World"));
        assert!(!result["content"].as_str().unwrap().contains("secret"));
        assert_eq!(result["truncated"], false);
    }

    #[test]
    fn converts_html_to_plain_text() {
        let output = convert_html(
            "<main><h1>Hello &amp; goodbye</h1><p>Second line</p></main>",
            OutputFormat::Text,
        )
        .unwrap();
        assert!(output.contains("Hello & goodbye"));
        assert!(output.contains("Second line"));
        assert!(!output.contains('<'));
    }

    #[test]
    fn rejects_unsupported_urls_formats_and_timeouts() {
        assert!(parse_http_url("file:///tmp/example").is_err());
        assert!(parse_http_url("https://user:secret@example.com").is_err());
        assert!(OutputFormat::parse(Some("pdf")).is_err());
        assert!(timeout(Some(0.0)).is_err());
        assert!(timeout(Some(121.0)).is_err());
    }

    #[test]
    fn decodes_declared_response_charset() {
        assert_eq!(
            decode_body(
                &[0x63, 0x61, 0x66, 0xe9],
                "text/plain; charset=windows-1252"
            ),
            "café"
        );
    }

    #[test]
    fn rejects_non_text_responses() {
        let (url, server) = serve_once("image/png", vec![0_u8, 1, 2, 3]);
        let failure = execute(
            None,
            WebfetchArguments {
                url,
                format: None,
                timeout: None,
            },
            None,
            true,
            true,
            None,
        )
        .unwrap_err();
        server.join().unwrap();
        assert_eq!(failure.code, "webfetch_failed");
        assert_eq!(failure.message, "response content type is not textual");
    }

    #[test]
    fn rejects_cross_origin_redirects() {
        let target = TcpListener::bind("127.0.0.1:0").unwrap();
        let target_url = format!("http://{}/not-approved", target.local_addr().unwrap());
        let (url, redirect_server) = redirect_once(target_url);
        let failure = execute(
            None,
            WebfetchArguments {
                url,
                format: None,
                timeout: None,
            },
            None,
            true,
            true,
            None,
        )
        .unwrap_err();
        redirect_server.join().unwrap();
        assert_eq!(failure.code, "webfetch_failed");
        assert_eq!(failure.message, "request could not be completed");
    }

    #[test]
    fn retains_large_converted_output_as_an_artifact() {
        let checkpoints = temporary_checkpoints("artifact");
        let body = "a".repeat(MAX_MODEL_BYTES + 1024);
        let (url, server) = serve_once("text/plain", body);
        let result = execute(
            Some(&checkpoints),
            WebfetchArguments {
                url,
                format: None,
                timeout: None,
            },
            None,
            true,
            true,
            None,
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(result["truncated"], true);
        assert_eq!(result["content"].as_str().unwrap().len(), MAX_MODEL_BYTES);
        let artifact_id = result["artifact_id"].as_str().unwrap();
        let artifact =
            artifacts::artifact_directory(&checkpoints).join(format!("{artifact_id}.bin"));
        assert!(artifact.exists());
        assert!(std::fs::metadata(artifact).unwrap().len() > MAX_MODEL_BYTES as u64);
        fs::remove_dir_all(checkpoints.parent().unwrap()).unwrap();
    }
}
