use super::*;
use diesel::{connection::SimpleConnection, Connection, SqliteConnection};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn existing_fifteen_table_database_receives_additive_mcp_table() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    let mut connection = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
    for script in suncode_database::sqlite::schema_scripts()
        .iter()
        .filter(|script| !script.contains("CREATE TABLE IF NOT EXISTS mcp_server"))
    {
        connection.batch_execute(script).unwrap();
    }
    for script in suncode_database::sqlite::data_scripts() {
        connection.batch_execute(script).unwrap();
    }
    drop(connection);

    let store = Store::open(&path).unwrap();
    assert!(store.mcp_servers().unwrap().is_empty());
}

#[test]
fn diesel_store_round_trips_project_and_session() {
    let store = Store::open_memory().unwrap();
    assert_eq!(
        store
            .settings(None, None)
            .unwrap()
            .into_iter()
            .find(|setting| setting.key == "verify_https_certificates")
            .map(|setting| setting.value),
        Some(json!(true))
    );
    let project = store.project("/tmp/suncode-diesel", "Diesel").unwrap();
    let session = store
        .create_session(&project.project_id, Some("Test"), None)
        .unwrap();
    assert_eq!(
        store
            .project_by_id(&project.project_id)
            .unwrap()
            .unwrap()
            .display_name,
        "Diesel"
    );
    assert_eq!(
        store
            .session_by_id(&session.session_id)
            .unwrap()
            .unwrap()
            .title
            .as_deref(),
        Some("Test")
    );
}

#[test]
fn seeded_model_catalog_matches_current_provider_limits_and_capabilities() {
    let store = Store::open_memory().unwrap();
    let models = store.llm_models(false).unwrap();
    assert_eq!(models.len(), 12);
    let model = |id: &str| {
        models
            .iter()
            .find(|item| item.model_id == id)
            .unwrap_or_else(|| panic!("missing seeded model {id}"))
    };
    let deepseek_flash = model("deepseek-v4-flash");
    assert_eq!(deepseek_flash.request_model, "deepseek-v4-flash-vision-exp");
    assert_eq!(deepseek_flash.context_tokens, 1_000_000);
    assert_eq!(deepseek_flash.max_output_tokens, Some(128_000));
    assert!(deepseek_flash.supports_vision);
    let deepseek_pro = model("deepseek-v4-pro");
    assert_eq!(deepseek_pro.context_tokens, 1_000_000);
    assert!(!deepseek_pro.supports_vision);
    for id in ["glm-5.2", "glm-5.3"] {
        let glm = model(id);
        assert_eq!(glm.context_tokens, 1_000_000);
        assert_eq!(glm.max_output_tokens, Some(128_000));
        assert!(glm.supports_reasoning_effort);
    }
    assert_eq!(model("glm-5.3").reasoning_efforts, ["low", "high"]);
    for id in ["gpt-5.6-sol", "gpt-5.5"] {
        let openai = model(id);
        assert_eq!(openai.context_tokens, 1_048_576);
        assert_eq!(openai.max_output_tokens, Some(128_000));
        assert!(openai.supports_vision);
        assert!(openai.supports_structured_output);
        assert!(openai.supports_reasoning_effort);
    }
    assert_eq!(model("gpt-5.5").request_model, "gpt-5.6-terra");
    let kimi_k2 = model("kimi-k2.7-code");
    assert_eq!(kimi_k2.context_tokens, 262_144);
    assert_eq!(kimi_k2.max_output_tokens, Some(262_144));
    assert!(kimi_k2.supports_vision);
    let kimi_k3 = model("kimi-k3");
    assert_eq!(kimi_k3.context_tokens, 1_000_000);
    assert!(kimi_k3.supports_vision);
    for id in ["claude-opus-5", "claude-sonnet-5"] {
        let claude = model(id);
        assert_eq!(claude.context_tokens, 1_000_000);
        assert_eq!(claude.max_output_tokens, Some(128_000));
        assert!(claude.supports_vision);
        assert!(claude.supports_structured_output);
        assert!(claude.supports_reasoning_effort);
    }
    for id in ["gemini-3.6-flash", "gemini-3.5"] {
        let gemini = model(id);
        assert_eq!(gemini.context_tokens, 1_048_576);
        assert_eq!(gemini.max_output_tokens, Some(65_536));
        assert!(gemini.supports_vision);
        assert!(gemini.supports_structured_output);
        assert!(gemini.supports_reasoning_effort);
    }
    assert_eq!(model("gemini-3.5").request_model, "gemini-3.5-flash");
}

#[test]
fn diesel_projection_persists_messages_tools_and_todos() {
    let store = Store::open_memory().unwrap();
    let project = store
        .project("/tmp/suncode-diesel-projection", "Projection")
        .unwrap();
    let session = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    store
        .append_content(
            &session.session_id,
            "turn.state",
            &json!({"turn_id":"turn-1","state":"resolving_calls"}),
        )
        .unwrap();
    store.append_content(&session.session_id, "message.user", &json!({"message_id":"message-1","turn_id":"turn-1","message":Message::text("user", "hello")})).unwrap();
    store.append_content(&session.session_id, "todo.updated", &json!({"turn_id":"turn-1","todos":[{"content":"Inspect","status":"completed","priority":"high"}]})).unwrap();
    let conversation = store
        .session_conversation_turns(&session.session_id)
        .unwrap();
    assert_eq!(
        conversation[0].messages[0].message,
        serde_json::to_value(Message::text("user", "hello")).unwrap()
    );
    assert_eq!(conversation[0].todos[0].status, "completed");
}

#[test]
fn latest_failed_turn_input_returns_persisted_submission() {
    let store = Store::open_memory().unwrap();
    let project = store.project("/tmp/suncode-retry", "Retry").unwrap();
    let session = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    let admission = store
        .begin_turn_with_images(
            &session.session_id,
            "key-1",
            "retry me",
            "model-x",
            Some("high"),
            &["img-1".to_string()],
        )
        .unwrap();
    let updated_session = store.session_by_id(&session.session_id).unwrap().unwrap();
    assert_eq!(updated_session.model_id.as_deref(), Some("model-x"));
    assert_eq!(updated_session.reasoning_effort.as_deref(), Some("high"));
    store
        .fail_turn(
            &session.session_id,
            "key-1",
            &json!({"code":"provider_failed","message":"boom"}),
        )
        .unwrap();
    let latest = store
        .latest_failed_turn_input(&session.session_id)
        .unwrap()
        .unwrap();
    assert_eq!(latest.0, "retry me");
    assert_eq!(latest.1, "model-x");
    assert_eq!(latest.2, vec!["img-1"]);
    assert!(!admission.turn_id.is_empty());
}

#[test]
fn session_ui_state_tracks_running_failure_and_idle_turns() {
    let store = Store::open_memory().unwrap();
    let project = store
        .project("/tmp/suncode-session-state", "State")
        .unwrap();
    let session = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    assert_eq!(store.session_ui_state(&session.session_id).unwrap(), "idle");
    store
        .begin_turn(&session.session_id, "state-1", "run", "gpt-5.5")
        .unwrap();
    assert_eq!(
        store.session_ui_state(&session.session_id).unwrap(),
        "running"
    );
    store
        .fail_turn(&session.session_id, "state-1", &json!({"code":"test"}))
        .unwrap();
    assert_eq!(
        store.session_ui_state(&session.session_id).unwrap(),
        "failed"
    );
    store
        .begin_turn(&session.session_id, "state-2", "finish", "gpt-5.5")
        .unwrap();
    store
        .complete_turn(
            &session.session_id,
            "state-2",
            &json!({"status":"completed"}),
        )
        .unwrap();
    assert_eq!(store.session_ui_state(&session.session_id).unwrap(), "idle");
}

#[test]
fn mcp_server_crud_preserves_prefix_and_enforces_revisions() {
    let store = Store::open_memory().unwrap();
    let input = McpServerInput {
        display_name: "Local Files".into(),
        transport: McpTransportConfig::Stdio {
            version: 1,
            command: "mcp-files".into(),
            arguments: vec!["--stdio".into()],
            working_directory: McpWorkingDirectory::Project,
            environment: BTreeMap::from([("TOKEN".into(), "secret".into())]),
            startup_timeout_seconds: 30,
            request_timeout_seconds: 60,
        },
        enabled: true,
        sort_order: 0,
    };
    let created = store.create_mcp_server("mcp-1", &input).unwrap();
    assert_eq!(created.tool_prefix, "local_files");
    assert_eq!(created.revision, 1);
    assert_eq!(store.create_mcp_server("mcp-1", &input).unwrap(), created);

    let mut update = input.clone();
    update.display_name = "Renamed Files".into();
    let updated = store.update_mcp_server("mcp-1", 1, &update).unwrap();
    assert_eq!(updated.tool_prefix, "local_files");
    assert_eq!(updated.revision, 2);
    let error = store.update_mcp_server("mcp-1", 1, &input).unwrap_err();
    assert_eq!(error.code, "mcp_server_revision_conflict");

    let disabled = store.set_mcp_server_enabled("mcp-1", 2, false).unwrap();
    assert!(!disabled.enabled);
    assert_eq!(disabled.revision, 3);
    assert_eq!(store.mcp_servers().unwrap(), vec![disabled.clone()]);
    assert!(store.delete_mcp_server("mcp-1", 3).unwrap());
    assert!(!store.delete_mcp_server("mcp-1", 3).unwrap());
}

#[test]
fn mcp_server_rejects_duplicate_names_prefixes_and_insecure_remote_urls() {
    let store = Store::open_memory().unwrap();
    let input = |name: &str, url: &str| McpServerInput {
        display_name: name.into(),
        transport: McpTransportConfig::StreamableHttp {
            version: 1,
            url: url.into(),
            headers: BTreeMap::new(),
            startup_timeout_seconds: 30,
            request_timeout_seconds: 60,
        },
        enabled: true,
        sort_order: 0,
    };
    store
        .create_mcp_server("mcp-1", &input("GitHub MCP", "https://example.com/mcp"))
        .unwrap();
    assert_eq!(
        store
            .create_mcp_server("mcp-2", &input("github mcp", "https://example.org/mcp"))
            .unwrap_err()
            .code,
        "mcp_server_conflict"
    );
    assert_eq!(
        store
            .create_mcp_server("mcp-3", &input("Other", "http://example.org/mcp"))
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
    store
        .create_mcp_server("mcp-4", &input("Loopback", "http://127.0.0.1:3000/mcp"))
        .unwrap();
}
