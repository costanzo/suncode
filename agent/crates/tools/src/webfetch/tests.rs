use super::convert::convert_html;
use super::*;
use crate::arguments::WebfetchArguments;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use suncode_common::{HttpProxyConfiguration, HttpProxyMode};

fn serve_once(content_type: &str, body: impl Into<Vec<u8>>) -> (String, thread::JoinHandle<()>) {
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
        &NetworkSettings::default(),
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
        &NetworkSettings::default(),
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
        &NetworkSettings::default(),
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
        &NetworkSettings::default(),
    )
    .unwrap();
    server.join().unwrap();
    assert_eq!(result["truncated"], true);
    assert_eq!(result["content"].as_str().unwrap().len(), MAX_MODEL_BYTES);
    let artifact_id = result["artifact_id"].as_str().unwrap();
    let artifact = artifacts::artifact_directory(&checkpoints).join(format!("{artifact_id}.bin"));
    assert!(artifact.exists());
    assert!(std::fs::metadata(artifact).unwrap().len() > MAX_MODEL_BYTES as u64);
    fs::remove_dir_all(checkpoints.parent().unwrap()).unwrap();
}

#[test]
fn routes_webfetch_through_the_custom_proxy() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 4096];
        let read = stream.read(&mut request).unwrap();
        let request = String::from_utf8_lossy(&request[..read]);
        assert!(request.starts_with("GET http://example.test/page HTTP/1.1"));
        let body = "proxied";
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });
    let result = execute(
        None,
        WebfetchArguments {
            url: "http://example.test/page".into(),
            format: Some("text".into()),
            timeout: None,
        },
        None,
        &NetworkSettings {
            proxy: HttpProxyConfiguration {
                mode: HttpProxyMode::Custom,
                url: format!("http://{address}"),
                username: String::new(),
                password: String::new(),
                bypass: Vec::new(),
            },
            ..NetworkSettings::default()
        },
    )
    .unwrap();
    server.join().unwrap();
    assert_eq!(result["content"], "proxied");
}
