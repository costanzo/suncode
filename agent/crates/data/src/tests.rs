use super::*;
use diesel::sql_types::BigInt;
use diesel::{
    connection::SimpleConnection, sql_query, Connection, QueryableByName, RunQueryDsl,
    SqliteConnection,
};
use serde_json::json;
use std::collections::BTreeMap;

const PREVIOUS_PROVIDER_SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS llm_model_provider (
        provider_id TEXT PRIMARY KEY CHECK(length(provider_id) > 0),
        display_name TEXT NOT NULL CHECK(length(display_name) > 0),
        endpoint TEXT NOT NULL CHECK(length(endpoint) > 0),
        default_endpoint TEXT NOT NULL CHECK(length(default_endpoint) > 0),
        adapter_type TEXT NOT NULL DEFAULT 'openai' CHECK(adapter_type IN ('openai')),
        api_key TEXT,
        enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
        sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS llm_model_provider_enabled_order_idx
        ON llm_model_provider(enabled, sort_order, provider_id);
"#;

#[test]
fn session_call_schema_does_not_retain_model_request_messages() {
    let store = Store::open_memory().unwrap();
    #[derive(QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let mut connection = store.connection.lock().unwrap();
    let columns = sql_query("PRAGMA table_info(session_call)")
        .load::<ColumnRow>(&mut *connection)
        .unwrap();
    assert!(columns
        .iter()
        .all(|column| column.name != "input_messages_json"));
}

#[test]
fn message_page_returns_latest_rows_and_walks_backward_without_duplicates() {
    let store = Store::open_memory().unwrap();
    let project = store.project("/tmp/page", "Page").unwrap();
    let session = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    for index in 0..25 {
        store.append_content(
            &session.session_id,
            "message.user",
            &json!({"message_id": format!("message-{index:02}"), "turn_id": format!("turn-{index:02}"), "message": Message::text("user", format!("message {index}"))}),
        ).unwrap();
    }
    let first = store
        .message_page(&session.session_id, 10, None, None)
        .unwrap();
    assert_eq!(first.messages.first().unwrap().message_id, "message-15");
    assert_eq!(first.messages.last().unwrap().message_id, "message-24");
    assert!(first.has_more);
    let second = store
        .message_page(
            &session.session_id,
            10,
            first.oldest_created_at.as_deref(),
            first.oldest_message_id.as_deref(),
        )
        .unwrap();
    assert_eq!(second.messages.first().unwrap().message_id, "message-05");
    assert_eq!(second.messages.last().unwrap().message_id, "message-14");
    assert!(second.messages.iter().all(|item| {
        !first
            .messages
            .iter()
            .any(|previous| previous.message_id == item.message_id)
    }));
    let third = store
        .message_page(
            &session.session_id,
            10,
            second.oldest_created_at.as_deref(),
            second.oldest_message_id.as_deref(),
        )
        .unwrap();
    assert!(!third.has_more);
    assert_eq!(third.messages.len(), 5);
    assert_eq!(third.messages.first().unwrap().message_id, "message-00");
}

#[test]
fn previous_provider_constraint_is_rebuilt_for_anthropic_without_losing_configuration() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    let mut connection = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
    for script in suncode_database::sqlite::schema_scripts() {
        if script.contains("CREATE TABLE IF NOT EXISTS llm_model_provider") {
            connection.batch_execute(PREVIOUS_PROVIDER_SCHEMA).unwrap();
        } else {
            connection.batch_execute(script).unwrap();
        }
    }
    for script in suncode_database::sqlite::data_scripts() {
        let compatible = script.replace("'anthropic', NULL, 1, 50", "'openai', NULL, 1, 50");
        connection.batch_execute(&compatible).unwrap();
    }
    connection
        .batch_execute(
            "UPDATE llm_model_provider SET api_key='preserved-key',enabled=0,sort_order=77 WHERE provider_id='claude';",
        )
        .unwrap();
    drop(connection);

    let store = Store::open(&path).unwrap();
    let claude = store
        .llm_model_providers(false)
        .unwrap()
        .into_iter()
        .find(|provider| provider.provider_id == "claude")
        .unwrap();
    assert_eq!(claude.adapter_type, "anthropic");
    assert_eq!(claude.endpoint, "https://api.anthropic.com/v1");
    assert_eq!(claude.default_endpoint, "https://api.anthropic.com/v1");
    assert!(!claude.enabled);
    assert_eq!(claude.sort_order, 77);
    assert_eq!(claude.created_at, "2026-08-19T00:00:00.000Z");
    assert_eq!(claude.updated_at, "2026-09-19T00:00:00.000Z");
    store
        .upsert_llm_model_provider(LlmModelProviderInput {
            provider_id: &claude.provider_id,
            display_name: &claude.display_name,
            endpoint: &claude.endpoint,
            default_endpoint: &claude.default_endpoint,
            adapter_type: &claude.adapter_type,
            enabled: true,
            sort_order: claude.sort_order,
        })
        .unwrap();
    assert_eq!(
        store.llm_provider_api_key("claude").unwrap(),
        Some("preserved-key".into())
    );
}

#[test]
fn existing_sixteen_table_database_without_mcp_receives_additive_table() {
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
fn immediately_previous_seventeen_table_database_receives_language_server_table() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    let mut connection = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
    for script in suncode_database::sqlite::schema_scripts()
        .iter()
        .filter(|script| !script.contains("CREATE TABLE IF NOT EXISTS language_server"))
    {
        connection.batch_execute(script).unwrap();
    }
    for script in suncode_database::sqlite::data_scripts() {
        connection.batch_execute(script).unwrap();
    }
    drop(connection);

    let store = Store::open(&path).unwrap();
    assert!(store.language_servers().unwrap().is_empty());
}

#[test]
fn immediately_previous_schema_receives_subagent_table_and_session_columns() {
    const PREVIOUS_SESSION_SCHEMA: &str = r#"
        CREATE TABLE IF NOT EXISTS session (
            session_id TEXT PRIMARY KEY CHECK(length(session_id) > 0),
            project_id TEXT NOT NULL,
            title TEXT,
            model_id TEXT,
            reasoning_effort TEXT,
            status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_activity_at TEXT NOT NULL,
            pin_at TEXT,
            archived_at TEXT,
            CHECK(
                (status = 'active' AND archived_at IS NULL)
                OR (status = 'archived' AND archived_at IS NOT NULL)
            )
        );
        CREATE INDEX IF NOT EXISTS session_project_activity_idx
            ON session(project_id, status, last_activity_at DESC, session_id);
    "#;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    let mut connection = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
    for script in suncode_database::sqlite::schema_scripts() {
        if script.contains("CREATE TABLE IF NOT EXISTS session_subagent_invocation") {
            continue;
        }
        if script.contains("CREATE TABLE IF NOT EXISTS session (") {
            connection.batch_execute(PREVIOUS_SESSION_SCHEMA).unwrap();
        } else {
            connection.batch_execute(script).unwrap();
        }
    }
    for script in suncode_database::sqlite::data_scripts() {
        connection.batch_execute(script).unwrap();
    }
    drop(connection);

    let store = Store::open(&path).unwrap();
    let project = store
        .project("/tmp/suncode-migrated-child", "Migrated")
        .unwrap();
    let parent = store
        .create_session(&project.project_id, Some("Parent"), Some("gpt-5.5"))
        .unwrap();
    let child = store
        .create_child_session(
            &project.project_id,
            &parent.session_id,
            "builtin.architect.v1",
            1,
            "Review architecture",
            "gpt-5.5",
        )
        .unwrap();
    assert_eq!(child.kind, "child");
    assert_eq!(
        child.parent_session_id.as_deref(),
        Some(parent.session_id.as_str())
    );
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
fn attention_candidates_include_only_eligible_normalized_state() {
    let store = Store::open_memory().unwrap();
    let project = store
        .project_for_user("attention-user", "/tmp/suncode-attention", "Attention")
        .unwrap();
    let primary = store
        .create_session(&project.project_id, Some("Primary"), Some("gpt-5.5"))
        .unwrap();
    let child = store
        .create_child_session(
            &project.project_id,
            &primary.session_id,
            "builtin.swe.v1",
            1,
            "Child",
            "gpt-5.5",
        )
        .unwrap();

    let completed = store
        .begin_turn(&primary.session_id, "completed", "done", "gpt-5.5")
        .unwrap();
    store
        .complete_turn(&primary.session_id, "completed", &json!({"ok":true}))
        .unwrap();
    let failed = store
        .begin_turn(&primary.session_id, "failed", "fail", "gpt-5.5")
        .unwrap();
    store
        .fail_turn(
            &primary.session_id,
            "failed",
            &json!({"code":"provider_error"}),
        )
        .unwrap();
    let child_completed = store
        .begin_turn(&child.session_id, "child-completed", "done", "gpt-5.5")
        .unwrap();
    store
        .complete_turn(&child.session_id, "child-completed", &json!({"ok":true}))
        .unwrap();

    let child_waiting = store
        .begin_turn(&child.session_id, "child-approval", "write", "gpt-5.5")
        .unwrap();
    let child_approval = store
        .create_approval(ApprovalInput {
            project_id: Some(&project.project_id),
            session_id: &child.session_id,
            turn_id: &child_waiting.turn_id,
            tool_call_id: "tool-1",
            operation: "write",
            arguments: &json!({"path":"README.md"}),
            snapshot: &json!({"turn_id":child_waiting.turn_id,"pending_call":{"name":"write"}}),
        })
        .unwrap();

    let question_turn = store
        .begin_turn(&primary.session_id, "question", "ask", "gpt-5.5")
        .unwrap();
    store
        .create_question(
            "question-1",
            &question_turn.turn_id,
            &json!({"turn_id":question_turn.turn_id,"pending_call":{"name":"question","call_id":"call-1","arguments":{"questions":[]}}}),
        )
        .unwrap();

    let candidates = store
        .attention_candidates("attention-user", "2000-01-01T00:00:00.000Z", 512)
        .unwrap();
    let keys = candidates
        .iter()
        .map(|candidate| (candidate.kind.as_str(), candidate.correlation_id.as_str()))
        .collect::<Vec<_>>();

    assert!(keys.contains(&("primary_turn_completed", completed.turn_id.as_str())));
    assert!(keys.contains(&("primary_turn_failed", failed.turn_id.as_str())));
    assert!(keys.contains(&("approval_requested", child_approval.approval_id.as_str())));
    assert!(keys.contains(&("question_asked", "question-1")));
    assert!(!keys.iter().any(|(_, id)| *id == child_completed.turn_id));
    let child_candidate = candidates
        .iter()
        .find(|candidate| candidate.correlation_id == child_approval.approval_id)
        .unwrap();
    assert_eq!(child_candidate.session_kind, "child");
    assert_eq!(
        child_candidate.parent_session_id.as_deref(),
        Some(primary.session_id.as_str())
    );
}

#[test]
fn child_sessions_are_linked_hidden_from_primary_lists_and_interrupted_on_recovery() {
    let store = Store::open_memory().unwrap();
    let project = store.project("/tmp/suncode-child", "Child").unwrap();
    let parent = store
        .create_session(&project.project_id, Some("Parent"), Some("gpt-5.5"))
        .unwrap();
    let child = store
        .create_child_session(
            &project.project_id,
            &parent.session_id,
            "builtin.swe.v1",
            1,
            "Implement the change",
            "gpt-5.5",
        )
        .unwrap();
    store
        .create_subagent_invocation(
            "invocation-1",
            &parent.session_id,
            "parent-turn-1",
            "parent-call-1",
            &child.session_id,
            "builtin.swe.v1",
            1,
            &json!({"text":"Implement the change"}),
            &json!(["read", "write"]),
            "gpt-5.5",
        )
        .unwrap();
    store
        .update_subagent_invocation(&child.session_id, "running", None, None)
        .unwrap();
    store
        .begin_turn(&child.session_id, "child-turn-key", "Implement", "gpt-5.5")
        .unwrap();

    let primary_sessions = store
        .sessions_for_project(&project.project_id, false)
        .unwrap();
    assert_eq!(primary_sessions.len(), 1);
    assert_eq!(primary_sessions[0].session_id, parent.session_id);
    let child_sessions = store.child_sessions_for_parent(&parent.session_id).unwrap();
    assert_eq!(child_sessions.len(), 1);
    assert_eq!(child_sessions[0].session_id, child.session_id);

    store.recover_startup().unwrap();
    let invocation = store
        .subagent_invocations_for_parent(&parent.session_id)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(invocation.state, "interrupted");
    assert_eq!(invocation.error_code.as_deref(), Some("runtime_restarted"));
    assert_eq!(store.session_ui_state(&child.session_id).unwrap(), "failed");
    #[derive(QueryableByName)]
    struct TurnError {
        #[diesel(sql_type = diesel::sql_types::Text)]
        state: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        error_json: Option<String>,
    }
    let turn = {
        let mut connection = store.connection.lock().unwrap();
        sql_query("SELECT state,error_json FROM session_turn WHERE session_id=?")
            .bind::<diesel::sql_types::Text, _>(&child.session_id)
            .get_result::<TurnError>(&mut *connection)
            .unwrap()
    };
    assert_eq!(turn.state, "interrupted");
    let error: serde_json::Value = serde_json::from_str(&turn.error_json.unwrap()).unwrap();
    assert_eq!(error["code"], "runtime_restarted");
}

#[test]
fn seeded_model_catalog_matches_current_provider_limits_and_capabilities() {
    let store = Store::open_memory().unwrap();
    let models = store.llm_models(false).unwrap();
    assert_eq!(models.len(), 25);
    let model = |id: &str| {
        models
            .iter()
            .find(|item| item.model_id == id)
            .unwrap_or_else(|| panic!("missing seeded model {id}"))
    };
    let deepseek_flash = model("deepseek-v4-flash");
    assert_eq!(deepseek_flash.request_model, "deepseek-flash");
    assert_eq!(deepseek_flash.context_tokens, 1_000_000);
    assert_eq!(deepseek_flash.max_output_tokens, Some(384_000));
    assert!(deepseek_flash.supports_vision);
    assert!(deepseek_flash.supports_reasoning_effort);
    let deepseek_pro = model("deepseek-v4-pro");
    assert_eq!(deepseek_pro.context_tokens, 1_000_000);
    assert!(!deepseek_pro.supports_vision);
    for id in ["qwen3.8-max", "qwen3.8-flash"] {
        let qwen = model(id);
        assert_eq!(qwen.request_model, id);
        assert_eq!(qwen.context_tokens, 1_000_000);
        assert_eq!(qwen.max_output_tokens, Some(131_072));
        assert!(qwen.supports_vision);
        assert!(qwen.supports_structured_output);
        assert!(!qwen.supports_reasoning_effort);
        assert!(!qwen.supports_computer_use);
    }
    for id in ["glm-5.2", "glm-5.3"] {
        let glm = model(id);
        assert_eq!(glm.context_tokens, 1_000_000);
        assert_eq!(glm.max_output_tokens, Some(128_000));
        assert!(glm.supports_reasoning_effort);
    }
    assert_eq!(model("glm-5.3").reasoning_efforts, ["low", "high", "max"]);
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
    assert_eq!(kimi_k2.max_output_tokens, Some(32_768));
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
    for id in ["gemini-3.6-flash", "gemini-3.8-flash"] {
        let gemini = model(id);
        assert_eq!(gemini.context_tokens, 1_048_576);
        assert_eq!(gemini.max_output_tokens, Some(65_536));
        assert!(gemini.supports_vision);
        assert!(gemini.supports_structured_output);
        assert!(gemini.supports_reasoning_effort);
    }
    assert_eq!(model("gemini-3.8-flash").request_model, "gemini-3.8-flash");
    assert_eq!(
        model("gemini-3.8-flash").reasoning_efforts,
        ["low", "medium", "high"]
    );
    assert!(models.iter().all(|item| item.model_id != "gemini-3.5"));
    let providers = store.llm_model_providers(true).unwrap();
    assert_eq!(providers.len(), 11);
    for (id, endpoint) in [
        ("minimax", "https://api.minimax.io/v1"),
        ("meta", "https://api.meta.ai/v1"),
        ("mimo", "https://api.xiaomimimo.com/v1"),
    ] {
        let provider = providers
            .iter()
            .find(|item| item.provider_id == id)
            .unwrap();
        assert_eq!(provider.endpoint, endpoint);
        assert_eq!(provider.default_endpoint, endpoint);
        assert_eq!(provider.adapter_type, "openai");
        assert!(!provider.api_key_configured);
    }
    for (id, provider, context, output, structured, efforts) in [
        (
            "MiniMax-M3.1-Flash-Preview",
            "minimax",
            1_000_000,
            None,
            false,
            vec!["low", "medium", "high", "xhigh", "max"],
        ),
        ("MiniMax-M3", "minimax", 1_000_000, None, false, vec![]),
        (
            "muse-wonder-1.0",
            "meta",
            1_048_576,
            None,
            true,
            vec!["minimal", "low", "medium", "high", "xhigh", "max"],
        ),
        (
            "muse-spark-1.3",
            "meta",
            1_048_576,
            Some(131_072),
            true,
            vec!["minimal", "low", "medium", "high", "xhigh", "max"],
        ),
        (
            "mimo-v2.6-pro",
            "mimo",
            1_048_576,
            Some(131_072),
            true,
            vec!["none", "high"],
        ),
        (
            "mimo-v2.6-flash",
            "mimo",
            1_048_576,
            Some(131_072),
            true,
            vec!["none", "high"],
        ),
    ] {
        let entry = model(id);
        assert_eq!(entry.provider_id, provider);
        assert_eq!(entry.request_model, id);
        assert_eq!(entry.context_tokens, context);
        assert_eq!(entry.auto_compact_tokens, context - 16_384);
        assert_eq!(entry.max_output_tokens, output);
        assert!(entry.supports_streaming);
        assert!(entry.supports_tool_use);
        assert!(entry.supports_vision);
        assert_eq!(entry.supports_structured_output, structured);
        assert!(entry.supports_cancellation);
        assert_eq!(entry.supports_reasoning_effort, !efforts.is_empty());
        assert_eq!(entry.reasoning_efforts, efforts);
        assert!(!entry.supports_computer_use);
    }
    let xai = providers
        .iter()
        .find(|item| item.provider_id == "xai")
        .unwrap();
    assert_eq!(xai.display_name, "xAI");
    assert_eq!(xai.endpoint, "https://api.x.ai/v1");
    assert_eq!(xai.default_endpoint, xai.endpoint);
    assert_eq!(xai.adapter_type, "openai");
    assert!(!xai.api_key_configured);
    for (id, context, efforts) in [
        ("grok-4.7", 500_000, vec!["low", "medium", "high", "xhigh"]),
        ("grok-4.6", 500_000, vec!["low", "medium", "high", "xhigh"]),
        ("grok-4.5", 500_000, vec!["low", "medium", "high"]),
        (
            "grok-4.3",
            1_000_000,
            vec!["none", "low", "medium", "high", "xhigh"],
        ),
        ("grok-build-0.1", 256_000, vec![]),
    ] {
        let grok = model(id);
        assert_eq!(grok.provider_id, "xai");
        assert_eq!(grok.request_model, id);
        assert_eq!(grok.context_tokens, context);
        assert_eq!(grok.auto_compact_tokens, context - 16_384);
        assert_eq!(grok.max_output_tokens, None);
        assert!(grok.supports_streaming);
        assert!(grok.supports_tool_use);
        assert!(grok.supports_vision);
        assert!(grok.supports_structured_output);
        assert!(grok.supports_cancellation);
        assert_eq!(grok.supports_reasoning_effort, !efforts.is_empty());
        assert_eq!(grok.reasoning_efforts, efforts);
        assert!(!grok.supports_computer_use);
    }
}

#[test]
fn reopening_existing_catalog_adds_new_providers_and_preserves_provider_settings() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    {
        let store = Store::open(&path).unwrap();
        let mut connection = store.connection.lock().unwrap();
        connection.batch_execute(
            "DELETE FROM llm_model WHERE provider_id IN ('xai', 'minimax', 'meta', 'mimo');
             DELETE FROM llm_model_provider WHERE provider_id IN ('xai', 'minimax', 'meta', 'mimo');
             UPDATE llm_model_provider SET endpoint='https://gateway.example.test/v1', api_key='test-existing-key' WHERE provider_id='openai';",
        ).unwrap();
    }
    for _ in 0..2 {
        let store = Store::open(&path).unwrap();
        let providers = store.llm_model_providers(true).unwrap();
        assert_eq!(providers.len(), 11);
        let openai = providers
            .iter()
            .find(|item| item.provider_id == "openai")
            .unwrap();
        assert_eq!(openai.endpoint, "https://gateway.example.test/v1");
        assert_eq!(
            store.llm_provider_api_key("openai").unwrap().as_deref(),
            Some("test-existing-key")
        );
        let models = store.llm_models(true).unwrap();
        assert_eq!(models.len(), 25);
        for (provider, ids) in [
            ("minimax", ["MiniMax-M3.1-Flash-Preview", "MiniMax-M3"]),
            ("meta", ["muse-wonder-1.0", "muse-spark-1.3"]),
            ("mimo", ["mimo-v2.6-pro", "mimo-v2.6-flash"]),
        ] {
            assert_eq!(
                models
                    .iter()
                    .filter(|item| item.provider_id == provider)
                    .map(|item| item.model_id.as_str())
                    .collect::<Vec<_>>(),
                ids
            );
        }
        let grok_ids = store
            .llm_models(true)
            .unwrap()
            .into_iter()
            .filter(|item| item.provider_id == "xai")
            .map(|item| item.model_id)
            .collect::<Vec<_>>();
        assert_eq!(
            grok_ids,
            [
                "grok-4.7",
                "grok-4.6",
                "grok-4.5",
                "grok-4.3",
                "grok-build-0.1"
            ]
        );
    }
}

#[test]
fn reopening_legacy_gemini_catalog_migrates_selections_and_preserves_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    let (project_id, session_id);
    {
        let store = Store::open(&path).unwrap();
        let project = store
            .project("/tmp/gemini-upgrade", "Gemini upgrade")
            .unwrap();
        let session = store
            .create_session(&project.project_id, None, Some("gemini-3.5"))
            .unwrap();
        project_id = project.project_id;
        session_id = session.session_id;
        store
            .set_setting(
                "project",
                &project_id,
                "default_model",
                &json!("gemini-3.5"),
            )
            .unwrap();
        store
            .set_setting("session", &session_id, "custom_note", &json!("gemini-3.5"))
            .unwrap();
        store
            .begin_turn(&session_id, "legacy-turn", "Hello", "gemini-3.5")
            .unwrap();
        let mut connection = store.connection.lock().unwrap();
        connection.batch_execute(
            "UPDATE llm_model SET model_id='gemini-3.5', display_name='Gemini 3.5 Flash (legacy SunCode ID)', request_model='gemini-3.5-flash' WHERE model_id='gemini-3.8-flash';
             UPDATE session SET reasoning_effort='minimal' WHERE model_id='gemini-3.5';",
        ).unwrap();
    }
    for _ in 0..2 {
        let store = Store::open(&path).unwrap();
        let session = store.session_by_id(&session_id).unwrap().unwrap();
        assert_eq!(session.model_id.as_deref(), Some("gemini-3.8-flash"));
        assert_eq!(session.reasoning_effort.as_deref(), Some("low"));
        assert_eq!(
            store
                .project_settings(&project_id)
                .unwrap()
                .default_model
                .as_deref(),
            Some("gemini-3.8-flash")
        );
        assert_eq!(
            store
                .settings(Some(&project_id), Some(&session_id))
                .unwrap()
                .into_iter()
                .find(|item| item.key == "custom_note")
                .unwrap()
                .value,
            json!("gemini-3.5")
        );
        let models = store.llm_models(true).unwrap();
        assert!(models.iter().all(|model| model.model_id != "gemini-3.5"));
        assert_eq!(
            models
                .iter()
                .filter(|model| model.provider_id == "gemini")
                .count(),
            2
        );
        assert_eq!(
            store.session_trace_turns(&session_id).unwrap()[0]
                .model_id
                .as_deref(),
            Some("gemini-3.5")
        );
    }
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
    assert!(conversation[0].changed_paths.is_empty());
}

#[test]
fn conversation_turns_list_checkpointed_paths_once_in_first_touched_order() {
    let store = Store::open_memory().unwrap();
    let project = store
        .project("/tmp/suncode-changed-paths", "Changed paths")
        .unwrap();
    let session = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    for turn in ["turn-1", "turn-2"] {
        store
            .append_content(
                &session.session_id,
                "turn.state",
                &json!({"turn_id":turn,"state":"resolving_calls"}),
            )
            .unwrap();
    }
    let checkpoints = [
        ("cp-1", "turn-1", Some("src/main.rs"), 0),
        ("cp-2", "turn-1", Some("README.md"), 1),
        ("cp-3", "turn-1", Some("src/main.rs"), 2),
        ("cp-4", "turn-1", Some(""), 3),
        ("cp-5", "turn-1", None, 4),
    ];
    for (checkpoint_id, turn_id, path, ordinal) in checkpoints {
        store
            .append_content(
                &session.session_id,
                "checkpoint.captured",
                &json!({"turn_id":turn_id,"tool_call_id":"call-1","manifest_id":"manifest-1","checkpoint_id":checkpoint_id,"path":path,"ordinal":ordinal}),
            )
            .unwrap();
    }
    store
        .append_content(
            &session.session_id,
            "checkpoint.item_restored",
            &json!({"manifest_id":"manifest-1","checkpoint_id":"cp-2","path":"README.md"}),
        )
        .unwrap();

    let conversation = store
        .session_conversation_turns(&session.session_id)
        .unwrap();

    assert_eq!(conversation[0].changed_paths, ["src/main.rs", "README.md"]);
    assert!(conversation[1].changed_paths.is_empty());
    let json = serde_json::to_value(&conversation[0]).unwrap();
    assert_eq!(json["changedPaths"], json!(["src/main.rs", "README.md"]));
}

#[test]
fn context_messages_replay_persisted_compaction_and_new_messages() {
    let store = Store::open_memory().unwrap();
    let project = store
        .project("/tmp/suncode-compaction-replay", "Compaction")
        .unwrap();
    let session = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    store
        .append_content(
            &session.session_id,
            "turn.state",
            &json!({"turn_id":"turn-1","state":"calling_model"}),
        )
        .unwrap();
    store.append_content(&session.session_id, "message.user", &json!({"message_id":"old","turn_id":"turn-1","message":Message::text("user", "old objective")})).unwrap();
    store.append_content(&session.session_id, "context.compacted", &json!({
        "exchange_id":"compact-1","turn_id":"turn-1","provider":"SunCode","model_id":"context-compaction","wire_model":"internal","iteration":1,
        "original_characters":1000,"retained_characters":200,"original_tokens":250,"retained_tokens":50,"dropped_messages":1,
        "summary":{"objective":"old objective","important_constraints":[],"completed_work":["old"],"active_work":[],"blockers":[],"next_action":"continue"},
        "retained_messages":[{"role":"user","content":[{"type":"text","text":"retained objective"}],"tool_calls":[],"tool_call_id":null}]
    })).unwrap();
    store.append_content(&session.session_id, "message.user", &json!({"message_id":"new","turn_id":"turn-2","message":Message::text("user", "new request")})).unwrap();
    {
        let mut connection = store.connection.lock().unwrap();
        sql_query("UPDATE session_message SET created_at=(SELECT completed_at FROM session_call WHERE call_id='compact-1') WHERE message_id='new'")
            .execute(&mut *connection)
            .unwrap();
    }
    let context = store.context_messages(&session.session_id).unwrap();
    assert!(context
        .iter()
        .any(|message| message.text_content().contains("suncode_context_summary")));
    assert!(context
        .iter()
        .any(|message| message.text_content() == "retained objective"));
    assert!(context
        .iter()
        .any(|message| message.text_content() == "new request"));
    assert!(!context
        .iter()
        .any(|message| message.text_content() == "old objective"));
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
fn deleting_archived_session_removes_descendants_and_all_owned_rows() {
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let store = Store::open_memory().unwrap();
    let project = store.project("/tmp/suncode-delete", "Delete").unwrap();
    let parent = store
        .create_session(&project.project_id, Some("Parent"), Some("gpt-5.5"))
        .unwrap();
    let child = store
        .create_child_session(
            &project.project_id,
            &parent.session_id,
            "builtin.qa.v1",
            1,
            "Child",
            "gpt-5.5",
        )
        .unwrap();
    let parent_turn = store
        .begin_turn(&parent.session_id, "parent-key", "Inspect", "gpt-5.5")
        .unwrap();
    let _child_turn = store
        .begin_turn(&child.session_id, "child-key", "Inspect", "gpt-5.5")
        .unwrap();
    store
        .append_content(
            &parent.session_id,
            "message.user",
            &json!({"message_id":"message-delete","turn_id":parent_turn.turn_id,"message":{"role":"user","content":[{"type":"text","text":"Inspect"}]}}),
        )
        .unwrap();
    store
        .append_content(
            &parent.session_id,
            "todo.updated",
            &json!({"turn_id":parent_turn.turn_id,"todos":[{"content":"Inspect","status":"completed","priority":"high"}]}),
        )
        .unwrap();
    store
        .append_content(
            &parent.session_id,
            "tool.requested",
            &json!({"turn_id":parent_turn.turn_id,"tool_call_id":"tool-delete","name":"read","arguments":{"path":"README.md"}}),
        )
        .unwrap();
    store
        .append_content(
            &parent.session_id,
            "provider.exchange.started",
            &json!({"exchange_id":"call-delete","turn_id":parent_turn.turn_id,"provider":"openai","model_id":"gpt-5.5","wire_model":"gpt-5.5","iteration":1}),
        )
        .unwrap();
    store
        .append_content(
            &parent.session_id,
            "checkpoint.captured",
            &json!({"checkpoint_id":"checkpoint-delete","manifest_id":"manifest-delete","turn_id":parent_turn.turn_id,"path":"README.md"}),
        )
        .unwrap();
    {
        let mut connection = store.connection.lock().unwrap();
        sql_query("INSERT INTO session_checkpoint_manifest(manifest_id,session_id,turn_id,status,created_at,updated_at,expires_at) VALUES ('manifest-delete',? ,? ,'available','now','now','2099-01-01')")
            .bind::<diesel::sql_types::Text, _>(&parent.session_id)
            .bind::<diesel::sql_types::Text, _>(&parent_turn.turn_id)
            .execute(&mut *connection)
            .unwrap();
    }
    store
        .create_approval(ApprovalInput {
            project_id: Some(&project.project_id),
            session_id: &parent.session_id,
            turn_id: &parent_turn.turn_id,
            tool_call_id: "tool-delete",
            operation: "read",
            arguments: &json!({"path":"README.md"}),
            snapshot: &json!({"turn_id":parent_turn.turn_id}),
        })
        .unwrap();
    store
        .create_subagent_invocation(
            "invocation-delete",
            &parent.session_id,
            &parent_turn.turn_id,
            "tool-delete",
            &child.session_id,
            "builtin.qa.v1",
            1,
            &json!({"text":"Inspect"}),
            &json!(["read"]),
            "gpt-5.5",
        )
        .unwrap();
    store
        .set_setting("session", &parent.session_id, "full_control", &json!(true))
        .unwrap();
    store
        .set_session_archived(&parent.session_id, true)
        .unwrap();

    let managed_path = tempfile::NamedTempFile::new().unwrap();
    let managed_path = managed_path.path().to_string_lossy().to_string();
    store
        .insert_session_image(
            "image-delete",
            &parent.session_id,
            "diagram.png",
            "file",
            Some("/source/diagram.png"),
            std::path::Path::new(&managed_path),
            "dGh1bWI=",
        )
        .unwrap();

    let returned_paths = store.delete_session(&parent.session_id).unwrap();
    assert_eq!(returned_paths, vec![managed_path]);
    assert!(store.session_by_id(&parent.session_id).unwrap().is_none());
    assert!(store.session_by_id(&child.session_id).unwrap().is_none());

    let mut connection = store.connection.lock().unwrap();
    for table in [
        "session_turn_todo",
        "session_tool_use",
        "session_approval_request",
        "session_call",
        "session_message",
        "session_checkpoint",
        "session_checkpoint_manifest",
        "session_image",
        "configuration",
        "session_subagent_invocation",
        "session_turn",
        "session",
    ] {
        let query = if table == "configuration" {
            format!("SELECT COUNT(*) AS count FROM {table} WHERE session_id IS NOT NULL")
        } else {
            format!("SELECT COUNT(*) AS count FROM {table}")
        };
        let count = sql_query(query)
            .get_result::<CountRow>(&mut *connection)
            .unwrap()
            .count;
        assert_eq!(count, 0, "table {table} still contains session data");
    }
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

#[test]
fn language_server_crud_preserves_configuration_and_enforces_revisions() {
    let store = Store::open_memory().unwrap();
    let input = LanguageServerInput {
        display_name: "Rust Analyzer".into(),
        config: LanguageServerConfig {
            version: 1,
            project_id: None,
            server_kind: None,
            command: "rust-analyzer".into(),
            arguments: Vec::new(),
            language_ids: vec!["rust".into()],
            root_markers: vec!["Cargo.toml".into()],
            initialization_options: json!({"check":{"command":"clippy"}}),
            configuration: json!({}),
            environment: BTreeMap::from([("RUST_LOG".into(), "warn".into())]),
            startup_timeout_seconds: 30,
            request_timeout_seconds: 30,
            java_home: None,
            data_path_mode: None,
            data_path: None,
            vtsls_executable: None,
            node_path: None,
            typescript_sdk_mode: None,
            typescript_sdk_path: None,
            max_ts_server_memory: None,
            basedpyright_executable: None,
            python_path: None,
            diagnostic_mode: None,
            type_checking_mode: None,
            config_path_mode: None,
            config_path: None,
        },
        enabled: true,
        sort_order: 0,
    };
    let created = store.create_language_server("lsp-1", &input).unwrap();
    assert_eq!(created.revision, 1);
    assert_eq!(created.config.environment["RUST_LOG"], "warn");
    assert_eq!(
        store.create_language_server("lsp-1", &input).unwrap(),
        created
    );

    let mut updated_input = input.clone();
    updated_input.display_name = "Rust".into();
    let updated = store
        .update_language_server("lsp-1", 1, &updated_input)
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(
        store
            .update_language_server("lsp-1", 1, &input)
            .unwrap_err()
            .code,
        "language_server_revision_conflict"
    );
    let disabled = store
        .set_language_server_enabled("lsp-1", 2, false)
        .unwrap();
    assert!(!disabled.enabled);
    assert!(store.delete_language_server("lsp-1", 3).unwrap());
}

#[test]
fn language_server_version_two_json_defaults_new_runtime_configuration_fields() {
    let config: LanguageServerConfig = serde_json::from_value(json!({
        "version": 2,
        "projectId": "project-1",
        "serverKind": "jdtls",
        "command": "jdtls",
        "arguments": [],
        "languageIds": ["java"],
        "rootMarkers": ["pom.xml"],
        "initializationOptions": {},
        "environment": {},
        "startupTimeoutSeconds": 120,
        "requestTimeoutSeconds": 30,
        "javaHome": null,
        "dataPathMode": "managed",
        "dataPath": "/tmp/jdtls"
    }))
    .unwrap();

    assert_eq!(config.configuration, json!({}));
    assert!(config.vtsls_executable.is_none());
    assert!(config.node_path.is_none());
    assert!(config.typescript_sdk_mode.is_none());
    assert!(config.typescript_sdk_path.is_none());
    assert!(config.max_ts_server_memory.is_none());
    assert!(config.basedpyright_executable.is_none());
    assert!(config.python_path.is_none());
    assert!(config.diagnostic_mode.is_none());
    assert!(config.type_checking_mode.is_none());
    assert!(config.config_path_mode.is_none());
    assert!(config.config_path.is_none());
}

#[test]
fn language_server_validation_rejects_unsafe_configuration() {
    let store = Store::open_memory().unwrap();
    let input = LanguageServerInput {
        display_name: "Broken".into(),
        config: LanguageServerConfig {
            version: 1,
            project_id: None,
            server_kind: None,
            command: "server".into(),
            arguments: Vec::new(),
            language_ids: vec!["rust".into(), "RUST".into()],
            root_markers: vec!["../outside".into()],
            initialization_options: json!([]),
            configuration: json!({}),
            environment: BTreeMap::new(),
            startup_timeout_seconds: 0,
            request_timeout_seconds: 0,
            java_home: None,
            data_path_mode: None,
            data_path: None,
            vtsls_executable: None,
            node_path: None,
            typescript_sdk_mode: None,
            typescript_sdk_path: None,
            max_ts_server_memory: None,
            basedpyright_executable: None,
            python_path: None,
            diagnostic_mode: None,
            type_checking_mode: None,
            config_path_mode: None,
            config_path: None,
        },
        enabled: true,
        sort_order: 0,
    };
    assert_eq!(
        store
            .create_language_server("lsp-invalid", &input)
            .unwrap_err()
            .code,
        "invalid_arguments"
    );

    let oversized_order = LanguageServerInput {
        display_name: "Oversized order".into(),
        config: LanguageServerConfig {
            version: 1,
            project_id: None,
            server_kind: None,
            command: "server".into(),
            arguments: Vec::new(),
            language_ids: vec!["rust".into()],
            root_markers: Vec::new(),
            initialization_options: json!({}),
            configuration: json!({}),
            environment: BTreeMap::new(),
            startup_timeout_seconds: 30,
            request_timeout_seconds: 30,
            java_home: None,
            data_path_mode: None,
            data_path: None,
            vtsls_executable: None,
            node_path: None,
            typescript_sdk_mode: None,
            typescript_sdk_path: None,
            max_ts_server_memory: None,
            basedpyright_executable: None,
            python_path: None,
            diagnostic_mode: None,
            type_checking_mode: None,
            config_path_mode: None,
            config_path: None,
        },
        enabled: true,
        sort_order: i64::from(i32::MAX) + 1,
    };
    assert_eq!(
        store
            .create_language_server("lsp-oversized-order", &oversized_order)
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
}

#[test]
fn new_database_records_schema_version_and_reopens() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    for _ in 0..2 {
        let store = Store::open(&path).unwrap();
        let mut connection = store.connection.lock().unwrap();
        assert_eq!(
            crate::store::user_version(&mut connection).unwrap(),
            crate::store::SCHEMA_VERSION
        );
    }
}

#[test]
fn newer_schema_version_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent.sqlite3");
    {
        let store = Store::open(&path).unwrap();
        let mut connection = store.connection.lock().unwrap();
        connection
            .batch_execute(&format!(
                "PRAGMA user_version={}",
                crate::store::SCHEMA_VERSION + 1
            ))
            .unwrap();
    }
    let error = Store::open(&path).err().unwrap();
    assert!(error.message.contains("newer than this build supports"));
}

#[test]
fn new_rows_get_kind_prefixed_ids_and_legacy_uuid_rows_still_load() {
    let store = Store::open_memory().unwrap();
    let project = store.project("/tmp/ids", "Ids").unwrap();
    assert!(
        project.project_id.starts_with("prj_"),
        "{}",
        project.project_id
    );
    let dependency = store
        .add_project_dependency(&project.project_id, "/tmp/ids-dependency", "Dependency")
        .unwrap();
    assert!(dependency.dependency_id.starts_with("dep_"));
    let first = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    let second = store
        .create_session(&project.project_id, None, None)
        .unwrap();
    assert!(first.session_id.starts_with("ses_"));
    assert!(first.session_id < second.session_id, "ids sort by creation");
    let turn = store
        .begin_turn(&second.session_id, "key-1", "hello", "model")
        .unwrap();
    assert!(turn.turn_id.starts_with("trn_"), "{}", turn.turn_id);

    // Rows written before prefixed ids keep their bare UUIDs.
    let legacy = "6f1d3c1e-8b4a-4f6e-9a7d-2c5b8e9f0a1b";
    {
        let mut connection = store.connection.lock().unwrap();
        sql_query("UPDATE session SET session_id=? WHERE session_id=?")
            .bind::<diesel::sql_types::Text, _>(legacy)
            .bind::<diesel::sql_types::Text, _>(&first.session_id)
            .execute(&mut *connection)
            .unwrap();
    }
    let loaded = store.session_by_id(legacy).unwrap().unwrap();
    assert_eq!(loaded.session_id, legacy);
}
