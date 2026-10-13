//! LSP base-protocol framing: `Content-Length` headers followed by a JSON body.
//!
//! Every read and write is bounded so a misbehaving server cannot exhaust memory
//! or block the client indefinitely.

use crate::error::{lsp_error, redact};
use serde_json::Value;
use std::time::Duration;
use suncode_common::BusinessError;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) const MAX_MESSAGE_BYTES: usize = 4 * 1024 * 1024;
/// Upper bound for one header line, including its line terminator.
pub(crate) const MAX_HEADER_LINE_BYTES: usize = 1024;
/// Upper bound for the whole header section, including the blank separator line.
pub(crate) const MAX_HEADER_BYTES: usize = 8 * 1024;

/// Encodes and writes one framed message, failing with `lsp_write_timeout` when the
/// peer does not accept the bytes within `timeout`.
pub(crate) async fn write_value<W>(
    writer: &mut W,
    value: &Value,
    timeout: Duration,
    secret_values: &[String],
) -> Result<(), BusinessError>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    let body = serde_json::to_vec(value).map_err(|error| {
        lsp_error(
            "lsp_serialization_failed",
            redact(&error.to_string(), secret_values),
        )
    })?;
    if body.len() > MAX_MESSAGE_BYTES {
        return Err(lsp_error(
            "lsp_message_too_large",
            "Language server message exceeds 4 MiB",
        ));
    }
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let write = async {
        writer.write_all(header.as_bytes()).await?;
        writer.write_all(&body).await?;
        writer.flush().await
    };
    match tokio::time::timeout(timeout, write).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(lsp_error(
            "lsp_io_failed",
            format!(
                "Language server write failed: {}",
                redact(&error.to_string(), secret_values)
            ),
        )),
        Err(_) => Err(lsp_error(
            "lsp_write_timeout",
            "Language server did not accept input before the request timeout",
        )),
    }
}

/// Reads one framed message. Returns `Ok(None)` when the stream ends before a
/// complete header section.
pub(crate) async fn read_value<R>(reader: &mut R) -> Result<Option<Value>, BusinessError>
where
    R: AsyncBufRead + Unpin,
{
    let mut content_length = None;
    let mut header_bytes = 0_usize;
    loop {
        let limit = MAX_HEADER_LINE_BYTES.min(MAX_HEADER_BYTES - header_bytes);
        let Some(line) = read_header_line(reader, limit).await? else {
            return Ok(None);
        };
        header_bytes += line.len();
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                content_length = Some(value.trim().parse::<usize>().map_err(|_| {
                    lsp_error(
                        "lsp_framing_invalid",
                        "Language server Content-Length is invalid",
                    )
                })?);
            }
        }
    }
    let content_length = content_length.ok_or_else(|| {
        lsp_error(
            "lsp_framing_invalid",
            "Language server message omitted Content-Length",
        )
    })?;
    if content_length > MAX_MESSAGE_BYTES {
        return Err(lsp_error(
            "lsp_message_too_large",
            "Language server message exceeds 4 MiB",
        ));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).await.map_err(|error| {
        lsp_error(
            "lsp_io_failed",
            format!("Language server body failed: {error}"),
        )
    })?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| lsp_error("lsp_json_invalid", error.to_string()))
}

/// Reads one `\n`-terminated header line of at most `limit` bytes. Returns
/// `Ok(None)` at end of stream.
async fn read_header_line<R>(reader: &mut R, limit: usize) -> Result<Option<String>, BusinessError>
where
    R: AsyncBufRead + Unpin,
{
    let mut line = Vec::new();
    (&mut *reader)
        .take(limit as u64)
        .read_until(b'\n', &mut line)
        .await
        .map_err(|error| {
            lsp_error(
                "lsp_io_failed",
                format!("Language server read failed: {error}"),
            )
        })?;
    if line.last() != Some(&b'\n') {
        if line.len() >= limit {
            return Err(lsp_error(
                "lsp_header_too_large",
                "Language server message header exceeds the supported size",
            ));
        }
        // The stream ended before the header line was terminated.
        return Ok(None);
    }
    String::from_utf8(line).map(Some).map_err(|_| {
        lsp_error(
            "lsp_framing_invalid",
            "Language server header is not valid UTF-8",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::io::{duplex, BufReader};

    async fn read_bytes(data: &[u8]) -> Result<Option<Value>, BusinessError> {
        let mut reader = BufReader::new(data);
        read_value(&mut reader).await
    }

    #[tokio::test]
    async fn rejects_oversized_framing() {
        let data = format!("Content-Length: {}\r\n\r\n", MAX_MESSAGE_BYTES + 1);
        let error = read_bytes(data.as_bytes()).await.unwrap_err();
        assert_eq!(error.code, "lsp_message_too_large");
    }

    #[tokio::test]
    async fn rejects_an_unterminated_oversized_header_line() {
        let data = vec![b'a'; MAX_HEADER_LINE_BYTES * 4];
        let error = read_bytes(&data).await.unwrap_err();
        assert_eq!(error.code, "lsp_header_too_large");
    }

    #[tokio::test]
    async fn rejects_an_oversized_header_section() {
        let line = format!("X-Padding: {}\r\n", "a".repeat(500));
        let mut data = line.repeat(MAX_HEADER_BYTES / line.len() + 1);
        data.push_str("Content-Length: 2\r\n\r\n{}");
        let error = read_bytes(data.as_bytes()).await.unwrap_err();
        assert_eq!(error.code, "lsp_header_too_large");
    }

    #[tokio::test]
    async fn accepts_headers_within_bounds() {
        let data = "Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(read_bytes(data.as_bytes()).await.unwrap(), Some(json!({})));
    }

    #[tokio::test]
    async fn matches_content_length_case_insensitively() {
        for name in [
            "Content-Length",
            "content-length",
            "CONTENT-LENGTH",
            "cOnTeNt-LeNgTh",
        ] {
            let data = format!("{name}:   7  \r\n\r\n{{\"a\":1}}");
            assert_eq!(
                read_bytes(data.as_bytes()).await.unwrap(),
                Some(json!({"a":1})),
                "header name {name}"
            );
        }
    }

    #[tokio::test]
    async fn end_of_stream_before_headers_is_not_an_error() {
        assert_eq!(read_bytes(b"").await.unwrap(), None);
        assert_eq!(read_bytes(b"Content-Len").await.unwrap(), None);
    }

    #[tokio::test]
    async fn write_round_trips_through_read() {
        let (mut client, server) = duplex(1024);
        write_value(&mut client, &json!({"id":1}), Duration::from_secs(1), &[])
            .await
            .unwrap();
        let mut reader = BufReader::new(server);
        assert_eq!(
            read_value(&mut reader).await.unwrap(),
            Some(json!({"id":1}))
        );
    }

    #[tokio::test]
    async fn write_times_out_when_the_peer_stops_reading() {
        // The peer half is kept alive but never read, so the 64-byte pipe fills up.
        let (mut client, _server) = duplex(64);
        let value = json!({"payload":"x".repeat(1024)});
        let error = write_value(&mut client, &value, Duration::from_millis(50), &[])
            .await
            .unwrap_err();
        assert_eq!(error.code, "lsp_write_timeout");
    }
}
