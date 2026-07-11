use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use silica_catalog::{CatalogFlagError, PhotoFlags};

use super::common::{bool_to_sql, current_timestamp_string, stable_catalog_id, unique_catalog_id};
use super::{
    open_catalog, open_existing_library_for_read, open_local_library, BatchEditGraphCommit,
    BatchEditGraphCommitResult, HistoryCommandResult, LibraryStorageError, PhotoHistoryItem,
    PhotoHistoryPanel, ACTION_SCHEMA, ACTION_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryActionRow {
    id: String,
    action_kind: String,
    action_json: String,
}

struct PreparedBatchEditGraphCommit {
    photo_id: String,
    before_graph: silica_edit::EditGraph,
    graph: silica_edit::EditGraph,
    label: String,
}

/// Load the active committed edit graph for a photo without creating a default draft.
pub fn load_active_edit_graph(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_edit::EditGraph>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;

    connection
        .query_row(
            r#"
            SELECT edit_graph_json
            FROM edit_states
            WHERE photo_id = ?1 AND active = 1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![photo_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|json| {
            let graph: silica_edit::EditGraph = serde_json::from_str(&json)?;
            silica_edit::validate_edit_graph(&graph)?;
            Ok(graph)
        })
        .transpose()
}

/// Load the active edit graph for a photo, or build a default draft without writing it.
pub fn load_active_edit_graph_or_default(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_edit::EditGraph>, LibraryStorageError> {
    let library_root_path = library_root_path.as_ref();
    if let Some(graph) = load_active_edit_graph(library_root_path, photo_id)? {
        return Ok(Some(graph));
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;

    let source = connection
        .query_row(
            r#"
            SELECT id, path, file_size, modified_at, partial_hash, full_hash
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(silica_edit::EditGraphSource {
                    photo_id: row.get(0)?,
                    path: row.get(1)?,
                    file_size: row.get(2)?,
                    modified_at: row.get(3)?,
                    partial_hash: row.get(4)?,
                    full_hash: row.get(5)?,
                })
            },
        )
        .optional()?;

    Ok(source.map(|source| silica_edit::default_edit_graph(source, current_timestamp_string())))
}

fn load_active_edit_graph_or_default_from_transaction(
    transaction: &Transaction<'_>,
    photo_id: &str,
    updated_at: &str,
) -> Result<Option<silica_edit::EditGraph>, LibraryStorageError> {
    let active = transaction
        .query_row(
            r#"
            SELECT edit_graph_json
            FROM edit_states
            WHERE photo_id = ?1 AND active = 1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![photo_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|json| {
            let graph: silica_edit::EditGraph = serde_json::from_str(&json)?;
            silica_edit::validate_edit_graph(&graph)?;
            Ok::<silica_edit::EditGraph, LibraryStorageError>(graph)
        })
        .transpose()?;
    if active.is_some() {
        return Ok(active);
    }

    let source = transaction
        .query_row(
            r#"
            SELECT id, path, file_size, modified_at, partial_hash, full_hash
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(silica_edit::EditGraphSource {
                    photo_id: row.get(0)?,
                    path: row.get(1)?,
                    file_size: row.get(2)?,
                    modified_at: row.get(3)?,
                    partial_hash: row.get(4)?,
                    full_hash: row.get(5)?,
                })
            },
        )
        .optional()?;

    Ok(source.map(|source| silica_edit::default_edit_graph(source, updated_at)))
}

pub(super) fn active_edit_state_id(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<String>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id
            FROM edit_states
            WHERE photo_id = ?1 AND active = 1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![photo_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

pub(super) fn mark_clean_sidecar_catalog_newer_after_history_commit(
    transaction: &Transaction<'_>,
    photo_id: &str,
) -> Result<(), LibraryStorageError> {
    transaction.execute(
        r#"
        UPDATE sidecar_status
        SET conflict_state = 'catalog_newer'
        WHERE photo_id = ?1
          AND conflict_state IN ('clean', 'in_sync')
        "#,
        params![photo_id],
    )?;
    Ok(())
}

/// Persist the active edit graph for a photo. Draft preview updates should not call this.
pub fn commit_edit_graph(
    library_root_path: impl AsRef<Path>,
    graph: silica_edit::EditGraph,
) -> Result<silica_edit::EditGraph, LibraryStorageError> {
    silica_edit::validate_edit_graph(&graph)?;

    let library = open_local_library(library_root_path)?;
    let before_graph =
        load_active_edit_graph_or_default(&library.root_path, &graph.source.photo_id)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let photo_id = graph.source.photo_id.clone();
    let edit_state_id = unique_catalog_id("edit-state");
    let edit_history_id = stable_catalog_id("edit-history", &edit_state_id);
    let edit_graph_json = serde_json::to_string(&graph)?;
    let label = edit_graph_history_label(before_graph.as_ref(), &graph);
    let action_json = serde_json::to_string(&serde_json::json!({
        "schema": ACTION_SCHEMA,
        "version": ACTION_VERSION,
        "class": "undoable",
        "kind": "edit_commit",
        "photo_id": photo_id.clone(),
        "label": label,
        "before": {
            "edit_graph": &before_graph,
        },
        "after": {
            "edit_graph": &graph,
        },
        "created_by": "core",
    }))?;

    let transaction = connection.transaction()?;
    invalidate_redo_history(&transaction, &photo_id)?;
    let sequence = next_history_sequence(&transaction, &photo_id)?;
    transaction.execute(
        "UPDATE edit_states SET active = 0 WHERE photo_id = ?1",
        params![photo_id],
    )?;
    transaction.execute(
        r#"
        INSERT INTO edit_states(id, photo_id, active, edit_graph_json, updated_at)
        VALUES (?1, ?2, 1, ?3, CURRENT_TIMESTAMP)
        "#,
        params![edit_state_id, photo_id, edit_graph_json],
    )?;
    transaction.execute(
        r#"
        INSERT INTO edit_history(
          id,
          photo_id,
          edit_state_id,
          action_json,
          sequence,
          action_class,
          action_kind,
          history_state
        )
        VALUES (?1, ?2, ?3, ?4, ?5, 'undoable', 'edit_commit', 'applied')
        "#,
        params![
            edit_history_id,
            photo_id,
            edit_state_id,
            action_json,
            sequence
        ],
    )?;
    transaction.execute(
        r#"
        UPDATE photo_flags
        SET edited = 1, updated_at = CURRENT_TIMESTAMP
        WHERE photo_id = ?1
        "#,
        params![graph.source.photo_id],
    )?;
    mark_clean_sidecar_catalog_newer_after_history_commit(&transaction, &photo_id)?;
    transaction.commit()?;

    Ok(graph)
}

/// Persist multiple active edit graphs as one all-or-none batch transaction.
pub fn commit_edit_graph_batch(
    library_root_path: impl AsRef<Path>,
    graphs: Vec<silica_edit::EditGraph>,
) -> Result<BatchEditGraphCommitResult, LibraryStorageError> {
    if graphs.is_empty() {
        return Err(LibraryStorageError::HistoryValidation(
            "batch edit graph commit requires at least one graph".to_string(),
        ));
    }

    let library = open_local_library(library_root_path)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let transaction = connection.transaction()?;
    let mut seen_photo_ids = Vec::new();
    let mut skipped_photo_ids = Vec::new();
    let mut prepared = Vec::new();

    for graph in graphs {
        silica_edit::validate_edit_graph(&graph)?;
        let photo_id = graph.source.photo_id.clone();
        if seen_photo_ids.iter().any(|seen| seen == &photo_id) {
            return Err(LibraryStorageError::HistoryValidation(format!(
                "duplicate batch target photo id: {photo_id}"
            )));
        }
        seen_photo_ids.push(photo_id.clone());

        let before_graph = load_active_edit_graph_or_default_from_transaction(
            &transaction,
            &photo_id,
            &current_timestamp_string(),
        )?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.clone()))?;

        validate_batch_edit_graph_identity(&before_graph, &graph)?;
        if edit_graph_content_equal_ignoring_updated_at(&before_graph, &graph) {
            skipped_photo_ids.push(photo_id);
            continue;
        }

        let label = edit_graph_history_label(Some(&before_graph), &graph).to_string();
        prepared.push(PreparedBatchEditGraphCommit {
            photo_id,
            before_graph,
            graph,
            label,
        });
    }

    if prepared.is_empty() {
        transaction.commit()?;
        return Ok(BatchEditGraphCommitResult {
            commits: Vec::new(),
            skipped_photo_ids,
        });
    }

    let mut commits = Vec::new();
    for item in prepared {
        invalidate_redo_history(&transaction, &item.photo_id)?;
        let sequence = next_history_sequence(&transaction, &item.photo_id)?;
        let edit_state_id = stable_catalog_id(
            "edit-state",
            &format!(
                "{}\nbatch_edit_commit\n{}\n{}",
                item.photo_id, sequence, item.graph.updated_at
            ),
        );
        let history_id = stable_catalog_id("edit-history", &edit_state_id);
        let edit_graph_json = serde_json::to_string(&item.graph)?;
        let action_json = serde_json::to_string(&serde_json::json!({
            "schema": ACTION_SCHEMA,
            "version": ACTION_VERSION,
            "class": "undoable",
            "kind": "edit_commit",
            "photo_id": item.photo_id.clone(),
            "label": item.label,
            "before": {
                "edit_graph": &item.before_graph,
            },
            "after": {
                "edit_graph": &item.graph,
            },
            "created_by": "core",
        }))?;

        transaction.execute(
            "UPDATE edit_states SET active = 0 WHERE photo_id = ?1",
            params![item.photo_id],
        )?;
        transaction.execute(
            r#"
            INSERT INTO edit_states(id, photo_id, active, edit_graph_json, updated_at)
            VALUES (?1, ?2, 1, ?3, CURRENT_TIMESTAMP)
            "#,
            params![edit_state_id, item.photo_id, edit_graph_json],
        )?;
        transaction.execute(
            r#"
            INSERT INTO edit_history(
              id,
              photo_id,
              edit_state_id,
              action_json,
              sequence,
              action_class,
              action_kind,
              history_state
            )
            VALUES (?1, ?2, ?3, ?4, ?5, 'undoable', 'edit_commit', 'applied')
            "#,
            params![
                history_id,
                item.photo_id,
                edit_state_id,
                action_json,
                sequence
            ],
        )?;
        transaction.execute(
            r#"
            UPDATE photo_flags
            SET edited = 1, updated_at = CURRENT_TIMESTAMP
            WHERE photo_id = ?1
            "#,
            params![item.photo_id],
        )?;
        mark_clean_sidecar_catalog_newer_after_history_commit(&transaction, &item.photo_id)?;
        commits.push(BatchEditGraphCommit {
            photo_id: item.photo_id,
            edit_state_id,
            history_id,
            sequence,
            label: item.label,
        });
    }
    transaction.commit()?;

    Ok(BatchEditGraphCommitResult {
        commits,
        skipped_photo_ids,
    })
}

fn validate_batch_edit_graph_identity(
    before_graph: &silica_edit::EditGraph,
    after_graph: &silica_edit::EditGraph,
) -> Result<(), LibraryStorageError> {
    if after_graph.source != before_graph.source {
        return Err(LibraryStorageError::HistoryValidation(format!(
            "batch edit graph source identity mismatch for photo {}",
            after_graph.source.photo_id
        )));
    }
    if after_graph.profile != before_graph.profile {
        return Err(LibraryStorageError::HistoryValidation(format!(
            "batch edit graph profile identity mismatch for photo {}",
            after_graph.source.photo_id
        )));
    }
    if after_graph.metadata != before_graph.metadata
        || after_graph.masks != before_graph.masks
        || after_graph.extensions != before_graph.extensions
    {
        return Err(LibraryStorageError::HistoryValidation(format!(
            "batch edit graph non-edit identity mismatch for photo {}",
            after_graph.source.photo_id
        )));
    }
    Ok(())
}

fn edit_graph_content_equal_ignoring_updated_at(
    before_graph: &silica_edit::EditGraph,
    after_graph: &silica_edit::EditGraph,
) -> bool {
    let mut normalized_before = before_graph.clone();
    let mut normalized_after = after_graph.clone();
    normalized_before.updated_at.clear();
    normalized_after.updated_at.clear();
    normalized_before == normalized_after
}

fn edit_graph_history_label(
    before_graph: Option<&silica_edit::EditGraph>,
    after_graph: &silica_edit::EditGraph,
) -> &'static str {
    let Some(before_graph) = before_graph else {
        return "Develop edit";
    };
    let before = &before_graph.basic;
    let after = &after_graph.basic;
    let exposure_contrast_changed =
        before.exposure != after.exposure || before.contrast != after.contrast;
    let white_balance_changed = before.white_balance != after.white_balance
        || before.temperature != after.temperature
        || before.tint != after.tint;
    let tone_recovery_changed = before.highlights != after.highlights
        || before.shadows != after.shadows
        || before.whites != after.whites
        || before.blacks != after.blacks;
    let color_presence_changed =
        before.vibrance != after.vibrance || before.saturation != after.saturation;
    let tone_curve_changed = before_graph.tone != after_graph.tone;
    let hsl_color_mixer_changed = before_graph.color.hsl != after_graph.color.hsl;
    let geometry_crop_changed = before_graph.geometry.crop != after_graph.geometry.crop;
    let geometry_orientation_changed = before_graph.geometry.rotation
        != after_graph.geometry.rotation
        || before_graph.geometry.flip_horizontal != after_graph.geometry.flip_horizontal
        || before_graph.geometry.flip_vertical != after_graph.geometry.flip_vertical;
    let geometry_transform_changed =
        before_graph.geometry.transform != after_graph.geometry.transform;
    let lens_changed = before_graph.lens != after_graph.lens;

    match (
        exposure_contrast_changed,
        white_balance_changed,
        tone_recovery_changed,
        color_presence_changed,
        tone_curve_changed,
        hsl_color_mixer_changed,
        geometry_crop_changed,
        geometry_orientation_changed,
        geometry_transform_changed,
        lens_changed,
    ) {
        (true, false, false, false, false, false, false, false, false, false) => {
            "Exposure / contrast"
        }
        (false, true, false, false, false, false, false, false, false, false) => "White balance",
        (false, false, true, false, false, false, false, false, false, false) => "Tone recovery",
        (false, false, false, true, false, false, false, false, false, false) => "Color presence",
        (false, false, false, false, true, false, false, false, false, false) => "Tone curve",
        (false, false, false, false, false, true, false, false, false, false) => "HSL color mixer",
        (false, false, false, false, false, false, true, false, false, false) => "Geometry crop",
        (false, false, false, false, false, false, false, true, false, false) => {
            "Geometry orientation"
        }
        (false, false, false, false, false, false, false, false, true, false) => {
            "Geometry transform"
        }
        (false, false, false, false, false, false, false, false, false, true) => "Lens correction",
        _ => "Develop edit",
    }
}

pub fn undo_last_history_action(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<HistoryCommandResult, LibraryStorageError> {
    apply_history_action(library_root_path, photo_id, "undo")
}

pub fn redo_last_history_action(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<HistoryCommandResult, LibraryStorageError> {
    apply_history_action(library_root_path, photo_id, "redo")
}

pub fn list_photo_history(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<PhotoHistoryPanel, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let mut statement = connection.prepare(
        r#"
        SELECT id, sequence, action_kind, action_json, history_state, created_at
        FROM edit_history
        WHERE photo_id = ?1
          AND action_class = 'undoable'
          AND history_state IN ('applied', 'undone')
        ORDER BY sequence DESC
        "#,
    )?;
    let mut items = statement
        .query_map(params![photo_id], |row| {
            let history_id: String = row.get(0)?;
            let sequence: i64 = row.get(1)?;
            let action_kind: String = row.get(2)?;
            let action_json: String = row.get(3)?;
            let history_state: String = row.get(4)?;
            let created_at: String = row.get(5)?;
            Ok((
                history_id,
                sequence,
                action_kind,
                action_json,
                history_state,
                created_at,
            ))
        })?
        .map(|row| {
            let (history_id, sequence, action_kind, action_json, history_state, created_at) = row?;
            let action: serde_json::Value = serde_json::from_str(&action_json)?;
            validate_history_action_header(&action, photo_id, &action_kind)?;
            let label = action
                .get("label")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&action_kind)
                .to_string();
            Ok(PhotoHistoryItem {
                history_id,
                photo_id: photo_id.to_string(),
                sequence,
                action_kind,
                label,
                can_undo: false,
                can_redo: false,
                history_state,
                created_at,
            })
        })
        .collect::<Result<Vec<_>, LibraryStorageError>>()?;

    let undo_sequence = items
        .iter()
        .filter(|item| item.history_state == "applied")
        .map(|item| item.sequence)
        .max();
    let redo_sequence = items
        .iter()
        .filter(|item| item.history_state == "undone")
        .map(|item| item.sequence)
        .min();
    for item in &mut items {
        item.can_undo = Some(item.sequence) == undo_sequence && item.history_state == "applied";
        item.can_redo = Some(item.sequence) == redo_sequence && item.history_state == "undone";
    }

    let can_undo = undo_sequence.is_some();
    let can_redo = redo_sequence.is_some();
    let (status, message) = if items.is_empty() {
        ("empty", "No committed history yet.")
    } else {
        ("ready", "History checkpoints loaded.")
    };

    Ok(PhotoHistoryPanel {
        photo_id: photo_id.to_string(),
        items,
        can_undo,
        can_redo,
        status: status.to_string(),
        message: message.to_string(),
    })
}

fn apply_history_action(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    command: &str,
) -> Result<HistoryCommandResult, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_local_library(library_root_path)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let transaction = connection.transaction()?;
    let row = next_history_action_for_command(&transaction, photo_id, command)?;
    let Some(row) = row else {
        transaction.commit()?;
        return Ok(HistoryCommandResult {
            photo_id: photo_id.to_string(),
            command: command.to_string(),
            applied: false,
            action_kind: None,
            history_id: None,
            message: format!("No {command} history is available."),
        });
    };

    apply_history_row(&transaction, photo_id, command, &row)?;
    let next_state = if command == "undo" {
        "undone"
    } else {
        "applied"
    };
    transaction.execute(
        "UPDATE edit_history SET history_state = ?1 WHERE id = ?2",
        params![next_state, row.id],
    )?;
    mark_clean_sidecar_catalog_newer_after_history_commit(&transaction, photo_id)?;
    transaction.commit()?;

    Ok(HistoryCommandResult {
        photo_id: photo_id.to_string(),
        command: command.to_string(),
        applied: true,
        action_kind: Some(row.action_kind),
        history_id: Some(row.id),
        message: format!("{command} applied."),
    })
}

fn next_history_action_for_command(
    transaction: &Transaction<'_>,
    photo_id: &str,
    command: &str,
) -> Result<Option<HistoryActionRow>, LibraryStorageError> {
    let sql = match command {
        "undo" => {
            r#"
            SELECT id, action_kind, action_json
            FROM edit_history
            WHERE photo_id = ?1
              AND action_class = 'undoable'
              AND history_state = 'applied'
            ORDER BY sequence DESC
            LIMIT 1
            "#
        }
        "redo" => {
            r#"
            SELECT id, action_kind, action_json
            FROM edit_history
            WHERE photo_id = ?1
              AND action_class = 'undoable'
              AND history_state = 'undone'
            ORDER BY sequence ASC
            LIMIT 1
            "#
        }
        other => {
            return Err(LibraryStorageError::HistoryValidation(format!(
                "unsupported history command: {other}"
            )));
        }
    };

    transaction
        .query_row(sql, params![photo_id], |row| {
            Ok(HistoryActionRow {
                id: row.get(0)?,
                action_kind: row.get(1)?,
                action_json: row.get(2)?,
            })
        })
        .optional()
        .map_err(LibraryStorageError::from)
}

fn apply_history_row(
    transaction: &Transaction<'_>,
    photo_id: &str,
    command: &str,
    row: &HistoryActionRow,
) -> Result<(), LibraryStorageError> {
    let action: serde_json::Value = serde_json::from_str(&row.action_json)?;
    validate_history_action_header(&action, photo_id, &row.action_kind)?;
    let snapshot_key = if command == "undo" { "before" } else { "after" };

    match row.action_kind.as_str() {
        "edit_commit" => {
            let graph_value = action
                .get(snapshot_key)
                .and_then(|snapshot| snapshot.get("edit_graph"))
                .ok_or_else(|| {
                    LibraryStorageError::HistoryValidation(format!(
                        "{snapshot_key}.edit_graph is required"
                    ))
                })?;
            let graph: silica_edit::EditGraph = serde_json::from_value(graph_value.clone())?;
            silica_edit::validate_edit_graph(&graph)?;
            restore_edit_graph_in_transaction(transaction, &graph)?;
        }
        "flag_change" => {
            let flags_value = action
                .get(snapshot_key)
                .and_then(|snapshot| snapshot.get("flags"))
                .ok_or_else(|| {
                    LibraryStorageError::HistoryValidation(format!(
                        "{snapshot_key}.flags is required"
                    ))
                })?;
            let flags = photo_flags_from_action_value(photo_id, flags_value)?;
            restore_photo_flags_in_transaction(transaction, &flags)?;
        }
        other => {
            return Err(LibraryStorageError::HistoryValidation(format!(
                "unsupported history action kind: {other}"
            )));
        }
    }

    Ok(())
}

fn validate_history_action_header(
    action: &serde_json::Value,
    photo_id: &str,
    action_kind: &str,
) -> Result<(), LibraryStorageError> {
    if action.get("schema").and_then(serde_json::Value::as_str) != Some(ACTION_SCHEMA) {
        return Err(LibraryStorageError::HistoryValidation(
            "history action schema mismatch".to_string(),
        ));
    }
    if action.get("version").and_then(serde_json::Value::as_i64) != Some(ACTION_VERSION) {
        return Err(LibraryStorageError::HistoryValidation(
            "history action version mismatch".to_string(),
        ));
    }
    if action.get("class").and_then(serde_json::Value::as_str) != Some("undoable") {
        return Err(LibraryStorageError::HistoryValidation(
            "history action must be undoable".to_string(),
        ));
    }
    if action.get("kind").and_then(serde_json::Value::as_str) != Some(action_kind) {
        return Err(LibraryStorageError::HistoryValidation(
            "history action kind mismatch".to_string(),
        ));
    }
    if action.get("photo_id").and_then(serde_json::Value::as_str) != Some(photo_id) {
        return Err(LibraryStorageError::HistoryValidation(
            "history action photo_id mismatch".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn next_history_sequence(
    transaction: &Transaction<'_>,
    photo_id: &str,
) -> Result<i64, LibraryStorageError> {
    transaction
        .query_row(
            "SELECT COALESCE(MAX(sequence), 0) + 1 FROM edit_history WHERE photo_id = ?1",
            params![photo_id],
            |row| row.get(0),
        )
        .map_err(LibraryStorageError::from)
}

pub(super) fn invalidate_redo_history(
    transaction: &Transaction<'_>,
    photo_id: &str,
) -> Result<(), LibraryStorageError> {
    transaction.execute(
        r#"
        UPDATE edit_history
        SET history_state = 'invalidated'
        WHERE photo_id = ?1 AND history_state = 'undone'
        "#,
        params![photo_id],
    )?;
    Ok(())
}

fn restore_edit_graph_in_transaction(
    transaction: &Transaction<'_>,
    graph: &silica_edit::EditGraph,
) -> Result<(), LibraryStorageError> {
    silica_edit::validate_edit_graph(graph)?;
    let photo_id = graph.source.photo_id.clone();
    let edit_state_id = unique_catalog_id("edit-state");
    let edit_graph_json = serde_json::to_string(graph)?;

    transaction.execute(
        "UPDATE edit_states SET active = 0 WHERE photo_id = ?1",
        params![photo_id],
    )?;
    transaction.execute(
        r#"
        INSERT INTO edit_states(id, photo_id, active, edit_graph_json, updated_at)
        VALUES (?1, ?2, 1, ?3, CURRENT_TIMESTAMP)
        "#,
        params![edit_state_id, photo_id, edit_graph_json],
    )?;
    transaction.execute(
        r#"
        UPDATE photo_flags
        SET edited = 1, updated_at = CURRENT_TIMESTAMP
        WHERE photo_id = ?1
        "#,
        params![graph.source.photo_id],
    )?;
    Ok(())
}

pub(super) fn restore_photo_flags_in_transaction(
    transaction: &Transaction<'_>,
    flags: &PhotoFlags,
) -> Result<(), LibraryStorageError> {
    transaction.execute(
        r#"
        INSERT INTO photo_flags(photo_id, rating, picked, rejected, color_label, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
        ON CONFLICT(photo_id) DO UPDATE SET
          rating = excluded.rating,
          picked = excluded.picked,
          rejected = excluded.rejected,
          color_label = excluded.color_label,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            flags.photo_id,
            i64::from(flags.rating),
            bool_to_sql(flags.picked),
            bool_to_sql(flags.rejected),
            flags.color_label,
        ],
    )?;
    Ok(())
}

pub(super) fn photo_flags_action_value(flags: &PhotoFlags) -> serde_json::Value {
    serde_json::json!({
        "rating": flags.rating,
        "picked": flags.picked,
        "rejected": flags.rejected,
        "color_label": flags.color_label,
    })
}

fn photo_flags_from_action_value(
    photo_id: &str,
    value: &serde_json::Value,
) -> Result<PhotoFlags, LibraryStorageError> {
    let rating = value
        .get("rating")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            LibraryStorageError::HistoryValidation("history flags.rating is required".to_string())
        })?;
    let picked = value
        .get("picked")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::HistoryValidation("history flags.picked is required".to_string())
        })?;
    let rejected = value
        .get("rejected")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::HistoryValidation("history flags.rejected is required".to_string())
        })?;
    let color_label = match value.get("color_label") {
        Some(serde_json::Value::Null) | None => None,
        Some(serde_json::Value::String(label)) => Some(label.clone()),
        _ => {
            return Err(LibraryStorageError::HistoryValidation(
                "history flags.color_label must be string or null".to_string(),
            ));
        }
    };
    PhotoFlags::new(
        photo_id.to_string(),
        rating as u8,
        picked,
        rejected,
        color_label,
    )
    .map_err(LibraryStorageError::from)
}
