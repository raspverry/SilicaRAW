use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use rusqlite::{params, Connection};

use crate::common::{stable_catalog_id, unique_catalog_id, validate_sidecar_photo_id};
use crate::{
    open_catalog, open_existing_library_for_read, ActionLogEntry, AiResult, LibraryStorageError,
    NewActionLogEntry, NewAiResult, AI_RESULT_PROPOSE_PERMISSION_ID, AI_RESULT_SCHEMA,
    AI_RESULT_VERSION,
};

pub fn append_action_log_entry(
    library_root_path: impl AsRef<Path>,
    entry: NewActionLogEntry,
) -> Result<ActionLogEntry, LibraryStorageError> {
    validate_new_action_log_entry(&entry)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let id = action_log_id(&entry);
    connection.execute(
        r#"
        INSERT INTO action_log(
          id,
          actor_type,
          actor_id,
          action_type,
          subject_type,
          subject_id,
          payload_json,
          side_effect_category,
          evidence_ref
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            id,
            entry.actor_type,
            entry.actor_id,
            entry.action_type,
            entry.subject_type,
            entry.subject_id,
            entry.payload_json,
            entry.side_effect_category,
            entry.evidence_ref,
        ],
    )?;
    action_log_entry_by_id(&connection, &id)
}

pub fn list_action_log_entries(
    library_root_path: impl AsRef<Path>,
    limit: u16,
) -> Result<Vec<ActionLogEntry>, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let limit = i64::from(limit.clamp(1, 500));
    let mut statement = connection.prepare(
        r#"
        SELECT
          id,
          actor_type,
          actor_id,
          action_type,
          subject_type,
          subject_id,
          side_effect_category,
          evidence_ref,
          payload_json,
          created_at
        FROM action_log
        ORDER BY rowid DESC
        LIMIT ?1
        "#,
    )?;
    let entries = statement
        .query_map(params![limit], action_log_entry_from_row)?
        .map(|row| row.map_err(LibraryStorageError::from))
        .collect();
    entries
}

pub fn append_ai_result(
    library_root_path: impl AsRef<Path>,
    result: NewAiResult,
) -> Result<AiResult, LibraryStorageError> {
    let result_json = build_ai_result_json(&result)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let id = unique_catalog_id("ai-result");
    connection.execute(
        r#"
        INSERT INTO ai_results(
          id,
          photo_id,
          task_type,
          model_id,
          result_json,
          approved
        )
        VALUES (?1, ?2, ?3, ?4, ?5, 0)
        "#,
        params![
            id,
            result.photo_id,
            result.task_type,
            result.model_id,
            result_json,
        ],
    )?;
    ai_result_by_id(&connection, &id)
}

pub fn get_ai_result(
    library_root_path: impl AsRef<Path>,
    result_id: impl AsRef<str>,
) -> Result<AiResult, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    ai_result_by_id(&connection, result_id.as_ref())
}

pub fn approve_ai_result(
    library_root_path: impl AsRef<Path>,
    result_id: impl AsRef<str>,
) -> Result<AiResult, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection.execute(
        "UPDATE ai_results SET approved = 1 WHERE id = ?1",
        params![result_id.as_ref()],
    )?;
    ai_result_by_id(&connection, result_id.as_ref())
}

pub fn list_ai_results_for_photo(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    limit: u16,
) -> Result<Vec<AiResult>, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let limit = i64::from(limit.clamp(1, 500));
    let mut statement = connection.prepare(
        r#"
        SELECT id, photo_id, task_type, model_id, result_json, approved, created_at
        FROM ai_results
        WHERE photo_id = ?1
        ORDER BY rowid DESC
        LIMIT ?2
        "#,
    )?;
    let results = statement
        .query_map(params![photo_id, limit], ai_result_from_row)?
        .map(|row| row.map_err(LibraryStorageError::from))
        .collect();
    results
}

fn ai_result_by_id(connection: &Connection, id: &str) -> Result<AiResult, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id, photo_id, task_type, model_id, result_json, approved, created_at
            FROM ai_results
            WHERE id = ?1
            "#,
            params![id],
            ai_result_from_row,
        )
        .map_err(LibraryStorageError::from)
}

fn ai_result_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiResult> {
    let approved: i64 = row.get(5)?;
    Ok(AiResult {
        id: row.get(0)?,
        photo_id: row.get(1)?,
        task_type: row.get(2)?,
        model_id: row.get(3)?,
        result_json: row.get(4)?,
        approved: approved != 0,
        created_at: row.get(6)?,
    })
}

fn build_ai_result_json(result: &NewAiResult) -> Result<String, LibraryStorageError> {
    validate_new_ai_result(result)?;
    let output: serde_json::Value = serde_json::from_str(&result.output_json)?;
    let payload = serde_json::json!({
        "schema": AI_RESULT_SCHEMA,
        "version": AI_RESULT_VERSION,
        "local_only": true,
        "permission_id": result.permission_id,
        "photo_id": result.photo_id,
        "task_type": result.task_type,
        "model_id": result.model_id,
        "output": output,
    });
    serde_json::to_string(&payload).map_err(LibraryStorageError::from)
}

fn validate_new_ai_result(result: &NewAiResult) -> Result<(), LibraryStorageError> {
    validate_sidecar_photo_id(&result.photo_id)?;
    for (field, value) in [
        ("task_type", result.task_type.as_str()),
        ("model_id", result.model_id.as_str()),
        ("permission_id", result.permission_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(LibraryStorageError::AiResultValidation(format!(
                "{field} must not be empty"
            )));
        }
    }
    if result.permission_id != AI_RESULT_PROPOSE_PERMISSION_ID {
        return Err(LibraryStorageError::AiResultValidation(format!(
            "AI result permission must be {AI_RESULT_PROPOSE_PERMISSION_ID}"
        )));
    }

    let output: serde_json::Value = serde_json::from_str(&result.output_json)?;
    let output_object = output.as_object().ok_or_else(|| {
        LibraryStorageError::AiResultValidation("AI result output must be an object".to_string())
    })?;
    if output_object.is_empty() {
        return Err(LibraryStorageError::AiResultValidation(
            "AI result output must not be empty".to_string(),
        ));
    }
    if contains_direct_ai_mutation_claim(&output) {
        return Err(LibraryStorageError::AiResultValidation(
            "direct edit mutation is not allowed in AI result output".to_string(),
        ));
    }

    Ok(())
}

fn contains_direct_ai_mutation_claim(value: &serde_json::Value) -> bool {
    const BLOCKED_KEYS: &[&str] = &[
        "edit_graph",
        "edit_state",
        "edit_states",
        "photo_flags",
        "rating",
        "picked",
        "rejected",
        "color_label",
    ];

    match value {
        serde_json::Value::Object(object) => object.iter().any(|(key, value)| {
            BLOCKED_KEYS.contains(&key.as_str()) || contains_direct_ai_mutation_claim(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(contains_direct_ai_mutation_claim),
        _ => false,
    }
}

fn action_log_entry_by_id(
    connection: &Connection,
    id: &str,
) -> Result<ActionLogEntry, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT
              id,
              actor_type,
              actor_id,
              action_type,
              subject_type,
              subject_id,
              side_effect_category,
              evidence_ref,
              payload_json,
              created_at
            FROM action_log
            WHERE id = ?1
            "#,
            params![id],
            action_log_entry_from_row,
        )
        .map_err(LibraryStorageError::from)
}

fn action_log_entry_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ActionLogEntry> {
    Ok(ActionLogEntry {
        id: row.get(0)?,
        actor_type: row.get(1)?,
        actor_id: row.get(2)?,
        action_type: row.get(3)?,
        subject_type: row.get(4)?,
        subject_id: row.get(5)?,
        side_effect_category: row.get(6)?,
        evidence_ref: row.get(7)?,
        payload_json: row.get(8)?,
        created_at: row.get(9)?,
    })
}

fn validate_new_action_log_entry(entry: &NewActionLogEntry) -> Result<(), LibraryStorageError> {
    if entry.actor_type.trim().is_empty() {
        return Err(LibraryStorageError::ActionLogValidation(
            "actor_type is required".to_string(),
        ));
    }
    if entry.action_type.trim().is_empty() {
        return Err(LibraryStorageError::ActionLogValidation(
            "action_type is required".to_string(),
        ));
    }
    if entry.side_effect_category.trim().is_empty() {
        return Err(LibraryStorageError::ActionLogValidation(
            "side_effect_category is required".to_string(),
        ));
    }
    if entry.action_type == "original_mutation" || entry.side_effect_category == "original_mutation"
    {
        return Err(LibraryStorageError::ActionLogValidation(
            "original mutation action logging is blocked".to_string(),
        ));
    }
    if extension_database_bypass_claim(entry) {
        return Err(LibraryStorageError::ActionLogValidation(
            "extension database bypass action logging is blocked".to_string(),
        ));
    }
    let payload: serde_json::Value = serde_json::from_str(&entry.payload_json)?;
    if !payload.is_object() {
        return Err(LibraryStorageError::ActionLogValidation(
            "payload_json must be a JSON object".to_string(),
        ));
    }
    Ok(())
}

fn extension_database_bypass_claim(entry: &NewActionLogEntry) -> bool {
    let actor_type = entry.actor_type.trim().to_ascii_lowercase();
    if !matches!(actor_type.as_str(), "plugin" | "mcp" | "ai" | "agent") {
        return false;
    }
    let subject_type = entry.subject_type.as_deref().unwrap_or_default();
    [
        entry.action_type.as_str(),
        entry.side_effect_category.as_str(),
        subject_type,
    ]
    .iter()
    .any(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        matches!(
            normalized.as_str(),
            "raw_sql" | "direct_sql" | "direct_database_access" | "database_write" | "sqlite"
        )
    })
}

fn action_log_id(entry: &NewActionLogEntry) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    stable_catalog_id(
        "action-log",
        &format!(
            "{}\n{}\n{}\n{}\n{}",
            entry.actor_type,
            entry.action_type,
            entry.subject_id.as_deref().unwrap_or(""),
            entry.evidence_ref.as_deref().unwrap_or(""),
            nanos
        ),
    )
}
