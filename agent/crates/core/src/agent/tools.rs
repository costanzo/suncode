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
                    EventPayload::ToolRequested(ToolRequestedPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), name: call.name.clone(), arguments: call.arguments.clone(), ordinal: index }),
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
            self.emit(&context.session_id, EventPayload::ToolRequested(ToolRequestedPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), name: call.name.clone(), arguments: call.arguments.clone(), ordinal: index }))?;
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
            if computer::is_computer_call(call) && context.computer_batch_failed {
                self.record_computer_skipped(context, call)?;
                continue;
            }
            if !context.allowed_tools.is_empty()
                && !context.allowed_tools.iter().any(|allowed| allowed == &call.name)
            {
                let error = BusinessError::new(
                    "agent_tool_denied",
                    format!("Tool `{}` is not allowed for this agent", call.name),
                );
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if !call.arguments.is_object() {
                let error =
                    BusinessError::new("malformed_tool_call", "Tool arguments must be an object");
                if !self.record_recoverable_call_error(context, call, &error)? {
                    return Err(error);
                }
                continue;
            }
            if call.name == "delegate_agent" {
                self.execute_allowed_calls(
                    context,
                    std::mem::take(&mut allowed_calls),
                    token.clone(),
                )
                .await?;
                self.execute_delegate_agent(context, call, token.clone()).await?;
                continue;
            }
            if !mcp::is_mcp_tool(&call.name) && !computer::is_computer_call(call) {
                if let Err(error) = self.validate_dependency_call(context, call) {
                    if !self.record_recoverable_call_error(context, call, &error)? {
                        return Err(error);
                    }
                    continue;
                }
            }
            let mut mcp_generation = None;
            let (mut approval_operation, mut approval_arguments) = if mcp::is_mcp_tool(&call.name) {
                match self
                    .mcp
                    .approval_target(&context.project_id, &call.name, &call.arguments)
                    .await
                {
                    Ok(Some(target)) => {
                        mcp_generation = Some(target.generation);
                        (
                            target.label,
                        json!({
                            "serverId": target.server_id,
                            "toolName": target.remote_name,
                            "arguments": call.arguments,
                        }),
                        )
                    }
                    Ok(None) => {
                        let error = BusinessError::new(
                            "mcp_tool_unavailable",
                            "MCP tool is no longer available for this project",
                        );
                        if !self.record_recoverable_call_error(context, call, &error)? {
                            return Err(error);
                        }
                        continue;
                    }
                    Err(error) => {
                        if !self.record_recoverable_call_error(context, call, &error)? {
                            return Err(error);
                        }
                        continue;
                    }
                }
            } else if computer::is_computer_call(call) {
                (
                    format!("computer.{}", call.name),
                    json!({"toolsetName":"computer","member":call.name,"input":call.arguments}),
                )
            } else {
                (call.name.clone(), call.arguments.clone())
            };
            let validation = if computer::is_computer_call(call) {
                suncode_computer::ComputerAction::from_member(&call.name, &call.arguments)
                    .map(|_| ())
                    .map_err(|error| BusinessError::invalid(error.to_string()))
            } else {
                validate_before_policy(&call.name, &call.arguments)
            };
            if let Err(error) = validation {
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
                let snapshot = continuation_snapshot(context).map_err(|_| {
                    BusinessError::new("agent_unavailable", "turn continuation could not be stored")
                })?;
                self.store
                    .create_question(&request_id, &context.turn_id, &snapshot)?;
                self.emit(&context.session_id, EventPayload::QuestionAsked(QuestionAskedPayload { request_id: request_id.clone(), turn_id: context.turn_id.clone(), tool_call_id: call.call_id.clone(), questions: call.arguments["questions"].clone() }))?;
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
            let risk = if computer::is_computer_call(call) {
                computer::risk(&call.name)
            } else {
                tool_risk(&call.name)
            };
            let computer_batch_approved = computer::is_computer_call(call)
                && context
                    .approved_computer_call_ids
                    .iter()
                    .any(|call_id| call_id == &call.call_id);
            let decision = if computer_batch_approved {
                Decision::Allow
            } else {
                evaluate(
                    risk,
                    self.non_interactive,
                    self.store.session_full_control(&context.session_id)?,
                )
            };
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
                    if computer::is_computer_call(call) {
                        let batch = calls[index..]
                            .iter()
                            .take_while(|candidate| computer::is_computer_call(candidate))
                            .cloned()
                            .collect::<Vec<_>>();
                        context.approved_computer_call_ids =
                            batch.iter().map(|candidate| candidate.call_id.clone()).collect();
                        approval_operation = "computer.batch".into();
                        approval_arguments = json!({
                            "toolsetName":"computer",
                            "actions":batch.iter().map(|candidate| json!({
                                "toolCallId":candidate.call_id,
                                "member":candidate.name,
                                "input":candidate.arguments,
                            })).collect::<Vec<_>>(),
                        });
                    }
                    context.pending_call = Some(call.clone());
                    context.pending_mcp_generation = mcp_generation;
                    context.remaining_calls = calls[index + 1..].to_vec();
                    let snapshot = continuation_snapshot(context).map_err(|_| {
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
                        operation: &approval_operation,
                        arguments: &approval_arguments,
                        snapshot: &snapshot,
                    })?;
                    self.emit(&context.session_id, EventPayload::ApprovalRequested(ApprovalRequestedPayload { turn_id: context.turn_id.clone(), tool_call_id: call.call_id.clone(), approval_id: approval.approval_id.clone(), operation: approval_operation, arguments: approval_arguments }))?;
                    return Err(BusinessError::new("approval_required",format!("Tool call requires approval: {}",call.name)).details(json!({"turn_id":context.turn_id,"tool_call_id":call.call_id,"approval_id":approval.approval_id})));
                }
                Decision::Allow => {
                    if computer::is_computer_call(call) {
                        context
                            .approved_computer_call_ids
                            .retain(|call_id| call_id != &call.call_id);
                        self.execute_allowed_calls(
                            context,
                            std::mem::take(&mut allowed_calls),
                            token.clone(),
                        )
                        .await?;
                        self.execute_call(context, call, token.clone(), None).await?;
                    } else {
                        allowed_calls.push(call.clone());
                    }
                }
            }
        }
        self.execute_allowed_calls(context, allowed_calls, token)
            .await?;
        context.computer_batch_failed = false;
        Ok(())
    }

    async fn execute_delegate_agent(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        token: CancellationToken,
    ) -> Result<(), BusinessError> {
        if context.agent_id.is_some() {
            let error = BusinessError::new("agent_tool_denied", "child agents cannot delegate");
            return self.record_call_success(
                context,
                call,
                json!({"status":"failed","error":{"code":error.code,"message":error.message}}),
            );
        }
        let agent_name = call
            .arguments
            .get("agent")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid("agent is required"))?;
        let task = call
            .arguments
            .get("task")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|task| !task.is_empty() && task.chars().count() <= 16_000)
            .ok_or_else(|| BusinessError::invalid("task must contain 1 through 16000 characters"))?;
        let definition = builtin_agents::by_name(agent_name)
            .ok_or_else(|| BusinessError::new("unknown_agent", "built-in agent was not found"))?;
        self.tool_state(context, call, "policy_check", None)?;
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        if token.is_cancelled() {
            return Err(BusinessError::new("cancelled", "Turn was cancelled"));
        }
        let title = task.chars().take(80).collect::<String>();
        let child = self.store.create_child_session(
            &context.project_id,
            &context.session_id,
            definition.id,
            definition.version,
            &title,
            &context.model,
        )?;
        let invocation_id = Uuid::new_v4().to_string();
        self.store.create_subagent_invocation(
            &invocation_id,
            &context.session_id,
            &context.turn_id,
            &call.call_id,
            &child.session_id,
            definition.id,
            definition.version,
            &json!({"text":task}),
            &json!(definition.allowed_tools),
            &context.model,
        )?;
        self.store.update_subagent_invocation(&child.session_id, "running", None, None)?;
        let outcome = Box::pin(self.submit_child(
            &child.session_id,
            task,
            &context.model,
            context.reasoning_effort.as_deref(),
            token,
        ))
        .await;
        let result = match outcome {
            Ok(TurnResponse::Completed { message, usage, iterations, tool_calls, .. }) => {
                let result = json!({
                    "status":"completed",
                    "agentId":definition.id,
                    "agentName":definition.name,
                    "agentDisplayName":definition.display_name,
                    "childSessionId":child.session_id,
                    "result":message.text_content(),
                    "usage":usage,
                    "iterations":iterations,
                    "toolCalls":tool_calls
                });
                self.store.update_subagent_invocation(&child.session_id, "completed", Some(&result), None)?;
                result
            }
            Ok(TurnResponse::AwaitingApproval { approval_id, .. }) => {
                let result = json!({"status":"awaiting_approval","agentId":definition.id,"childSessionId":child.session_id,"approvalId":approval_id});
                self.store.update_subagent_invocation(&child.session_id, "awaiting_approval", Some(&result), None)?;
                result
            }
            Ok(TurnResponse::AwaitingQuestion { .. }) => {
                let result = json!({"status":"failed","agentId":definition.id,"childSessionId":child.session_id,"error":{"code":"agent_question_denied","message":"child agents cannot ask the user questions"}});
                self.store.update_subagent_invocation(&child.session_id, "failed", Some(&result), Some("agent_question_denied"))?;
                result
            }
            Ok(TurnResponse::Queued { .. }) => {
                let result = json!({"status":"failed","agentId":definition.id,"childSessionId":child.session_id,"error":{"code":"agent_busy","message":"child agent unexpectedly queued its initial task"}});
                self.store.update_subagent_invocation(&child.session_id, "failed", Some(&result), Some("agent_busy"))?;
                result
            }
            Err(error) if error.code == "approval_required" => {
                let approval_id = error
                    .details
                    .get("approval_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| BusinessError::unavailable("child approval outcome is missing approval_id"))?;
                let result = json!({"status":"awaiting_approval","agentId":definition.id,"childSessionId":child.session_id,"approvalId":approval_id});
                self.store.update_subagent_invocation(&child.session_id, "awaiting_approval", Some(&result), None)?;
                result
            }
            Err(error) => {
                let state = if error.code == "cancelled" { "cancelled" } else { "failed" };
                let error_code = error.code.clone();
                let result = json!({"status":state,"agentId":definition.id,"childSessionId":child.session_id,"error":{"code":error.code,"message":error.message,"details":error.details}});
                self.store.update_subagent_invocation(&child.session_id, state, Some(&result), Some(&error_code))?;
                result
            }
        };
        self.record_call_success(context, call, result)
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
            && !calls.iter().any(|call| lsp::is_lsp_tool(&call.name))
            && calls
                .iter()
                .all(|call| tool_risk(&call.name) == Some(Risk::ReadOnly));
        if !parallel_read_only {
            for call in calls {
                self.execute_call(context, &call, token.clone(), None)
                    .await?;
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
                        agent.emit_live(&session_id, EventPayload::ToolOutput(ToolOutputPayload { turn_id: turn_id.clone(), call_id: call_id.clone(), tool_call_id: tool_call_id.clone(), stream: stream.to_owned(), chunk_base64: STANDARD.encode(chunk) }));
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
            EventPayload::TodoUpdated(TodoUpdatedPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), todos: todos.into_iter().map(|todo| TodoEventItem { content: todo.content, status: todo.status, priority: todo.priority }).collect() }),
        )?;
        self.tool_state(context, call, "succeeded", None)?;
        let result = json!({"todos": context.todos});
        self.emit(
            &context.session_id,
            EventPayload::ToolResult(ToolResultPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), result: result.clone() }),
        )?;
        let mut tool = Message::text(
            "tool",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            EventPayload::MessageTool(MessageToolPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), message: tool }),
        )?;
        Ok(())
    }

    async fn execute_call(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        token: CancellationToken,
        expected_mcp_generation: Option<u64>,
    ) -> Result<(), BusinessError> {
        self.tool_state(context, call, "authorized", None)?;
        self.tool_state(context, call, "executing", None)?;
        if mcp::is_mcp_tool(&call.name) {
            let result = match self
                .mcp
                .call(
                    &context.project_id,
                    &call.name,
                    call.arguments.clone(),
                    token,
                    expected_mcp_generation,
                )
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
            return self.record_call_success(context, call, result);
        }
        if lsp::is_lsp_tool(&call.name) {
            let result = match self
                .execute_lsp_call(context, call, token)
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
            return self.record_call_success(context, call, result);
        }
        if browser::is_browser_tool(&call.name) {
            let result = match self
                .browser
                .call(
                    &context.project_id,
                    &call.name,
                    call.arguments.clone(),
                    token,
                )
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
            return self.record_call_success(context, call, result);
        }
        if computer::is_computer_call(call) {
            let result = match self
                .computer
                .execute(&call.name, &call.arguments, token)
                .await
            {
                Ok(result) => result,
                Err(error) => {
                    if !self.record_recoverable_call_error(context, call, &error)? {
                        return Err(error);
                    }
                    context.computer_batch_failed = true;
                    return Ok(());
                }
            };
            return self.record_computer_success(context, call, result);
        }
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
                    agent.emit_live(&session_id, EventPayload::ToolOutput(ToolOutputPayload { turn_id: turn_id.clone(), call_id: call_id.clone(), tool_call_id: tool_call_id.clone(), stream: stream.to_owned(), chunk_base64: STANDARD.encode(chunk) }));
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
            if !error.code.starts_with("mcp_")
                && !error.code.starts_with("lsp_")
                && !error.code.starts_with("browser_")
                && !error.code.starts_with("computer_")
            {
                return Ok(false);
            }
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
            EventPayload::ToolResult(ToolResultPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), result: result.clone() }),
        )?;
        let mut tool = Message::text(
            "tool",
            serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\":{}}".into()),
        );
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            EventPayload::MessageTool(MessageToolPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), message: tool }),
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
        let process_failed = (call.name == "bash"
            && normalized_result.get("success").and_then(Value::as_bool) == Some(false))
            || (mcp::is_mcp_tool(&call.name)
                && normalized_result.get("isError").and_then(Value::as_bool) == Some(true));
        self.tool_state(
            context,
            call,
            if process_failed {
                "failed"
            } else {
                "succeeded"
            },
            if process_failed {
                if mcp::is_mcp_tool(&call.name) {
                    Some("mcp_server_error")
                } else {
                    normalized_result.get("status").and_then(Value::as_str)
                }
            } else {
                None
            },
        )?;
        self.emit(
            &context.session_id,
            EventPayload::ToolResult(ToolResultPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), result: normalized_result.clone() }),
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
                self.emit(&context.session_id, EventPayload::CheckpointCaptured(CheckpointCapturedPayload { turn_id: context.turn_id.clone(), tool_call_id: call.call_id.clone(), manifest_id: manifest.manifest_id.clone(), checkpoint_id: (*id).to_owned(), path: path.map(str::to_owned), ordinal: existing + index as i64 }))?;
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
            EventPayload::MessageTool(MessageToolPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), message: tool }),
        )?;
        Ok(())
    }

    fn record_computer_success(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
        outcome: suncode_computer::ActionOutcome,
    ) -> Result<(), BusinessError> {
        let (metadata, message) = match outcome {
            suncode_computer::ActionOutcome::Text(text) => {
                let metadata = json!({"type":"computer_text","text":text});
                (metadata, Message::text("tool", text))
            }
            suncode_computer::ActionOutcome::Image(frame) => {
                let png = frame
                    .png()
                    .map_err(|error| BusinessError::new("computer_capture_failed", error.to_string()))?;
                let metadata = json!({
                    "type":"computer_image",
                    "pixelWidth":frame.display.pixel_width,
                    "pixelHeight":frame.display.pixel_height,
                    "displayGeneration":frame.display.generation,
                });
                let mut message = Message::text("tool", "Computer screenshot captured.");
                message.content.push(crate::domain::ContentPart {
                    kind: "image_url".into(),
                    text: format!("data:image/png;base64,{}", STANDARD.encode(png)),
                });
                (metadata, message)
            }
        };
        self.tool_state(context, call, "succeeded", None)?;
        self.emit(
            &context.session_id,
            EventPayload::ToolResult(ToolResultPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), result: metadata.clone() }),
        )?;
        let mut provider_message = message;
        provider_message.tool_call_id = Some(call.call_id.clone());
        context.messages.push(provider_message);
        let mut redacted = Message::text(
            "tool",
            serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".into()),
        );
        redacted.tool_call_id = Some(call.call_id.clone());
        self.emit(
            &context.session_id,
            EventPayload::MessageTool(MessageToolPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), message: redacted }),
        )?;
        Ok(())
    }

    fn record_computer_skipped(
        &self,
        context: &mut Continuation,
        call: &ToolCall,
    ) -> Result<(), BusinessError> {
        self.tool_state(
            context,
            call,
            "failed",
            Some("computer_batch_halted"),
        )?;
        let result = json!({
            "error": {
                "code": "computer_batch_halted",
                "message": suncode_computer::HALT_MESSAGE,
            }
        });
        self.emit(
            &context.session_id,
            EventPayload::ToolResult(ToolResultPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), result: result.clone() }),
        )?;
        let mut tool = Message::text("tool", suncode_computer::HALT_MESSAGE);
        tool.tool_call_id = Some(call.call_id.clone());
        context.messages.push(tool.clone());
        self.emit(
            &context.session_id,
            EventPayload::MessageTool(MessageToolPayload { turn_id: context.turn_id.clone(), call_id: context.active_call_id.clone(), tool_call_id: call.call_id.clone(), message: tool }),
        )?;
        Ok(())
    }

    async fn execute_lsp_call(
        &self,
        context: &Continuation,
        call: &ToolCall,
        token: CancellationToken,
    ) -> Result<Value, BusinessError> {
        let display_path = call
            .arguments
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid("path is required"))?;
        let (scope_root, prepared) = self.prepare_call(context, call)?;
        let relative_path = prepared
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid("path is required"))?;
        let read = self
            .operation_in_project(
                &scope_root,
                "tool/read",
                json!({"path":relative_path,"offset":1,"max_bytes":1024 * 1024}),
                token.clone(),
                None,
            )
            .await?;
        if read.get("truncated").and_then(Value::as_bool) == Some(true) {
            return Err(BusinessError::new(
                "lsp_document_too_large",
                "Language server document exceeds the 1 MiB semantic-tool bound",
            ));
        }
        let bytes = read
            .get("data_base64")
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::new("read_failed", "file content is unavailable"))
            .and_then(|encoded| {
                STANDARD.decode(encoded).map_err(|_| {
                    BusinessError::new("read_failed", "file content could not be decoded")
                })
            })?;
        let text = String::from_utf8(bytes).map_err(|_| {
            BusinessError::new(
                "encoding_unsupported",
                "language server tools require UTF-8 text",
            )
        })?;
        let language_id = call
            .arguments
            .get("languageId")
            .and_then(Value::as_str)
            .or_else(|| lsp::infer_language_id(display_path))
            .ok_or_else(|| {
                BusinessError::new(
                    "lsp_language_unknown",
                    "Language could not be inferred; provide languageId",
                )
            })?;
        let line = call
            .arguments
            .get("line")
            .and_then(Value::as_u64)
            .map(|value| u32::try_from(value).unwrap_or(u32::MAX));
        let column = call
            .arguments
            .get("column")
            .and_then(Value::as_u64)
            .map(|value| u32::try_from(value).unwrap_or(u32::MAX));
        let operation = lsp::operation(&call.name)
            .ok_or_else(|| BusinessError::new("authorization_denied", "Unknown LSP tool"))?;
        self.lsp
            .execute(
                lsp::SemanticRequest {
                    project_id: &context.project_id,
                    project_root: Path::new(&context.project_root),
                    scope_root: Path::new(&scope_root),
                    display_path,
                    relative_path,
                    language_id,
                    text: &text,
                    line,
                    column,
                    operation,
                },
                token,
            )
            .await
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
