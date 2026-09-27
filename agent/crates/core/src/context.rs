use crate::domain::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const DEFAULT_CONTEXT_WINDOW_TOKENS: usize = 64_000;
pub const DEFAULT_RESERVE_TOKENS: usize = 16_384;
pub const DEFAULT_KEEP_RECENT_TOKENS: usize = 20_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub objective: String,
    pub important_constraints: Vec<String>,
    pub completed_work: Vec<String>,
    pub active_work: Vec<String>,
    pub blockers: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone)]
pub struct ContextBuildResult {
    pub messages: Vec<Message>,
    pub compacted: bool,
    pub max_retained_tokens: usize,
    pub original_characters: usize,
    pub retained_characters: usize,
    pub original_tokens: usize,
    pub retained_tokens: usize,
    pub dropped_messages: usize,
    pub summary: Option<ContextSummary>,
}

#[cfg(test)]
fn build_for_model(
    messages: &[Message],
    max_input_tokens: Option<u64>,
    auto_compact_tokens: Option<u64>,
) -> ContextBuildResult {
    build_for_model_with_overhead(messages, max_input_tokens, auto_compact_tokens, 0)
}

pub fn build_for_model_with_overhead(
    messages: &[Message],
    max_input_tokens: Option<u64>,
    auto_compact_tokens: Option<u64>,
    fixed_request_tokens: usize,
) -> ContextBuildResult {
    let context_window = max_input_tokens
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(DEFAULT_CONTEXT_WINDOW_TOKENS)
        .clamp(DEFAULT_RESERVE_TOKENS + 1, 2_000_000);
    let compact_at = auto_compact_tokens
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(context_window.saturating_sub(DEFAULT_RESERVE_TOKENS))
        .clamp(1_000, context_window.saturating_sub(1));
    let reserve_tokens = context_window
        .saturating_sub(compact_at)
        .saturating_add(fixed_request_tokens);
    build_with_token_limits(
        messages,
        context_window,
        reserve_tokens,
        DEFAULT_KEEP_RECENT_TOKENS,
    )
}

pub fn force_compact(messages: &[Message], context_window_tokens: usize) -> ContextBuildResult {
    let context_window_tokens = context_window_tokens.max(DEFAULT_RESERVE_TOKENS + 1);
    let target = (context_window_tokens / 2).max(1_000);
    build_with_token_limits(
        messages,
        context_window_tokens,
        context_window_tokens - target,
        target,
    )
}

pub fn apply_generated_summary(result: &mut ContextBuildResult, summary: ContextSummary) -> bool {
    let Some(previous) = result.messages.first().cloned() else {
        return false;
    };
    if let Some(first) = result.messages.first_mut() {
        *first = Message::text(
            "system",
            serde_json::to_string(&json!({"type":"suncode_context_summary","summary":summary}))
                .expect("context summary is serializable"),
        );
    }
    result.retained_characters = serialized_characters(&result.messages);
    result.retained_tokens = estimate_tokens(&result.messages);
    if result.retained_tokens > result.max_retained_tokens {
        result.messages[0] = previous;
        result.retained_characters = serialized_characters(&result.messages);
        result.retained_tokens = estimate_tokens(&result.messages);
        return false;
    }
    result.summary = Some(summary);
    true
}

pub fn persistable_messages(messages: &[Message]) -> Vec<Message> {
    messages
        .iter()
        .cloned()
        .map(|mut message| {
            let original = message.content.len();
            message.content.retain(|part| part.kind != "image_url");
            if message.content.len() != original {
                message.content.push(crate::domain::ContentPart {
                    kind: "text".into(),
                    text: "[transient image omitted from persisted context]".into(),
                });
            }
            message
        })
        .collect()
}

#[cfg(test)]
fn build_with_limits(
    messages: &[Message],
    max_characters: usize,
    recent_tail: usize,
) -> ContextBuildResult {
    let max_characters = max_characters.clamp(16_000, 1_000_000);
    let recent_tail = recent_tail.clamp(2, 32);
    let original_characters = serialized_characters(messages);
    if original_characters <= max_characters {
        return ContextBuildResult {
            messages: messages.to_vec(),
            compacted: false,
            max_retained_tokens: max_characters.div_ceil(4),
            original_characters,
            retained_characters: original_characters,
            original_tokens: estimate_tokens(messages),
            retained_tokens: estimate_tokens(messages),
            dropped_messages: 0,
            summary: None,
        };
    }

    let mut start = messages.len().saturating_sub(recent_tail);
    while start > 0 && messages[start].role == "tool" {
        start -= 1;
    }
    let tail = &messages[start..];
    let dropped = &messages[..start];
    let summary = summarize(dropped, tail);
    let summary_message = Message::text(
        "system",
        serde_json::to_string(&json!({
            "type": "suncode_context_summary",
            "objective": summary.objective,
            "important_constraints": summary.important_constraints,
            "completed_work": summary.completed_work,
            "active_work": summary.active_work,
            "blockers": summary.blockers,
            "next_action": summary.next_action,
        }))
        .expect("context summary is serializable"),
    );
    let mut retained = vec![summary_message];
    retained.extend_from_slice(tail);
    while serialized_characters(&retained) > max_characters && retained.len() > 1 {
        remove_oldest_context_unit(&mut retained);
    }
    let retained_characters = serialized_characters(&retained);
    let retained_tokens = estimate_tokens(&retained);
    let dropped_messages = messages.len().saturating_sub(retained.len() - 1);
    ContextBuildResult {
        messages: retained,
        compacted: true,
        max_retained_tokens: max_characters.div_ceil(4),
        original_characters,
        retained_characters,
        original_tokens: estimate_tokens(messages),
        retained_tokens,
        dropped_messages,
        summary: Some(summary),
    }
}

pub fn build_with_token_limits(
    messages: &[Message],
    context_window_tokens: usize,
    reserve_tokens: usize,
    keep_recent_tokens: usize,
) -> ContextBuildResult {
    let context_window_tokens = context_window_tokens.clamp(16_000, 2_000_000);
    let reserve_tokens = reserve_tokens.min(context_window_tokens.saturating_sub(1));
    let max_context_tokens = context_window_tokens.saturating_sub(reserve_tokens).max(1);
    let keep_recent_tokens = keep_recent_tokens.clamp(1_000, max_context_tokens);
    let original_characters = serialized_characters(messages);
    let original_tokens = estimate_tokens(messages);
    if original_tokens <= max_context_tokens {
        return ContextBuildResult {
            messages: messages.to_vec(),
            compacted: false,
            max_retained_tokens: max_context_tokens,
            original_characters,
            retained_characters: original_characters,
            original_tokens,
            retained_tokens: original_tokens,
            dropped_messages: 0,
            summary: None,
        };
    }

    let start = recent_token_tail_start(messages, keep_recent_tokens);
    let tail = &messages[start..];
    let dropped = &messages[..start];
    let summary = summarize(dropped, tail);
    let summary_message = Message::text(
        "system",
        serde_json::to_string(&json!({
            "type": "suncode_context_summary",
            "objective": summary.objective,
            "important_constraints": summary.important_constraints,
            "completed_work": summary.completed_work,
            "active_work": summary.active_work,
            "blockers": summary.blockers,
            "next_action": summary.next_action,
        }))
        .expect("context summary is serializable"),
    );
    let mut retained = vec![summary_message];
    retained.extend_from_slice(tail);
    while estimate_tokens(&retained) > max_context_tokens && retained.len() > 1 {
        remove_oldest_context_unit(&mut retained);
    }
    let retained_characters = serialized_characters(&retained);
    let retained_tokens = estimate_tokens(&retained);
    let dropped_messages = messages.len().saturating_sub(retained.len() - 1);
    ContextBuildResult {
        messages: retained,
        compacted: true,
        max_retained_tokens: max_context_tokens,
        original_characters,
        retained_characters,
        original_tokens,
        retained_tokens,
        dropped_messages,
        summary: Some(summary),
    }
}

fn remove_oldest_context_unit(messages: &mut Vec<Message>) {
    if messages.len() <= 1 {
        return;
    }
    let end = if messages[1].role == "assistant" && !messages[1].tool_calls.is_empty() {
        let expected = messages[1].tool_calls.len();
        let mut end = 2;
        while end < messages.len() && messages[end].role == "tool" && end - 2 < expected {
            end += 1;
        }
        end
    } else {
        2
    };
    messages.drain(1..end.min(messages.len()));
}

fn serialized_characters(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| {
            message.text_content().chars().count()
                + message
                    .content
                    .iter()
                    .filter(|part| part.kind == "image_ref" || part.kind == "image_url")
                    .count()
                    * 4_800
                + serde_json::to_string(&message.tool_calls)
                    .map(|value| value.chars().count())
                    .unwrap_or_default()
                + 32
        })
        .sum()
}

fn estimate_tokens(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| {
            let characters = message.text_content().chars().count()
                + message
                    .content
                    .iter()
                    .filter(|part| part.kind == "image_ref" || part.kind == "image_url")
                    .count()
                    * 4_800
                + serde_json::to_string(&message.tool_calls)
                    .map(|value| value.chars().count())
                    .unwrap_or_default()
                + 32;
            characters.div_ceil(4)
        })
        .sum()
}

fn recent_token_tail_start(messages: &[Message], keep_recent_tokens: usize) -> usize {
    let mut tokens = 0usize;
    let mut start = messages.len();
    while start > 0 {
        let next = &messages[start - 1..start];
        let next_tokens = estimate_tokens(next);
        if tokens > 0 && tokens + next_tokens > keep_recent_tokens {
            break;
        }
        tokens += next_tokens;
        start -= 1;
    }
    while start > 0 && messages[start].role == "tool" {
        start -= 1;
    }
    start
}

fn summarize(dropped: &[Message], tail: &[Message]) -> ContextSummary {
    let texts = dropped
        .iter()
        .chain(tail.iter())
        .map(Message::text_content)
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>();
    let latest_user = tail
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(Message::text_content)
        .or_else(|| texts.last().cloned())
        .unwrap_or_default();
    ContextSummary {
        objective: latest_user.chars().take(2_000).collect(),
        important_constraints: matching_tail(&texts, r"must|cannot|only|never|required", 8, 500),
        completed_work: dropped
            .iter()
            .filter(|message| message.role == "tool")
            .map(Message::text_content)
            .filter(|text| !text.is_empty())
            .rev()
            .take(8)
            .map(|text| text.chars().take(500).collect())
            .collect(),
        active_work: tail
            .iter()
            .filter(|message| message.role == "assistant")
            .map(Message::text_content)
            .filter(|text| !text.is_empty())
            .map(|text| text.chars().take(500).collect())
            .collect(),
        blockers: matching_tail(&texts, r"error|failed|conflict|denied|blocked", 8, 500),
        next_action: latest_user.chars().take(1_000).collect(),
    }
}

fn matching_tail(texts: &[String], pattern: &str, limit: usize, width: usize) -> Vec<String> {
    texts
        .iter()
        .filter(|text| {
            text.to_ascii_lowercase()
                .split_whitespace()
                .any(|word| pattern.split('|').any(|needle| word.contains(needle)))
        })
        .rev()
        .take(limit)
        .map(|text| text.chars().take(width).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_recent_messages_and_tool_groups_when_compacting() {
        let mut messages = vec![Message::text("user", "old ".repeat(5_000))];
        messages.push(Message::text("assistant", "tool request"));
        let mut tool = Message::text("tool", "result");
        tool.tool_call_id = Some("call-1".into());
        messages.push(tool);
        messages.extend((0..7).map(|index| Message::text("user", format!("recent {index}"))));

        let result = build_with_limits(&messages, 16_000, 8);
        assert!(result.compacted);
        assert!(result.messages[0]
            .text_content()
            .contains("suncode_context_summary"));
        assert_eq!(result.messages[1].role, "assistant");
        assert_eq!(result.messages[2].role, "tool");
        assert!(result.retained_characters <= 16_000);
    }

    #[test]
    fn does_not_compact_within_budget() {
        let messages = vec![Message::text("user", "hello")];
        let result = build_with_limits(&messages, 16_000, 8);
        assert!(!result.compacted);
        assert_eq!(result.dropped_messages, 0);
    }

    #[test]
    fn compacts_when_estimated_tokens_exceed_model_window_reserve() {
        let messages = (0..20)
            .map(|index| Message::text("user", format!("message {index} {}", "wide ".repeat(500))))
            .collect::<Vec<_>>();

        let result = build_with_token_limits(&messages, 16_000, 8_000, 2_000);

        assert!(result.compacted);
        assert!(result.original_tokens > result.retained_tokens);
        assert!(result.retained_tokens <= 8_000);
        assert!(result.messages[0]
            .text_content()
            .contains("suncode_context_summary"));
    }

    #[test]
    fn uses_model_auto_compact_threshold_before_context_limit() {
        let messages = (0..100)
            .map(|index| Message::text("user", format!("message {index} {}", "wide ".repeat(40))))
            .collect::<Vec<_>>();
        let result = build_for_model(&messages, Some(64_000), Some(2_000));

        assert!(result.compacted);
        assert!(result.retained_tokens <= 2_000);
    }

    #[test]
    fn removes_old_tool_call_and_results_as_one_unit() {
        let mut messages = vec![Message::text("user", "old objective ".repeat(2_000))];
        messages.push(Message {
            role: "assistant".into(),
            content: vec![crate::domain::ContentPart {
                kind: "text".into(),
                text: "call tools".into(),
            }],
            tool_calls: vec![crate::domain::ToolCall {
                call_id: "one".into(),
                name: "read".into(),
                arguments: serde_json::json!({"path":"a"}),
                toolset_name: None,
            }],
            tool_call_id: None,
        });
        let mut tool = Message::text("tool", "result");
        tool.tool_call_id = Some("one".into());
        messages.push(tool);
        messages.push(Message::text("user", "recent"));
        let result = build_with_token_limits(&messages, 16_000, 15_000, 1_000);
        assert!(result.compacted);
        let has_result = result
            .messages
            .iter()
            .any(|message| message.tool_call_id.as_deref() == Some("one"));
        let has_call = result
            .messages
            .iter()
            .any(|message| message.tool_calls.iter().any(|call| call.call_id == "one"));
        assert_eq!(has_result, has_call);
    }

    #[test]
    fn request_overhead_reduces_available_history_budget() {
        let messages = vec![Message::text("user", "wide ".repeat(5_000))];
        let without = build_for_model(&messages, Some(16_000), Some(15_000));
        let with = build_for_model_with_overhead(&messages, Some(16_000), Some(15_000), 14_000);
        assert!(!without.compacted);
        assert!(with.compacted);
    }

    #[test]
    fn image_references_contribute_to_context_budget() {
        let mut message = Message::text("user", "inspect");
        message.content.push(crate::domain::ContentPart {
            kind: "image_ref".into(),
            text: "image-id".into(),
        });
        assert!(estimate_tokens(&[message]) >= 1_200);
    }

    #[test]
    fn persisted_checkpoint_omits_transient_image_bytes() {
        let mut message = Message::text("tool", "Computer screenshot captured.");
        message.content.push(crate::domain::ContentPart {
            kind: "image_url".into(),
            text: "data:image/png;base64,secret".into(),
        });
        let persisted = persistable_messages(&[message]);
        assert!(!serde_json::to_string(&persisted)
            .unwrap()
            .contains("secret"));
        assert!(persisted[0]
            .text_content()
            .contains("transient image omitted"));
    }

    #[test]
    fn generated_summary_replaces_local_checkpoint_only_when_it_fits() {
        let messages = vec![
            Message::text("user", "old ".repeat(8_000)),
            Message::text("user", "recent"),
        ];
        let mut result = build_with_token_limits(&messages, 16_000, 15_000, 1_000);
        assert!(result.compacted);
        let local = result.messages[0].clone();
        let summary = ContextSummary {
            objective: "implement feature".into(),
            important_constraints: vec!["preserve API".into()],
            completed_work: vec![],
            active_work: vec![],
            blockers: vec![],
            next_action: "run tests".into(),
        };
        assert!(apply_generated_summary(&mut result, summary));
        assert!(result.messages[0]
            .text_content()
            .contains("implement feature"));
        assert_ne!(result.messages[0], local);
        let previous = result.messages[0].clone();
        let oversized = ContextSummary {
            objective: "x".repeat(8_000),
            important_constraints: vec![],
            completed_work: vec![],
            active_work: vec![],
            blockers: vec![],
            next_action: "continue".into(),
        };
        assert!(!apply_generated_summary(&mut result, oversized));
        assert_eq!(result.messages[0], previous);
    }
}
