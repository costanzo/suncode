use super::*;
use suncode_agent::domain::{SessionImageRecord, SessionRecord};

impl AgentSdk {
    pub fn create_session(
        &self,
        project_id: &str,
        title: Option<&str>,
        model: Option<&str>,
    ) -> SdkResult<SessionRecord> {
        let selected_model = match model {
            Some(model) => Some(model.to_string()),
            None => self.state.store.project_default_model(project_id)?,
        };
        if let Some(model) = selected_model.as_deref() {
            if self.state.providers.route(model).is_none() {
                return Err(BusinessError::new(
                    "model_unavailable",
                    "model is not advertised",
                ));
            }
        }
        self.state
            .store
            .create_session(project_id, title, selected_model.as_deref())
    }

    pub fn rename_session(&self, session_id: &str, title: &str) -> SdkResult<SessionRecord> {
        if title.trim().is_empty() {
            return Err(BusinessError::invalid("title is required"));
        }
        self.state.store.rename_session(session_id, title.trim())
    }

    pub fn archive_session(&self, session_id: &str) -> SdkResult<SessionRecord> {
        self.state.store.set_session_archived(session_id, true)
    }

    pub fn set_session_pinned(&self, session_id: &str, pinned: bool) -> SdkResult<SessionRecord> {
        self.state.store.set_session_pinned(session_id, pinned)
    }

    pub fn reopen_session(&self, session_id: &str) -> SdkResult<SessionRecord> {
        self.state.store.set_session_archived(session_id, false)
    }

    pub fn list_session_images(&self, session_id: &str) -> SdkResult<SessionImagesResult> {
        if self.state.store.session_by_id(session_id)?.is_none() {
            return Err(BusinessError::missing("session"));
        }
        let referenced = self
            .state
            .store
            .messages(session_id)?
            .into_iter()
            .flat_map(|message| message.content)
            .filter(|part| part.kind == "image_ref")
            .map(|part| part.text)
            .collect::<std::collections::HashSet<_>>();
        Ok(SessionImagesResult {
            session_id: session_id.to_string(),
            images: self
                .state
                .store
                .session_images(session_id)?
                .into_iter()
                .filter(|image| !referenced.contains(&image.image_id))
                .collect(),
        })
    }

    pub fn add_session_image(
        &self,
        session_id: &str,
        payload: &Value,
    ) -> SdkResult<SessionImageRecord> {
        if self.state.store.session_by_id(session_id)?.is_none() {
            return Err(BusinessError::missing("session"));
        }
        let request: AddSessionImageRequest = serde_json::from_value(payload.clone())?;
        let display_name = request.display_name.trim();
        let source_kind = request.source_kind.trim();
        let extension = sanitize_image_extension(&request.extension)?;
        if display_name.is_empty() {
            return Err(BusinessError::invalid("display_name is required"));
        }
        if !matches!(source_kind, "file" | "clipboard") {
            return Err(BusinessError::invalid(
                "source_kind must be file or clipboard",
            ));
        }
        if source_kind == "file"
            && request
                .original_path
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(BusinessError::invalid(
                "original_path is required for file uploads",
            ));
        }
        let image_bytes = STANDARD
            .decode(request.bytes_base64.trim())
            .map_err(|_| BusinessError::invalid("bytes_base64 is not valid Base64"))?;
        if image_bytes.is_empty() {
            return Err(BusinessError::invalid("image bytes are required"));
        }
        if image_bytes.len() > 20 * 1024 * 1024 {
            return Err(BusinessError::invalid(
                "image bytes exceed the 20 MiB limit",
            ));
        }
        let thumbnail_base64 = request.thumbnail_base64.trim();
        if thumbnail_base64.is_empty() {
            return Err(BusinessError::invalid("thumbnail_base64 is required"));
        }
        let thumbnail_bytes = STANDARD
            .decode(thumbnail_base64)
            .map_err(|_| BusinessError::invalid("thumbnail_base64 is not valid Base64"))?;
        if thumbnail_bytes.is_empty() || thumbnail_bytes.len() > 1024 * 1024 {
            return Err(BusinessError::invalid(
                "thumbnail must be between 1 byte and 1 MiB",
            ));
        }
        let image_id = uuid::Uuid::new_v4().to_string();
        let directory = self.resolved_image_directory()?.join(session_id);
        std::fs::create_dir_all(&directory)
            .map_err(|error| BusinessError::unavailable(error.to_string()))?;
        let storage_path = directory.join(format!("{image_id}.{extension}"));
        if let Err(error) = std::fs::write(&storage_path, &image_bytes) {
            return Err(BusinessError::unavailable(error.to_string()));
        }
        match self.state.store.insert_session_image(
            &image_id,
            session_id,
            display_name,
            source_kind,
            request.original_path.as_deref().map(str::trim),
            &storage_path,
            thumbnail_base64,
        ) {
            Ok(record) => Ok(record),
            Err(error) => {
                let _ = std::fs::remove_file(&storage_path);
                Err(error)
            }
        }
    }

    pub fn remove_session_image(
        &self,
        session_id: &str,
        image_id: &str,
    ) -> SdkResult<SessionImageRemoval> {
        let referenced = self
            .state
            .store
            .messages(session_id)?
            .into_iter()
            .flat_map(|message| message.content)
            .any(|part| part.kind == "image_ref" && part.text == image_id);
        if referenced {
            return Err(BusinessError::new(
                "conflict",
                "an image attached to a submitted message cannot be removed",
            ));
        }
        let removed = self
            .state
            .store
            .remove_session_image(session_id, image_id)?
            .ok_or_else(|| BusinessError::missing("session_image"))?;
        let path = PathBuf::from(&removed.storage_path);
        if let Err(error) = std::fs::remove_file(&path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                logging::write(
                    Level::Warn,
                    "session_image",
                    format!(
                        "remove_file_failed session={} image={} error={}",
                        session_id, image_id, error
                    ),
                );
            }
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
        Ok(SessionImageRemoval {
            session_id: session_id.to_string(),
            image_id: image_id.to_string(),
            removed: true,
        })
    }

    pub fn session_snapshot(&self, session_id: &str, _after: i64) -> SdkResult<SessionSnapshot> {
        logging::write(
            Level::Debug,
            "session_snapshot",
            format!("begin session={session_id}"),
        );
        let session = self
            .state
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::missing("session"))?;
        let messages = self.state.store.messages(session_id)?;
        let images = self.state.store.session_images(session_id)?;
        let conversation_turns = self.state.store.session_conversation_turns(session_id)?;
        let pending_question = self.state.store.pending_question(session_id)?;
        logging::write(
            Level::Debug,
            "session_snapshot",
            format!("end session={session_id} messages={}", messages.len()),
        );
        Ok(SessionSnapshot {
            session,
            messages,
            conversation_turns,
            images,
            pending_question,
        })
    }

    pub fn session_usage(&self, session_id: &str) -> SdkResult<SessionUsageResult> {
        if self.state.store.session_by_id(session_id)?.is_none() {
            return Err(BusinessError::missing("session"));
        }
        let usage = self.state.store.session_usage(session_id)?;
        Ok(SessionUsageResult {
            session_id: session_id.to_string(),
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
        })
    }

    pub fn list_provider_exchanges(&self, session_id: &str) -> SdkResult<ProviderExchangesResult> {
        if self.state.store.session_by_id(session_id)?.is_none() {
            return Err(BusinessError::missing("session"));
        }
        Ok(ProviderExchangesResult {
            session_id: session_id.to_string(),
            turns: self.state.store.session_trace_turns(session_id)?,
            exchanges: self.state.store.provider_exchanges(session_id)?,
        })
    }

    pub fn provider_exchange(
        &self,
        session_id: &str,
        exchange_id: &str,
    ) -> SdkResult<ProviderExchangeDetails> {
        if exchange_id.trim().is_empty() {
            return Err(BusinessError::invalid("exchange_id is required"));
        }
        let exchange = self
            .state
            .store
            .provider_exchange(session_id, exchange_id)?
            .ok_or_else(|| BusinessError::missing("provider_exchange"))?;
        Ok(ProviderExchangeDetails {
            messages: self
                .state
                .store
                .session_call_messages(session_id, exchange_id)?,
            tool_uses: self.state.store.session_call_tool_uses(exchange_id)?,
            exchange,
        })
    }

    fn resolved_image_directory(&self) -> SdkResult<PathBuf> {
        let configured =
            global_string_setting(&self.state.store, "image_directory")?.unwrap_or_default();
        let configured = configured.trim();
        if configured.is_empty() {
            Ok(self.data_dir.join("data/images"))
        } else {
            Ok(PathBuf::from(configured))
        }
    }
}
