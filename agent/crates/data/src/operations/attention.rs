use crate::{
    domain::AttentionCandidateRecord,
    store::{lock, Store},
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text};
use suncode_common::BusinessError;

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    correlation_id: String,
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Text)]
    project_display_name: String,
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    session_title: String,
    #[diesel(sql_type = Text)]
    session_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    parent_session_id: Option<String>,
    #[diesel(sql_type = Text)]
    turn_id: String,
    #[diesel(sql_type = Text)]
    occurred_at: String,
}

impl From<Row> for AttentionCandidateRecord {
    fn from(row: Row) -> Self {
        Self {
            kind: row.kind,
            correlation_id: row.correlation_id,
            project_id: row.project_id,
            project_display_name: row.project_display_name,
            session_id: row.session_id,
            session_title: row.session_title,
            session_kind: row.session_kind,
            parent_session_id: row.parent_session_id,
            turn_id: row.turn_id,
            occurred_at: row.occurred_at,
        }
    }
}

impl Store {
    pub fn attention_candidates(
        &self,
        user_id: &str,
        since: &str,
        limit: usize,
    ) -> Result<Vec<AttentionCandidateRecord>, BusinessError> {
        let limit = limit.clamp(1, 512) as i64;
        let mut connection = lock(&self.connection)?;
        let rows = sql_query(
            "SELECT kind,correlation_id,project_id,project_display_name,session_id,session_title,session_kind,parent_session_id,turn_id,occurred_at FROM (\
             SELECT CASE turn.state WHEN 'completed' THEN 'primary_turn_completed' ELSE 'primary_turn_failed' END AS kind,turn.turn_id AS correlation_id,project.project_id,project.display_name AS project_display_name,session.session_id,COALESCE(session.title,'') AS session_title,session.kind AS session_kind,session.parent_session_id,turn.turn_id,COALESCE(turn.completed_at,turn.updated_at) AS occurred_at \
             FROM session_turn AS turn JOIN session ON session.session_id=turn.session_id JOIN project ON project.project_id=session.project_id \
             WHERE project.user_id=? AND session.kind='primary' AND turn.state IN ('completed','failed') AND COALESCE(turn.completed_at,turn.updated_at)>=? \
             UNION ALL \
             SELECT 'approval_requested' AS kind,approval.approval_id AS correlation_id,project.project_id,project.display_name AS project_display_name,session.session_id,COALESCE(session.title,'') AS session_title,session.kind AS session_kind,session.parent_session_id,approval.turn_id,approval.created_at AS occurred_at \
             FROM approval_request AS approval JOIN session ON session.session_id=approval.session_id JOIN project ON project.project_id=session.project_id \
             WHERE project.user_id=? AND approval.status='pending' AND approval.created_at>=? \
             UNION ALL \
             SELECT 'question_asked' AS kind,turn.recovery_approval_id AS correlation_id,project.project_id,project.display_name AS project_display_name,session.session_id,COALESCE(session.title,'') AS session_title,session.kind AS session_kind,session.parent_session_id,turn.turn_id,COALESCE(turn.recovery_created_at,turn.updated_at) AS occurred_at \
             FROM session_turn AS turn JOIN session ON session.session_id=turn.session_id JOIN project ON project.project_id=session.project_id \
             WHERE project.user_id=? AND session.kind='primary' AND turn.recovery_status='pending' AND json_extract(turn.recovery_snapshot_json,'$.pending_call.name')='question' AND COALESCE(turn.recovery_created_at,turn.updated_at)>=?\
             ) ORDER BY occurred_at,correlation_id LIMIT ?",
        )
        .bind::<Text, _>(user_id)
        .bind::<Text, _>(since)
        .bind::<Text, _>(user_id)
        .bind::<Text, _>(since)
        .bind::<Text, _>(user_id)
        .bind::<Text, _>(since)
        .bind::<BigInt, _>(limit)
        .load::<Row>(&mut *connection)
        .map_err(crate::database_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
