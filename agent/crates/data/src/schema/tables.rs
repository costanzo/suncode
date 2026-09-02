//! Diesel table declarations for the current SQLite schema.

diesel::table! {
    approval_request (approval_id) {
        approval_id -> Text,
        project_id -> Nullable<Text>,
        session_id -> Text,
        turn_id -> Text,
        tool_call_id -> Text,
        operation -> Text,
        arguments_json -> Text,
        idempotency_key -> Text,
        status -> Text,
        decision -> Nullable<Text>,
        decision_source -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    checkpoint (checkpoint_id) {
        checkpoint_id -> Text,
        manifest_id -> Nullable<Text>,
        session_id -> Text,
        turn_id -> Nullable<Text>,
        tool_call_id -> Nullable<Text>,
        relative_path -> Nullable<Text>,
        status -> Text,
        created_at -> Text,
        restored_at -> Nullable<Text>,
        invalidated_at -> Nullable<Text>,
        ordinal -> Nullable<Integer>,
    }
}

diesel::table! {
    checkpoint_manifest (manifest_id) {
        manifest_id -> Text,
        session_id -> Text,
        turn_id -> Nullable<Text>,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        expires_at -> Text,
        restored_at -> Nullable<Text>,
    }
}

diesel::table! {
    configuration (configuration_id) {
        configuration_id -> Integer,
        scope -> Text,
        project_id -> Nullable<Text>,
        session_id -> Nullable<Text>,
        key -> Text,
        value_json -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    llm_model (model_id) {
        model_id -> Text,
        provider_id -> Text,
        display_name -> Text,
        request_model -> Text,
        context_tokens -> Integer,
        auto_compact_tokens -> Integer,
        max_output_tokens -> Nullable<Integer>,
        supports_streaming -> Integer,
        supports_tool_use -> Integer,
        supports_vision -> Integer,
        supports_structured_output -> Integer,
        supports_cancellation -> Integer,
        supports_reasoning_effort -> Integer,
        reasoning_efforts -> Text,
        enabled -> Integer,
        sort_order -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    llm_model_provider (provider_id) {
        provider_id -> Text,
        display_name -> Text,
        endpoint -> Text,
        adapter_type -> Text,
        api_key -> Nullable<Text>,
        enabled -> Integer,
        sort_order -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    project (project_id) {
        project_id -> Text,
        canonical_root -> Text,
        display_name -> Text,
        created_at -> Text,
        updated_at -> Text,
        last_opened_at -> Text,
        archived_at -> Nullable<Text>,
    }
}

diesel::table! {
    project_dependency (dependency_id) {
        dependency_id -> Text,
        project_id -> Text,
        canonical_root -> Text,
        display_name -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    session (session_id) {
        session_id -> Text,
        project_id -> Text,
        title -> Nullable<Text>,
        model_id -> Nullable<Text>,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        last_activity_at -> Text,
        pin_at -> Nullable<Text>,
        archived_at -> Nullable<Text>,
    }
}

diesel::table! {
    session_call (call_id) {
        call_id -> Text,
        session_id -> Text,
        turn_id -> Text,
        provider -> Text,
        model_id -> Text,
        wire_model -> Text,
        provider_request_id -> Nullable<Text>,
        provider_response_id -> Nullable<Text>,
        state -> Text,
        iteration -> Integer,
        started_at -> Text,
        completed_at -> Nullable<Text>,
        input_messages_json -> Text,
        output_message_json -> Nullable<Text>,
        tool_calls_json -> Text,
        usage_json -> Nullable<Text>,
        finish_reason -> Nullable<Text>,
        error_json -> Nullable<Text>,
    }
}

diesel::table! {
    session_image (image_id) {
        image_id -> Text,
        session_id -> Text,
        display_name -> Text,
        source_kind -> Text,
        original_path -> Nullable<Text>,
        storage_path -> Text,
        thumbnail_base64 -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    session_message (message_id) {
        message_id -> Text,
        session_id -> Text,
        turn_id -> Nullable<Text>,
        session_call_id -> Nullable<Text>,
        role -> Text,
        message_json -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    session_tool_use (turn_id, tool_call_id) {
        turn_id -> Text,
        tool_call_id -> Text,
        session_call_id -> Nullable<Text>,
        name -> Text,
        request_json -> Nullable<Text>,
        result_json -> Nullable<Text>,
        state -> Text,
        ordinal -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        completed_at -> Nullable<Text>,
        error_code -> Nullable<Text>,
    }
}

diesel::table! {
    session_turn (turn_id) {
        turn_id -> Text,
        session_id -> Text,
        submission_idempotency_key -> Nullable<Text>,
        state -> Text,
        model_id -> Nullable<Text>,
        input_json -> Nullable<Text>,
        response_json -> Nullable<Text>,
        error_json -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        admitted_at -> Nullable<Text>,
        started_at -> Nullable<Text>,
        completed_at -> Nullable<Text>,
        error_code -> Nullable<Text>,
        input_tokens -> Integer,
        output_tokens -> Integer,
        total_tokens -> Integer,
        recovery_approval_id -> Nullable<Text>,
        recovery_snapshot_json -> Nullable<Text>,
        recovery_status -> Nullable<Text>,
        recovery_created_at -> Nullable<Text>,
        recovery_updated_at -> Nullable<Text>,
    }
}

diesel::table! {
    session_turn_todo (turn_id, ordinal) {
        turn_id -> Text,
        ordinal -> Integer,
        content -> Text,
        status -> Text,
        priority -> Text,
        created_at -> Text,
        updated_at -> Text,
        completed_at -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    approval_request,
    checkpoint,
    checkpoint_manifest,
    configuration,
    llm_model,
    llm_model_provider,
    project,
    project_dependency,
    session,
    session_call,
    session_image,
    session_message,
    session_tool_use,
    session_turn,
    session_turn_todo,
);
