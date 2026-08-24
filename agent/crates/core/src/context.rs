use crate::domain::Message;
use serde::Serialize;
use serde_json::json;

pub const DEFAULT_CONTEXT_WINDOW_TOKENS: usize = 64_000;
pub const DEFAULT_RESERVE_TOKENS: usize = 16_384;
pub const DEFAULT_KEEP_RECENT_TOKENS: usize = 20_000;

#[derive(Debug, Clone, Serialize)]
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
    pub original_characters: usize,
    pub retained_characters: usize,
    pub original_tokens: usize,
    pub retained_tokens: usize,
    pub dropped_messages: usize,
    pub summary: Option<ContextSummary>,
}

pub fn build_for_model(
    messages: &[Message],
    max_input_tokens: Option<u64>,
    auto_compact_tokens: Option<u64>,
) -> ContextBuildResult {
    let context_window = max_input_tokens
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(DEFAULT_CONTEXT_WINDOW_TOKENS)
        .clamp(DEFAULT_RESERVE_TOKENS + 1, 2_000_000);
    let compact_at = auto_compact_tokens
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(context_window.saturating_sub(DEFAULT_RESERVE_TOKENS))
        .clamp(1_000, context_window.saturating_sub(1));
    let reserve_tokens = context_window.saturating_sub(compact_at);
    build_with_token_limits(
        messages,
        context_window,
        reserve_tokens,
        DEFAULT_KEEP_RECENT_TOKENS,
    )
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
    while serialized_characters(&retained) > max_characters && retained.len() > 2 {
        retained.remove(1);
    }
    let retained_characters = serialized_characters(&retained);
    let retained_tokens = estimate_tokens(&retained);
    let dropped_messages = messages.len().saturating_sub(retained.len() - 1);
    ContextBuildResult {
        messages: retained,
        compacted: true,
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
    while estimate_tokens(&retained) > max_context_tokens && retained.len() > 2 {
        retained.remove(1);
    }
    let retained_characters = serialized_characters(&retained);
    let retained_tokens = estimate_tokens(&retained);
    let dropped_messages = messages.len().saturating_sub(retained.len() - 1);
    ContextBuildResult {
        messages: retained,
        compacted: true,
        original_characters,
        retained_characters,
        original_tokens,
        retained_tokens,
        dropped_messages,
        summary: Some(summary),
    }
}

fn serialized_characters(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| {
            message.text_content().chars().count()
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
}
