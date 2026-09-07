use super::*;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};
use suncode_agent::domain::Message;

fn test_sdk(directory: &std::path::Path) -> AgentSdk {
    let state = test_state(directory);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("suncode-sdk-test")
        .build()
        .unwrap();
    AgentSdk {
        _lock: None,
        data_dir: directory.to_path_buf(),
        runtime,
        state,
    }
}

fn test_state(directory: &std::path::Path) -> AgentState {
    let store = Store::open_memory().unwrap();
    let verify_https_certificates = Arc::new(AtomicBool::new(true));
    let operations = Arc::new(
        suncode_tool::Operations::new_with_https_certificate_verification(
            directory.join("operations"),
            verify_https_certificates.clone(),
        )
        .unwrap(),
    );
    store
        .set_llm_provider_api_key("deepseek", "test-key")
        .unwrap();
    let (events, _) = broadcast::channel(16);
    let providers = Arc::new(
        registry_from_store(
            &store,
            Arc::new(SqliteApiKeyResolver {
                store: store.clone(),
            }),
            verify_https_certificates.clone(),
            Arc::new(AtomicBool::new(true)),
            Arc::new(RwLock::new(None)),
        )
        .unwrap(),
    );
    let agent = Agent::new(
        store.clone(),
        providers.clone(),
        operations.clone(),
        events.clone(),
        false,
    );
    AgentState {
        store,
        operations,
        active_project: Arc::new(Mutex::new(None)),
        events,
        verify_https_certificates,
        use_system_certificates: Arc::new(AtomicBool::new(true)),
        certificate_path: Arc::new(RwLock::new(None)),
        agent,
        providers,
    }
}

#[test]
fn named_sdk_methods_serve_project_session_and_model_dtos() {
    let directory = tempfile::tempdir().unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));
    assert_eq!(AgentSdk::version().version, suncode_agent::version());
    assert!(sdk.health().unwrap().ok);
    assert_eq!(sdk.list_credentials().unwrap().credentials.len(), 6);
    assert_eq!(sdk.list_models().unwrap().models.len(), 12);
    let project = sdk
        .open_project(directory.path().to_str().unwrap(), None)
        .unwrap();
    let session = sdk
        .create_session(&project.project_id, Some("First"), Some("gpt-5.5"))
        .unwrap();
    assert_eq!(
        session.project_id.as_deref(),
        Some(project.project_id.as_str())
    );
    assert_eq!(session.model_id.as_deref(), Some("gpt-5.5"));
    let usage = sdk.session_usage(&session.session_id).unwrap();
    assert_eq!(usage.session_id, session.session_id);
    assert_eq!(usage.total_tokens, 0);
    sdk.state
        .store
        .append_content(
            &session.session_id,
            "turn.state",
            &json!({
                "turn_id":"turn-1",
                "state":"calling_model",
                "model_id":"gpt-5.5"
            }),
        )
        .unwrap();
    sdk.state
            .store
            .append_content(
                &session.session_id,
                "provider.exchange.started",
                &json!({
                    "exchange_id":"exchange-1",
                    "turn_id":"turn-1",
                    "provider":"openai",
                    "model_id":"gpt-5.5",
                    "wire_model":"gpt-5.5",
                    "iteration":1,
                    "input_messages":[{"role":"user","content":[{"type":"text","text":"Inspect package.json"}]}]
                }),
            )
            .unwrap();
    let traces = sdk.list_provider_exchanges(&session.session_id).unwrap();
    assert_eq!(traces.exchanges.len(), 1);
    assert_eq!(traces.turns.len(), 1);
    assert_eq!(
        sdk.provider_exchange(&session.session_id, "exchange-1")
            .unwrap()
            .exchange
            .provider,
        "openai"
    );
    let details = sdk
        .provider_exchange(&session.session_id, "exchange-1")
        .unwrap();
    assert!(details.messages.is_empty());
    assert!(details.tool_uses.is_empty());
    assert_eq!(
        sdk.session_usage("missing-session").unwrap_err().code,
        "session_not_found"
    );
}

#[test]
fn project_default_model_is_used_when_session_model_is_omitted() {
    let directory = tempfile::tempdir().unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));
    let project = sdk
        .open_project(directory.path().to_str().unwrap(), None)
        .unwrap();
    sdk.set_setting(
        "project",
        Some(&project.project_id),
        None,
        "default_model",
        &json!("gpt-5.5"),
    )
    .unwrap();

    let session = sdk.create_session(&project.project_id, None, None).unwrap();
    assert_eq!(session.model_id.as_deref(), Some("gpt-5.5"));
    sdk.set_setting(
        "session",
        None,
        Some(&session.session_id),
        "full_control",
        &json!(true),
    )
    .unwrap();
    let setting = sdk
        .list_settings(Some(&project.project_id), Some(&session.session_id))
        .unwrap()
        .settings
        .into_iter()
        .find(|setting| setting.key == "full_control")
        .unwrap();
    assert_eq!(setting.value, json!(true));
    assert_eq!(setting.scope, "session");
}

#[test]
fn project_dependencies_are_read_only_and_browsed_on_demand() {
    let directory = tempfile::tempdir().unwrap();
    let project_root = directory.path().join("project");
    let dependency_root = directory.path().join("dependency");
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::create_dir_all(dependency_root.join("lib")).unwrap();
    std::fs::create_dir_all(dependency_root.join("nested")).unwrap();
    std::fs::write(dependency_root.join("lib/code.rs"), "pub fn shared() {}\n").unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));
    let project = sdk
        .open_project(project_root.to_str().unwrap(), None)
        .unwrap();
    let dependency = sdk
        .add_project_dependency(&project.project_id, dependency_root.to_str().unwrap())
        .unwrap();
    assert_eq!(
        sdk.list_project_dependencies(&project.project_id)
            .unwrap()
            .dependencies
            .len(),
        1
    );
    let root = sdk
        .list_project_directory(&project.project_id, Some(&dependency.dependency_id), ".")
        .unwrap();
    assert_eq!(root["entries"][0]["name"], "lib");
    let nested = sdk
        .list_project_directory(&project.project_id, Some(&dependency.dependency_id), "lib")
        .unwrap();
    assert_eq!(nested["entries"][0]["name"], "code.rs");
    assert!(sdk
        .add_project_dependency(&project.project_id, project_root.to_str().unwrap())
        .is_err());
    assert!(sdk
        .add_project_dependency(
            &project.project_id,
            dependency_root.join("nested").to_str().unwrap()
        )
        .is_err());
    assert!(sdk
        .add_project_dependency(&project.project_id, directory.path().to_str().unwrap())
        .is_err());
    assert!(
        sdk.remove_project_dependency(&project.project_id, &dependency.dependency_id)
            .unwrap()
            .removed
    );
}

#[test]
fn logging_settings_are_global_and_typed() {
    assert!(validate_setting("global", "log_level", &json!("TRACE")).is_ok());
    assert!(validate_setting("global", "log_directory", &json!("")).is_ok());
    assert!(validate_setting("global", "log_max_bytes", &json!(1024)).is_ok());
    assert!(validate_setting("global", "log_retention", &json!(0)).is_ok());
    assert!(validate_setting("global", "verify_https_certificates", &json!(true)).is_ok());
    assert!(validate_setting("global", "image_directory", &json!("")).is_ok());

    assert!(validate_setting("project", "log_level", &json!("INFO")).is_err());
    assert!(validate_setting("global", "log_level", &json!("VERBOSE")).is_err());
    assert!(validate_setting("global", "log_directory", &json!(7)).is_err());
    assert!(validate_setting("global", "log_max_bytes", &json!(1023)).is_err());
    assert!(validate_setting("global", "log_retention", &json!(101)).is_err());
    assert!(validate_setting("project", "image_directory", &json!("/tmp/images")).is_err());
    assert!(validate_setting("global", "image_directory", &json!(9)).is_err());
    assert!(validate_setting("session", "full_control", &json!(true)).is_ok());
    assert!(validate_setting("global", "full_control", &json!(true)).is_err());
    assert!(validate_setting("session", "full_control", &json!("yes")).is_err());
    assert!(validate_setting("project", "tool_call_limit", &json!(1)).is_ok());
    assert!(validate_setting("project", "tool_call_limit", &json!(256)).is_ok());
    assert!(validate_setting("global", "tool_call_limit", &json!(64)).is_err());
    assert!(validate_setting("session", "tool_call_limit", &json!(64)).is_err());
    assert!(validate_setting("project", "tool_call_limit", &json!(0)).is_err());
    assert!(validate_setting("project", "tool_call_limit", &json!(257)).is_err());
    assert!(validate_setting("project", "tool_call_limit", &json!(64.0)).is_err());
    assert!(validate_setting("project", "verify_https_certificates", &json!(true)).is_err());
    assert!(validate_setting("global", "verify_https_certificates", &json!("yes")).is_err());
}

#[test]
fn https_certificate_verification_setting_updates_live_state() {
    let directory = tempfile::tempdir().unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));

    assert!(sdk.state.verify_https_certificates.load(Ordering::SeqCst));
    sdk.set_setting(
        "global",
        None,
        None,
        "verify_https_certificates",
        &json!(false),
    )
    .unwrap();

    assert!(!sdk.state.verify_https_certificates.load(Ordering::SeqCst));
    assert!(!global_bool_setting(&sdk.state.store, "verify_https_certificates", true).unwrap());
}

#[test]
fn session_image_methods_persist_and_remove_files() {
    let directory = tempfile::tempdir().unwrap();
    let project_root = directory.path().join("project");
    std::fs::create_dir_all(&project_root).unwrap();
    let sdk = test_sdk(directory.path());
    let project = sdk
        .open_project(project_root.to_str().unwrap(), None)
        .unwrap();
    let session = sdk
        .create_session(&project.project_id, Some("Images"), Some("deepseek-v4-pro"))
        .unwrap();

    let payload = json!({
        "displayName": "diagram.png",
        "sourceKind": "file",
        "originalPath": project_root.join("diagram.png").to_str().unwrap(),
        "extension": "png",
        "bytesBase64": STANDARD.encode(b"png-bytes"),
        "thumbnailBase64": STANDARD.encode(b"thumb")
    });
    let image = sdk
        .add_session_image(&session.session_id, &payload)
        .unwrap();
    assert_eq!(image.source_kind, "file");
    assert!(std::path::Path::new(&image.storage_path).is_file());
    assert_eq!(
        sdk.list_session_images(&session.session_id)
            .unwrap()
            .images
            .len(),
        1
    );
    assert_eq!(
        sdk.submit_turn_with_attachments(
            &session.session_id,
            "inspect",
            "image-unsupported",
            Some("deepseek-v4-pro"),
            None,
            std::slice::from_ref(&image.image_id),
        )
        .unwrap_err()
        .code,
        "unsupported_capability"
    );
    assert_eq!(
        sdk.submit_turn_with_attachments(
            &session.session_id,
            "inspect",
            "image-count",
            Some("deepseek-v4-pro"),
            None,
            &["1".into(), "2".into(), "3".into(), "4".into()],
        )
        .unwrap_err()
        .code,
        "invalid_arguments"
    );
    assert!(
        sdk.remove_session_image(&session.session_id, &image.image_id)
            .unwrap()
            .removed
    );
    assert_eq!(
        sdk.list_session_images(&session.session_id)
            .unwrap()
            .images
            .len(),
        0
    );
}

#[test]
fn session_snapshot_associates_images_with_messages_and_protects_them() {
    let directory = tempfile::tempdir().unwrap();
    let project_root = directory.path().join("project-with-image");
    std::fs::create_dir_all(&project_root).unwrap();
    let sdk = test_sdk(directory.path());
    let project = sdk
        .open_project(project_root.to_str().unwrap(), None)
        .unwrap();
    let session = sdk
        .create_session(&project.project_id, Some("Attached"), Some("gpt-5.5"))
        .unwrap();
    let image = sdk
        .add_session_image(
            &session.session_id,
            &json!({
                "displayName": "diagram.png",
                "sourceKind": "file",
                "originalPath": project_root.join("diagram.png").to_str().unwrap(),
                "extension": "png",
                "bytesBase64": STANDARD.encode(b"png-bytes"),
                "thumbnailBase64": STANDARD.encode(b"thumb")
            }),
        )
        .unwrap();
    let turn = sdk
        .state
        .store
        .begin_turn_with_images(
            &session.session_id,
            "image-turn",
            "inspect",
            "gpt-5.5",
            std::slice::from_ref(&image.image_id),
        )
        .unwrap();
    sdk.state.store.append_content(&session.session_id, "message.user", &json!({
            "message_id":"message-image",
            "turn_id":turn.turn_id,
            "message":Message {
                role:"user".into(),
                content:vec![
                    suncode_agent::domain::ContentPart { kind:"text".into(), text:"inspect".into() },
                    suncode_agent::domain::ContentPart { kind:"image_ref".into(), text:image.image_id.clone() }
                ],
                tool_calls:vec![],
                tool_call_id:None
            }
        })).unwrap();

    let snapshot = sdk.session_snapshot(&session.session_id, 0).unwrap();
    assert_eq!(snapshot.images.len(), 1);
    assert_eq!(snapshot.messages[0].content[1].kind, "image_ref");
    assert_eq!(snapshot.messages[0].content[1].text, image.image_id);
    assert!(sdk
        .list_session_images(&session.session_id)
        .unwrap()
        .images
        .is_empty());
    assert_eq!(
        sdk.remove_session_image(&session.session_id, &image.image_id)
            .unwrap_err()
            .code,
        "conflict"
    );
}

#[test]
fn session_image_upload_enforces_payload_bounds() {
    let directory = tempfile::tempdir().unwrap();
    let project_root = directory.path().join("project-image-bounds");
    std::fs::create_dir_all(&project_root).unwrap();
    let sdk = test_sdk(directory.path());
    let project = sdk
        .open_project(project_root.to_str().unwrap(), None)
        .unwrap();
    let session = sdk.create_session(&project.project_id, None, None).unwrap();
    let payload = json!({
        "displayName":"large.png",
        "sourceKind":"file",
        "originalPath":project_root.join("large.png").to_str().unwrap(),
        "extension":"png",
        "bytesBase64":STANDARD.encode(vec![0_u8; 20 * 1024 * 1024 + 1]),
        "thumbnailBase64":STANDARD.encode(b"thumb")
    });
    assert_eq!(
        sdk.add_session_image(&session.session_id, &payload)
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
}

#[test]
fn named_git_methods_return_project_scoped_status_and_diff() {
    let directory = tempfile::tempdir().unwrap();
    let project_root = directory.path().join("project");
    git2::Repository::init(&project_root).unwrap();
    std::fs::write(project_root.join("new.txt"), "first\nsecond\n").unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));
    let project = sdk
        .open_project(project_root.to_str().unwrap(), None)
        .unwrap();
    let status = sdk.git_status(&project.project_id).unwrap();
    assert_eq!(status.changed_files, 1);
    assert_eq!(status.files[0].path, "new.txt");
    assert_eq!(status.files[0].status, "untracked");
    let diff = sdk
        .git_diff_file(&project.project_id, "all", "new.txt")
        .unwrap();
    assert_eq!(diff.path, "new.txt");
    assert_eq!(diff.additions, 2);
    assert!(!diff.hunks.is_empty());
    assert_eq!(
        sdk.git_diff_file(&project.project_id, "invalid", "new.txt")
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
}

#[test]
fn named_credential_methods_update_model_availability() {
    let directory = tempfile::tempdir().unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));
    sdk.set_credential("claude", "claude-key").unwrap();
    assert!(sdk
        .list_models()
        .unwrap()
        .models
        .iter()
        .any(|model| model.provider == "claude" && model.availability == "configured"));
    sdk.remove_credential("claude").unwrap();
    assert_eq!(
        sdk.set_credential("unsupported", "unused")
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
}

#[test]
fn provider_endpoint_updates_are_validated_persisted_and_live() {
    let directory = tempfile::tempdir().unwrap();
    let sdk = AgentSdk::from_state_for_test(test_state(directory.path()));
    let update = sdk
        .set_provider_endpoint("openai", " https://gateway.example.test/v1/ ")
        .unwrap();
    assert_eq!(update.provider_id, "openai");
    assert_eq!(update.endpoint, "https://gateway.example.test/v1");
    assert!(sdk.list_models().unwrap().models.iter().any(|model| {
        model.provider == "openai" && model.api_base == "https://gateway.example.test/v1"
    }));
    let stored = sdk
        .state
        .store
        .llm_model_providers(false)
        .unwrap()
        .into_iter()
        .find(|provider| provider.provider_id == "openai")
        .unwrap();
    assert_eq!(stored.endpoint, "https://gateway.example.test/v1");
    assert_eq!(
        sdk.set_provider_endpoint("openai", "file:///tmp/provider")
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
    assert_eq!(
        sdk.set_provider_endpoint("openai", "https://user:secret@example.test/v1")
            .unwrap_err()
            .code,
        "invalid_arguments"
    );
    assert_eq!(
        sdk.set_provider_endpoint("missing", "https://example.test/v1")
            .unwrap_err()
            .code,
        "provider_not_found"
    );
}

#[test]
fn session_snapshot_serializes_normalized_conversation_turns() {
    let directory = tempfile::tempdir().unwrap();
    let state = test_state(directory.path());
    let project = state
        .store
        .project(directory.path().to_str().unwrap(), "Test")
        .unwrap();
    let session = state
        .store
        .create_session(
            &project.project_id,
            Some("First"),
            Some("deepseek-v4-flash"),
        )
        .unwrap();
    state
        .store
        .append_content(
            &session.session_id,
            "turn.state",
            &json!({"turn_id":"turn-1","state":"completed"}),
        )
        .unwrap();
    state
            .store
            .append_content(
                &session.session_id,
                "message.user",
                &json!({"message_id":"user-1","turn_id":"turn-1","message":Message::text("user","inspect")}),
            )
            .unwrap();
    state
            .store
            .append_content(
                &session.session_id,
                "todo.updated",
                &json!({"turn_id":"turn-1","todos":[{"content":"Persisted progress","status":"in_progress","priority":"high"}]}),
            )
            .unwrap();
    let sdk = AgentSdk::from_state_for_test(state);

    let snapshot =
        serde_json::to_value(sdk.session_snapshot(&session.session_id, 0).unwrap()).unwrap();

    assert_eq!(snapshot["messages"][0]["role"], "user");
    assert_eq!(snapshot["conversationTurns"][0]["turnId"], "turn-1");
    assert_eq!(
        snapshot["conversationTurns"][0]["messages"][0]["messageId"],
        "user-1"
    );
    assert_eq!(
        snapshot["conversationTurns"][0]["todos"][0]["content"],
        "Persisted progress"
    );
}

unsafe extern "C" fn collect_event(event_json: *const c_char, user_data: *mut c_void) {
    let sender = &*(user_data as *const std::sync::mpsc::Sender<String>);
    let value = CStr::from_ptr(event_json).to_string_lossy().to_string();
    let _ = sender.send(value);
}

#[test]
fn subscription_delivers_live_events_without_replay() {
    let directory = tempfile::tempdir().unwrap();
    let state = test_state(directory.path());
    let project = state
        .store
        .project(directory.path().to_str().unwrap(), "Test")
        .unwrap();
    let session = state
        .store
        .create_session(
            &project.project_id,
            Some("First"),
            Some("deepseek-v4-flash"),
        )
        .unwrap();
    let sender_for_live = state.events.clone();
    let store_for_live = state.store.clone();
    let sdk = AgentSdk::from_state_for_test(state);
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    let subscription = sdk
        .subscribe_session_events(
            session.session_id.clone(),
            0,
            collect_event,
            &sender as *const _ as *mut c_void,
        )
        .unwrap();
    let live = store_for_live
        .append_content(
            &session.session_id,
            "turn.state",
            &json!({"turn_id": "turn-1", "state": "calling_model"}),
        )
        .unwrap();
    let _ = sender_for_live.send(live.clone());
    let received: SessionEvent = serde_json::from_str(
        &receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(received.event_type, live.event_type);
    subscription.close();
}
