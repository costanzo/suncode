use crate::{
    domain::{LanguageServerInput, LanguageServerRecord},
    store::{business_transaction, lock, nonnegative, now, Store},
};
use diesel::{
    prelude::*,
    sql_query,
    sql_types::{Integer, Text},
    sqlite::SqliteConnection,
    QueryableByName,
};
use serde_json::json;
use std::{collections::HashSet, path::Path};
use suncode_common::BusinessError;

const SELECT: &str = "SELECT language_server_id,display_name,config_json,enabled,sort_order,revision,created_at,updated_at FROM language_server";

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    language_server_id: String,
    #[diesel(sql_type = Text)]
    display_name: String,
    #[diesel(sql_type = Text)]
    config_json: String,
    #[diesel(sql_type = Integer)]
    enabled: i32,
    #[diesel(sql_type = Integer)]
    sort_order: i32,
    #[diesel(sql_type = Integer)]
    revision: i32,
    #[diesel(sql_type = Text)]
    created_at: String,
    #[diesel(sql_type = Text)]
    updated_at: String,
}

fn to_record(row: Row) -> Result<LanguageServerRecord, BusinessError> {
    Ok(LanguageServerRecord {
        language_server_id: row.language_server_id,
        display_name: row.display_name,
        config: serde_json::from_str(&row.config_json).map_err(|_| {
            BusinessError::database("stored language server configuration is invalid")
        })?,
        enabled: row.enabled != 0,
        sort_order: i64::from(row.sort_order),
        revision: nonnegative(i64::from(row.revision))?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn by_id(
    connection: &mut SqliteConnection,
    language_server_id: &str,
) -> Result<Option<LanguageServerRecord>, BusinessError> {
    sql_query(format!("{SELECT} WHERE language_server_id=?"))
        .bind::<Text, _>(language_server_id)
        .get_result::<Row>(connection)
        .optional()
        .map_err(crate::database_error)?
        .map(to_record)
        .transpose()
}

fn validate_input(input: &LanguageServerInput) -> Result<(), BusinessError> {
    let display_name = input.display_name.trim();
    if display_name.is_empty() || display_name.chars().count() > 128 {
        return Err(BusinessError::invalid(
            "Language server name is required and must not exceed 128 characters",
        ));
    }
    if input.sort_order < 0 || input.sort_order > i64::from(i32::MAX) {
        return Err(BusinessError::invalid(
            "Language server sort order is outside the supported range",
        ));
    }
    let config = &input.config;
    if config.version != 1 {
        return Err(BusinessError::invalid(
            "Language server configuration version is unsupported",
        ));
    }
    if config.command.trim().is_empty()
        || config.command.chars().count() > 2_048
        || config.command.contains('\0')
    {
        return Err(BusinessError::invalid(
            "Language server executable is required and must not exceed 2048 characters",
        ));
    }
    if config.arguments.len() > 128
        || config
            .arguments
            .iter()
            .any(|value| value.chars().count() > 8_192 || value.contains('\0'))
    {
        return Err(BusinessError::invalid(
            "Language server arguments exceed the supported bounds",
        ));
    }
    if config.language_ids.is_empty() || config.language_ids.len() > 32 {
        return Err(BusinessError::invalid(
            "Language server must declare between 1 and 32 language ids",
        ));
    }
    let mut language_ids = HashSet::new();
    for language_id in &config.language_ids {
        let language_id = language_id.trim();
        if language_id.is_empty()
            || language_id.chars().count() > 64
            || !language_id
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || "_.+-".contains(value))
            || !language_ids.insert(language_id.to_ascii_lowercase())
        {
            return Err(BusinessError::invalid(
                "Language server language ids must be unique safe identifiers",
            ));
        }
    }
    if config.root_markers.len() > 32 {
        return Err(BusinessError::invalid(
            "Language server root markers exceed the supported bound",
        ));
    }
    let mut root_markers = HashSet::new();
    for marker in &config.root_markers {
        let marker = marker.trim();
        let path = Path::new(marker);
        if marker.is_empty()
            || marker.chars().count() > 512
            || marker.contains('\0')
            || path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
            || !root_markers.insert(marker.to_ascii_lowercase())
        {
            return Err(BusinessError::invalid(
                "Language server root markers must be unique project-relative paths",
            ));
        }
    }
    if !config.initialization_options.is_null() && !config.initialization_options.is_object() {
        return Err(BusinessError::invalid(
            "Language server initialization options must be a JSON object",
        ));
    }
    if config.environment.len() > 128 {
        return Err(BusinessError::invalid(
            "Language server environment has too many entries",
        ));
    }
    for (key, value) in &config.environment {
        if key.is_empty()
            || key.chars().count() > 128
            || !key
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
            || key
                .chars()
                .next()
                .is_some_and(|value| value.is_ascii_digit())
            || value.chars().count() > 32_768
            || value.contains('\0')
        {
            return Err(BusinessError::invalid(
                "Language server environment entry is invalid",
            ));
        }
    }
    if !(1..=120).contains(&config.startup_timeout_seconds)
        || !(1..=600).contains(&config.request_timeout_seconds)
    {
        return Err(BusinessError::invalid(
            "Language server timeout is outside the supported range",
        ));
    }
    Ok(())
}

fn revision_conflict(expected: u64, actual: u64) -> BusinessError {
    BusinessError::new(
        "language_server_revision_conflict",
        "Language server was changed by another operation",
    )
    .details(json!({"expectedRevision":expected,"actualRevision":actual}))
}

impl Store {
    pub fn language_servers(&self) -> Result<Vec<LanguageServerRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        sql_query(format!(
            "{SELECT} ORDER BY sort_order,display_name COLLATE NOCASE,language_server_id"
        ))
        .load::<Row>(&mut *connection)
        .map_err(crate::database_error)?
        .into_iter()
        .map(to_record)
        .collect()
    }

    pub fn language_server_by_id(
        &self,
        language_server_id: &str,
    ) -> Result<Option<LanguageServerRecord>, BusinessError> {
        let mut connection = lock(&self.connection)?;
        by_id(&mut connection, language_server_id.trim())
    }

    pub fn create_language_server(
        &self,
        language_server_id: &str,
        input: &LanguageServerInput,
    ) -> Result<LanguageServerRecord, BusinessError> {
        validate_input(input)?;
        let language_server_id = language_server_id.trim();
        if language_server_id.is_empty() || language_server_id.chars().count() > 128 {
            return Err(BusinessError::invalid("Language server id is invalid"));
        }
        let mut connection = lock(&self.connection)?;
        business_transaction(&mut connection, |connection| {
            if let Some(existing) = by_id(connection, language_server_id)? {
                if existing.display_name == input.display_name.trim()
                    && existing.config == input.config
                    && existing.enabled == input.enabled
                    && existing.sort_order == input.sort_order
                {
                    return Ok(existing);
                }
                return Err(BusinessError::new(
                    "idempotency_conflict",
                    "Language server create id was already used with different input",
                ));
            }
            let timestamp = now();
            let config_json = serde_json::to_string(&input.config)?;
            sql_query("INSERT INTO language_server(language_server_id,display_name,config_json,enabled,sort_order,revision,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?)")
                .bind::<Text, _>(language_server_id)
                .bind::<Text, _>(input.display_name.trim())
                .bind::<Text, _>(&config_json)
                .bind::<Integer, _>(input.enabled as i32)
                .bind::<Integer, _>(input.sort_order as i32)
                .bind::<Integer, _>(1)
                .bind::<Text, _>(&timestamp)
                .bind::<Text, _>(&timestamp)
                .execute(connection)
                .map_err(|error| match error {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => BusinessError::new(
                        "language_server_conflict",
                        "Language server name already exists",
                    ),
                    other => crate::database_error(other),
                })?;
            by_id(connection, language_server_id)?
                .ok_or_else(|| BusinessError::database("Language server was not stored"))
        })
    }

    pub fn update_language_server(
        &self,
        language_server_id: &str,
        expected_revision: u64,
        input: &LanguageServerInput,
    ) -> Result<LanguageServerRecord, BusinessError> {
        validate_input(input)?;
        let mut connection = lock(&self.connection)?;
        business_transaction(&mut connection, |connection| {
            let current = by_id(connection, language_server_id)?
                .ok_or_else(|| BusinessError::missing("language_server"))?;
            if current.revision != expected_revision {
                if current.display_name == input.display_name.trim()
                    && current.config == input.config
                    && current.enabled == input.enabled
                    && current.sort_order == input.sort_order
                {
                    return Ok(current);
                }
                return Err(revision_conflict(expected_revision, current.revision));
            }
            let revision = i32::try_from(current.revision + 1)
                .map_err(|_| BusinessError::invalid("Language server revision overflow"))?;
            let config_json = serde_json::to_string(&input.config)?;
            sql_query("UPDATE language_server SET display_name=?,config_json=?,enabled=?,sort_order=?,revision=?,updated_at=? WHERE language_server_id=? AND revision=?")
                .bind::<Text, _>(input.display_name.trim())
                .bind::<Text, _>(&config_json)
                .bind::<Integer, _>(input.enabled as i32)
                .bind::<Integer, _>(input.sort_order as i32)
                .bind::<Integer, _>(revision)
                .bind::<Text, _>(&now())
                .bind::<Text, _>(language_server_id)
                .bind::<Integer, _>(expected_revision as i32)
                .execute(connection)
                .map_err(|error| match error {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => BusinessError::new(
                        "language_server_conflict",
                        "Language server name already exists",
                    ),
                    other => crate::database_error(other),
                })?;
            by_id(connection, language_server_id)?
                .ok_or_else(|| BusinessError::database("Language server update was not stored"))
        })
    }

    pub fn set_language_server_enabled(
        &self,
        language_server_id: &str,
        expected_revision: u64,
        enabled: bool,
    ) -> Result<LanguageServerRecord, BusinessError> {
        let current = self
            .language_server_by_id(language_server_id)?
            .ok_or_else(|| BusinessError::missing("language_server"))?;
        self.update_language_server(
            language_server_id,
            expected_revision,
            &LanguageServerInput {
                display_name: current.display_name,
                config: current.config,
                enabled,
                sort_order: current.sort_order,
            },
        )
    }

    pub fn delete_language_server(
        &self,
        language_server_id: &str,
        expected_revision: u64,
    ) -> Result<bool, BusinessError> {
        let mut connection = lock(&self.connection)?;
        business_transaction(&mut connection, |connection| {
            let Some(current) = by_id(connection, language_server_id)? else {
                return Ok(false);
            };
            if current.revision != expected_revision {
                return Err(revision_conflict(expected_revision, current.revision));
            }
            Ok(
                sql_query("DELETE FROM language_server WHERE language_server_id=? AND revision=?")
                    .bind::<Text, _>(language_server_id)
                    .bind::<Integer, _>(expected_revision as i32)
                    .execute(connection)
                    .map_err(crate::database_error)?
                    == 1,
            )
        })
    }
}
