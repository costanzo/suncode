//! Conversion between agent and LLM messages, and continuation snapshots.

use super::super::*;

pub(in crate::agent) fn is_context_overflow_error(error: &BusinessError) -> bool {
    if error.code == states::error_codes::CONTEXT_OVERFLOW {
        return true;
    }
    let text = format!("{} {}", error.code, error.message).to_ascii_lowercase();
    text.contains("context")
        && (text.contains("length")
            || text.contains("token")
            || text.contains("too large")
            || text.contains("maximum"))
}

pub(in crate::agent) fn message_with_image_refs(
    input: &str,
    images: &[suncode_data::SessionImageRecord],
) -> Message {
    let mut message = Message::text("user", input);
    message
        .content
        .extend(images.iter().map(|image| suncode_data::ContentPart {
            kind: "image_ref".into(),
            text: image.image_id.clone(),
        }));
    message
}

pub(in crate::agent) fn image_mime_type(storage_path: &str) -> Result<&'static str, BusinessError> {
    let extension = Path::new(storage_path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "gif" => Ok("image/gif"),
        "webp" => Ok("image/webp"),
        "bmp" => Ok("image/bmp"),
        "avif" => Ok("image/avif"),
        _ => Err(BusinessError::invalid(
            "message image format is unsupported",
        )),
    }
}

pub(in crate::agent) fn continuation_snapshot(
    context: &Continuation,
) -> Result<Value, serde_json::Error> {
    let mut redacted = context.clone();
    for message in &mut redacted.messages {
        if message.role != "tool" {
            continue;
        }
        crate::context::redact_accessibility_outlines(
            message,
            "[computer accessibility tree omitted from durable continuation]",
        );
        for part in &mut message.content {
            if part.kind == "image_url" {
                part.kind = "text".into();
                part.text = "[computer screenshot omitted from durable continuation]".into();
            }
        }
    }
    serde_json::to_value(redacted)
}
