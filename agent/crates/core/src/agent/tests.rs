#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::CredentialStore;
    use axum::{http::header, response::IntoResponse, routing::post, Json, Router};
    use suncode_llm::{
        ModelCapabilities, ModelDescriptor, ModelLimits, ModelProviderRegistry,
        OpenAiCompatibleProvider,
    };

    #[test]
    fn bash_translation_uses_opencode_command_and_millisecond_timeout() {
        let translated = translate_arguments(
            "bash",
            &json!({"command":"echo hello","timeout":120_000,"workdir":"src"}),
        )
        .unwrap();
        assert_eq!(translated["timeout_ms"], 120_000);
        assert_eq!(translated["cwd"], "src");
        assert!(translated.get("command").is_none());
        assert!(translate_arguments("bash", &json!({"command":"echo hello","timeout":0})).is_err());
        assert!(
            translate_arguments("bash", &json!({"command":"echo hello","timeout":600_001}))
                .is_err()
        );
    }

    #[test]
    fn grep_translation_recurses_directories_and_preserves_files() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src/nested")).unwrap();
        fs::write(root.path().join("src/nested/Main.java"), b"class Main {}").unwrap();

        let directory = translate_arguments_with_root(
            "grep",
            &json!({
                "pattern": "orderRefund",
                "path": "src",
                "include": "*.java"
            }),
            Some(root.path()),
        )
        .unwrap();
        assert_eq!(directory["query"], "orderRefund");
        assert_eq!(directory["pattern"], "src/**/*.java");

        let file = translate_arguments_with_root(
            "grep",
            &json!({"pattern": "Main", "path": "src/nested/Main.java"}),
            Some(root.path()),
        )
        .unwrap();
        assert_eq!(file["query"], "Main");
        assert_eq!(file["pattern"], "src/nested/Main.java");
    }

    #[test]
    fn edit_translation_accepts_multiple_disjoint_edits() {
        let translated = translate_arguments(
            "edit",
            &json!({
                "path":"file.txt",
                "expected_base64":"YmVmb3Jl",
                "edits":[
                    {"oldText":"one","newText":"two"},
                    {"oldText":"three","newText":"four"}
                ]
            }),
        )
        .unwrap();
        assert_eq!(translated["replacements"].as_array().unwrap().len(), 2);
        assert!(translated.get("edits").is_none());
    }

    #[test]
    fn webfetch_arguments_are_validated_before_policy() {
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://example.com","format":"markdown","timeout":30})
        )
        .is_ok());
        assert!(validate_before_policy("webfetch", &json!({"url":"file:///tmp/example"})).is_err());
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://user:secret@example.com"})
        )
        .is_err());
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://example.com","format":"pdf"})
        )
        .is_err());
        assert!(validate_before_policy(
            "webfetch",
            &json!({"url":"https://example.com","timeout":121})
        )
        .is_err());
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
            dependency_path("dependency:dependency-1/src/lib.rs"),
            Some(("dependency-1", "src/lib.rs"))
        );
        assert_eq!(
            dependency_path("dependency:dependency-1"),
            Some(("dependency-1", "."))
        );
        assert_eq!(dependency_path("src/lib.rs"), None);
        assert_eq!(dependency_path("dependency:"), None);
        assert!(dependency_tool_allowed("read"));
        assert!(!dependency_tool_allowed("write"));
    }

    #[test]
    fn dependency_results_preserve_the_stable_alias() {
        let read = normalize_result(
            "read",
            json!({"path":"src/lib.rs","data_base64":STANDARD.encode("hello")}),
            Some("dependency-1"),
        );
        assert_eq!(read["path"], "dependency:dependency-1/src/lib.rs");

        let glob = normalize_result(
            "glob",
            json!({"paths":["src/lib.rs","README.md"]}),
            Some("dependency-1"),
        );
        assert_eq!(
            glob["paths"],
            json!([
                "dependency:dependency-1/src/lib.rs",
                "dependency:dependency-1/README.md"
            ])
        );

        let grep = normalize_result(
            "grep",
            json!({"matches":[{"path":"src/lib.rs","line":1}]}),
            Some("dependency-1"),
        );
        assert_eq!(
            grep["matches"][0]["path"],
            "dependency:dependency-1/src/lib.rs"
        );
    }

    #[test]
    fn legacy_bash_translation_uses_the_platform_shell() {
        let translated = translate_arguments("bash", &json!({"command":"echo hello"})).unwrap();
        #[cfg(target_os = "windows")]
        {
            assert_eq!(translated["program"], "powershell.exe");
            assert_eq!(
                translated["args"],
                json!([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "echo hello"
                ])
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(translated["program"], "/bin/sh");
            assert_eq!(translated["args"], json!(["-lc", "echo hello"]));
        }
        assert!(translated.get("command").is_none());
    }

    #[test]
    fn host_context_identifies_platform_and_stable_session_time() {
        let session_started_at = "2026-08-23T01:02:03.000Z";
        let message = host_environment_message(session_started_at);
        let text = message.content[0].text.as_str();
        assert!(text.contains(std::env::consts::OS));
        assert!(text.contains(std::env::consts::ARCH));
        assert!(text.contains(&format!("session started at={session_started_at}")));
        assert!(text.contains(
            "use glob, grep, and read instead of running find, grep, or rg through bash"
        ));
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
        assert!(!text.contains(directory.path().to_str().unwrap()));
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
            nearby_instruction_files(project.path().to_str().unwrap(), "src/file.rs", &[],)
                .is_empty()
        );
    }

    async fn mock_deepseek(Json(body): Json<Value>) -> impl IntoResponse {
        let messages = body
            .get("messages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let last_role = messages
            .last()
            .and_then(|message| message.get("role"))
            .and_then(Value::as_str);
        let has_tool_error = messages.iter().any(|message| {
            message.get("role").and_then(Value::as_str) == Some("tool")
                && message
                    .get("content")
                    .and_then(Value::as_str)
                    .map(|content| content.contains("invalid_arguments"))
                    .unwrap_or(false)
        });
        let user_text = messages
            .iter()
            .rev()
            .find(|message| message.get("role").and_then(Value::as_str) == Some("user"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .unwrap_or_default();
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
        let data = if user_text.contains("invalid arguments") && !has_tool_error {
            vec![
                json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"invalid-read-call","function":{"name":"read","arguments":"{\"path\":123}"}}]},"finish_reason":"tool_calls"}]}),
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
        let body = data
            .into_iter()
            .map(|value| format!("data: {value}\n\n"))
            .collect::<String>()
            + "data: [DONE]\n\n";
        ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
    }

    async fn fixture() -> (
        Agent,
        Store,
        std::path::PathBuf,
        tokio::task::JoinHandle<()>,
        String,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/chat/completions", post(mock_deepseek)),
            )
            .await
            .unwrap();
        });
        let directory = tempfile::tempdir().unwrap().keep();
        std::fs::write(directory.join("README.md"), "hello").unwrap();
        let store = Store::open_memory().unwrap();
        let root = directory.canonicalize().unwrap();
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
        let (events, _) = broadcast::channel(64);
        let credentials = Arc::new(CredentialStore::memory(
            Some("test-key"),
            None,
            None,
            None,
            None,
            None,
        ));
        let provider = Arc::new(OpenAiCompatibleProvider::new(
            "deepseek",
            "DeepSeek",
            format!("http://{address}"),
            credentials,
        ));
        let mut registry = ModelProviderRegistry::new();
        let models = ["deepseek-v4-flash", "deepseek-v4-pro"]
            .into_iter()
            .enumerate()
            .map(|(_index, model_id)| ModelDescriptor {
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
                },
                reasoning_efforts: Vec::new(),
                limits: ModelLimits {
                    max_input_tokens: Some(64_000),
                    auto_compact_tokens: Some(47_616),
                    max_output_tokens: None,
                },
                availability: "configured".into(),
            })
            .collect();
        registry.register("deepseek", provider, models).unwrap();
        (
            Agent::new(store.clone(), Arc::new(registry), operations, events, false),
            store,
            root,
            server,
            session.session_id,
        )
    }

    #[tokio::test]
    async fn read_tool_round_trip_completes() {
        let (agent, store, root, server, session_id) = fixture().await;
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
        assert_eq!(
            exchanges[0].input_messages[0],
            exchanges[1].input_messages[0]
        );
        assert!(exchanges.iter().all(|exchange| exchange
            .input_messages
            .as_array()
            .unwrap()
            .iter()
            .any(|message| message["role"] == "system"
                && message["content"][0]["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("Always run focused tests.")))));
        let usage = exchanges[0].usage.as_ref().unwrap();
        assert_eq!(usage["cache_read_tokens"], 6);
        assert_eq!(usage["cache_miss_tokens"], 2);
        assert_eq!(usage["cache_write_tokens"], serde_json::Value::Null);
        assert_eq!(usage["reasoning_tokens"], 1);
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
            std::fs::read_to_string(root.join("dependency-source/lib.rs")).unwrap(),
            "pub fn shared() {}\n"
        );
        server.abort();
    }

    #[tokio::test]
    async fn queued_submit_is_injected_before_completion() {
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
            TurnResponse::Completed { iterations: 2, .. }
        ));
        assert!(store
            .messages(&session_id)
            .unwrap()
            .iter()
            .filter(|message| message.role == "user")
            .any(|message| message.text_content() == "follow up while running"));
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
        assert!(agent
            .resolve_approval(&approval_id, "allow_once")
            .await
            .unwrap());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if std::fs::read_to_string(root.join("README.md")).unwrap() == "updated" {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "approval continuation did not complete"
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
        assert!(store.session_full_control(&session_id).unwrap());

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
}
