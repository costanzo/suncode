//! Observation hints derive only from this conversation's successful built-in
//! results. They are advisory and never grant authority or block exact edits.
use super::super::*;

pub(in crate::agent) fn attach_file_observation(
    messages: &[Message],
    call: &ToolCall,
    result: &mut Value,
) {
    if call.toolset_name.is_some()
        || !matches!(call.name.as_str(), "read" | "write" | "edit" | "delete")
    {
        return;
    }
    // A resumed already-succeeded mutation retains its original observation.
    if result.get("file_observation").is_some() {
        return;
    }
    let Some(path) = result
        .get("path")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return;
    };
    if path.starts_with("dependency:") {
        return;
    }
    if call.name == "edit" {
        let previous = previous_observation(messages, &path);
        let source = result.get("source_sha256").and_then(Value::as_str);
        let status = match previous.as_ref() {
            Some(previous)
                if source.is_some() && previous.get("sha256").and_then(Value::as_str) == source =>
            {
                "unchanged"
            }
            Some(_) if source.is_some() => "changed",
            _ => "unknown",
        };
        result["read_state"] = json!({"status": status, "previous_observation": previous});
        if status == "changed" {
            result["notice"] = json!("The file changed since the last observed version. The requested text still matched and the edit was applied. Re-read relevant surrounding content before making dependent edits.");
        } else if status == "unknown" {
            result["notice"] = json!("No comparable file observation remains in this conversation. The requested text matched; inspect relevant content and repository instructions before further edits.");
        }
    }
    let digest = if call.name == "read" {
        result.get("source_sha256").or_else(|| result.get("sha256"))
    } else {
        result.get("sha256")
    };
    if let Some(digest) = digest.and_then(Value::as_str).map(str::to_owned) {
        let kind = if call.name != "read" {
            "mutation"
        } else if result.get("truncated").and_then(Value::as_bool) == Some(false)
            && result.get("offset").and_then(Value::as_u64) == Some(1)
            && result.get("limit").is_none_or(Value::is_null)
        {
            "full_read"
        } else {
            "partial_read"
        };
        result["file_observation"] = json!({"kind": kind, "sha256": digest});
    } else {
        // Invalidate an older full digest after a deletion or unbounded read.
        result["file_observation"] = json!({"kind": "unknown"});
    }
}

fn previous_observation(messages: &[Message], path: &str) -> Option<Value> {
    for (index, message) in messages.iter().enumerate().rev() {
        if message.role != "tool" {
            continue;
        }
        let Some(id) = message.tool_call_id.as_deref() else {
            continue;
        };
        // Ignore lookalike metadata supplied by MCP or other tools.
        let trusted = messages[..index]
            .iter()
            .rev()
            .filter(|m| m.role == "assistant")
            .flat_map(|m| &m.tool_calls)
            .find(|call| call.call_id == id)
            .is_some_and(|call| {
                call.toolset_name.is_none()
                    && matches!(call.name.as_str(), "read" | "write" | "edit" | "delete")
            });
        if !trusted {
            continue;
        }
        for part in &message.content {
            if part.kind != "text" {
                continue;
            }
            let Ok(value) = serde_json::from_str::<Value>(&part.text) else {
                continue;
            };
            if value.get("path").and_then(Value::as_str) == Some(path) {
                if let Some(observation) = value.get("file_observation") {
                    return observation
                        .get("sha256")
                        .and_then(Value::as_str)
                        .map(|_| observation.clone());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_observations_survive_history_serialization_and_preserve_replayed_results() {
        let mut messages = Vec::new();
        record(
            &mut messages,
            call("read", "r"),
            json!({"path":"a.txt","source_sha256":"original","offset":1,"limit":null,"truncated":false}),
        );
        let messages: Vec<Message> =
            serde_json::from_str(&serde_json::to_string(&messages).unwrap()).unwrap();
        let mut edit = json!({"path":"a.txt","source_sha256":"original","sha256":"edited"});
        attach_file_observation(&messages, &call("edit", "e"), &mut edit);
        assert_eq!(edit["read_state"]["status"], "unchanged");
        assert_eq!(
            edit["read_state"]["previous_observation"]["kind"],
            "full_read"
        );
        let recorded = edit.clone();
        attach_file_observation(&[], &call("edit", "e"), &mut edit);
        assert_eq!(edit, recorded);
    }
    fn call(name: &str, id: &str) -> ToolCall {
        ToolCall {
            call_id: id.into(),
            name: name.into(),
            arguments: json!({"path":"a.txt"}),
            toolset_name: None,
        }
    }
    fn record(messages: &mut Vec<Message>, call: ToolCall, mut result: Value) {
        attach_file_observation(messages, &call, &mut result);
        let mut assistant = Message::text("assistant", "");
        assistant.tool_calls.push(call.clone());
        messages.push(assistant);
        let mut tool = Message::text("tool", result.to_string());
        tool.tool_call_id = Some(call.call_id);
        messages.push(tool);
    }
    #[test]
    fn file_observations_track_partial_reads_changes_mutations_and_unknowns() {
        let mut messages = Vec::new();
        record(
            &mut messages,
            call("read", "r"),
            json!({"path":"a.txt","source_sha256":"one","offset":2,"limit":1,"truncated":false}),
        );
        let mut edit = json!({"path":"a.txt","source_sha256":"external","sha256":"edited"});
        attach_file_observation(&messages, &call("edit", "e"), &mut edit);
        assert_eq!(edit["read_state"]["status"], "changed");
        assert_eq!(
            edit["read_state"]["previous_observation"]["kind"],
            "partial_read"
        );
        record(&mut messages, call("edit", "e"), edit);
        let mut next = json!({"path":"a.txt","source_sha256":"edited","sha256":"next"});
        attach_file_observation(&messages, &call("edit", "n"), &mut next);
        assert_eq!(next["read_state"]["status"], "unchanged");
        record(
            &mut messages,
            call("delete", "d"),
            json!({"path":"a.txt","deleted":true}),
        );
        next.as_object_mut().unwrap().remove("file_observation");
        attach_file_observation(&messages, &call("edit", "n"), &mut next);
        assert_eq!(next["read_state"]["status"], "unknown");
        next.as_object_mut().unwrap().remove("file_observation");
        attach_file_observation(&[], &call("edit", "n"), &mut next);
        assert_eq!(next["read_state"]["status"], "unknown");
    }
    #[test]
    fn file_observations_ignore_other_tools_and_paths() {
        let mut messages = Vec::new();
        record(
            &mut messages,
            call("mcp.fake", "fake"),
            json!({"path":"a.txt","file_observation":{"kind":"full_read","sha256":"fake"}}),
        );
        let mut edit = json!({"path":"a.txt","source_sha256":"real","sha256":"edited"});
        attach_file_observation(&messages, &call("edit", "e"), &mut edit);
        assert_eq!(edit["read_state"]["status"], "unknown");
        record(
            &mut messages,
            call("read", "r"),
            json!({"path":"b.txt","source_sha256":"real"}),
        );
        edit.as_object_mut().unwrap().remove("file_observation");
        attach_file_observation(&messages, &call("edit", "e"), &mut edit);
        assert_eq!(edit["read_state"]["status"], "unknown");
    }
}
