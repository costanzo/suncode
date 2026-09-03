impl Agent {
    async fn run(
        &self,
        mut context: Continuation,
        user_input: Option<Message>,
        token: CancellationToken,
        provider: ModelRoute,
    ) -> Result<TurnResponse, BusinessError> {
        let started = Instant::now();
        if let Some(message) = user_input {
            self.turn_state(&context, "admitted", None)?;
            context.messages.push(message.clone());
            self.emit(
                &context.session_id,
                "message.user",
                json!({"message_id":Uuid::new_v4(),"turn_id":context.turn_id,"message":message}),
            )?;
        }
        self.turn_state(&context, "preparing", None)?;
        while context.iterations < 32 {
            if token.is_cancelled() {
                return self.fail_context(&context, "cancelled", "Turn was cancelled");
            }
            if started.elapsed() > Duration::from_secs(600) {
                return self.fail_context(
                    &context,
                    "turn_timeout",
                    "Turn exceeded its wall-clock budget",
                );
            }
            self.drain_queued_messages(&mut context)?;
            context.iterations += 1;
            self.turn_state(&context, "calling_model", None)?;
            let prompt = context::build_for_model(
                &context.messages,
                self.providers
                    .limits(&context.model)
                    .and_then(|limits| limits.max_input_tokens),
                self.providers
                    .limits(&context.model)
                    .and_then(|limits| limits.auto_compact_tokens),
            );
            if prompt.compacted && !context.context_compacted {
                context.context_compacted = true;
                let compaction_id = Uuid::new_v4().to_string();
                self.emit(
                    &context.session_id,
                    "context.compacted",
                    json!({
                        "exchange_id": compaction_id,
                        "turn_id": context.turn_id,
                        "provider": "SunCode",
                        "model_id": "context-compaction",
                        "wire_model": "internal",
                        "iteration": context.iterations,
                        "started_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                        "original_characters": prompt.original_characters,
                        "retained_characters": prompt.retained_characters,
                        "original_tokens": prompt.original_tokens,
                        "retained_tokens": prompt.retained_tokens,
                        "dropped_messages": prompt.dropped_messages,
                        "summary": prompt.summary,
                    }),
                )?;
            }
            if prompt.compacted {
                context.messages = prompt.messages;
            }
            let exchange_id = Uuid::new_v4().to_string();
            context.active_call_id = Some(exchange_id.clone());
            let mut llm_messages = vec![host_environment_message(&context.session_started_at)];
            if let Some(message) = project_instruction_message(&context.project_root) {
                llm_messages.push(message);
            }
            if let Some(message) = self.dependency_context_message(&context.project_id)? {
                llm_messages.push(message);
            }
            llm_messages.extend(
                context
                    .messages
                    .iter()
                    .map(|message| self.to_llm_message_with_images(&context.session_id, message))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            let trace_messages = llm_messages
                .iter()
                .map(redacted_trace_message)
                .collect::<Vec<_>>();
            self.emit(
                &context.session_id,
                "provider.exchange.started",
                json!({
                    "exchange_id": exchange_id,
                    "turn_id": context.turn_id,
                    "provider": provider.provider_id,
                    "model_id": context.model,
                    "wire_model": provider.wire_model,
                    "iteration": context.iterations,
                    "input_messages": trace_messages,
                }),
            )?;
            let result = {
                let (delta_sender, mut delta_receiver) = mpsc::unbounded_channel();
                let tool_definitions = suncode_tool::definitions::all()
                    .into_iter()
                    .map(|definition| suncode_llm::ToolDefinition {
                        name: definition.name.into(),
                        description: definition.description.into(),
                        parameters: definition.parameters,
                    })
                    .collect::<Vec<_>>();
                let provider_call = provider.provider.complete(
                    CompletionRequest {
                        messages: &llm_messages,
                        wire_model: &provider.wire_model,
                        tools: &tool_definitions,
                        reasoning_effort: context.reasoning_effort.as_deref(),
                    },
                    &token,
                    delta_sender,
                );
                tokio::pin!(provider_call);
                let result = loop {
                    tokio::select! {
                        value = &mut provider_call => break value,
                        Some(delta) = delta_receiver.recv() => {
                            self.emit_live(&context.session_id, "assistant.delta", json!({"turn_id":context.turn_id,"text":delta}));
                        }
                    }
                };
                while let Ok(delta) = delta_receiver.try_recv() {
                    self.emit_live(
                        &context.session_id,
                        "assistant.delta",
                        json!({"turn_id":context.turn_id,"text":delta}),
                    );
                }
                result
            };
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    self.emit(
                        &context.session_id,
                        "provider.exchange.failed",
                        json!({
                            "exchange_id": exchange_id,
                            "turn_id": context.turn_id,
                            "error": {
                                "code": error.code,
                                "message": error.message.clone(),
                                "retryable": error.retryable,
                            },
                            "provider_request_id": error.provider_request_id,
                        }),
                    )?;
                    return Err(error);
                }
            };
            if result.text.len() > 8 * 1024 * 1024 {
                return self.fail_context(
                    &context,
                    "output_budget_exceeded",
                    "Provider output exceeded its budget",
                );
            }
            let usage = result.usage.as_ref().map(|usage| Usage {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                total_tokens: usage.total_tokens,
            });
            let cache_read_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.cache_read_tokens);
            let cache_miss_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.cache_miss_tokens);
            let cache_write_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.cache_write_tokens);
            let reasoning_tokens = result
                .usage
                .as_ref()
                .and_then(|usage| usage.reasoning_tokens);
            let tool_calls = result
                .tool_calls
                .iter()
                .map(|call| ToolCall {
                    call_id: call.call_id.clone(),
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                })
                .collect::<Vec<_>>();
            if let Some(usage) = &usage {
                context.usage.add(usage);
                self.emit(
                    &context.session_id,
                    "usage.updated",
                    json!({"turn_id":context.turn_id,"usage":context.usage}),
                )?;
            }
            let assistant = Message {
                role: "assistant".into(),
                content: if result.text.is_empty() {
                    Vec::new()
                } else {
                    Message::text("assistant", result.text).content
                },
                tool_calls: tool_calls.clone(),
                tool_call_id: None,
            };
            self.emit(
                &context.session_id,
                "provider.exchange.completed",
                json!({
                    "exchange_id": exchange_id,
                    "turn_id": context.turn_id,
                    "output_message": assistant,
                    "tool_calls": tool_calls.clone(),
                    "usage": usage.as_ref().map(|usage| json!({
                        "input_tokens": usage.input_tokens,
                        "output_tokens": usage.output_tokens,
                        "total_tokens": usage.total_tokens,
                        "cache_read_tokens": cache_read_tokens,
                        "cache_miss_tokens": cache_miss_tokens,
                        "cache_write_tokens": cache_write_tokens,
                        "reasoning_tokens": reasoning_tokens,
                    })),
                    "provider_request_id": result.provider_request_id,
                    "provider_response_id": result.provider_response_id,
                    "finish_reason": result.finish_reason.clone(),
                }),
            )?;
            context.messages.push(assistant.clone());
            self.emit(&context.session_id, "message.assistant", json!({"message_id":Uuid::new_v4(),"turn_id":context.turn_id,"call_id":context.active_call_id,"message":assistant,"usage":context.usage,"finish_reason":result.finish_reason}))?;
            if tool_calls.is_empty() {
                if self.drain_queued_messages(&mut context)? {
                    self.turn_state(&context, "preparing", None)?;
                    continue;
                }
                self.turn_state(&context, "completed", None)?;
                self.emit(&context.session_id, "turn.completed", json!({"turn_id":context.turn_id,"usage":context.usage,"iterations":context.iterations,"tool_calls":context.tool_calls}))?;
                let response = TurnResponse::Completed {
                    turn_id: context.turn_id.clone(),
                    message: assistant,
                    usage: context.usage.clone(),
                    iterations: context.iterations,
                    tool_calls: context.tool_calls,
                };
                self.store.complete_turn(
                    &context.session_id,
                    &context.submission_key,
                    &serde_json::to_value(&response).map_err(|_| {
                        BusinessError::new("agent_unavailable", "turn response could not be stored")
                    })?,
                )?;
                return Ok(response);
            }
            self.turn_state(&context, "resolving_calls", None)?;
            self.resolve_calls(&mut context, tool_calls, token.clone())
                .await?;
            self.drain_queued_messages(&mut context)?;
            self.turn_state(&context, "preparing", None)?;
        }
        self.fail_context(
            &context,
            "iteration_budget_exceeded",
            "Turn exceeded its iteration budget",
        )
    }

    fn drain_queued_messages(&self, context: &mut Continuation) -> Result<bool, BusinessError> {
        let queued = {
            let mut queues = self
                .queued_messages
                .lock()
                .map_err(|_| BusinessError::new("agent_unavailable", "turn queue unavailable"))?;
            queues.remove(&context.session_id).unwrap_or_default()
        };
        if queued.is_empty() {
            return Ok(false);
        }
        for item in queued {
            let images =
                self.validate_message_images(&context.session_id, &context.model, &item.image_ids)?;
            let message = message_with_image_refs(&item.input, &images);
            context.messages.push(message.clone());
            self.emit(
                &context.session_id,
                "message.user",
                json!({
                    "message_id": Uuid::new_v4(),
                    "turn_id": context.turn_id,
                    "queued_id": item.queued_id,
                    "queued_idempotency_key": item.idempotency_key,
                    "message": message
                }),
            )?;
        }
        Ok(true)
    }

    fn validate_message_images(
        &self,
        session_id: &str,
        model: &str,
        image_ids: &[String],
    ) -> Result<Vec<suncode_data::SessionImageRecord>, BusinessError> {
        if image_ids.len() > 3 {
            return Err(BusinessError::invalid(
                "a message can include at most three images",
            ));
        }
        if !image_ids.is_empty() && !self.providers.supports_vision(model) {
            return Err(BusinessError::new(
                "unsupported_capability",
                "selected model does not support image input",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        let mut images = Vec::with_capacity(image_ids.len());
        for image_id in image_ids {
            if image_id.trim().is_empty() || !seen.insert(image_id.as_str()) {
                return Err(BusinessError::invalid(
                    "image IDs must be non-empty and unique",
                ));
            }
            let image = self
                .store
                .session_image_by_id(session_id, image_id)?
                .ok_or_else(|| {
                    BusinessError::new("not_found", "message image was not found in this session")
                })?;
            let size = fs::metadata(&image.storage_path)
                .map_err(|_| BusinessError::new("not_found", "message image file is unavailable"))?
                .len();
            if size == 0 || size > 20 * 1024 * 1024 {
                return Err(BusinessError::invalid(
                    "message image must be between 1 byte and 20 MiB",
                ));
            }
            images.push(image);
        }
        Ok(images)
    }

    fn to_llm_message_with_images(
        &self,
        session_id: &str,
        message: &Message,
    ) -> Result<suncode_llm::Message, BusinessError> {
        let mut converted = to_llm_message(message);
        for part in &mut converted.content {
            if part.kind != "image_ref" {
                continue;
            }
            let image = self
                .store
                .session_image_by_id(session_id, &part.text)?
                .ok_or_else(|| BusinessError::new("not_found", "message image is unavailable"))?;
            let bytes = fs::read(&image.storage_path).map_err(|_| {
                BusinessError::new("not_found", "message image file is unavailable")
            })?;
            if bytes.is_empty() || bytes.len() > 20 * 1024 * 1024 {
                return Err(BusinessError::invalid(
                    "message image must be between 1 byte and 20 MiB",
                ));
            }
            part.kind = "image_url".into();
            part.text = format!(
                "data:{};base64,{}",
                image_mime_type(&image.storage_path)?,
                STANDARD.encode(bytes)
            );
        }
        Ok(converted)
    }

}
