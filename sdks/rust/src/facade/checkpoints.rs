use super::*;

impl AsyncAgentSdk {
    pub fn list_checkpoints(&self, session_id: &str) -> SdkResult<CheckpointsResult> {
        self.session_for_user(session_id)?;
        Ok(CheckpointsResult {
            session_id: session_id.to_string(),
            checkpoints: self.state.store.manifests(session_id)?,
        })
    }

    pub fn checkpoint_manifest(&self, manifest_id: &str) -> SdkResult<CheckpointDetails> {
        let manifest = self
            .state
            .store
            .manifest(manifest_id)?
            .ok_or_else(|| BusinessError::missing("checkpoint"))?;
        Ok(CheckpointDetails {
            manifest,
            items: self.state.store.checkpoint_items(manifest_id)?,
        })
    }

    pub fn restore_checkpoint(
        &self,
        manifest_id: &str,
        session_id: &str,
    ) -> SdkResult<RestoreOutcome> {
        let manifest = self
            .state
            .store
            .manifest(manifest_id)?
            .ok_or_else(|| BusinessError::missing("checkpoint"))?;
        if manifest.session_id != session_id {
            return Err(BusinessError::new(
                "scope_denied",
                "checkpoint does not belong to session",
            ));
        }
        if manifest.status != "available" {
            return Err(BusinessError::new(
                "checkpoint_unavailable",
                "checkpoint is not available",
            ));
        }
        let session = self.session_for_user(session_id)?;
        let project = self.project_for_user(session.project_id.as_deref().unwrap_or(""))?;
        self.state
            .store
            .set_manifest_status(manifest_id, "restoring")?;
        let items = self.state.store.checkpoint_items(manifest_id)?;
        let mut restored = 0;
        for item in &items {
            if item.status != "available" {
                continue;
            }
            match self.state.operations.execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "checkpoint/restore",
                json!({"checkpoint_id": item.checkpoint_id}),
            ) {
                Ok(result) => {
                    restored += 1;
                    emit_event(
                        &self.state,
                        session_id,
                        EventPayload::CheckpointItemRestored(CheckpointItemRestoredPayload {
                            manifest_id: manifest_id.to_string(),
                            checkpoint_id: item.checkpoint_id.clone(),
                            path: result
                                .get("path")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                        }),
                    )?;
                }
                Err(error) => {
                    let status = if restored > 0 { "partial" } else { "conflict" };
                    self.state.store.set_manifest_status(manifest_id, status)?;
                    emit_event(
                        &self.state,
                        session_id,
                        EventPayload::CheckpointRestoreFailed(CheckpointRestoreFailedPayload {
                            manifest_id: manifest_id.to_string(),
                            status: status.to_string(),
                            code: error
                                .get("code")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                        }),
                    )?;
                    return Err(BusinessError::new(
                        error
                            .get("code")
                            .and_then(Value::as_str)
                            .unwrap_or("restore_conflict"),
                        error
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("checkpoint restore failed"),
                    ));
                }
            }
        }
        self.state
            .store
            .set_manifest_status(manifest_id, "restored")?;
        emit_event(
            &self.state,
            session_id,
            EventPayload::CheckpointRestored(CheckpointRestoredPayload {
                manifest_id: manifest_id.to_string(),
                restored_items: restored,
            }),
        )?;
        Ok(RestoreOutcome {
            manifest_id: manifest_id.to_string(),
            status: "restored",
            restored_items: restored,
        })
    }
}
