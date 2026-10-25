use super::*;
use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use std::collections::BTreeSet;
use suncode_llm::{
    ApiKeyResolver, ModelCapabilities, ModelDescriptor, ModelLimits, ModelProviderRegistry,
    OpenAiCompatibleProvider,
};

struct TestApiKey;

impl ApiKeyResolver for TestApiKey {
    fn api_key(&self, provider_id: &str) -> Result<Option<String>, suncode_common::BusinessError> {
        Ok((provider_id == "deepseek").then(|| "test-key".to_string()))
    }
}

#[test]
fn question_arguments_and_answers_are_validated() {
    let arguments = json!({"questions":[{"question":"Choose","header":"Mode","options":[{"label":"Fast","description":"Quick"}],"custom":false}]});
    assert!(validate_before_policy("question", &arguments).is_ok());
    assert!(validate_question_answers(&arguments, &[vec!["Fast".into()]]).is_ok());
    assert!(validate_question_answers(&arguments, &[vec!["Other".into()]]).is_err());
    assert!(validate_question_answers(&arguments, &[]).is_err());
}

#[test]
fn todo_write_arguments_require_one_active_task_at_most() {
    let valid = json!({"todos":[{"content":"Implement tool","status":"in_progress","priority":"high"},{"content":"Run tests","status":"pending","priority":"medium"}]});
    assert!(validate_before_policy("todowrite", &valid).is_ok());
    assert!(validate_before_policy("todowrite", &json!({"todos":[]})).is_ok());
    assert!(validate_before_policy("todowrite", &json!({"todos":[{"content":"one","status":"in_progress","priority":"low"},{"content":"two","status":"in_progress","priority":"low"}]})).is_err());
    assert!(validate_before_policy(
        "todowrite",
        &json!({"todos":[{"content":"one","status":"blocked","priority":"low"}]})
    )
    .is_err());
}

#[test]
fn dependency_paths_are_parsed_without_exposing_absolute_roots() {
    assert_eq!(
        providers::dependency_path("dependency:dependency-1/src/lib.rs"),
        Some(("dependency-1", "src/lib.rs"))
    );
    assert_eq!(
        providers::dependency_path("dependency:dependency-1"),
        Some(("dependency-1", "."))
    );
    assert_eq!(providers::dependency_path("src/lib.rs"), None);
    assert_eq!(providers::dependency_path("dependency:"), None);
}

#[test]
fn host_context_identifies_platform_without_session_identity() {
    let text = host_environment_message().text_content();
    assert!(text.contains(std::env::consts::OS));
    assert!(text.contains(std::env::consts::ARCH));
    assert!(text.contains("non-interactive=false"));
    assert!(text.contains("browser capability=true"));
    assert!(text.contains("computer capability=true"));
    assert!(text.contains(
        "Use the bash tool for terminal commands and write commands in the stated shell dialect."
    ));
    assert!(
        text.contains("use glob, grep, and read instead of running find, grep, or rg through bash")
    );
    assert!(!text.contains("provider="));
    assert!(!text.contains("model="));
    assert!(!text.contains("session started at="));
}

#[test]
fn host_tool_guidance_follows_the_advertised_tools() {
    let read_only = host_environment_message_with_context(
        false,
        AgentHostCapabilities::default(),
        &names(&["read", "glob", "grep", "webfetch", "todowrite"]),
    )
    .text_content();
    assert!(read_only.contains("For file discovery and content search, use glob, grep, and read."));
    assert!(!read_only.contains("Use the bash tool"));
    assert!(!read_only.contains("through bash"));
    assert!(read_only.contains("shell tool dialect="));
    assert!(read_only.contains("authority boundary"));

    let search_subset = host_environment_message_with_context(
        false,
        AgentHostCapabilities::default(),
        &names(&["read", "grep"]),
    )
    .text_content();
    assert!(search_subset.contains("use grep and read."));
    assert!(!search_subset.contains("glob"));
    assert!(!search_subset.contains("Use the bash tool"));

    let shell_only = host_environment_message_with_context(
        true,
        AgentHostCapabilities {
            browser_use: false,
            computer_use: false,
        },
        &names(&["bash"]),
    )
    .text_content();
    assert!(shell_only.contains("Use the bash tool"));
    assert!(!shell_only.contains("file discovery"));
}

fn names(tools: &[&str]) -> Vec<String> {
    tools.iter().map(|tool| (*tool).to_string()).collect()
}

#[test]
fn session_context_carries_provider_model_and_start_time() {
    let session_started_at = "2026-08-23T01:02:03.000Z";
    let text = session_context_message(session_started_at, "model-1", "provider-1").text_content();
    assert!(text.contains("provider=provider-1"));
    assert!(text.contains("model=model-1"));
    assert!(text.contains(&format!("session started at={session_started_at}")));
    assert!(session_context_message("", "model-1", "provider-1")
        .text_content()
        .contains("session started at=unavailable"));
}

#[test]
fn project_agents_file_is_loaded_as_repository_instructions() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# Project rules\nRun focused tests.",
    )
    .unwrap();

    let message = project_instruction_message(directory.path().to_str().unwrap()).unwrap();

    assert_eq!(message.role, "system");
    let text = message.text_content();
    assert!(text.contains("Repository instructions from AGENTS.md"));
    assert!(text.contains("Run focused tests."));
    assert!(text.contains("cannot grant authority"));
    assert!(!text.contains(directory.path().to_str().unwrap()));
}

#[test]
fn project_instruction_precedence_prefers_override_then_claude() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("AGENTS.md"), "agents").unwrap();
    fs::write(directory.path().join("CLAUDE.md"), "claude").unwrap();
    fs::write(directory.path().join("AGENTS.override.md"), "override").unwrap();

    let message = project_instruction_message(directory.path().to_str().unwrap()).unwrap();
    let text = message.text_content();
    assert!(text.contains("AGENTS.override.md"));
    assert!(text.contains("override"));
    assert!(!text.contains("\nagents\n"));
    assert!(!text.contains("\nclaude\n"));
}

#[test]
fn system_prompt_builder_contains_authority_workflow_and_capabilities() {
    let messages = super::system_prompt::build_messages(super::system_prompt::PromptContext {
        model_id: "model-1",
        provider_id: "provider-1",
        project_root: "project",
        session_started_at: "2026-01-01T00:00:00Z",
        non_interactive: true,
        host_capabilities: AgentHostCapabilities {
            browser_use: false,
            computer_use: true,
        },
        allowed_tools: &["read".into(), "grep".into()],
        agent: &suncode_agents::PRIMARY,
        dependency_context: None,
        skill_guidance: None,
    })
    .unwrap();
    let text = messages
        .iter()
        .map(|message| message.text_content())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("cannot grant permission"));
    assert!(text.contains("Inspect relevant files"));
    assert!(text.contains("translate it to the equivalent relative path"));
    assert!(text.contains("provider-1"));
    assert!(text.contains("model-1"));
    assert!(text.contains("You are SunCode"));
    assert!(text.contains("Non-interactive execution"));
    assert!(text.contains("disabled in non-interactive mode"));
    assert!(!text.contains("Advertised tool scope"));
    assert!(!text.contains("Browser Use host capability"));
    assert!(!text.contains("Computer Use host capability"));
    assert!(!text.contains("Computer Use workflow"));
    assert!(!text.contains("todowrite and keep its statuses current"));
    assert!(messages[0].text_content().starts_with("You are SunCode"));
    assert!(messages[1].text_content().contains("authority and trust"));
    let workflow = messages[2].text_content();
    assert!(workflow.contains("coding workflow"));
    assert!(workflow.contains("File tools (read and grep)"));
    assert!(!workflow.contains("bash"));
    let host = messages[3].text_content();
    assert!(host.contains("use grep and read."));
    assert!(!host.contains("Use the bash tool"));
    assert!(host.contains("shell tool dialect="));
    assert!(!workflow.contains("todowrite"));
    assert!(!workflow.contains("ask a focused question"));
    assert!(messages[3].text_content().contains("host environment"));
    assert!(!messages[3].text_content().contains("model-1"));
    let session = messages.last().unwrap().text_content();
    assert!(session.contains("provider=provider-1"));
    assert!(session.contains("model=model-1"));
    assert!(session.contains("session started at=2026-01-01T00:00:00Z"));
    assert!(messages[..messages.len() - 1].iter().all(|message| {
        let earlier = message.text_content();
        !earlier.contains("provider=provider-1") && !earlier.contains("model=model-1")
    }));
}

#[test]
fn system_prompt_prefix_is_stable_across_model_and_session_time() {
    let first = prompt_messages("model-a", "provider-a", "2026-01-01T00:00:00Z");
    let second = prompt_messages("model-b", "provider-b", "2026-06-01T00:00:00Z");
    assert_eq!(first.len(), second.len());
    assert!(first.len() > 1);
    assert_eq!(&first[..first.len() - 1], &second[..second.len() - 1]);
    assert!(first.last().unwrap().text_content().contains("model-a"));
    assert!(second.last().unwrap().text_content().contains("model-b"));
}

#[test]
fn capability_section_keeps_question_policy_and_computer_workflow() {
    let primary = capability_section(&suncode_agents::PRIMARY);
    assert!(primary.contains("available to the host"));
    assert!(primary.contains("policy and approvals still apply"));
    assert!(primary.contains("Computer Use workflow"));
    assert!(primary.contains("computer_screenshot"));
    assert!(!primary.contains("Advertised tool scope"));
    assert!(!primary.contains("Browser Use host capability"));
    assert!(!primary.contains("Computer Use host capability"));

    let architect = suncode_agents::specialist_by_name("architect-agent").unwrap();
    let specialist = capability_section(architect);
    assert!(specialist.contains("disabled for this specialist"));
    assert!(specialist.contains("specialist restrictions apply"));
    assert!(!specialist.contains("Computer Use workflow"));
}

fn capability_section(agent: &AgentDefinition) -> String {
    super::system_prompt::build_messages(super::system_prompt::PromptContext {
        model_id: "model-1",
        provider_id: "provider-1",
        project_root: "project",
        session_started_at: "2026-01-01T00:00:00Z",
        non_interactive: false,
        host_capabilities: AgentHostCapabilities {
            browser_use: true,
            computer_use: true,
        },
        allowed_tools: &[],
        agent,
        dependency_context: None,
        skill_guidance: None,
    })
    .unwrap()
    .into_iter()
    .map(|message| message.text_content())
    .find(|text| text.contains("effective capabilities"))
    .unwrap()
}

#[test]
fn model_family_appendix_is_only_for_gpt_and_claude() {
    let gpt_a = family_messages("gpt-5.5", "openai");
    let gpt_b = family_messages("gpt-5.6-sol", "openai");
    assert_eq!(gpt_a.len(), gpt_b.len());
    assert_eq!(&gpt_a[..gpt_a.len() - 1], &gpt_b[..gpt_b.len() - 1]);
    assert!(gpt_a[gpt_a.len() - 2]
        .text_content()
        .contains("model guidance for GPT"));
    assert!(gpt_a
        .last()
        .unwrap()
        .text_content()
        .contains("model=gpt-5.5"));
    assert!(!gpt_a[gpt_a.len() - 2].text_content().contains("git reset"));

    let claude = family_messages("claude-opus-5", "claude");
    assert!(claude[claude.len() - 2]
        .text_content()
        .contains("model guidance for Claude"));
    assert_eq!(&gpt_a[..gpt_a.len() - 2], &claude[..claude.len() - 2]);

    let qwen = family_messages("qwen-max", "qwen");
    assert!(!qwen
        .iter()
        .any(|message| message.text_content().contains("model guidance for")));
    assert!(qwen
        .last()
        .unwrap()
        .text_content()
        .contains("model=qwen-max"));
    assert_eq!(&gpt_a[..gpt_a.len() - 2], &qwen[..qwen.len() - 1]);

    let custom_gpt = family_messages("gpt-4.1", "custom-gateway");
    assert!(custom_gpt[custom_gpt.len() - 2]
        .text_content()
        .contains("model guidance for GPT"));
    let provider_claude = family_messages("future-model", "anthropic");
    assert!(provider_claude[provider_claude.len() - 2]
        .text_content()
        .contains("model guidance for Claude"));
}

fn family_messages(model_id: &str, provider_id: &str) -> Vec<Message> {
    super::system_prompt::build_messages(super::system_prompt::PromptContext {
        model_id,
        provider_id,
        project_root: "project",
        session_started_at: "2026-01-01T00:00:00Z",
        non_interactive: true,
        host_capabilities: AgentHostCapabilities {
            browser_use: false,
            computer_use: true,
        },
        allowed_tools: &["read".into(), "grep".into()],
        agent: &suncode_agents::PRIMARY,
        dependency_context: None,
        skill_guidance: None,
    })
    .unwrap()
}

fn prompt_messages(model_id: &str, provider_id: &str, session_started_at: &str) -> Vec<Message> {
    super::system_prompt::build_messages(super::system_prompt::PromptContext {
        model_id,
        provider_id,
        project_root: "project",
        session_started_at,
        non_interactive: true,
        host_capabilities: AgentHostCapabilities {
            browser_use: false,
            computer_use: true,
        },
        allowed_tools: &["read".into(), "grep".into()],
        agent: &suncode_agents::PRIMARY,
        dependency_context: None,
        skill_guidance: None,
    })
    .unwrap()
}

#[test]
fn provider_context_overflow_errors_are_detected_for_one_retry() {
    assert!(is_context_overflow_error(&BusinessError::new(
        "context_overflow",
        "request too large",
    )));
    assert!(is_context_overflow_error(&BusinessError::new(
        "invalid_request",
        "maximum context length exceeded",
    )));
    assert!(!is_context_overflow_error(&BusinessError::new(
        "authentication",
        "invalid API key",
    )));
}

#[test]
fn nearby_agents_files_are_loaded_nearest_first_and_deduplicated() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir_all(directory.path().join("src/nested")).unwrap();
    fs::write(directory.path().join("AGENTS.md"), "root").unwrap();
    fs::write(directory.path().join("src/AGENTS.md"), "src").unwrap();
    fs::write(directory.path().join("src/nested/AGENTS.md"), "nested").unwrap();
    fs::write(directory.path().join("src/nested/file.rs"), "fn main() {}").unwrap();

    let instructions = nearby_instruction_files(
        directory.path().to_str().unwrap(),
        "src/nested/file.rs",
        &[],
    );

    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].path, "src/nested/AGENTS.md");
    assert_eq!(instructions[0].scope, "directory_tree");
    assert_eq!(instructions[0].precedence, "nearest_applicable");
    assert!(instructions[0].content.contains("nested"));
    assert_eq!(instructions[1].path, "src/AGENTS.md");
    assert!(instructions[1].content.contains("src"));
    assert!(nearby_instruction_files(
        directory.path().to_str().unwrap(),
        "src/nested/file.rs",
        &["src/nested/AGENTS.md".into(), "src/AGENTS.md".into()],
    )
    .is_empty());
    assert!(nearby_instruction_files(
        directory.path().to_str().unwrap(),
        "src/nested/AGENTS.md",
        &[],
    )
    .is_empty());
}

#[cfg(unix)]
#[test]
fn instruction_symlinks_cannot_escape_the_project() {
    use std::os::unix::fs::symlink;

    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/file.rs"), "fn main() {}").unwrap();
    fs::write(outside.path().join("AGENTS.md"), "outside rules").unwrap();
    symlink(
        outside.path().join("AGENTS.md"),
        project.path().join("src/AGENTS.md"),
    )
    .unwrap();

    assert!(
        nearby_instruction_files(project.path().to_str().unwrap(), "src/file.rs", &[],).is_empty()
    );
}

async fn mock_deepseek(Json(body): Json<Value>) -> impl IntoResponse {
    let messages = body
        .get("input")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let last_role = messages.last().and_then(|message| {
        message.get("role").and_then(Value::as_str).or_else(|| {
            (message.get("type").and_then(Value::as_str) == Some("function_call_output"))
                .then_some("tool")
        })
    });
    let has_tool_error = messages.iter().any(|message| {
        message.get("type").and_then(Value::as_str) == Some("function_call_output")
            && message
                .get("output")
                .and_then(Value::as_str)
                .map(|content| {
                    content.contains("invalid_arguments") || content.contains("path_unavailable")
                })
                .unwrap_or(false)
    });
    let user_text = messages
        .iter()
        .rev()
        .find(|message| message.get("role").and_then(Value::as_str) == Some("user"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let advertised_tools = body
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tool| tool.get("name"))
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let advertised_count = body
        .get("tools")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    assert_eq!(
        advertised_tools.len(),
        advertised_count,
        "tool names must be unique"
    );
    let is_child = messages.iter().any(|message| {
        message.get("role").and_then(Value::as_str) == Some("system")
            && message
                .get("content")
                .and_then(Value::as_str)
                .is_some_and(|content| content.contains("Software Engineering Agent"))
    });
    let is_summary_call = messages.iter().any(|message| {
        message.get("role").and_then(Value::as_str) == Some("system")
            && message
                .get("content")
                .and_then(Value::as_str)
                .is_some_and(|content| {
                    content.contains("Summarize the provided coding-agent history")
                })
    });
    if user_text == "assert primary tools" {
        assert!(!is_child);
        assert!(advertised_tools.contains("delegate_agent"));
        let delegate = body["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some("delegate_agent"))
            .expect("delegate_agent is advertised");
        let description = delegate["description"].as_str().unwrap_or_default();
        for agent in suncode_agents::specialists() {
            assert!(description.contains(&format!("- {}: {}", agent.name, agent.description)));
        }
    }
    if user_text == "assert child tools" {
        assert!(is_child);
        assert_eq!(
            advertised_tools,
            [
                "bash",
                "edit",
                "glob",
                "grep",
                "read",
                "todowrite",
                "webfetch",
                "write"
            ]
            .into_iter()
            .collect()
        );
    }
    let dependency_alias = messages
        .iter()
        .filter(|message| message.get("role").and_then(Value::as_str) == Some("system"))
        .filter_map(|message| message.get("content").and_then(Value::as_str))
        .find_map(|content| {
            content
                .split_whitespace()
                .find(|value| value.starts_with("dependency:") && !value.contains('<'))
                .map(|value| {
                    value.trim_end_matches(|character: char| character.is_ascii_punctuation())
                })
        });
    if user_text.contains("slow") {
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    if user_text.starts_with("overflow test") && !is_summary_call {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":{"message":"maximum context length exceeded"}})),
        )
            .into_response();
    }
    let data = if is_summary_call {
        vec![
            json!({"choices":[{"delta":{"content":"{\"objective\":\"generated objective\",\"important_constraints\":[\"keep tests\"],\"completed_work\":[],\"active_work\":[],\"blockers\":[],\"next_action\":\"continue\"}"},"finish_reason":"stop"}],"usage":{"prompt_tokens":100,"completion_tokens":20,"total_tokens":120}}),
        ]
    } else if user_text == "assert compaction replay" {
        assert!(messages.iter().any(|message| {
            message
                .get("content")
                .and_then(Value::as_str)
                .is_some_and(|content| content.contains("generated objective"))
        }));
        vec![json!({"choices":[{"delta":{"content":"replay verified"},"finish_reason":"stop"}]})]
    } else if last_role != Some("tool") && user_text == "delegate SWE to assert child tools" {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"delegate-tools","function":{"name":"delegate_agent","arguments":"{\"agent\":\"swe-agent\",\"task\":\"assert child tools\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if last_role != Some("tool") && user_text == "delegate SWE to read" {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"delegate-read","function":{"name":"delegate_agent","arguments":"{\"agent\":\"swe-agent\",\"task\":\"read the file as child\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if last_role != Some("tool") && user_text == "delegate SWE to write" {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"delegate-write","function":{"name":"delegate_agent","arguments":"{\"agent\":\"swe-agent\",\"task\":\"write the file as child\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if user_text == "assert primary tools" || user_text == "assert child tools" {
        vec![
            json!({"choices":[{"delta":{"content":"tools verified"},"finish_reason":"stop"}],"usage":{"prompt_tokens":4,"completion_tokens":2,"total_tokens":6}}),
        ]
    } else if user_text.contains("invalid arguments") && !has_tool_error {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"invalid-read-call","function":{"name":"read","arguments":"{\"path\":123}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if user_text.contains("missing file") && !has_tool_error {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"missing-read-call","function":{"name":"read","arguments":"{\"path\":\"missing.txt\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if last_role == Some("tool") {
        vec![
            json!({"choices":[{"delta":{"content":"done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":8,"completion_tokens":1,"total_tokens":9,"prompt_cache_hit_tokens":6,"prompt_cache_miss_tokens":2,"completion_tokens_details":{"reasoning_tokens":1}}}),
        ]
    } else if user_text.contains("slow") || user_text.contains("follow up") {
        vec![
            json!({"choices":[{"delta":{"content":"queued done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":4,"completion_tokens":2,"total_tokens":6}}),
        ]
    } else if user_text.contains("dependency read") {
        let path = format!("{}/lib.rs", dependency_alias.unwrap());
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"dependency-read","function":{"name":"read","arguments":serde_json::to_string(&json!({"path":path})).unwrap()}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if user_text.contains("dependency write") {
        let path = format!("{}/lib.rs", dependency_alias.unwrap());
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"dependency-write","function":{"name":"write","arguments":serde_json::to_string(&json!({"path":path,"content":"changed"})).unwrap()}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if user_text.contains("read nested") {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"nested-read","function":{"name":"read","arguments":"{\"path\":\"src/nested/file.rs\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if user_text.contains("read two") {
        vec![json!({"choices":[{"delta":{"tool_calls":[
                    {"index":0,"id":"read-call-1","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}},
                    {"index":1,"id":"read-call-2","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}}
                ]},"finish_reason":"tool_calls"}]})]
    } else if user_text.contains("write again") {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"write-again-call","function":{"name":"write","arguments":"{\"path\":\"README.md\",\"content\":\"updated again\",\"expected_base64\":\"dXBkYXRlZA==\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else if user_text.contains("write") {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"write-call","function":{"name":"write","arguments":"{\"path\":\"README.md\",\"content\":\"updated\",\"expected_base64\":\"aGVsbG8=\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    } else {
        vec![
            json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"read-call","function":{"name":"read","arguments":"{\"path\":\"README.md\"}"}}]},"finish_reason":"tool_calls"}]}),
        ]
    };
    let body = responses_stream(data);
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn responses_stream(chunks: Vec<Value>) -> String {
    let mut events = vec![json!({"type":"response.created","response":{"id":"resp-test"}})];
    let mut usage = None;
    for chunk in chunks {
        if let Some(value) = chunk
            .pointer("/choices/0/delta/content")
            .and_then(Value::as_str)
        {
            events.push(json!({"type":"response.output_text.delta","delta":value}));
        }
        if let Some(calls) = chunk
            .pointer("/choices/0/delta/tool_calls")
            .and_then(Value::as_array)
        {
            for call in calls {
                let index = call.get("index").and_then(Value::as_u64).unwrap_or(0);
                let call_id = call.get("id").and_then(Value::as_str).unwrap_or_default();
                let name = call
                    .pointer("/function/name")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let arguments = call
                    .pointer("/function/arguments")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                events.push(json!({"type":"response.output_item.added","output_index":index,"item":{"type":"function_call","call_id":call_id,"name":name,"arguments":""}}));
                events.push(json!({"type":"response.function_call_arguments.delta","output_index":index,"call_id":call_id,"name":name,"delta":arguments}));
                events.push(json!({"type":"response.function_call_arguments.done","output_index":index,"call_id":call_id,"name":name,"arguments":arguments}));
                events.push(json!({"type":"response.output_item.done","output_index":index,"item":{"type":"function_call","call_id":call_id,"name":name,"arguments":arguments}}));
            }
        }
        if let Some(value) = chunk.get("usage") {
            usage = Some(value.clone());
        }
    }
    let mut response = json!({"id":"resp-test","status":"completed"});
    if let Some(value) = usage {
        response["usage"] = value;
    }
    events.push(json!({"type":"response.completed","response":response}));
    events
        .into_iter()
        .map(|value| format!("data: {value}\n\n"))
        .collect()
}

async fn fixture() -> (
    AgentRuntime,
    Store,
    std::path::PathBuf,
    tokio::task::JoinHandle<()>,
    String,
) {
    fixture_with_compaction_threshold(47_616).await
}

async fn fixture_with_compaction_threshold(
    auto_compact_tokens: u64,
) -> (
    AgentRuntime,
    Store,
    std::path::PathBuf,
    tokio::task::JoinHandle<()>,
    String,
) {
    fixture_with_router(
        auto_compact_tokens,
        Router::new().route("/responses", post(mock_deepseek)),
    )
    .await
}

async fn fixture_with_router(
    auto_compact_tokens: u64,
    router: Router,
) -> (
    AgentRuntime,
    Store,
    std::path::PathBuf,
    tokio::task::JoinHandle<()>,
    String,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let directory = tempfile::tempdir().unwrap().keep().canonicalize().unwrap();
    // Keep the dependency outside the project, using canonical paths on every OS.
    let root = directory.join("project");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("README.md"), "hello").unwrap();
    let store = Store::open_memory().unwrap();
    let project = store.project(root.to_str().unwrap(), "Fixture").unwrap();
    let dependency_root = directory.join("dependency-source");
    std::fs::create_dir_all(&dependency_root).unwrap();
    std::fs::write(dependency_root.join("lib.rs"), "pub fn shared() {}\n").unwrap();
    store
        .add_project_dependency(
            &project.project_id,
            dependency_root.to_str().unwrap(),
            "Shared source",
        )
        .unwrap();
    let session = store
        .create_session(&project.project_id, None, Some("deepseek-v4-flash"))
        .unwrap();
    let operations =
        Arc::new(suncode_tool::Operations::new(directory.join(".operations")).unwrap());
    let events = SessionEventHub::new(64);
    let provider = Arc::new(OpenAiCompatibleProvider::new(
        "deepseek",
        "DeepSeek",
        format!("http://{address}"),
        Arc::new(TestApiKey),
    ));
    let mut registry = ModelProviderRegistry::new();
    let models = ["deepseek-v4-flash", "deepseek-v4-pro"]
        .into_iter()
        .map(|model_id| ModelDescriptor {
            provider: "deepseek".into(),
            provider_label: "DeepSeek".into(),
            id: model_id.into(),
            wire_model: model_id.into(),
            api_base: format!("http://{address}"),
            default_api_base: format!("http://{address}"),
            capabilities: ModelCapabilities {
                streaming: true,
                tool_use: true,
                vision: false,
                structured_output: false,
                cancellation: true,
                reasoning_effort: false,
                computer_use: false,
            },
            reasoning_efforts: Vec::new(),
            limits: ModelLimits {
                max_input_tokens: Some(64_000),
                auto_compact_tokens: Some(auto_compact_tokens),
                max_output_tokens: None,
            },
            availability: "configured".into(),
        })
        .collect();
    registry.register("deepseek", provider, models).unwrap();
    (
        AgentRuntime::new(store.clone(), Arc::new(registry), operations, events, false),
        store,
        root,
        server,
        session.session_id,
    )
}

#[tokio::test]
async fn read_tool_round_trip_completes() {
    let (agent, store, root, server, session_id) = fixture().await;
    let mut attention = agent.attention_event_hub().subscribe();
    fs::write(root.join("AGENTS.md"), "Always run focused tests.").unwrap();
    let response = agent
        .submit(&session_id, "read-1", "read the file", None, None)
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed { tool_calls: 1, .. }
    ));
    let messages = store.messages(&session_id).unwrap();
    assert!(messages.iter().all(|message| message.role != "tool"));
    assert_eq!(messages.last().unwrap().role, "assistant");
    let context = store.context_messages(&session_id).unwrap();
    assert!(context.iter().any(|message| message.role == "tool"));
    let exchanges = store.provider_exchanges(&session_id).unwrap();
    assert_eq!(exchanges.len(), 2);
    assert!(exchanges
        .iter()
        .all(|exchange| exchange.output_message.is_some()));
    let usage = exchanges[0].usage.as_ref().unwrap();
    assert_eq!(usage["cache_read_tokens"], 6);
    assert_eq!(usage["cache_miss_tokens"], 2);
    assert_eq!(usage["cache_write_tokens"], serde_json::Value::Null);
    assert_eq!(usage["reasoning_tokens"], 1);
    let event = attention.recv().await.unwrap();
    assert_eq!(event.kind, AttentionKind::PrimaryTurnCompleted);
    assert_eq!(event.session_id, session_id);
    assert_eq!(event.correlation_id, event.turn_id);
    server.abort();
}

#[tokio::test]
async fn generated_compaction_summary_survives_a_new_turn() {
    let (agent, store, _root, server, session_id) = fixture_with_compaction_threshold(12_000).await;
    let long_request = format!("compact test {}", "history ".repeat(8_000));
    let first = agent
        .submit(&session_id, "compact-1", &long_request, None, None)
        .await
        .unwrap();
    assert!(matches!(first, TurnResponse::Completed { .. }));
    let context = store.context_messages(&session_id).unwrap();
    assert!(context
        .iter()
        .any(|message| message.text_content().contains("generated objective")));
    assert!(!context
        .iter()
        .any(|message| message.text_content().contains(&"history ".repeat(8_000))));
    let exchanges = store.provider_exchanges(&session_id).unwrap();
    assert!(exchanges.iter().any(|exchange| {
        exchange
            .usage
            .as_ref()
            .and_then(|usage| usage.get("total_tokens"))
            .and_then(Value::as_u64)
            == Some(120)
    }));
    assert!(exchanges
        .iter()
        .any(
            |exchange| exchange.finish_reason.as_deref() == Some("context_compacted")
                && exchange.usage.is_none()
        ));
    let second = agent
        .submit(
            &session_id,
            "compact-2",
            "assert compaction replay",
            None,
            None,
        )
        .await
        .unwrap();
    assert!(matches!(second, TurnResponse::Completed { .. }));
    server.abort();
}

async fn retry_fixture(
    status: u16,
    failures: usize,
    retry_after: &'static str,
    partial: bool,
) -> (
    AgentRuntime,
    Store,
    tokio::task::JoinHandle<()>,
    String,
    Arc<std::sync::atomic::AtomicUsize>,
) {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = calls.clone();
    let router = Router::new().route("/responses", post(move |Json(_): Json<Value>| {
            let count = count.clone();
            async move {
                let attempt = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt < failures {
                    if partial {
                        return ([(header::CONTENT_TYPE, "text/event-stream")],
                            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"partial\"}\n\n").into_response();
                    }
                    return (StatusCode::from_u16(status).unwrap(), [(header::RETRY_AFTER, retry_after)],
                        Json(json!({"error":{"message":"temporarily unavailable"}}))).into_response();
                }
                ([(header::CONTENT_TYPE, "text/event-stream")],
                    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"success\"}\n\ndata: {\"type\":\"response.completed\",\"response\":{}}\n\n").into_response()
            }
        }));
    let (agent, store, _root, server, session) = fixture_with_router(47_616, router).await;
    (agent, store, server, session, calls)
}

#[tokio::test]
async fn provider_retry_recovers_and_records_independent_attempts() {
    let (agent, store, server, session, calls) = retry_fixture(503, 1, "0", false).await;
    let response = agent
        .submit(&session, "retry-success", "hello", None, None)
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed {
            iterations: 1,
            tool_calls: 0,
            ..
        }
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    let exchanges = store.provider_exchanges(&session).unwrap();
    assert_eq!(exchanges.len(), 2);
    assert_eq!(
        exchanges
            .iter()
            .filter(|item| item.state == "failed")
            .count(),
        1
    );
    assert_ne!(exchanges[0].exchange_id, exchanges[1].exchange_id);
    server.abort();
}

#[tokio::test]
async fn provider_retry_preserves_tool_result_without_reexecution() {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = calls.clone();
    let inputs = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = inputs.clone();
    let router = Router::new().route("/responses", post(move |Json(body): Json<Value>| {
            let count = count.clone();
            let captured = captured.clone();
            async move {
                captured.lock().unwrap().push(body);
                match count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) {
                    0 => ([(header::CONTENT_TYPE, "text/event-stream")], concat!(
                        "data: {\"type\":\"response.output_item.done\",\"output_index\":0,\"item\":{\"type\":\"function_call\",\"call_id\":\"read-once\",\"name\":\"read\",\"arguments\":\"{\\\"path\\\":\\\"README.md\\\"}\"}}\n\n",
                        "data: {\"type\":\"response.completed\",\"response\":{}}\n\n"
                    )).into_response(),
                    1 => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error":{"message":"busy"}}))).into_response(),
                    2 => ([(header::CONTENT_TYPE, "text/event-stream")],
                        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"done\"}\n\ndata: {\"type\":\"response.completed\",\"response\":{}}\n\n").into_response(),
                    _ => panic!("unexpected request"),
                }
            }
        }));
    let (agent, store, _root, server, session) = fixture_with_router(47_616, router).await;
    let response = agent
        .submit(&session, "retry-after-tool", "read the file", None, None)
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed {
            iterations: 2,
            tool_calls: 1,
            ..
        }
    ));
    let inputs = inputs.lock().unwrap();
    assert_eq!(inputs.len(), 3);
    assert_eq!(
        inputs[1], inputs[2],
        "retry must reuse the same tool-result context"
    );
    let turns = store.session_conversation_turns(&session).unwrap();
    assert_eq!(turns.last().unwrap().tool_uses.len(), 1);
    assert_eq!(turns.last().unwrap().tool_uses[0].state, "succeeded");
    assert_eq!(store.provider_exchanges(&session).unwrap().len(), 3);
    server.abort();
}

#[tokio::test]
async fn provider_retry_exhaustion_is_bounded() {
    let (agent, store, server, session, calls) = retry_fixture(429, 10, "0", false).await;
    let error = agent
        .submit(&session, "retry-exhausted", "hello", None, None)
        .await
        .unwrap_err();
    assert_eq!(error.code, "transient");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    assert_eq!(store.provider_exchanges(&session).unwrap().len(), 3);
    server.abort();
}

#[tokio::test]
async fn provider_retry_rejects_authentication_and_partial_output() {
    for (status, partial, expected) in [
        (401, false, "authentication"),
        (200, true, "provider_protocol"),
    ] {
        let (agent, store, server, session, calls) = retry_fixture(status, 10, "0", partial).await;
        let error = agent
            .submit(&session, "retry-denied", "hello", None, None)
            .await
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(store.provider_exchanges(&session).unwrap().len(), 1);
        server.abort();
    }
}

#[tokio::test]
async fn provider_retry_wait_honors_retry_after_and_cancellation() {
    let (agent, store, server, session, calls) = retry_fixture(503, 10, "30", false).await;
    let submit = agent.submit(&session, "retry-cancel", "hello", None, None);
    let cancel = async {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if store
                .provider_exchanges(&session)
                .unwrap()
                .iter()
                .any(|item| item.state == "failed")
            {
                break;
            }
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // The server's 30-second minimum must prevent a second attempt.
        tokio::time::sleep(Duration::from_millis(1_500)).await;
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        let turn_id = agent
            .active_turns
            .lock()
            .unwrap()
            .get(&session)
            .unwrap()
            .clone();
        assert!(agent.cancel(&turn_id));
    };
    let (result, _) = tokio::time::timeout(Duration::from_secs(4), async {
        tokio::join!(submit, cancel)
    })
    .await
    .unwrap();
    assert_eq!(result.unwrap_err().code, "cancelled");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn provider_retry_cannot_outlive_turn_budget() {
    let (agent, _store, server, session, calls) = retry_fixture(503, 10, "99999999", false).await;
    let error = agent
        .submit(&session, "retry-budget", "hello", None, None)
        .await
        .unwrap_err();
    assert_eq!(error.code, "turn_timeout");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    server.abort();
}
#[tokio::test]
async fn provider_overflow_forces_one_compaction_and_retries() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let request = format!("overflow test {}", "history ".repeat(18_000));
    let response = agent
        .submit(&session_id, "overflow-1", &request, None, None)
        .await
        .unwrap();
    assert!(matches!(response, TurnResponse::Completed { .. }));
    let exchanges = store.provider_exchanges(&session_id).unwrap();
    assert_eq!(
        exchanges
            .iter()
            .filter(|exchange| exchange.state == "failed")
            .count(),
        1
    );
    assert_eq!(
        exchanges
            .iter()
            .filter(|exchange| exchange.finish_reason.as_deref() == Some("context_compacted"))
            .count(),
        1
    );
    server.abort();
}

#[tokio::test]
async fn primary_and_child_requests_advertise_their_respective_tool_catalogs() {
    let (agent, _store, _root, server, session_id) = fixture().await;
    agent
        .submit(
            &session_id,
            "primary-tools-1",
            "assert primary tools",
            None,
            None,
        )
        .await
        .unwrap();
    agent
        .submit(
            &session_id,
            "child-tools-1",
            "delegate SWE to assert child tools",
            None,
            None,
        )
        .await
        .unwrap();
    server.abort();
}

#[tokio::test]
async fn delegation_creates_a_linked_child_and_public_submission_is_denied() {
    let (agent, store, _root, server, session_id) = fixture().await;
    agent
        .submit(
            &session_id,
            "delegate-read-1",
            "delegate SWE to read",
            None,
            None,
        )
        .await
        .unwrap();
    let invocations = store.subagent_invocations_for_parent(&session_id).unwrap();
    assert_eq!(invocations.len(), 1);
    assert_eq!(invocations[0].state, "completed");
    assert_eq!(invocations[0].agent_id, "builtin.swe.v1");
    let child = store
        .session_by_id(&invocations[0].child_session_id)
        .unwrap()
        .unwrap();
    assert_eq!(child.kind, "child");
    assert_eq!(
        child.parent_session_id.as_deref(),
        Some(session_id.as_str())
    );
    let error = agent
        .submit(
            &child.session_id,
            "direct-child-1",
            "talk directly",
            None,
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "child_session_read_only");
    server.abort();
}

#[tokio::test]
async fn child_approval_transitions_invocation_to_completion() {
    let (agent, store, root, server, session_id) = fixture().await;
    agent
        .submit(
            &session_id,
            "delegate-write-1",
            "delegate SWE to write",
            None,
            None,
        )
        .await
        .unwrap();
    let invocation = store
        .subagent_invocations_for_parent(&session_id)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(invocation.state, "awaiting_approval");
    let approval_id = invocation.result.unwrap()["approvalId"]
        .as_str()
        .unwrap()
        .to_string();
    agent
        .resolve_approval(&approval_id, "allow_once")
        .await
        .unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let invocation = store
            .subagent_invocations_for_parent(&session_id)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        if invocation.state == "completed" {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "child approval continuation did not complete"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "updated"
    );
    server.abort();
}

#[tokio::test]
async fn read_tool_attaches_nearby_agents_instructions_to_its_result() {
    let (agent, store, root, server, session_id) = fixture().await;
    fs::create_dir_all(root.join("src/nested")).unwrap();
    fs::write(
        root.join("src/AGENTS.md"),
        "Use the src module conventions.",
    )
    .unwrap();
    fs::write(root.join("src/nested/file.rs"), "pub fn nested() {}\n").unwrap();

    agent
        .submit(&session_id, "nested-read-1", "read nested", None, None)
        .await
        .unwrap();

    let turns = store.session_conversation_turns(&session_id).unwrap();
    let result = turns[0].tool_uses[0].result.as_ref().unwrap();
    assert_eq!(
        result["repository_instructions"][0]["path"],
        "src/AGENTS.md"
    );
    assert!(result["repository_instructions"][0]["content"]
        .as_str()
        .unwrap()
        .contains("Use the src module conventions."));
    server.abort();
}

#[tokio::test]
async fn invalid_tool_arguments_are_returned_to_model_for_recovery() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let response = agent
        .submit(
            &session_id,
            "invalid-arguments-1",
            "invalid arguments",
            None,
            None,
        )
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed {
            iterations: 2,
            tool_calls: 1,
            ..
        }
    ));
    let context = store.context_messages(&session_id).unwrap();
    let tool = context
        .iter()
        .find(|message| message.role == "tool")
        .expect("invalid tool result should be retained in context");
    assert_eq!(tool.tool_call_id.as_deref(), Some("invalid-read-call"));
    assert!(tool.text_content().contains("invalid_arguments"));
    assert!(tool.text_content().contains("path is required"));
    server.abort();
}

#[tokio::test]
async fn tool_execution_errors_are_returned_to_model_for_recovery() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let response = agent
        .submit(
            &session_id,
            "missing-file-1",
            "read the missing file",
            None,
            None,
        )
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed {
            iterations: 2,
            tool_calls: 1,
            ..
        }
    ));
    let context = store.context_messages(&session_id).unwrap();
    let tool = context
        .iter()
        .find(|message| message.role == "tool")
        .expect("tool execution error should be retained in context");
    assert_eq!(tool.tool_call_id.as_deref(), Some("missing-read-call"));
    assert!(tool.text_content().contains("path_unavailable"));
    assert!(tool.text_content().contains("path is unavailable"));
    let turns = store.session_conversation_turns(&session_id).unwrap();
    assert_eq!(turns[0].tool_uses[0].state, "failed");
    assert_eq!(
        turns[0].tool_uses[0].error_code.as_deref(),
        Some("path_unavailable")
    );
    server.abort();
}

#[tokio::test]
async fn reasoning_effort_rejects_invalid_values_and_unsupported_models() {
    let (agent, _store, _root, server, session_id) = fixture().await;

    let invalid = agent
        .submit(
            &session_id,
            "invalid-reasoning-effort-1",
            "reasoning effort",
            None,
            Some("xhigh"),
        )
        .await
        .unwrap_err();
    assert_eq!(invalid.code, "invalid_arguments");
    assert!(invalid
        .message
        .contains("does not support reasoning effort"));

    let unsupported = agent
        .submit(
            &session_id,
            "invalid-reasoning-effort-2",
            "reasoning effort",
            None,
            Some("high"),
        )
        .await
        .unwrap_err();
    assert_eq!(unsupported.code, "invalid_arguments");
    assert!(unsupported
        .message
        .contains("does not support reasoning effort"));
    server.abort();
}

#[tokio::test]
async fn dependency_read_is_routed_and_write_is_rejected_before_approval() {
    let (agent, store, root, server, session_id) = fixture().await;
    let project_id = store
        .session_by_id(&session_id)
        .unwrap()
        .unwrap()
        .project_id
        .unwrap();
    let dependency = store.project_dependencies(&project_id).unwrap().remove(0);
    let response = agent
        .submit(
            &session_id,
            "dependency-read-1",
            "dependency read",
            None,
            None,
        )
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed { tool_calls: 1, .. }
    ));
    let result = store
        .context_messages(&session_id)
        .unwrap()
        .into_iter()
        .find(|message| message.role == "tool")
        .unwrap()
        .text_content();
    assert!(result.contains("dependency:"));
    assert!(!result.contains(root.to_str().unwrap()));

    let error = agent
        .submit(
            &session_id,
            "dependency-write-1",
            "dependency write",
            None,
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "scope_denied");
    assert_eq!(
        std::fs::read_to_string(Path::new(&dependency.canonical_root).join("lib.rs")).unwrap(),
        "pub fn shared() {}\n"
    );
    server.abort();
}

#[tokio::test]
async fn queued_submit_starts_a_distinct_followup_turn_after_completion() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let running = {
        let agent = agent.clone();
        let session_id = session_id.clone();
        tokio::spawn(async move {
            agent
                .submit(&session_id, "slow-1", "slow initial request", None, None)
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(30)).await;

    let queued = agent
        .submit(
            &session_id,
            "queued-1",
            "follow up while running",
            None,
            None,
        )
        .await
        .unwrap();
    assert!(matches!(queued, TurnResponse::Queued { position: 1, .. }));
    let response = running.await.unwrap().unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed { iterations: 1, .. }
    ));
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let messages = store.messages(&session_id).unwrap();
            let user_messages = messages
                .iter()
                .filter(|message| message.role == "user")
                .collect::<Vec<_>>();
            if user_messages.len() == 2 {
                assert_eq!(user_messages[1].text_content(), "follow up while running");
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("queued follow-up should be admitted after the active turn");
    assert_eq!(store.session_trace_turns(&session_id).unwrap().len(), 2);
    server.abort();
}

#[tokio::test]
async fn steering_is_added_to_the_next_request_without_cancelling_the_active_request() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let running = {
        let agent = agent.clone();
        let session_id = session_id.clone();
        tokio::spawn(async move {
            agent
                .submit(
                    &session_id,
                    "slow-steer-1",
                    "slow initial request",
                    None,
                    None,
                )
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(30)).await;
    let queued = agent
        .submit(&session_id, "steer-1", "apply the correction", None, None)
        .await
        .unwrap();
    let queued_id = match queued {
        TurnResponse::Queued { queued_id, .. } => queued_id,
        other => panic!("expected queued response, got {other:?}"),
    };
    agent.steer_queued_message(&session_id, &queued_id).unwrap();
    let response = running.await.unwrap().unwrap();
    assert!(matches!(response, TurnResponse::Completed { .. }));
    let messages = store.messages(&session_id).unwrap();
    let users = messages
        .iter()
        .filter(|message| message.role == "user")
        .collect::<Vec<_>>();
    assert_eq!(users.len(), 2);
    assert_eq!(store.session_trace_turns(&session_id).unwrap().len(), 1);
    assert_eq!(users[1].text_content(), "apply the correction");
    assert!(agent.queued_messages(&session_id).unwrap().is_empty());
    server.abort();
}

#[tokio::test]
async fn multiple_queued_followups_start_in_fifo_order() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let running = {
        let agent = agent.clone();
        let session_id = session_id.clone();
        tokio::spawn(async move {
            agent
                .submit(
                    &session_id,
                    "slow-multiple-1",
                    "slow initial request",
                    None,
                    None,
                )
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(30)).await;
    for (key, input) in [
        ("follow-up-1", "first queued input"),
        ("follow-up-2", "second queued input"),
    ] {
        assert!(matches!(
            agent
                .submit(&session_id, key, input, None, None)
                .await
                .unwrap(),
            TurnResponse::Queued { .. }
        ));
    }
    running.await.unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            let messages = store.messages(&session_id).unwrap();
            let users = messages
                .iter()
                .filter(|message| message.role == "user")
                .map(|message| message.text_content())
                .collect::<Vec<_>>();
            if users.len() == 3 {
                assert_eq!(
                    users,
                    [
                        "slow initial request",
                        "first queued input",
                        "second queued input"
                    ]
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("all queued follow-ups should be admitted");
    assert_eq!(store.session_trace_turns(&session_id).unwrap().len(), 3);
    server.abort();
}

#[tokio::test]
async fn queued_message_actions_preserve_payload_and_order() {
    let (agent, _store, _root, server, session_id) = fixture().await;
    let running = {
        let agent = agent.clone();
        let session_id = session_id.clone();
        tokio::spawn(async move {
            agent
                .submit(
                    &session_id,
                    "slow-actions-1",
                    "slow initial request",
                    None,
                    None,
                )
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(30)).await;
    let first = match agent
        .submit(&session_id, "actions-1", "keep first", None, None)
        .await
        .unwrap()
    {
        TurnResponse::Queued { queued_id, .. } => queued_id,
        other => panic!("expected first queued response, got {other:?}"),
    };
    let second = match agent
        .submit(&session_id, "actions-2", "take second", None, None)
        .await
        .unwrap()
    {
        TurnResponse::Queued { queued_id, .. } => queued_id,
        other => panic!("expected second queued response, got {other:?}"),
    };
    let steered = agent.steer_queued_message(&session_id, &second).unwrap();
    assert_eq!(steered.mode, QueuedMessageMode::Steering);
    let taken = agent.take_queued_message(&session_id, &first).unwrap();
    assert_eq!(taken.input, "keep first");
    assert_eq!(taken.position, 1);
    let restored = agent.restore_queued_message(&session_id, taken).unwrap();
    assert_eq!(restored.position, 1);
    assert_eq!(
        agent.queued_messages(&session_id).unwrap()[0].input,
        "keep first"
    );
    let taken = agent
        .take_queued_message_replacing_with_followup(&session_id, &second, "unsent draft", &[])
        .unwrap();
    assert_eq!(taken.input, "take second");
    let queue = agent.queued_messages(&session_id).unwrap();
    assert_eq!(queue[0].input, "keep first");
    assert_eq!(queue[1].input, "unsent draft");
    assert_eq!(queue[1].mode, QueuedMessageMode::Followup);
    let _ = running.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn read_only_tool_batch_is_preflighted_before_execution() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let response = agent
        .submit(&session_id, "read-two-1", "read two files", None, None)
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed { tool_calls: 2, .. }
    ));
    assert!(store
        .messages(&session_id)
        .unwrap()
        .iter()
        .all(|message| message.role != "tool"));
    assert_eq!(
        store
            .context_messages(&session_id)
            .unwrap()
            .iter()
            .filter(|message| message.role == "tool")
            .count(),
        2
    );
    server.abort();
}

#[tokio::test]
async fn over_budget_batch_is_rejected_before_any_call_executes() {
    let (agent, store, _root, server, session_id) = fixture().await;
    let project_id = store
        .session_by_id(&session_id)
        .unwrap()
        .unwrap()
        .project_id
        .unwrap();
    store
        .set_setting("project", &project_id, "tool_call_limit", &json!(1))
        .unwrap();

    let error = agent
        .submit(&session_id, "over-budget-1", "read two files", None, None)
        .await
        .unwrap_err();
    assert_eq!(error.code, "tool_budget_exceeded");
    assert_eq!(error.details["limit"], 1);
    assert_eq!(error.details["rejected_batch_size"], 2);

    let turns = store.session_conversation_turns(&session_id).unwrap();
    let turn = turns.last().unwrap();
    assert_eq!(turn.state, "failed");
    assert_eq!(turn.tool_uses.len(), 2);
    assert!(turn.tool_uses.iter().all(|tool| {
        tool.state == "failed"
            && tool.error_code.as_deref() == Some("tool_budget_exceeded")
            && tool.result.is_none()
    }));
    assert!(store
        .context_messages(&session_id)
        .unwrap()
        .iter()
        .all(|message| message.role != "tool"));
    server.abort();
}

#[tokio::test]
async fn write_waits_for_approval_and_captures_checkpoint() {
    let (agent, store, root, server, session_id) = fixture().await;
    let mut attention = agent.attention_event_hub().subscribe();
    let error = agent
        .submit(&session_id, "write-1", "write the file", None, None)
        .await
        .unwrap_err();
    assert_eq!(error.code, "approval_required");
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "hello"
    );
    let approval_id = error.details["approval_id"].as_str().unwrap().to_string();
    let event = attention.recv().await.unwrap();
    assert_eq!(event.kind, AttentionKind::ApprovalRequested);
    assert_eq!(event.correlation_id, approval_id);
    assert_eq!(event.session_kind, "primary");
    assert!(agent
        .resolve_approval(&approval_id, "allow_once")
        .await
        .unwrap());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let file_updated = std::fs::read_to_string(root.join("README.md")).unwrap() == "updated";
        let checkpoint_ready = store
            .manifests(&session_id)
            .unwrap()
            .first()
            .map(|manifest| store.checkpoint_items(&manifest.manifest_id).unwrap().len() == 1)
            .unwrap_or(false);
        if file_updated && checkpoint_ready {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "approval continuation did not complete its file and checkpoint updates"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "updated"
    );
    let manifests = store.manifests(&session_id).unwrap();
    assert_eq!(manifests.len(), 1);
    assert_eq!(
        store
            .checkpoint_items(&manifests[0].manifest_id)
            .unwrap()
            .len(),
        1
    );
    server.abort();
}

#[tokio::test]
async fn allow_session_skips_later_approvals_for_the_same_session() {
    let (agent, store, root, server, session_id) = fixture().await;
    let error = agent
        .submit(&session_id, "write-session-1", "write the file", None, None)
        .await
        .unwrap_err();
    assert_eq!(error.code, "approval_required");
    let approval_id = error.details["approval_id"].as_str().unwrap();
    assert!(agent
        .resolve_approval(approval_id, "allow_session")
        .await
        .unwrap());

    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let completed = store
            .session_trace_turns(&session_id)
            .unwrap()
            .first()
            .map(|turn| turn.state == "completed")
            .unwrap_or(false);
        if completed {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "session approval continuation did not complete"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        store.session_settings(&session_id).unwrap().full_control,
        Some(true)
    );

    let response = agent
        .submit(&session_id, "write-session-2", "write again", None, None)
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed { tool_calls: 1, .. }
    ));
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "updated again"
    );
    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn db_blocking_runs_store_work_on_multi_thread_runtime() {
    let store = Store::open_memory().unwrap();
    let health = db_blocking(|| store.health()).unwrap();
    assert_eq!(health["ok"], true);
}

#[tokio::test(flavor = "current_thread")]
async fn db_blocking_runs_inline_on_current_thread_runtime() {
    assert_eq!(db_blocking(|| 7), 7);
}

#[test]
fn db_blocking_runs_inline_outside_a_runtime() {
    assert_eq!(db_blocking(|| 7), 7);
}

/// A provider with one read-only tool that echoes its argument.
struct EchoProvider {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

impl suncode_common::tool::ToolProvider for EchoProvider {
    fn owns(&self, name: &str) -> bool {
        name == "fake_echo"
    }

    fn catalog<'a>(
        &'a self,
        _context: &'a suncode_common::tool::CatalogContext<'a>,
    ) -> suncode_common::tool::BoxFuture<'a, Vec<suncode_llm::ToolDefinition>> {
        Box::pin(async {
            vec![suncode_llm::ToolDefinition {
                name: "fake_echo".into(),
                description: "Echo text".into(),
                parameters: json!({"type":"object","properties":{"text":{"type":"string"}}}),
            }]
        })
    }

    fn validate(&self, _name: &str, arguments: &Value) -> Result<(), BusinessError> {
        arguments
            .get("text")
            .and_then(Value::as_str)
            .map(|_| ())
            .ok_or_else(|| BusinessError::invalid("text is required"))
    }

    fn risk(&self, _name: &str) -> Option<suncode_common::tool::Risk> {
        Some(suncode_common::tool::Risk::ReadOnly)
    }

    fn execute<'a>(
        &'a self,
        invocation: suncode_common::tool::ToolInvocation<'a>,
        _context: suncode_common::tool::ToolContext<'a>,
    ) -> suncode_common::tool::BoxFuture<'a, Result<suncode_common::tool::ToolResult, BusinessError>>
    {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move { Ok(json!({"echo": invocation.arguments["text"]}).into()) })
    }
}

#[tokio::test]
async fn provider_tool_is_advertised_validated_and_executed() {
    let router = Router::new().route(
        "/responses",
        post(|Json(body): Json<Value>| async move {
            let advertised = body["tools"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|tool| tool["name"] == "fake_echo");
            let lsp_advertised = body["tools"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|tool| tool["name"] == "lsp_hover");
            let tool_output = body["input"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|item| item["type"] == "function_call_output")
                .and_then(|item| item["output"].as_str())
                .map(str::to_string);
            let chunks = match tool_output {
                None => vec![json!({"choices":[{"delta":{"tool_calls":[{
                    "index":0,"id":"call-echo","function":{
                        "name":"fake_echo","arguments":"{\"text\":\"hi\"}"
                    }
                }]}}]})],
                Some(output) => vec![json!({"choices":[{"delta":{
                    "content": format!("advertised={advertised} lsp_advertised={lsp_advertised} output={output}")
                }}]})],
            };
            (
                [(header::CONTENT_TYPE, "text/event-stream")],
                responses_stream(chunks),
            )
        }),
    );
    let (mut agent, store, _root, server, session_id) = fixture_with_router(47_616, router).await;
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    agent.services.tool_providers = providers::ToolProviders::new(vec![Arc::new(EchoProvider {
        calls: calls.clone(),
    })]);
    let response = agent
        .submit(&session_id, "echo-1", "echo", None, None)
        .await
        .unwrap();
    assert!(matches!(
        response,
        TurnResponse::Completed { tool_calls: 1, .. }
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    let reply = store
        .messages(&session_id)
        .unwrap()
        .last()
        .unwrap()
        .text_content();
    assert!(reply.contains("advertised=true"), "{reply}");
    assert!(reply.contains("lsp_advertised=false"), "{reply}");
    assert!(reply.contains("hi"), "{reply}");
    server.abort();
}

#[test]
fn approval_continuations_omit_accessibility_trees() {
    let tree = format!(
        "{}\napp=\"Bank\" nodes=1\n[0] text \"account 1234\"",
        suncode_computer::OUTLINE_MARKER
    );
    let continuation = serde_json::from_value::<Continuation>(json!({
        "session_id": "s", "project_id": "p", "project_root": "/",
        "turn_id": "t", "submission_key": "k", "model": "m",
        "messages": [{"role":"tool","content":[{"type":"text","text":tree}],"tool_call_id":"call-1"}],
        "iterations": 0, "tool_calls": 0,
        "usage": {"input_tokens":0,"output_tokens":0,"total_tokens":0},
        "pending_call": null, "remaining_calls": []
    }))
    .unwrap();
    let snapshot = support::continuation_snapshot(&continuation)
        .unwrap()
        .to_string();
    assert!(!snapshot.contains("account 1234"), "{snapshot}");
    assert!(snapshot.contains("omitted from durable continuation"));
}

#[test]
fn legacy_mcp_generation_snapshot_resumes_as_an_approval_pin() {
    let mut snapshot = serde_json::to_value(Continuation {
        pending_approval_pin: None,
        ..serde_json::from_value::<Continuation>(json!({
            "session_id": "s", "project_id": "p", "project_root": "/",
            "turn_id": "t", "submission_key": "k", "model": "m",
            "messages": [], "iterations": 0, "tool_calls": 0,
            "usage": {"input_tokens":0,"output_tokens":0,"total_tokens":0},
            "pending_call": null, "remaining_calls": []
        }))
        .unwrap()
    })
    .unwrap();
    let object = snapshot.as_object_mut().unwrap();
    object.remove("pending_approval_pin");
    object.insert("pending_mcp_generation".into(), json!(7));
    object.remove("approved_batch_call_ids");
    object.remove("approval_batch_failed");
    object.insert("approved_computer_call_ids".into(), json!(["call-2"]));
    object.insert("computer_batch_failed".into(), json!(true));
    let restored: Continuation = serde_json::from_value(snapshot).unwrap();
    assert_eq!(restored.pending_approval_pin, Some(json!(7)));
    assert_eq!(restored.approved_batch_call_ids, ["call-2"]);
    assert!(restored.approval_batch_failed);
}

#[tokio::test]
async fn documents_are_read_through_audited_operations_and_paths_map_back() {
    use suncode_common::tool::{DocumentReader, PathScopes};
    let (agent, store, root, server, session_id) = fixture().await;
    let project_id = store
        .session_by_id(&session_id)
        .unwrap()
        .unwrap()
        .project_id
        .unwrap();
    let dependency = store.project_dependencies(&project_id).unwrap().remove(0);
    let paths = providers::StorePathScopes {
        store: &store,
        project_id: &project_id,
        project_root: &root,
    };
    let documents = providers::OperationsDocumentReader {
        services: &agent.services,
        paths: &paths,
        token: CancellationToken::new(),
    };

    let readme = documents.read_text("README.md", 1024).await.unwrap();
    assert_eq!(readme.text, "hello");
    assert_eq!(readme.path, root.join("README.md"));
    let alias = format!("dependency:{}/lib.rs", dependency.dependency_id);
    let shared = documents.read_text(&alias, 1024).await.unwrap();
    assert_eq!(shared.text, "pub fn shared() {}\n");
    assert_eq!(shared.display_path, alias);
    assert_eq!(
        documents.read_text("README.md", 2).await.unwrap_err().code,
        "file_too_large"
    );

    assert_eq!(
        paths.display_path(&root.join("src/main.rs")).as_deref(),
        Some("src/main.rs")
    );
    assert_eq!(
        paths
            .display_path(&Path::new(&dependency.canonical_root).join("lib.rs"))
            .as_deref(),
        Some(alias.as_str())
    );
    assert_eq!(paths.display_path(Path::new("/elsewhere/file.rs")), None);
    server.abort();
}

#[test]
fn project_paths_take_precedence_over_nested_dependencies() {
    use suncode_common::tool::PathScopes;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let dependency_root = root.join("dependency-source");
    fs::create_dir(&dependency_root).unwrap();
    fs::write(dependency_root.join("lib.rs"), "pub fn shared() {}\n").unwrap();
    let store = Store::open_memory().unwrap();
    let project = store.project(root.to_str().unwrap(), "Fixture").unwrap();
    store
        .add_project_dependency(
            &project.project_id,
            dependency_root.to_str().unwrap(),
            "Nested source",
        )
        .unwrap();
    let paths = providers::StorePathScopes {
        store: &store,
        project_id: &project.project_id,
        project_root: &root,
    };

    assert_eq!(
        paths
            .display_path(&dependency_root.join("lib.rs"))
            .as_deref(),
        Some("dependency-source/lib.rs")
    );
}

/// A batched read-only provider: `fake_step` fails when `fail` is set and
/// returns an image otherwise. Owns toolset calls as `fake_<member>`.
struct BatchProvider {
    calls: Arc<std::sync::Mutex<Vec<String>>>,
}

impl suncode_common::tool::ToolProvider for BatchProvider {
    fn owns(&self, name: &str) -> bool {
        name == "fake_step"
    }
    fn catalog<'a>(
        &'a self,
        _context: &'a suncode_common::tool::CatalogContext<'a>,
    ) -> suncode_common::tool::BoxFuture<'a, Vec<suncode_llm::ToolDefinition>> {
        Box::pin(async { Vec::new() })
    }
    fn validate(&self, _name: &str, _arguments: &Value) -> Result<(), BusinessError> {
        Ok(())
    }
    fn risk(&self, _name: &str) -> Option<suncode_common::tool::Risk> {
        Some(suncode_common::tool::Risk::ReadOnly)
    }
    fn approval_batch(&self, _name: &str) -> Option<&'static str> {
        Some("fake")
    }
    fn concurrent(&self, _name: &str) -> bool {
        false
    }
    fn execute<'a>(
        &'a self,
        invocation: suncode_common::tool::ToolInvocation<'a>,
        _context: suncode_common::tool::ToolContext<'a>,
    ) -> suncode_common::tool::BoxFuture<'a, Result<suncode_common::tool::ToolResult, BusinessError>>
    {
        let label = invocation.arguments["label"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        self.calls.lock().unwrap().push(label.clone());
        Box::pin(async move {
            if invocation.arguments["fail"] == true {
                return Err(BusinessError::new("fake_failed", "step failed"));
            }
            Ok(suncode_common::tool::ToolResult {
                value: json!({"label": label}),
                model_text: Some(format!("step {label}")),
                image: Some(suncode_common::tool::ToolImage {
                    mime_type: "image/png".into(),
                    bytes: b"png".to_vec(),
                }),
            })
        })
    }
}

#[tokio::test]
async fn approval_batch_halts_after_a_failure_and_toolset_calls_route_by_prefix() {
    let router = Router::new().route(
        "/responses",
        post(|Json(body): Json<Value>| async move {
            let answered = body["input"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|item| item["type"] == "function_call_output");
            let chunks = if answered {
                vec![json!({"choices":[{"delta":{"content":"done"}}]})]
            } else {
                let call = |index: u64, id: &str, arguments: Value| {
                    json!({"index":index,"id":id,"function":{
                        "name":"fake_step","arguments":arguments.to_string()
                    }})
                };
                vec![json!({"choices":[{"delta":{"tool_calls":[
                    call(0, "one", json!({"label":"one"})),
                    call(1, "two", json!({"label":"two","fail":true})),
                    call(2, "three", json!({"label":"three"}))
                ]}}]})]
            };
            (
                [(header::CONTENT_TYPE, "text/event-stream")],
                responses_stream(chunks),
            )
        }),
    );
    let (mut agent, store, _root, server, session_id) = fixture_with_router(47_616, router).await;
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    agent.services.tool_providers = providers::ToolProviders::new(vec![Arc::new(BatchProvider {
        calls: calls.clone(),
    })]);
    agent
        .submit(&session_id, "batch-1", "batch", None, None)
        .await
        .unwrap();
    assert_eq!(
        *calls.lock().unwrap(),
        ["one", "two"],
        "third call must be skipped"
    );
    let context = store.context_messages(&session_id).unwrap();
    let tool_text = |id: &str| {
        context
            .iter()
            .find(|message| message.tool_call_id.as_deref() == Some(id))
            .map(|message| message.text_content())
            .unwrap_or_default()
    };
    assert!(tool_text("three").contains(suncode_computer::HALT_MESSAGE));
    // Persisted context keeps the redacted metadata; the image lives only in
    // the in-memory provider context of the running turn.
    let one = context
        .iter()
        .find(|message| message.tool_call_id.as_deref() == Some("one"))
        .unwrap();
    assert_eq!(one.text_content(), r#"{"label":"one"}"#);
    let history = store.messages(&session_id).unwrap();
    assert!(context
        .iter()
        .chain(history.iter())
        .flat_map(|message| message.content.iter())
        .all(|part| part.kind != "image_url"));
    server.abort();

    let toolset = ToolCall {
        call_id: "c".into(),
        name: "step".into(),
        arguments: json!({}),
        toolset_name: Some("fake".into()),
    };
    assert_eq!(tools::tool_name(&toolset), "fake_step");
}
