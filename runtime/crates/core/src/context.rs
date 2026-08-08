use crate::domain::Message;
use serde::Serialize;
use serde_json::json;

pub const DEFAULT_MAX_CHARACTERS: usize = 240_000;
pub const DEFAULT_RECENT_TAIL: usize = 8;

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
    pub dropped_messages: usize,
    pub summary: Option<ContextSummary>,
}

pub fn build(messages: &[Message]) -> ContextBuildResult {
    build_with_limits(messages, DEFAULT_MAX_CHARACTERS, DEFAULT_RECENT_TAIL)
}

pub fn build_with_limits(
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
    let dropped_messages = messages.len().saturating_sub(retained.len() - 1);
    ContextBuildResult {
        messages: retained,
        compacted: true,
        original_characters,
        retained_characters,
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
}
