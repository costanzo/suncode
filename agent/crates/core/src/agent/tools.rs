impl Agent {
    async fn resolve_calls(
        &self,
        context: &mut Continuation,
        calls: Vec<ToolCall>,
        token: CancellationToken,
    ) -> Result<(), BusinessError> {
        let batch_size = u32::try_from(calls.len()).unwrap_or(u32::MAX);
        let next_tool_calls = context.tool_calls.saturating_add(batch_size);
        if next_tool_calls > context.tool_call_limit {
            context.tool_calls = next_tool_calls;
            for (index, call) in calls.iter().enumerate() {
                self.emit(
                    &context.session_id,
                    "tool.requested",
                    json!({
                        "turn_id": context.turn_id,
                        "call_id": context.active_call_id,
                        "tool_call_id": call.call_id,
                        "name": call.name,
                        "arguments": call.arguments,
                        "ordinal": index
                    }),
                )?;
                self.tool_state(context, call, "failed", Some("tool_budget_exceeded"))?;
            }
            self.turn_state(context, "failed", Some("tool_budget_exceeded"))?;
            return Err(BusinessError::new(
                "tool_budget_exceeded",
                "Turn exceeded its tool-call budget",
            )
            .details(json!({
                "limit": context.tool_call_limit,
                "requested_total": context.tool_calls,
                "rejected_batch_size": batch_size
            })));
        }
        context.tool_calls = next_tool_calls;
        let mut allowed_calls = Vec::new();
        for (index, call) in calls.iter().enumerate() {
            self.emit(&context.session_id, "tool.requested", json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"name":call.name,"arguments":call.arguments,"ordinal":index}))?;
            let signature = tool_signature(call);
            if context.last_tool_signature.as_deref() == Some(signature.as_str()) {
                context.repeated_tool_stalls += 1;
            } else {
                context.last_tool_signature = Some(signature);
                context.repeated_tool_stalls = 1;
            }
            if context.repeated_tool_stalls >= 4 {
                self.tool_state(context, call, "failed", Some("repeated_equivalent_call"))?;
                return self.fail_context(
                    context,
                    "tool_stall_detected",
                    "Repeated equivalent tool calls indicate a stalled turn",
                );
            }
            self.tool_state(context, call, "requested", None)?;
            self.tool_state(context, call, "validating", None)?;
            if !call.arguments.is_object() {
                let error =
                    BusinessError::new("malformed_tool_call", "Tool arguments must be an object");
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if let Err(error) = self.validate_dependency_call(context, call) {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if let Err(error) = validate_before_policy(&call.name, &call.arguments) {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if call.name == "question" {
                self.execute_allowed_calls(
                    context,
                    std::mem::take(&mut allowed_calls),
                    token.clone(),
                )
                .await?;
                self.tool_state(
                    context,
                    call,
                    "awaiting_question",
                    Some("user_input_required"),
                )?;
                let request_id = format!("que_{}", Uuid::new_v4());
                context.pending_call = Some(call.clone());
                context.remaining_calls = calls[index + 1..].to_vec();
                let snapshot = serde_json::to_value(&*context).map_err(|_| {
                    BusinessError::new("agent_unavailable", "turn continuation could not be stored")
                })?;
                self.store
                    .create_question(&request_id, &context.turn_id, &snapshot)?;
                self.emit(&context.session_id, "question.asked", json!({"request_id":request_id,"turn_id":context.turn_id,"tool_call_id":call.call_id,"questions":call.arguments["questions"]}))?;
                return Err(BusinessError::new("question_required", "The user must answer the question tool").details(json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"request_id":request_id})));
            }
            if call.name == "todowrite" {
                self.execute_allowed_calls(
                    context,
                    std::mem::take(&mut allowed_calls),
                    token.clone(),
                )
                .await?;
                self.execute_todowrite(context, call)?;
                continue;
            }
            self.tool_state(context, call, "policy_check", None)?;
            let decision = evaluate(
                tool_risk(&call.name),
                self.non_interactive,
                self.store.session_full_control(&context.session_id)?,
            );
            match decision {
                Decision::Deny => {
                    self.execute_allowed_calls(
                        context,
                        std::mem::take(&mut allowed_calls),
                        token.clone(),
                    )
                    .await?;
                    self.tool_state(context, call, "denied", Some("authorization_denied"))?;
                    return Err(BusinessError::new(
                        "authorization_denied",
                        format!("Tool call denied: {}", call.name),
                    ));
                }
                Decision::ApprovalRequired => {
                    self.execute_allowed_calls(
                        context,
                        std::mem::take(&mut allowed_calls),
                        token.clone(),
                    )
                    .await?;
                    self.tool_state(
                        context,
                        call,
                        "awaiting_approval",
                        Some("risk_requires_approval"),
                    )?;
                    context.pending_call = Some(call.clone());
                    context.remaining_calls = calls[index + 1..].to_vec();
                    let snapshot = serde_json::to_value(&*context).map_err(|_| {
                        BusinessError::new(
                            "agent_unavailable",
                            "turn continuation could not be stored",
                        )
                    })?;
                    let approval = self.store.create_approval(ApprovalInput {
                        project_id: Some(&context.project_id),
                        session_id: &context.session_id,
                        turn_id: &context.turn_id,
                        tool_call_id: &call.call_id,
                        operation: &call.name,
                        arguments: &call.arguments,
                        snapshot: &snapshot,
                    })?;
                    self.emit(&context.session_id,"approval.requested",json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"approval_id":approval.approval_id,"operation":call.name,"arguments":call.arguments}))?;
                    return Err(BusinessError::new("approval_required",format!("Tool call requires approval: {}",call.name)).details(json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"approval_id":approval.approval_id})));
                }
                Decision::Allow => allowed_calls.push(call.clone()),
            }
        }
        self.execute_allowed_calls(context, allowed_calls, token)
            .await?;
        Ok(())
    }

    async fn execute_allowed_calls(
        &self,
        context: &mut Continuation,
        calls: Vec<ToolCall>,
        token: CancellationToken,
    ) -> Result<(), BusinessError> {
        if calls.is_empty() {
            return Ok(());
        }
        let parallel_read_only = calls.len() > 1
            && calls
                .iter()
                .all(|call| tool_risk(&call.name) == Some(Risk::ReadOnly));
        if !parallel_read_only {
            for call in calls {
                self.execute_call(context, &call, token.clone()).await?;
            }
            return Ok(());
        }

        let mut futures = Vec::with_capacity(calls.len());
        for call in calls {
            self.tool_state(context, &call, "authorized", None)?;
            self.tool_state(context, &call, "executing", None)?;
            let (project_root, mut params) = match self.prepare_call(context, &call) {
                Ok(prepared) => prepared,
                Err(error) => {
                    if !self.record_recoverable_call_error(context, &call, &error)? {
                        return Err(error);
                    }
                    continue;
                }
            };
            params["idempotency_key"] = json!(format!("{}:{}", context.turn_id, call.call_id));
            let method = match method_name(&call.name) {
                Some(method) => method.to_string(),
                None => {
                    let error = BusinessError::new("authorization_denied", "Unknown tool");
                    self.tool_state(context, &call, "failed", Some(&error.code))?;
                    return Err(error);
                }
            };
                let agent = self.clone();
                let token = token.clone();
                let output_callback = {
                    let agent = agent.clone();
                    let session_id = context.session_id.clone();
                    let turn_id = context.turn_id.clone();
                    let call_id = context.active_call_id.clone();
                    let tool_call_id = call.call_id.clone();
                    Some(std::sync::Arc::new(move |stream: &str, chunk: &[u8]| {
                        agent.emit_live(&session_id, "tool.output", json!({
                            "turn_id": turn_id,
                            "call_id": call_id,
                            "tool_call_id": tool_call_id,
                            "stream": stream,
                            "chunk_base64": STANDARD.encode(chunk),
                        }));
                    }) as suncode_tool::ProcessOutputCallback)
                };
                futures.push(async move {
                    let result = agent
                    .operation_in_project(&project_root, &method, params, token, output_callback)
                    .await;
                (call, result)
            });
        }

        for (call, result) in join_all(futures).await {
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    if !self.record_recoverable_call_error(context, &call, &error)? {
                        return Err(error);
                    }
                    continue;
                }
            };
            self.record_call_success(context, &call, result)?;
        }
        Ok(())
    }

    fn execute_todowrite(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
    ) -> Result<(), BusinessError> {
        let todos = parse_todos(&call.arguments)?;
        self.tool_state(context, call, "policy_check", None)?;
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        context.todos = todos.clone();
        self.emit(
            &context.session_id,
            "todo.updated",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"todos":todos}),
        )?;
        self.tool_state(context, call, "succeeded", None)?;
        let result = json!({"todos": context.todos});
        self.emit(
            &context.session_id,
            "tool.result",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"result":result}),
        )?;
        let mut tool = Message::text(
            "tool",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"message":tool}),
        )?;
        Ok(())
    }

    async fn execute_call(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        token: CancellationToken,
    ) -> Result<(), BusinessError> {
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        let (project_root, mut params) = match self.prepare_call(context, call) {
            Ok(prepared) => prepared,
            Err(error) => {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                return Ok(());
            }
        };
        params["idempotency_key"] = json!(format!("{}:{}", context.turn_id, call.call_id));
        let method = match method_name(&call.name) {
            Some(method) => method,
            None => {
                let error = BusinessError::new("authorization_denied", "Unknown tool");
                self.tool_state(context, call, "failed", Some(&error.code))?;
                return Err(error);
            }
        };
        let result = match self
            .operation_in_project(&project_root, method, params, token, {
                let agent = self.clone();
                let session_id = context.session_id.clone();
                let turn_id = context.turn_id.clone();
                let call_id = context.active_call_id.clone();
                let tool_call_id = call.call_id.clone();
                Some(std::sync::Arc::new(move |stream: &str, chunk: &[u8]| {
                    agent.emit_live(&session_id, "tool.output", json!({
                        "turn_id": turn_id,
                        "call_id": call_id,
                        "tool_call_id": tool_call_id,
                        "stream": stream,
                        "chunk_base64": STANDARD.encode(chunk),
                    }));
                }) as suncode_tool::ProcessOutputCallback)
            })
            .await
        {
            Ok(result) => result,
            Err(error) => {
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                return Ok(());
            }
        };
        self.record_call_success(context, call, result)
    }

    fn record_recoverable_call_error(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        error: &BusinessError,
    ) -> Result<bool, BusinessError> {
        self.tool_state(context, call, "failed", Some(&error.code))?;
        if !matches!(
            error.code.as_str(),
            "invalid_arguments" | "malformed_tool_call"
        ) {
            return Ok(false);
        }
        let result = json!({
            "error": {
                "code": error.code,
                "message": error.message,
                "details": error.details,
            }
        });
        self.emit(
            &context.session_id,
            "tool.result",
            json!({
                "turn_id": context.turn_id,
                "call_id": context.active_call_id,
                "tool_call_id": call.call_id,
                "result": result,
            }),
        )?;
        let mut tool = Message::text(
            "tool",
            serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\":{}}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({
                "turn_id": context.turn_id,
                "call_id": context.active_call_id,
                "tool_call_id": call.call_id,
                "message": tool,
            }),
        )?;
        Ok(true)
    }

    fn record_call_success(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        result: Value,
    ) -> Result<(), BusinessError> {
        let dependency_id = call
            .arguments
            .get("path")
            .and_then(Value::as_str)
            .and_then(dependency_path)
            .map(|(dependency_id, _)| dependency_id);
        let mut normalized_result = normalize_result(&call.name, result.clone(), dependency_id);
        attach_nearby_instructions(context, call, &mut normalized_result);
        let process_failed = call.name == "bash"
            && normalized_result.get("success").and_then(Value::as_bool) == Some(false);
        self.tool_state(
            context,
            call,
            if process_failed {
                "failed"
            } else {
                "succeeded"
            },
            if process_failed {
                normalized_result.get("status").and_then(Value::as_str)
            } else {
                None
            },
        )?;
        self.emit(
            &context.session_id,
            "tool.result",
            json!({
                "turn_id": context.turn_id,
                "call_id": context.active_call_id,
                "tool_call_id": call.call_id,
                "result": normalized_result,
            }),
        )?;
        let checkpoint_ids = result
            .get("checkpoint_ids")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                result
                    .get("checkpoint_id")
                    .and_then(Value::as_str)
                    .map(|value| vec![value])
                    .unwrap_or_default()
            });
        if !checkpoint_ids.is_empty() {
            let manifest = self
                .store
                .ensure_manifest(&context.session_id, &context.turn_id)?;
            let existing = self.store.checkpoint_items(&manifest.manifest_id)?.len() as i64;
            for (index, id) in checkpoint_ids.iter().enumerate() {
                let path = if index == 0 {
                    result.get("path").or_else(|| result.get("from"))
                } else {
                    result.get("to")
                }
                .and_then(Value::as_str);
                self.emit(&context.session_id,"checkpoint.captured",json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"manifest_id":manifest.manifest_id,"checkpoint_id":id,"path":path,"ordinal":existing+index as i64}))?;
            }
        }
        let mut tool = Message::text(
            "tool",
            serde_json::to_string(&normalized_result).unwrap_or_else(|_| "{}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            "message.tool",
            json!({"turn_id":context.turn_id,"call_id":context.active_call_id,"tool_call_id":call.call_id,"message":tool}),
        )?;
        Ok(())
    }

    async fn operation_in_project(
        &self,
        project_root: &str,
        method: &str,
        params: Value,
        token: CancellationToken,
        output_callback: Option<suncode_tool::ProcessOutputCallback>,
    ) -> Result<Value, BusinessError> {
        let operations = self.operations.clone();
        let root = std::path::PathBuf::from(project_root);
        let method = method.to_string();
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled_for_operation = cancelled.clone();
        let mut operation = Box::pin(tokio::task::spawn_blocking(move || {
            operations.execute_in_project_with_cancellation_and_output(
                &root,
                &method,
                params,
                Some(&cancelled_for_operation),
                output_callback,
            )
        }));
        let join_result = tokio::select! {
            result = &mut operation => result,
            _ = token.cancelled() => {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                (&mut operation).await
            }
        };
        join_result
            .map_err(|_| BusinessError::new("agent_unavailable", "operation task failed"))?
            .map_err(|error| {
                let mut agent_error = BusinessError::new(
                    error
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or("operation_failed"),
                    error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("operation failed"),
                );
                if let Some(details) = error.get("details") {
                    agent_error = agent_error.details(details.clone());
                }
                agent_error
            })
    }

    fn prepare_call(
        &self,
        context: &Continuation,
        call: &ToolCall,
    ) -> Result<(String, Value), BusinessError> {
        let mut arguments = call.arguments.clone();
        let Some(path) = arguments.get("path").and_then(Value::as_str) else {
            return Ok((
                context.project_root.clone(),
                translate_arguments(&call.name, &arguments)?,
            ));
        };
        if path.starts_with("dependency:") && dependency_path(path).is_none() {
            return Err(BusinessError::new(
                "invalid_arguments",
                "dependency path must include a dependency ID",
            ));
        }
        let Some((dependency_id, relative_path)) = dependency_path(path) else {
            return Ok((
                context.project_root.clone(),
                translate_arguments_with_root(
                    &call.name,
                    &arguments,
                    Some(Path::new(&context.project_root)),
                )?,
            ));
        };
        if !dependency_tool_allowed(&call.name) {
            return Err(BusinessError::new(
                "scope_denied",
                "dependencies are read-only and support only read, glob, and grep",
            ));
        }
        let dependency = self
            .store
            .project_dependency_by_id(&context.project_id, dependency_id)?
            .ok_or_else(|| BusinessError::new("dependency_not_found", "dependency not found"))?;
        arguments["path"] = json!(relative_path);
        let dependency_root = dependency.canonical_root.clone();
        Ok((
            dependency_root.clone(),
            translate_arguments_with_root(
                &call.name,
                &arguments,
                Some(Path::new(&dependency_root)),
            )?,
        ))
    }

    fn validate_dependency_call(
        &self,
        context: &Continuation,
        call: &ToolCall,
    ) -> Result<(), BusinessError> {
        let Some(path) = call.arguments.get("path").and_then(Value::as_str) else {
            return Ok(());
        };
        if path.starts_with("dependency:") && dependency_path(path).is_none() {
            return Err(BusinessError::new(
                "invalid_arguments",
                "dependency path must include a dependency ID",
            ));
        }
        let Some((dependency_id, _)) = dependency_path(path) else {
            return Ok(());
        };
        if !dependency_tool_allowed(&call.name) {
            return Err(BusinessError::new(
                "scope_denied",
                "dependencies are read-only and support only read, glob, and grep",
            ));
        }
        if self
            .store
            .project_dependency_by_id(&context.project_id, dependency_id)?
            .is_none()
        {
            return Err(BusinessError::new(
                "dependency_not_found",
                "dependency not found",
            ));
        }
        Ok(())
    }

    fn dependency_context_message(
        &self,
        project_id: &str,
    ) -> Result<Option<suncode_llm::Message>, BusinessError> {
        let dependencies = self.store.project_dependencies(project_id)?;
        if dependencies.is_empty() {
            return Ok(None);
        }
        let roots = dependencies
            .iter()
            .map(|dependency| {
                format!(
                    "- {}: dependency:{}",
                    dependency.display_name, dependency.dependency_id
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(Some(suncode_llm::Message {
            role: "system".into(),
            content: vec![suncode_llm::ContentPart {
                kind: "text".into(),
                text: format!(
                    "Registered read-only source dependencies:\n{roots}\nUse dependency:<dependencyId>/<relativePath> with read, or dependency:<dependencyId> as the glob/grep path. Dependencies cannot be modified or used as a process working directory."
                ),
            }],
            tool_calls: Vec::new(),
            tool_call_id: None,
        }))
    }
}
