use super::*;
use crate::raw::{raw_preview_artifact_cache_key, raw_preview_artifact_output_path};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exposes_crate_name() {
    assert_eq!(CRATE_NAME, "silica-core");
}

#[test]
fn extension_permission_policy_is_default_deny() {
    let policy = ExtensionPermissionPolicy::default();

    for permission in ExtensionPermission::ALL {
        assert!(
            !policy.allows(permission),
            "default policy should deny {}",
            permission.stable_id()
        );
    }
}

#[test]
fn extension_permissions_cover_required_safe_categories() {
    let categories: BTreeSet<ExtensionPermissionCategory> = ExtensionPermission::ALL
        .iter()
        .map(|permission| permission.category())
        .collect();

    assert!(categories.contains(&ExtensionPermissionCategory::Metadata));
    assert!(categories.contains(&ExtensionPermissionCategory::EditSuggestion));
    assert!(categories.contains(&ExtensionPermissionCategory::Export));
    assert!(categories.contains(&ExtensionPermissionCategory::Filesystem));
    assert!(categories.contains(&ExtensionPermissionCategory::AiResult));
    assert!(categories.contains(&ExtensionPermissionCategory::McpMode));
}

#[test]
fn extension_permissions_do_not_include_raw_sql_or_original_mutation() {
    for permission in ExtensionPermission::ALL {
        assert_ne!(permission.stable_id(), "raw_sql");
        assert_ne!(permission.stable_id(), "database:raw_sql");
        assert!(
            !permission.allows_original_mutation(),
            "{} must not allow original mutation",
            permission.stable_id()
        );
    }
}

#[test]
fn extension_permission_policy_only_allows_explicit_grants() {
    let policy = ExtensionPermissionPolicy::default()
        .with_grant(ExtensionPermission::MetadataRead)
        .with_grant(ExtensionPermission::McpReadOnly);

    assert!(policy.allows(ExtensionPermission::MetadataRead));
    assert!(policy.allows(ExtensionPermission::McpReadOnly));
    assert!(!policy.allows(ExtensionPermission::MetadataWrite));
    assert!(!policy.allows(ExtensionPermission::McpEdit));
}

#[test]
fn product_raw_decode_plan_wraps_decode_contract_without_side_effects() {
    let plan = plan_product_raw_decode("/tmp/sample.dng");

    assert_eq!(plan.source_path, "/tmp/sample.dng");
    assert_eq!(
        plan.status,
        silica_decode::ProductRawDecodeStatus::BlockedPendingEvidence
    );
    assert_ne!(
        plan.status,
        silica_decode::ProductRawDecodeStatus::Supported
    );
}

#[test]
fn product_raw_decode_probe_plan_wraps_supported_fixture_evidence() {
    let probe = silica_decode::RawProbeResult {
        backend: silica_decode::RawProbeBackend::CoreImageRaw,
        platform: silica_decode::RawProbePlatform::Macos,
        macos_version: Some("26.4".to_string()),
        source_path: "/tmp/sample.cr2".to_string(),
        source_sha256: Some("fixture-hash".to_string()),
        original_file_size: Some(1024),
        original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
        status: silica_decode::RawProbeStatus::Success,
        width: Some(5184),
        height: Some(3456),
        orientation: None,
        error_category: None,
        message: "Core Image opened the RAW source.".to_string(),
    };

    let plan = plan_product_raw_decode_from_probe("A", &probe);

    assert_eq!(plan.source_path, "/tmp/sample.cr2");
    assert_eq!(
        plan.status,
        silica_decode::ProductRawDecodeStatus::Supported
    );
    assert_eq!(plan.width, Some(5184));
    assert_eq!(plan.height, Some(3456));
}

#[test]
fn decoded_image_viewer_handoff_wraps_decode_and_render_without_state_writes() {
    let probe = silica_decode::RawProbeResult {
        backend: silica_decode::RawProbeBackend::CoreImageRaw,
        platform: silica_decode::RawProbePlatform::Macos,
        macos_version: Some("26.4".to_string()),
        source_path: "/tmp/sample.cr2".to_string(),
        source_sha256: Some("fixture-hash".to_string()),
        original_file_size: Some(1024),
        original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
        status: silica_decode::RawProbeStatus::Success,
        width: Some(5184),
        height: Some(3456),
        orientation: None,
        error_category: None,
        message: "Core Image opened the RAW source.".to_string(),
    };

    let plan = plan_decoded_image_viewer_handoff("A", &probe, "previews/raw/photo-1");

    assert_eq!(
        plan.decoded.status,
        silica_decode::DecodedImageHandoffStatus::Ready
    );
    assert!(!plan.writes_catalog_state());
    assert!(!plan.writes_sidecars());
    assert!(!plan.writes_originals());
    assert!(!plan.writes_exports());
    match plan.viewer_input {
        silica_render::ViewerPreviewInput::DecodedImageArtifact { cache_key, .. } => {
            assert_eq!(cache_key, "previews/raw/photo-1");
        }
        other => panic!("expected decoded image artifact input, got {other:?}"),
    }
}

#[test]
fn edit_clipboard_contract_through_core_preserves_target_identity_without_catalog_write() {
    let mut source = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: "source-photo".to_string(),
            path: "/tmp/source.raw".to_string(),
            file_size: 2048,
            modified_at: Some("unix:11".to_string()),
            partial_hash: Some("source-partial".to_string()),
            full_hash: Some("source-full".to_string()),
        },
        "unix:12",
    );
    source.profile.input_profile = "source-profile".to_string();
    source.metadata.rating = 5;
    source.extensions.insert(
        "com.example.source".to_string(),
        serde_json::json!({ "owned_by": "source" }),
    );
    let source = silica_edit::apply_exposure_contrast(&source, 0.8, 12.0, "unix:13")
        .expect("source basic edit");
    let source = silica_edit::apply_geometry_orientation(&source, 90.0, true, false, "unix:14")
        .expect("source geometry edit");

    let mut target = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: "target-photo".to_string(),
            path: "/tmp/target.raw".to_string(),
            file_size: 4096,
            modified_at: Some("unix:21".to_string()),
            partial_hash: Some("target-partial".to_string()),
            full_hash: Some("target-full".to_string()),
        },
        "unix:22",
    );
    target.profile.input_profile = "target-profile".to_string();
    target.metadata.rejected = true;
    target.extensions.insert(
        "com.example.target".to_string(),
        serde_json::json!({ "owned_by": "target" }),
    );

    let payload = copy_edit_clipboard_payload(
        &source,
        silica_edit::EditClipboardSelection {
            basic: true,
            geometry: true,
            ..Default::default()
        },
    )
    .expect("copy clipboard through core");
    let pasted = apply_edit_clipboard_payload_to_graph(&target, &payload, "unix:30")
        .expect("paste clipboard through core");

    assert_eq!(pasted.source, target.source);
    assert_eq!(pasted.profile, target.profile);
    assert_eq!(pasted.metadata, target.metadata);
    assert_eq!(pasted.extensions, target.extensions);
    assert_eq!(pasted.masks, target.masks);
    assert_eq!(pasted.basic, source.basic);
    assert_eq!(pasted.geometry, source.geometry);
    assert_eq!(pasted.tone, target.tone);
    assert_eq!(pasted.color, target.color);
    assert_eq!(pasted.detail, target.detail);
    assert_eq!(pasted.lens, target.lens);
}

#[test]
fn copies_photo_edit_clipboard_payload_from_catalog_state() {
    let workspace = unique_library_root("core-copy-photo-edit-clipboard");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import root");
    write_source_jpeg(&supported_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");
    let photo = list_library_photos(&created.root_path)
        .expect("list photos")
        .into_iter()
        .find(|photo| photo.file_name == "sample.jpg")
        .expect("sample photo");
    commit_exposure_contrast_edit(&created.root_path, &photo.photo_id, 0.65, 9.0)
        .expect("commit source edit")
        .expect("committed edit");

    let payload = copy_photo_edit_clipboard_payload(
        &created.root_path,
        &photo.photo_id,
        silica_edit::EditClipboardSelection {
            basic: true,
            ..Default::default()
        },
    )
    .expect("copy catalog clipboard payload")
    .expect("payload");

    assert_eq!(payload.schema, silica_edit::EDIT_CLIPBOARD_SCHEMA);
    assert!(payload.basic.is_some());
    assert_eq!(
        payload
            .basic
            .as_ref()
            .and_then(|basic| basic.exposure.as_f64()),
        Some(0.65)
    );
    assert_eq!(
        payload
            .basic
            .as_ref()
            .and_then(|basic| basic.contrast.as_f64()),
        Some(9.0)
    );
    assert!(payload.tone.is_none());
    assert!(payload.color.is_none());
    assert!(payload.detail.is_none());
    assert!(payload.lens.is_none());
    assert!(payload.geometry.is_none());

    let missing = copy_photo_edit_clipboard_payload(
        &created.root_path,
        "missing-photo",
        silica_edit::EditClipboardSelection {
            basic: true,
            ..Default::default()
        },
    )
    .expect("missing photo is not an error");
    assert!(missing.is_none());

    remove_library_root(&workspace);
}

#[test]
fn batch_sync_edit_clipboard_applies_payload_with_per_photo_history() {
    let workspace = unique_library_root("core-batch-sync");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let first_file = import_root.join("first.jpg");
    let second_file = import_root.join("second.jpg");

    std::fs::create_dir_all(&import_root).expect("create import root");
    write_source_jpeg(&first_file);
    write_source_jpeg(&second_file);
    let first_hash = file_hash(&first_file);
    let second_hash = file_hash(&second_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");
    let photos = list_library_photos(&created.root_path).expect("list photos");
    assert_eq!(photos.len(), 2);
    let target_ids: Vec<String> = photos.iter().map(|photo| photo.photo_id.clone()).collect();

    let source = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: "source-photo".to_string(),
            path: "/tmp/source.jpg".to_string(),
            file_size: 256,
            modified_at: Some("unix:source".to_string()),
            partial_hash: Some("source-partial".to_string()),
            full_hash: None,
        },
        "unix:10",
    );
    let source = silica_edit::apply_exposure_contrast(&source, 0.9, 16.0, "unix:11")
        .expect("source basic edit");
    let payload = copy_edit_clipboard_payload(
        &source,
        silica_edit::EditClipboardSelection {
            basic: true,
            ..Default::default()
        },
    )
    .expect("copy clipboard payload");

    let plan = plan_edit_clipboard_sync(&created.root_path, &target_ids, &payload)
        .expect("plan batch sync payload");
    assert_eq!(plan.status, "ready");
    assert_eq!(plan.ready_count, 2);
    assert_eq!(plan.unchanged_count, 0);
    assert_eq!(plan.blocked_count, 0);

    let result = apply_edit_clipboard_sync(&created.root_path, &target_ids, &payload)
        .expect("batch sync payload");

    assert_eq!(result.status, "applied");
    assert_eq!(result.requested_count, 2);
    assert_eq!(result.applied_count, 2);
    assert_eq!(result.failed_count, 0);
    assert_eq!(result.blocked_count, 0);
    assert_eq!(result.commits.len(), 2);
    assert!(result.failures.is_empty());

    for photo in &photos {
        let graph = silica_storage::load_active_edit_graph(&created.root_path, &photo.photo_id)
            .expect("load active graph")
            .expect("active graph");
        assert_eq!(graph.source.photo_id, photo.photo_id);
        assert_eq!(graph.source.path, photo.path);
        assert_eq!(
            graph.profile.input_profile,
            silica_edit::INPUT_PROFILE_UNKNOWN
        );
        assert_eq!(graph.basic.exposure.as_f64(), Some(0.9));
        assert_eq!(graph.basic.contrast.as_f64(), Some(16.0));

        let history =
            list_photo_history(&created.root_path, &photo.photo_id).expect("read history");
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].action_kind, "edit_commit");
        assert_eq!(history.items[0].sequence, 1);
    }

    assert_original_hash(&first_file, &first_hash, "batch sync first original");
    assert_original_hash(&second_file, &second_hash, "batch sync second original");
    remove_library_root(&workspace);
}

#[test]
fn batch_sync_edit_clipboard_preflight_failure_writes_no_history() {
    let workspace = unique_library_root("core-batch-sync-preflight");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import root");
    write_source_jpeg(&supported_file);
    let original_hash = file_hash(&supported_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");
    let photo = list_library_photos(&created.root_path)
        .expect("list photos")
        .into_iter()
        .find(|photo| photo.file_name == "sample.jpg")
        .expect("sample photo");

    let source = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: "source-photo".to_string(),
            path: "/tmp/source.jpg".to_string(),
            file_size: 256,
            modified_at: Some("unix:source".to_string()),
            partial_hash: Some("source-partial".to_string()),
            full_hash: None,
        },
        "unix:10",
    );
    let source = silica_edit::apply_exposure_contrast(&source, 0.4, 8.0, "unix:11")
        .expect("source basic edit");
    let payload = copy_edit_clipboard_payload(
        &source,
        silica_edit::EditClipboardSelection {
            basic: true,
            ..Default::default()
        },
    )
    .expect("copy clipboard payload");

    let plan = plan_edit_clipboard_sync(
        &created.root_path,
        &[photo.photo_id.clone(), "missing-photo".to_string()],
        &payload,
    )
    .expect("plan preflight failure");

    assert_eq!(plan.status, "blocked");
    assert_eq!(plan.requested_count, 2);
    assert_eq!(plan.ready_count, 1);
    assert_eq!(plan.blocked_count, 1);
    assert_eq!(plan.targets[1].photo_id, "missing-photo");
    assert_eq!(plan.targets[1].code.as_deref(), Some("missing_photo"));

    let result = apply_edit_clipboard_sync(
        &created.root_path,
        &[photo.photo_id.clone(), "missing-photo".to_string()],
        &payload,
    )
    .expect("preflight failure returns result");

    assert_eq!(result.status, "blocked");
    assert_eq!(result.requested_count, 2);
    assert_eq!(result.applied_count, 0);
    assert_eq!(result.blocked_count, 1);
    assert_eq!(result.failed_count, 1);
    assert_eq!(result.failures[0].photo_id, "missing-photo");
    assert_eq!(result.targets[1].code.as_deref(), Some("missing_photo"));
    assert!(result.failures[0].message.contains("not found"));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo.photo_id)
            .expect("load active graph")
            .is_none(),
        "failed batch must not write active edit graph"
    );
    let history = list_photo_history(&created.root_path, &photo.photo_id)
        .expect("read history after failed batch");
    assert!(history.items.is_empty());
    assert_original_hash(
        &supported_file,
        &original_hash,
        "failed batch sync original",
    );
    remove_library_root(&workspace);
}

#[test]
fn batch_sync_edit_clipboard_blocks_unsupported_detail_without_writes() {
    let workspace = unique_library_root("core-batch-sync-detail-blocked");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import root");
    write_source_jpeg(&supported_file);
    let original_hash = file_hash(&supported_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");
    let photo = list_library_photos(&created.root_path)
        .expect("list photos")
        .into_iter()
        .find(|photo| photo.file_name == "sample.jpg")
        .expect("sample photo");

    let source = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: "source-photo".to_string(),
            path: "/tmp/source.jpg".to_string(),
            file_size: 256,
            modified_at: Some("unix:source".to_string()),
            partial_hash: Some("source-partial".to_string()),
            full_hash: None,
        },
        "unix:10",
    );
    let source = silica_edit::apply_detail_sharpening(&source, 40.0, 1.2, 35.0, 10.0, "unix:11")
        .expect("source detail edit");
    let payload = copy_edit_clipboard_payload(
        &source,
        silica_edit::EditClipboardSelection {
            detail: true,
            ..Default::default()
        },
    )
    .expect("copy detail clipboard payload");

    let result = apply_edit_clipboard_sync(&created.root_path, &[photo.photo_id.clone()], &payload)
        .expect("unsupported detail returns blocked result");

    assert_eq!(result.status, "blocked");
    assert_eq!(result.applied_count, 0);
    assert_eq!(result.blocked_count, 1);
    assert_eq!(
        result.targets[0].code.as_deref(),
        Some("unsupported_detail")
    );
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo.photo_id)
            .expect("load active graph")
            .is_none(),
        "blocked detail batch must not write active edit graph"
    );
    let history = list_photo_history(&created.root_path, &photo.photo_id)
        .expect("read history after blocked detail batch");
    assert!(history.items.is_empty());
    assert_original_hash(
        &supported_file,
        &original_hash,
        "blocked detail batch original",
    );
    remove_library_root(&workspace);
}

#[test]
fn edit_clipboard_blocks_unsupported_source_copy_and_batch_target_without_writes() {
    let workspace = unique_library_root("core-edit-clipboard-raw-blocked");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let source_file = import_root.join("source.jpg");
    let raw_file = import_root.join("target.DNG");

    std::fs::create_dir_all(&import_root).expect("create import root");
    write_source_jpeg(&source_file);
    std::fs::write(&raw_file, b"raw target placeholder").expect("write raw target");
    let raw_hash = file_hash(&raw_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");
    let photos = list_library_photos(&created.root_path).expect("list photos");
    let source_photo = photos
        .iter()
        .find(|photo| photo.file_name == "source.jpg")
        .expect("source photo");
    let raw_photo = photos
        .iter()
        .find(|photo| photo.file_name == "target.DNG")
        .expect("raw target photo");
    assert_eq!(raw_photo.file_type, "DNG");
    assert!(raw_photo.unsupported);

    let raw_copy = copy_photo_edit_clipboard_payload(
        &created.root_path,
        &raw_photo.photo_id,
        silica_edit::EditClipboardSelection {
            basic: true,
            ..Default::default()
        },
    )
    .expect_err("RAW copy must be blocked");
    assert!(matches!(raw_copy, CoreError::UnsupportedEdit(_)));
    assert!(raw_copy.to_string().contains("supported raster"));

    let unsupported_commit =
        commit_exposure_contrast_edit(&created.root_path, &raw_photo.photo_id, 0.2, 4.0)
            .expect_err("unsupported source Develop commit must be blocked");
    assert!(matches!(unsupported_commit, CoreError::UnsupportedEdit(_)));
    assert!(unsupported_commit.to_string().contains("supported raster"));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &raw_photo.photo_id)
            .expect("load unsupported source active graph")
            .is_none(),
        "unsupported source Develop commit must not write active edit graph"
    );

    commit_exposure_contrast_edit(&created.root_path, &source_photo.photo_id, 0.8, 12.0)
        .expect("commit source edit");
    let payload = copy_photo_edit_clipboard_payload(
        &created.root_path,
        &source_photo.photo_id,
        silica_edit::EditClipboardSelection {
            basic: true,
            ..Default::default()
        },
    )
    .expect("copy source payload")
    .expect("source payload");

    let plan = plan_edit_clipboard_sync(
        &created.root_path,
        std::slice::from_ref(&raw_photo.photo_id),
        &payload,
    )
    .expect("plan unsupported target");
    assert_eq!(plan.status, "blocked");
    assert_eq!(plan.ready_count, 0);
    assert_eq!(plan.blocked_count, 1);
    assert_eq!(plan.targets[0].code.as_deref(), Some("unsupported_target"));
    assert!(plan.targets[0].message.contains("supported raster"));

    let result = apply_edit_clipboard_sync(
        &created.root_path,
        std::slice::from_ref(&raw_photo.photo_id),
        &payload,
    )
    .expect("unsupported target returns blocked result");
    assert_eq!(result.status, "blocked");
    assert_eq!(result.applied_count, 0);
    assert_eq!(result.blocked_count, 1);
    assert_eq!(
        result.targets[0].code.as_deref(),
        Some("unsupported_target")
    );
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &raw_photo.photo_id)
            .expect("load raw active graph")
            .is_none(),
        "RAW target batch must not write active edit graph"
    );
    let history = list_photo_history(&created.root_path, &raw_photo.photo_id)
        .expect("read raw history after blocked batch");
    assert!(history.items.is_empty());
    assert_original_hash(&raw_file, &raw_hash, "blocked raw batch target");
    remove_library_root(&workspace);
}

#[test]
fn raw_preview_artifact_path_stays_under_library_previews() {
    let library_root = PathBuf::from("/tmp/SilicaRAW Library");
    let output_path = raw_preview_artifact_output_path(&library_root, "photo-1");

    assert_eq!(
        output_path,
        library_root.join("previews").join("raw-photo-1.jpg")
    );
    assert!(output_path.starts_with(library_root.join("previews")));
}

#[test]
fn raw_preview_artifact_cache_key_uses_source_hash_and_decode_settings() {
    let probe = silica_decode::RawProbeResult {
        backend: silica_decode::RawProbeBackend::CoreImageRaw,
        platform: silica_decode::RawProbePlatform::Macos,
        macos_version: Some("26.4".to_string()),
        source_path: "/tmp/sample.cr2".to_string(),
        source_sha256: Some("fixture-hash".to_string()),
        original_file_size: Some(1024),
        original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
        status: silica_decode::RawProbeStatus::Success,
        width: Some(5184),
        height: Some(3456),
        orientation: None,
        error_category: None,
        message: "Core Image opened the RAW source.".to_string(),
    };

    let cache_key = raw_preview_artifact_cache_key("photo-1", &probe);

    assert!(cache_key.contains("raw-preview:v1:photo-1"));
    assert!(cache_key.contains("fixture-hash"));
    assert!(cache_key.contains("core-image-raw"));
    assert!(cache_key.contains("2048"));
}

#[test]
fn raw_preview_artifact_wrapper_keeps_blocked_classes_reviewable_without_cache_write() {
    let workspace = unique_library_root("raw-preview-wrapper-blocked");
    let library_root = workspace.join("SilicaRAW Library");
    let created = create_library(&library_root).expect("create library through core");
    let source_path = workspace.join("sample.cr2");
    std::fs::write(&source_path, b"raw placeholder").expect("write raw placeholder");
    let probe = silica_decode::RawProbeResult {
        backend: silica_decode::RawProbeBackend::CoreImageRaw,
        platform: silica_decode::RawProbePlatform::Macos,
        macos_version: Some("26.4".to_string()),
        source_path: source_path.display().to_string(),
        source_sha256: Some("fixture-hash".to_string()),
        original_file_size: Some(1024),
        original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
        status: silica_decode::RawProbeStatus::Success,
        width: Some(1200),
        height: Some(800),
        orientation: None,
        error_category: None,
        message: "Core Image opened the RAW source.".to_string(),
    };

    let session = write_raw_preview_artifact_for_probe(&created.root_path, "photo-1", "E", &probe)
        .expect("blocked class remains reviewable");

    assert_eq!(
        session.handoff.decoded.status,
        silica_decode::DecodedImageHandoffStatus::BlockedPendingEvidence
    );
    assert_eq!(session.artifact_path, None);
    assert_eq!(session.cache_record, None);
    assert!(session
        .output_path
        .starts_with(created.root_path.join("previews")));
    assert!(!session.output_path.exists());

    remove_library_root(&workspace);
}

#[test]
fn metal_draft_preview_request_validates_exposure_contrast_without_state_writes() {
    let viewer_input = silica_render::ViewerPreviewInput::DecodedImageArtifact {
        cache_key: "raw-preview:v1:photo-1".to_string(),
        source_sha256: Some("fixture-hash".to_string()),
        width_px: 2048,
        height_px: 1365,
        pixel_format: silica_render::ViewerPreviewPixelFormat::JpegSrgb8,
        decoder_backend: "core_image_raw".to_string(),
        input_profile: "core_image_raw".to_string(),
        working_space: "srgb".to_string(),
    };

    let request = plan_exposure_contrast_metal_draft(
        "photo-1",
        "/tmp/sample.cr2",
        viewer_input,
        silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
        silica_render::ViewerPreviewRenderRequestId(51),
        3,
        0.5,
        -8.0,
    )
    .expect("valid metal draft request");

    assert_eq!(request.photo_id, "photo-1");
    assert_eq!(
        request.exposure_contrast_draft,
        Some(silica_render::ViewerExposureContrastDraft {
            exposure: 0.5,
            contrast: -8.0
        })
    );
    assert!(!request.writes_catalog_state());
    assert!(!request.contains_image_pixels());
}

#[test]
fn metal_draft_preview_request_rejects_invalid_edit_values() {
    let viewer_input = silica_render::ViewerPreviewInput::DecodedImageArtifact {
        cache_key: "raw-preview:v1:photo-1".to_string(),
        source_sha256: Some("fixture-hash".to_string()),
        width_px: 2048,
        height_px: 1365,
        pixel_format: silica_render::ViewerPreviewPixelFormat::JpegSrgb8,
        decoder_backend: "core_image_raw".to_string(),
        input_profile: "core_image_raw".to_string(),
        working_space: "srgb".to_string(),
    };

    let error = plan_exposure_contrast_metal_draft(
        "photo-1",
        "/tmp/sample.cr2",
        viewer_input,
        silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
        silica_render::ViewerPreviewRenderRequestId(52),
        3,
        8.0,
        0.0,
    )
    .expect_err("invalid exposure must fail edit graph validation");

    assert!(matches!(error, CoreError::EditGraph(_)));
}

#[test]
fn app_session_missing_file_returns_safe_defaults() {
    let workspace = unique_library_root("app-session-missing");
    let session_path = workspace
        .join("Application Support")
        .join("dev.silicaraw.desktop")
        .join("app-session.json");

    let loaded = load_app_session(&session_path).expect("load missing session");

    assert_eq!(loaded.session.schema, APP_SESSION_SCHEMA);
    assert_eq!(loaded.session.version, APP_SESSION_VERSION);
    assert_eq!(loaded.session.last_mode, AppSessionMode::Library);
    assert!(loaded.session.last_library_root_path.is_none());
    assert!(loaded.session.recents.is_empty());
    assert!(loaded.session.per_library.is_empty());
    assert_eq!(
        loaded.session.layout.thumbnail_size,
        DEFAULT_APP_SESSION_THUMBNAIL_SIZE
    );
    assert_eq!(loaded.warnings, vec![AppSessionWarning::Missing]);
    assert!(!session_path.exists());
    assert!(!workspace.join("catalog.db").exists());
    assert!(!workspace.join("sidecars").exists());

    remove_library_root(&workspace);
}

#[test]
fn app_session_round_trips_typed_state_with_atomic_write() {
    let workspace = unique_library_root("app-session-roundtrip");
    let session_path = workspace
        .join("Application Support")
        .join("dev.silicaraw.desktop")
        .join("app-session.json");
    let library_root = workspace.join("SilicaRAW Library");

    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_root.clone());
    session.last_mode = AppSessionMode::Develop;
    session.recents.push(AppRecentLibrary {
        root_path: library_root.clone(),
        display_name: "SilicaRAW Library".to_string(),
        last_opened_at: "unix:42".to_string(),
    });
    session.per_library.insert(
        library_root.display().to_string(),
        AppPerLibrarySession {
            selected_photo_id: Some("photo-1".to_string()),
            last_mode: AppSessionMode::Develop,
            last_opened_at: "unix:42".to_string(),
        },
    );

    let written = write_app_session(&session_path, &session).expect("write app session");
    assert_eq!(written.session_path, session_path);
    assert!(written.bytes_written > 0);
    assert!(session_path.is_file());
    assert!(!session_path.with_extension("tmp").exists());

    let loaded = load_app_session(&session_path).expect("load written session");
    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.session, session);

    let raw = std::fs::read_to_string(&session_path).expect("read app session json");
    assert!(raw.contains("\"schema\": \"silica.desktop_session\""));
    assert!(raw.contains("\"last_mode\": \"develop\""));

    remove_library_root(&workspace);
}

#[test]
fn app_session_corrupt_or_newer_files_return_defaults_with_warnings() {
    let workspace = unique_library_root("app-session-invalid");
    let session_path = workspace.join("app-session.json");
    std::fs::create_dir_all(&workspace).expect("create session workspace");

    std::fs::write(&session_path, b"{not json").expect("write corrupt session");
    let corrupt = load_app_session(&session_path).expect("load corrupt session");
    assert_eq!(corrupt.session, AppSession::default());
    assert_eq!(corrupt.warnings, vec![AppSessionWarning::Corrupt]);

    std::fs::write(
        &session_path,
        r#"{"schema":"silica.desktop_session","version":999,"recents":[],"per_library":{}}"#,
    )
    .expect("write newer session");
    let newer = load_app_session(&session_path).expect("load newer session");
    assert_eq!(newer.session, AppSession::default());
    assert_eq!(newer.warnings, vec![AppSessionWarning::UnsupportedVersion]);

    remove_library_root(&workspace);
}

#[test]
fn app_session_invalid_values_are_clamped_to_safe_defaults() {
    let workspace = unique_library_root("app-session-clamp");
    let session_path = workspace.join("app-session.json");
    std::fs::create_dir_all(&workspace).expect("create session workspace");
    std::fs::write(
        &session_path,
        r#"{
              "schema": "silica.desktop_session",
              "version": 1,
              "last_library_root_path": "/tmp/SilicaRAW Library",
              "last_mode": "unknown-mode",
              "recents": [],
              "appearance": {
                "theme": "neon",
                "density": "wide",
                "ui_scale": 1000
              },
              "layout": {
                "sidebar_collapsed": true,
                "inspector_collapsed": true,
                "filmstrip_visible": false,
                "thumbnail_size": 9999,
                "sort": "unknown-sort",
                "filters": {
                  "min_rating": 99,
                  "picked": true,
                  "rejected": false,
                  "file_type": "unsupported",
                  "metadata": "not-indexed",
                  "search": 123
                }
              },
              "per_library": {
                "/tmp/SilicaRAW Library": {
                  "selected_photo_id": "photo-2",
                  "last_mode": "not-real",
                  "last_opened_at": "unix:44"
                }
              }
            }"#,
    )
    .expect("write invalid value session");

    let loaded = load_app_session(&session_path).expect("load invalid value session");

    assert_eq!(loaded.session.last_mode, AppSessionMode::Library);
    assert_eq!(loaded.session.appearance.theme, AppAppearanceTheme::Dark);
    assert_eq!(
        loaded.session.appearance.density,
        AppAppearanceDensity::Compact
    );
    assert_eq!(loaded.session.appearance.ui_scale, MAX_APP_SESSION_UI_SCALE);
    assert_eq!(
        loaded.session.layout.thumbnail_size,
        MAX_APP_SESSION_THUMBNAIL_SIZE
    );
    assert_eq!(loaded.session.layout.sort, AppLibrarySort::ImportedAtDesc);
    assert_eq!(loaded.session.layout.filters.min_rating, Some(5));
    assert_eq!(loaded.session.layout.filters.search, "");
    let per_library = loaded
        .session
        .per_library
        .get("/tmp/SilicaRAW Library")
        .expect("per-library state");
    assert_eq!(per_library.last_mode, AppSessionMode::Library);
    assert_eq!(per_library.selected_photo_id.as_deref(), Some("photo-2"));
    assert_eq!(loaded.warnings, vec![AppSessionWarning::InvalidValues]);

    remove_library_root(&workspace);
}

#[test]
fn layout_preferences_defaults_and_reset_are_stable() {
    let workspace = unique_library_root("layout-preferences-reset");
    let session_path = workspace.join("app-session.json");
    let library_root = workspace.join("SilicaRAW Library");
    let defaults = default_app_layout_preferences();

    assert!(!defaults.sidebar_collapsed);
    assert!(!defaults.inspector_collapsed);
    assert!(defaults.filmstrip_visible);
    assert_eq!(defaults.thumbnail_size, DEFAULT_APP_SESSION_THUMBNAIL_SIZE);
    assert_eq!(defaults.sort, AppLibrarySort::ImportedAtDesc);
    assert_eq!(defaults.filters, AppSessionFilters::default());

    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_root.clone());
    write_app_session(&session_path, &session).expect("write app session");

    let mut changed_layout = default_app_layout_preferences();
    changed_layout.sidebar_collapsed = true;
    changed_layout.inspector_collapsed = true;
    changed_layout.filmstrip_visible = false;
    changed_layout.thumbnail_size = MAX_APP_SESSION_THUMBNAIL_SIZE;
    changed_layout.sort = AppLibrarySort::RatingDesc;
    changed_layout.filters.min_rating = Some(4);
    changed_layout.filters.metadata = Some(AppMetadataFilter::HasDimensions);
    changed_layout.filters.search = "portrait".to_string();
    let recorded =
        record_app_session_layout(&session_path, changed_layout.clone()).expect("record layout");
    assert_eq!(recorded.session.layout, changed_layout);

    let reset = reset_app_session_layout(&session_path).expect("reset layout");

    assert!(reset.warnings.is_empty());
    assert_eq!(reset.session.layout, defaults);
    assert_eq!(
        reset.session.last_library_root_path.as_deref(),
        Some(library_root.as_path())
    );
    let loaded = load_app_session(&session_path).expect("reload reset layout");
    assert_eq!(loaded.session.layout, defaults);

    remove_library_root(&workspace);
}

#[test]
fn appearance_preferences_defaults_and_reset_are_stable() {
    let workspace = unique_library_root("appearance-preferences-reset");
    let session_path = workspace.join("app-session.json");
    let library_root = workspace.join("SilicaRAW Library");
    let defaults = default_app_appearance_preferences();

    assert_eq!(defaults.theme, AppAppearanceTheme::Dark);
    assert_eq!(defaults.density, AppAppearanceDensity::Compact);
    assert_eq!(defaults.ui_scale, DEFAULT_APP_SESSION_UI_SCALE);

    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_root.clone());
    write_app_session(&session_path, &session).expect("write app session");

    let changed = AppAppearancePreferences {
        theme: AppAppearanceTheme::Light,
        density: AppAppearanceDensity::Comfortable,
        ui_scale: MAX_APP_SESSION_UI_SCALE,
    };
    let recorded =
        record_app_session_appearance(&session_path, changed.clone()).expect("record appearance");
    assert_eq!(recorded.session.appearance, changed);
    assert_eq!(
        recorded.session.last_library_root_path.as_deref(),
        Some(library_root.as_path())
    );

    let reset = reset_app_session_appearance(&session_path).expect("reset appearance");

    assert!(reset.warnings.is_empty());
    assert_eq!(reset.session.appearance, defaults);
    assert_eq!(
        reset.session.last_library_root_path.as_deref(),
        Some(library_root.as_path())
    );
    let loaded = load_app_session(&session_path).expect("reload reset appearance");
    assert_eq!(loaded.session.appearance, defaults);

    remove_library_root(&workspace);
}

#[test]
fn library_cache_status_and_preferences_are_scoped() {
    let workspace = unique_library_root("library-cache-preferences");
    let session_path = workspace.join("app-session.json");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let defaults = default_app_library_preferences();

    assert_eq!(defaults.default_library_root_path, None);

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&supported_file);
    create_library(&library_root).expect("create library");
    import_folder(&library_root, &import_root).expect("import folder");
    std::fs::write(
        library_root.join("thumbnails").join("status-thumb.cache"),
        b"cache bytes",
    )
    .expect("write cache bytes");
    std::fs::create_dir_all(library_root.join("exports")).expect("create exports");
    std::fs::write(library_root.join("exports").join("keep.jpg"), b"not cache")
        .expect("write export");

    let status = get_library_cache_status(&library_root).expect("cache status");

    assert_eq!(status.total_bytes, 11);
    assert_eq!(
        status
            .directories
            .iter()
            .map(|directory| directory.name.as_str())
            .collect::<Vec<_>>(),
        silica_storage::DISPOSABLE_CACHE_DIRECTORIES
    );
    assert!(status
        .directories
        .iter()
        .all(|directory| directory.path.starts_with(&library_root)));
    assert!(library_root.join("exports").join("keep.jpg").is_file());

    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_root.clone());
    write_app_session(&session_path, &session).expect("write app session");
    let changed = AppLibraryPreferences {
        default_library_root_path: Some(library_root.clone()),
    };
    let recorded = record_app_session_library_preferences(&session_path, changed.clone())
        .expect("record library preferences");
    assert_eq!(recorded.session.library, changed);
    assert_eq!(
        recorded.session.last_library_root_path.as_deref(),
        Some(library_root.as_path())
    );

    let reset = reset_app_session_library_preferences(&session_path).expect("reset library prefs");
    assert_eq!(reset.session.library, defaults);
    assert_eq!(
        reset.session.last_library_root_path.as_deref(),
        Some(library_root.as_path())
    );

    remove_library_root(&workspace);
}

#[test]
fn app_session_records_recents_with_dedupe_and_cap() {
    let workspace = unique_library_root("app-session-recents");
    let session_path = workspace.join("app-session.json");

    for index in 0..12 {
        let root_path = workspace.join(format!("Library {index}"));
        let session = LibrarySession {
            root_path: root_path.clone(),
            catalog_path: root_path.join("catalog.db"),
            schema_version: 1,
        };
        record_app_session_recent_library(&session_path, &session).expect("record app recent");
    }

    let loaded = load_app_session(&session_path).expect("load recents");
    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.session.recents.len(), APP_SESSION_RECENTS_LIMIT);
    assert_eq!(
        loaded.session.last_library_root_path.as_deref(),
        Some(workspace.join("Library 11").as_path())
    );
    assert_eq!(
        loaded
            .session
            .recents
            .first()
            .map(|recent| recent.root_path.as_path()),
        Some(workspace.join("Library 11").as_path())
    );
    assert!(!loaded
        .session
        .recents
        .iter()
        .any(|recent| recent.root_path == workspace.join("Library 0")));

    let repeated = LibrarySession {
        root_path: workspace.join("Library 5"),
        catalog_path: workspace.join("Library 5").join("catalog.db"),
        schema_version: 1,
    };
    record_app_session_recent_library(&session_path, &repeated).expect("record repeated recent");
    let loaded = load_app_session(&session_path).expect("reload recents");

    assert_eq!(loaded.session.recents.len(), APP_SESSION_RECENTS_LIMIT);
    assert_eq!(
        loaded
            .session
            .recents
            .first()
            .map(|recent| recent.root_path.as_path()),
        Some(workspace.join("Library 5").as_path())
    );
    assert_eq!(
        loaded
            .session
            .recents
            .iter()
            .filter(|recent| recent.root_path == workspace.join("Library 5"))
            .count(),
        1
    );
    assert!(!workspace.join("catalog.db").exists());
    assert!(!workspace.join("sidecars").exists());

    remove_library_root(&workspace);
}

#[test]
fn app_session_restore_plans_existing_library_without_support_dir_repair() {
    let workspace = unique_library_root("app-session-restore-existing");
    let session_path = workspace.join("app-session.json");
    let library_root = workspace.join("restore-library");
    create_library(&library_root).expect("create library");
    std::fs::remove_dir_all(library_root.join("thumbnails")).expect("remove thumbnails");

    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_root.clone());
    session.last_mode = AppSessionMode::Develop;
    write_app_session(&session_path, &session).expect("write app session");

    let restored = plan_app_session_restore(&session_path).expect("plan restore");

    assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
    assert_eq!(restored.requested_mode, AppSessionMode::Develop);
    assert_eq!(restored.resolved_mode, AppSessionMode::Library);
    assert_eq!(
        restored.library_root_path.as_deref(),
        Some(library_root.as_path())
    );
    assert_eq!(
        restored.catalog_path.as_deref(),
        Some(library_root.join("catalog.db").as_path())
    );
    assert!(!library_root.join("thumbnails").exists());

    remove_library_root(&workspace);
}

#[test]
fn app_session_restore_allows_older_catalog_for_grid_migration() {
    let workspace = unique_library_root("app-session-restore-legacy");
    let session_path = workspace.join("app-session.json");
    let library_root = workspace.join("restore-library");
    let created = create_library(&library_root).expect("create library");
    {
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        connection
            .execute("DELETE FROM schema_migrations WHERE version = 12", [])
            .expect("simulate legacy catalog version");
    }

    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_root.clone());
    session.last_mode = AppSessionMode::Library;
    write_app_session(&session_path, &session).expect("write app session");

    let restored = plan_app_session_restore(&session_path).expect("plan legacy restore");

    assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
    assert_eq!(restored.schema_version, Some(11));
    assert_eq!(
        restored.library_root_path.as_deref(),
        Some(library_root.as_path())
    );

    remove_library_root(&workspace);
}

#[test]
fn app_session_restore_falls_back_for_missing_library_or_catalog() {
    let workspace = unique_library_root("app-session-restore-missing");
    let session_path = workspace.join("app-session.json");

    let mut session = AppSession::default();
    session.last_library_root_path = Some(workspace.join("missing-library"));
    session.last_mode = AppSessionMode::Export;
    write_app_session(&session_path, &session).expect("write missing library app session");

    let missing_library_restore =
        plan_app_session_restore(&session_path).expect("plan missing library restore");
    assert_eq!(
        missing_library_restore.status,
        AppSessionRestoreStatus::MissingLibrary
    );
    assert_eq!(
        missing_library_restore.requested_mode,
        AppSessionMode::Export
    );
    assert_eq!(
        missing_library_restore.resolved_mode,
        AppSessionMode::Library
    );
    assert!(missing_library_restore.library_root_path.is_none());

    let library_without_catalog = workspace.join("library-without-catalog");
    std::fs::create_dir_all(&library_without_catalog).expect("create library dir");
    let mut session = AppSession::default();
    session.last_library_root_path = Some(library_without_catalog);
    write_app_session(&session_path, &session).expect("write missing catalog app session");

    let missing_catalog_restore =
        plan_app_session_restore(&session_path).expect("plan missing catalog restore");
    assert_eq!(
        missing_catalog_restore.status,
        AppSessionRestoreStatus::MissingCatalog
    );
    assert_eq!(
        missing_catalog_restore.requested_mode,
        AppSessionMode::Library
    );
    assert_eq!(
        missing_catalog_restore.resolved_mode,
        AppSessionMode::Library
    );
    assert!(missing_catalog_restore.catalog_path.is_none());

    remove_library_root(&workspace);
}

#[test]
fn selected_photo_restore_keeps_existing_photo_and_clears_missing_photo() {
    let workspace = unique_library_root("selected-photo-restore");
    let session_path = workspace.join("app-session.json");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let photo_id = list_library_photos(&created.root_path)
        .expect("list photos")
        .into_iter()
        .find(|photo| photo.file_name == "sample.jpg")
        .map(|photo| photo.photo_id)
        .expect("imported photo id");

    record_app_session_library_state(
        &session_path,
        &created.root_path,
        Some(photo_id.clone()),
        AppSessionMode::Develop,
    )
    .expect("record selected photo");

    let restored = plan_app_session_restore(&session_path).expect("restore selected photo");
    assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
    assert_eq!(
        restored.selected_photo_status,
        AppSessionSelectedPhotoStatus::Restored
    );
    assert_eq!(
        restored.selected_photo_id.as_deref(),
        Some(photo_id.as_str())
    );
    assert_eq!(restored.requested_mode, AppSessionMode::Develop);
    assert_eq!(restored.resolved_mode, AppSessionMode::Develop);

    record_app_session_library_state(
        &session_path,
        &created.root_path,
        Some("missing-photo".to_string()),
        AppSessionMode::Export,
    )
    .expect("record missing selected photo");

    let restored = plan_app_session_restore(&session_path).expect("restore missing selection");
    assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
    assert_eq!(
        restored.selected_photo_status,
        AppSessionSelectedPhotoStatus::Missing
    );
    assert!(restored.selected_photo_id.is_none());
    assert_eq!(restored.requested_mode, AppSessionMode::Export);
    assert_eq!(restored.resolved_mode, AppSessionMode::Library);

    remove_library_root(&workspace);
}

#[test]
fn exposes_metadata_policy_without_raw_decode_claim() {
    let jpeg_policy = metadata_extraction_policy_for_path(Path::new("sample.jpeg"));
    assert_eq!(
        jpeg_policy.dimension_source,
        silica_storage::MetadataDimensionSource::ExistingRasterPath
    );
    assert!(!jpeg_policy.raw_decode_supported);

    let raw_policy = metadata_extraction_policy_for_path(Path::new("sample.ARW"));
    assert_eq!(
        raw_policy.dimension_source,
        silica_storage::MetadataDimensionSource::Unavailable
    );
    assert!(!raw_policy.raw_decode_supported);
    assert!(!raw_policy.camera_lens_available);
}

#[test]
fn imports_jpeg_metadata_without_mutating_original() {
    let workspace = unique_library_root("jpeg-metadata");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");
    let raw_file = import_root.join("sample.DNG");
    let unsupported_file = import_root.join("notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");
    std::fs::write(&unsupported_file, b"unsupported note").expect("write unsupported");
    let jpeg_hash = file_hash(&jpeg_file);
    let raw_hash = file_hash(&raw_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let (width, height, camera_make, lens_model): (
        Option<i64>,
        Option<i64>,
        Option<String>,
        Option<String>,
    ) = connection
        .query_row(
            r#"
                SELECT photo_metadata.width,
                       photo_metadata.height,
                       photo_metadata.camera_make,
                       photo_metadata.lens_model
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'sample.jpg'
                "#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("jpeg metadata row");
    assert_eq!(width, Some(2));
    assert_eq!(height, Some(2));
    assert_eq!(camera_make, None);
    assert_eq!(lens_model, None);

    let raw_metadata_count: i64 = connection
        .query_row(
            r#"
                SELECT COUNT(*)
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'sample.DNG'
                  AND photo_metadata.width IS NULL
                  AND photo_metadata.height IS NULL
                  AND photo_metadata.camera_make IS NULL
                  AND photo_metadata.lens_model IS NULL
                "#,
            [],
            |row| row.get(0),
        )
        .expect("raw metadata count");
    assert_eq!(raw_metadata_count, 0);

    let unsupported_metadata_count: i64 = connection
        .query_row(
            r#"
                SELECT COUNT(*)
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'notes.txt'
                "#,
            [],
            |row| row.get(0),
        )
        .expect("unsupported metadata count");
    assert_eq!(unsupported_metadata_count, 0);

    assert_original_hash(&jpeg_file, &jpeg_hash, "JPEG metadata extraction");
    assert_original_hash(&raw_file, &raw_hash, "RAW metadata policy");

    remove_library_root(&workspace);
}

#[test]
fn queries_photo_metadata_without_reopening_original() {
    let workspace = unique_library_root("core-metadata-query");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");
    std::fs::remove_file(&jpeg_file).expect("remove original before metadata query");

    let photo_id = list_library_photos(&created.root_path)
        .expect("list imported photos")
        .into_iter()
        .find(|photo| photo.file_name == "sample.jpg")
        .expect("sample photo")
        .photo_id;
    let metadata = get_photo_metadata(&created.root_path, &photo_id)
        .expect("query metadata through core")
        .expect("photo metadata");
    assert_eq!(metadata.width.state, PhotoMetadataFieldState::Known);
    assert_eq!(metadata.width.value, Some(2));
    assert_eq!(metadata.height.state, PhotoMetadataFieldState::Known);
    assert_eq!(metadata.height.value, Some(2));
    assert_eq!(
        metadata.capture_time.state,
        PhotoMetadataFieldState::Unavailable
    );

    remove_library_root(&workspace);
}

#[test]
fn metadata_filter_returns_only_photos_with_dimensions() {
    let workspace = unique_library_root("core-metadata-filter");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");
    let raw_file = import_root.join("sample.DNG");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");
    std::fs::remove_file(&jpeg_file).expect("remove original after dimension import");
    std::fs::remove_file(&raw_file).expect("remove raw original before metadata filter query");

    let page = query_library_photos(
        &created.root_path,
        LibraryQueryRequest::new(
            0,
            100,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters {
                metadata: Some(LibraryQueryMetadataFilter::HasDimensions),
                ..LibraryQueryFilters::default()
            },
        ),
    )
    .expect("query metadata-backed filter");

    assert_eq!(page.total_count, 1);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].file_name, "sample.jpg");

    remove_library_root(&workspace);
}

#[test]
fn import_error_summary_survives_core_metadata_step() {
    let workspace = unique_library_root("core-import-errors");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let unsupported_file = import_root.join("notes.txt");
    let hidden_file = import_root.join(".hidden.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&supported_file);
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
    std::fs::write(&hidden_file, b"hidden jpeg").expect("write hidden");

    let created = create_library(&library_root).expect("create library through core");
    let summary = import_folder(&created.root_path, &import_root).expect("import through core");

    assert_eq!(summary.supported_files, 1);
    assert_eq!(summary.unsupported_files, 1);
    assert!(summary
        .issues
        .iter()
        .any(|issue| issue.kind == ImportIssueKind::UnsupportedFile));
    assert!(summary
        .issues
        .iter()
        .any(|issue| issue.kind == ImportIssueKind::HiddenEntrySkipped));

    let rows = list_library_photos(&created.root_path).expect("browse after import issues");
    assert_eq!(rows.len(), 2);

    remove_library_root(&workspace);
}

#[test]
fn recursive_import_opt_in_through_core_imports_nested_rows() {
    let workspace = unique_library_root("core-recursive-import");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let nested_root = import_root.join("Nested");
    let nested_file = nested_root.join("child.jpg");
    let unsupported_file = nested_root.join("notes.txt");

    std::fs::create_dir_all(&nested_root).expect("create nested import directory");
    write_source_jpeg(&nested_file);
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

    let created = create_library(&library_root).expect("create library through core");
    let default_summary = import_folder(&created.root_path, &import_root).expect("default import");
    assert_eq!(default_summary.scanned_files, 0);

    let summary = import_folder_with_options(
        &created.root_path,
        &import_root,
        FolderImportOptions { recursive: true },
    )
    .expect("recursive import through core");

    assert_eq!(summary.supported_files, 1);
    assert_eq!(summary.unsupported_files, 1);
    assert!(summary
        .issues
        .iter()
        .any(|issue| issue.kind == ImportIssueKind::UnsupportedFile));

    let rows = list_library_photos(&created.root_path).expect("browse recursive rows");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|row| row.file_name == "child.jpg"));
    assert!(rows
        .iter()
        .any(|row| row.file_name == "notes.txt" && row.unsupported));

    remove_library_root(&workspace);
}

#[test]
fn creates_and_reopens_local_library_session() {
    let root = unique_library_root("core");

    let created = create_library(&root).expect("create library through core");
    let reopened = open_library(&root).expect("open library through core");

    assert_eq!(created.root_path, root);
    assert_eq!(reopened.root_path, created.root_path);
    assert_eq!(reopened.catalog_path, created.catalog_path);
    assert_eq!(reopened.schema_version, created.schema_version);
    assert!(created.catalog_path.is_file());
    assert!(created.status_text().contains("Library:"));
    assert!(created.status_text().contains("catalog.db"));

    remove_library_root(&root);
}

#[test]
fn imports_and_persists_photo_flags_through_core() {
    let workspace = unique_library_root("core-flags");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&supported_file);

    let created = create_library(&library_root).expect("create library through core");
    let summary = import_folder(&created.root_path, &import_root).expect("import through core");
    assert_eq!(summary.supported_files, 1);

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");

    let updated = set_photo_flags(
        &created.root_path,
        photo_id,
        3,
        false,
        true,
        Some("red".to_string()),
    )
    .expect("set flags through core");

    let reopened = open_library(&library_root).expect("reopen library through core");
    let persisted = get_photo_flags(&reopened.root_path, &updated.photo_id)
        .expect("read flags through core")
        .expect("flags row");

    assert_eq!(persisted, updated);

    remove_library_root(&workspace);
}

#[test]
fn serializes_library_photo_grid_rows_for_desktop() {
    let workspace = unique_library_root("core-grid");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let unsupported_file = import_root.join("sample.DNG");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&supported_file);
    std::fs::write(&unsupported_file, b"unsupported raw candidate").expect("write unsupported");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    set_photo_flags(
        &created.root_path,
        photo_id,
        4,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set grid flags through core");

    let rows = list_library_photos_json(&created.root_path).expect("list grid rows as json");
    let rows: serde_json::Value = serde_json::from_str(&rows).expect("parse grid rows json");
    let rows = rows.as_array().expect("grid rows array");

    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|row| {
        row["fileName"] == "sample.jpg"
            && row["fileType"] == "JPG"
            && row["rating"] == 4
            && row["picked"] == true
            && row["colorLabel"] == "green"
    }));
    assert!(rows.iter().any(|row| {
        row["fileName"] == "sample.DNG" && row["fileType"] == "DNG" && row["unsupported"] == true
    }));

    remove_library_root(&workspace);
}

#[test]
fn library_query_returns_page_without_cache_hydration() {
    let workspace = unique_library_root("core-library-query");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");
    let raw_file = import_root.join("sample.DNG");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    std::fs::write(&raw_file, b"raw candidate").expect("write raw");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");

    let page = query_library_photos(
        &created.root_path,
        LibraryQueryRequest::new(
            0,
            1,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query page through core");

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.total_count, 2);
    assert!(page.has_next_page);

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let cache_records: i64 = connection
        .query_row("SELECT COUNT(*) FROM cache_records", [], |row| row.get(0))
        .expect("count cache records");
    assert_eq!(cache_records, 0);

    remove_library_root(&workspace);
}

#[test]
fn creates_jpeg_thumbnail_cache_for_grid_without_mutating_original() {
    let workspace = unique_library_root("core-thumbnail-grid");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");
    let raw_file = import_root.join("sample.DNG");
    let unsupported_file = import_root.join("notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    std::fs::write(&raw_file, b"supported raw candidate").expect("write raw candidate");
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

    let original_hash = file_hash(&jpeg_file);
    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");

    let rows = list_library_photos(&created.root_path).expect("list grid rows");

    let jpeg = rows
        .iter()
        .find(|row| row.file_name == "sample.jpg")
        .expect("jpeg grid row");
    let thumbnail_path = PathBuf::from(
        jpeg.thumbnail_path
            .as_ref()
            .expect("jpeg row exposes thumbnail path"),
    );
    assert!(thumbnail_path.starts_with(created.root_path.join("thumbnails")));
    assert!(thumbnail_path.is_file());
    let decoded = image::ImageReader::open(&thumbnail_path)
        .expect("open thumbnail")
        .with_guessed_format()
        .expect("guess thumbnail format")
        .decode()
        .expect("decode thumbnail");
    assert!(decoded.width() <= 320);
    assert!(decoded.height() <= 320);
    assert_original_hash(&jpeg_file, &original_hash, "thumbnail cache generation");

    let raw = rows
        .iter()
        .find(|row| row.file_name == "sample.DNG")
        .expect("raw grid row");
    assert!(raw.thumbnail_path.is_none());
    let unsupported = rows
        .iter()
        .find(|row| row.file_name == "notes.txt")
        .expect("unsupported grid row");
    assert!(unsupported.thumbnail_path.is_none());

    let cached_rows = list_library_photos(&created.root_path).expect("list cached grid rows");
    let cached_jpeg = cached_rows
        .iter()
        .find(|row| row.file_name == "sample.jpg")
        .expect("cached jpeg grid row");
    assert_eq!(
        cached_jpeg.thumbnail_path.as_deref(),
        jpeg.thumbnail_path.as_deref()
    );
    assert_eq!(
        cached_jpeg.thumbnail_cache_key.as_deref(),
        jpeg.thumbnail_cache_key.as_deref()
    );

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let cache_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'thumbnail'",
            [],
            |row| row.get(0),
        )
        .expect("count thumbnail cache rows");
    assert_eq!(cache_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn supports_png_and_tiff_sources_through_preview_develop_and_jpeg_export() {
    let workspace = unique_library_root("core-raster-source-flow");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let png_file = import_root.join("sample.png");
    let tiff_file = import_root.join("sample.tiff");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_image(&png_file, image::ImageFormat::Png);
    write_source_image(&tiff_file, image::ImageFormat::Tiff);
    let png_hash = file_hash(&png_file);
    let tiff_hash = file_hash(&tiff_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");

    let rows = list_library_photos(&created.root_path).expect("list grid rows");
    for (file_name, file_type) in [("sample.png", "PNG"), ("sample.tiff", "TIFF")] {
        let row = rows
            .iter()
            .find(|row| row.file_name == file_name)
            .expect("raster grid row");
        assert_eq!(row.file_type, file_type);
        assert!(!row.unsupported);
        assert!(row.thumbnail_path.is_some());
    }

    let png_page = query_library_photos(
        &created.root_path,
        LibraryQueryRequest::new(
            0,
            10,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters {
                file_type: Some(LibraryQueryFileType::Png),
                ..Default::default()
            },
        ),
    )
    .expect("query png files");
    assert_eq!(png_page.items.len(), 1);
    assert_eq!(png_page.items[0].file_name, "sample.png");

    let tiff_page = query_library_photos(
        &created.root_path,
        LibraryQueryRequest::new(
            0,
            10,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters {
                file_type: Some(LibraryQueryFileType::Tiff),
                ..Default::default()
            },
        ),
    )
    .expect("query tiff files");
    assert_eq!(tiff_page.items.len(), 1);
    assert_eq!(tiff_page.items[0].file_name, "sample.tiff");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    for (file_name, source_path, source_hash, output_name) in [
        ("sample.png", &png_file, &png_hash, "sample-png-export.jpg"),
        (
            "sample.tiff",
            &tiff_file,
            &tiff_hash,
            "sample-tiff-export.jpg",
        ),
    ] {
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = ?1",
                [file_name],
                |row| row.get(0),
            )
            .expect("photo id");

        let preview = open_photo_preview(&created.root_path, &photo_id)
            .expect("open preview")
            .expect("preview session");
        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert!(preview.preview_bytes.is_some());

        let develop_preview =
            preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.25, 6.0)
                .expect("preview develop edit")
                .expect("develop preview");
        assert_eq!(develop_preview.status, PhotoPreviewStatus::Ready);
        assert!(develop_preview.develop_preview_bytes.is_some());

        let committed = commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.25, 6.0)
            .expect("commit develop edit")
            .expect("committed edit");
        assert_eq!(committed.photo_id, photo_id);

        let histogram = get_photo_histogram(&created.root_path, &photo_id)
            .expect("compute histogram")
            .expect("histogram");
        assert_eq!(histogram.status, PhotoHistogramStatus::Ready);
        assert_eq!(histogram.pixel_count, 4);

        let output_path = export_root.join(output_name);
        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export jpeg")
            .expect("export result");
        assert_eq!(exported.format, "jpeg");
        assert_eq!(exported.decoder_backend.as_deref(), Some("raster"));
        assert_eq!(exported.input_profile.as_deref(), Some("assume_srgb"));
        assert_eq!(exported.working_space.as_deref(), Some("srgb"));
        assert!(output_path.is_file());
        assert_original_hash(source_path, source_hash, "raster source workflow");
    }

    remove_library_root(&workspace);
}

#[test]
fn hydrates_thumbnails_only_for_queried_page() {
    let workspace = unique_library_root("core-thumbnail-page");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let first_jpeg = import_root.join("a-first.jpg");
    let second_jpeg = import_root.join("b-second.jpg");
    let raw_file = import_root.join("c-raw.DNG");
    let unsupported_file = import_root.join("d-notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&first_jpeg);
    write_source_jpeg(&second_jpeg);
    std::fs::write(&raw_file, b"raw candidate").expect("write raw");
    std::fs::write(&unsupported_file, b"unsupported").expect("write unsupported");
    let first_hash = file_hash(&first_jpeg);
    let second_hash = file_hash(&second_jpeg);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import folder through core");

    let page = query_library_photos_with_thumbnail_hydration(
        &created.root_path,
        LibraryQueryRequest::new(
            0,
            1,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query hydrated page");

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].file_name, "a-first.jpg");
    assert!(page.items[0].thumbnail_path.is_some());
    assert!(page.items[0].thumbnail_cache_key.is_some());
    assert_original_hash(&first_jpeg, &first_hash, "page thumbnail hydration");
    assert_original_hash(&second_jpeg, &second_hash, "page thumbnail hydration");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let thumbnail_records: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'thumbnail'",
            [],
            |row| row.get(0),
        )
        .expect("count thumbnail records");
    assert_eq!(thumbnail_records, 1);

    let second_page = query_library_photos(
        &created.root_path,
        LibraryQueryRequest::new(
            1,
            1,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query second page without hydration");
    assert_eq!(second_page.items[0].file_name, "b-second.jpg");
    assert!(second_page.items[0].thumbnail_path.is_none());

    remove_library_root(&workspace);
}

#[test]
fn opens_preview_session_with_ready_and_blocked_states() {
    let workspace = unique_library_root("core-preview");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");
    let raw_file = import_root.join("sample.dng");
    let unsupported_file = import_root.join("notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

    let original_hash = file_hash(&jpeg_file);
    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let jpeg_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("jpeg photo id");
    let raw_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.dng'",
            [],
            |row| row.get(0),
        )
        .expect("raw photo id");
    let unsupported_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'notes.txt'",
            [],
            |row| row.get(0),
        )
        .expect("unsupported photo id");

    let jpeg_preview = open_photo_preview(&created.root_path, &jpeg_id)
        .expect("open jpeg preview")
        .expect("jpeg preview session");
    assert_eq!(jpeg_preview.file_name, "sample.jpg");
    assert_eq!(jpeg_preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(jpeg_preview.source_path, jpeg_file.display().to_string());
    assert!(jpeg_preview
        .preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert_original_hash(&jpeg_file, &original_hash, "loupe preview cache generation");

    let jpeg_preview_again = open_photo_preview(&created.root_path, &jpeg_id)
        .expect("reopen jpeg preview")
        .expect("cached jpeg preview session");
    assert_eq!(jpeg_preview_again.preview_bytes, jpeg_preview.preview_bytes);

    let raw_preview = open_photo_preview(&created.root_path, &raw_id)
        .expect("open raw preview")
        .expect("raw preview session");
    assert_eq!(raw_preview.status, PhotoPreviewStatus::Unsupported);
    assert!(raw_preview.message.contains("Unsupported file type"));
    assert!(raw_preview.preview_bytes.is_none());

    let unsupported_preview = open_photo_preview(&created.root_path, &unsupported_id)
        .expect("open unsupported preview")
        .expect("unsupported preview session");
    assert_eq!(unsupported_preview.status, PhotoPreviewStatus::Unsupported);

    assert!(open_photo_preview(&created.root_path, "missing-photo")
        .expect("missing preview lookup")
        .is_none());

    let cache_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'preview'",
            [],
            |row| row.get(0),
        )
        .expect("count preview cache rows");
    assert_eq!(cache_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn missing_original_blocks_ready_preview_develop_and_export_without_writes() {
    let workspace = unique_library_root("core-missing-original-readiness");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let cached_rows = list_library_photos(&created.root_path).expect("hydrate thumbnail");
    let cached = cached_rows
        .iter()
        .find(|row| row.photo_id == photo_id)
        .expect("cached row");
    assert!(!cached.missing);
    assert!(cached.thumbnail_path.is_some());

    std::fs::remove_file(&jpeg_file).expect("remove referenced original");
    let before_missing_counts = durable_catalog_counts(&created.catalog_path);
    let before_stored_missing: i64 = {
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        connection
            .query_row(
                "SELECT missing FROM photos WHERE id = ?1",
                [&photo_id],
                |row| row.get(0),
            )
            .expect("read stored missing before runtime downgrade")
    };
    assert_eq!(before_stored_missing, 0);

    let missing_rows = list_library_photos(&created.root_path).expect("list missing row");
    let missing = missing_rows
        .iter()
        .find(|row| row.photo_id == photo_id)
        .expect("missing row");
    assert!(missing.missing);
    assert!(missing.thumbnail_path.is_none());

    let missing_page = query_library_photos(
        &created.root_path,
        LibraryQueryRequest::new(
            0,
            10,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query missing row");
    assert_eq!(missing_page.items.len(), 1);
    assert!(missing_page.items[0].missing);
    assert!(missing_page.items[0].thumbnail_path.is_none());

    let preview = open_photo_preview(&created.root_path, &photo_id)
        .expect("open missing preview")
        .expect("missing preview session");
    assert_eq!(preview.status, PhotoPreviewStatus::BlockedByDecode);
    assert!(preview.message.contains("source file is missing"));
    assert!(preview.preview_bytes.is_none());

    let histogram = get_photo_histogram(&created.root_path, &photo_id)
        .expect("get missing histogram")
        .expect("missing histogram");
    assert_eq!(histogram.status, PhotoHistogramStatus::Missing);
    assert_eq!(histogram.pixel_count, 0);

    let develop_preview = preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, 4.0)
        .expect("preview missing develop")
        .expect("missing develop preview");
    assert_eq!(develop_preview.status, PhotoPreviewStatus::BlockedByDecode);
    assert!(develop_preview.develop_preview_bytes.is_none());

    let commit_error = commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, 4.0)
        .expect_err("missing source Develop commit must be blocked");
    assert!(matches!(commit_error, CoreError::UnsupportedEdit(_)));
    assert!(commit_error.to_string().contains("source file is missing"));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load missing source active graph")
            .is_none(),
        "missing source Develop commit must not write active edit graph"
    );

    let export_error = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect_err("missing source export must be blocked");
    assert!(matches!(export_error, CoreError::ExportBlocked(_)));
    assert!(export_error.to_string().contains("source file is missing"));
    assert!(!output_path.exists());
    let after_missing_counts = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after_missing_counts, before_missing_counts);
    let after_stored_missing: i64 = {
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        connection
            .query_row(
                "SELECT missing FROM photos WHERE id = ?1",
                [&photo_id],
                |row| row.get(0),
            )
            .expect("read stored missing after runtime downgrade")
    };
    assert_eq!(after_stored_missing, 0);

    remove_library_root(&workspace);
}

#[test]
fn computes_and_caches_histogram_without_mutating_original() {
    let workspace = unique_library_root("core-histogram-flow");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    commit_color_presence_edit(&created.root_path, &photo_id, 24.0, -8.5)
        .expect("commit color presence")
        .expect("commit result");
    let histogram = get_photo_histogram(&created.root_path, &photo_id)
        .expect("get histogram")
        .expect("histogram result");

    assert_eq!(histogram.status, PhotoHistogramStatus::Ready);
    assert_eq!(histogram.pixel_count, 4);
    assert_eq!(histogram.red.len(), 256);
    assert_eq!(histogram.green.len(), 256);
    assert_eq!(histogram.blue.len(), 256);
    assert_eq!(histogram.luminance.len(), 256);
    assert!(histogram.cache_path.contains("render-cache"));
    assert_original_hash(&jpeg_file, &original_hash, "histogram generation");

    let cached = silica_storage::get_photo_cache_record(
        &created.root_path,
        &photo_id,
        silica_storage::HISTOGRAM_CACHE_TYPE,
    )
    .expect("read histogram cache")
    .expect("histogram cache row");
    assert_eq!(cached.path, histogram.cache_path);

    remove_library_root(&workspace);
}

#[test]
fn previews_without_write_and_commits_exposure_contrast_edit() {
    let workspace = unique_library_root("core-edit-flow");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);

    let original_hash = file_hash(&jpeg_file);
    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    let edit_state_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
        .expect("count edit states");
    assert_eq!(edit_state_count, 0);
    drop(connection);

    let preview = preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("preview edit")
        .expect("preview edit request");

    assert_eq!(preview.photo_id, photo_id);
    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.exposure, 0.5);
    assert_eq!(preview.contrast, -8.0);
    assert!(preview.message.contains("exposure/contrast"));
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert_original_hash(&jpeg_file, &original_hash, "develop preview generation");

    let default_edit_state = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read default edit state")
        .expect("default edit state");
    assert_eq!(default_edit_state.exposure, 0.0);
    assert_eq!(default_edit_state.contrast, 0.0);
    assert!(!default_edit_state.persisted);

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let edit_state_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
        .expect("count edit states");
    assert_eq!(
        edit_state_count, 0,
        "preview edit must not write edit_states"
    );
    drop(connection);

    let committed = commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("committed edit");
    assert_eq!(committed.photo_id, photo_id);
    assert_eq!(committed.exposure, 0.5);
    assert_eq!(committed.contrast, -8.0);
    assert!(committed.persisted);

    let reopened = open_library(&library_root).expect("reopen library through core");
    let persisted =
        silica_storage::load_active_edit_graph_or_default(&reopened.root_path, &committed.photo_id)
            .expect("load active graph")
            .expect("active graph");
    assert_eq!(persisted.basic.exposure.as_f64(), Some(0.5));
    assert_eq!(persisted.basic.contrast.as_f64(), Some(-8.0));

    let restored = get_photo_edit_state(&reopened.root_path, &committed.photo_id)
        .expect("read restored edit state")
        .expect("restored edit state");
    assert_eq!(restored.exposure, 0.5);
    assert_eq!(restored.contrast, -8.0);
    assert!(restored.persisted);

    remove_library_root(&workspace);
}

#[test]
fn previews_without_write_and_commits_manual_linear_gradient_mask() {
    let workspace = unique_library_root("core-manual-mask-flow");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let before_preview = durable_catalog_counts(&created.catalog_path);
    let preview = preview_manual_linear_gradient_mask(
        &created.root_path,
        &photo_id,
        "mask-linear-1",
        "Top burn",
        100.0,
        0.0,
        false,
        0.0,
        0.0,
        1.0,
        1.0,
        Some(0.75),
        Some(0.0),
    )
    .expect("preview manual mask")
    .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.masks.len(), 1);
    assert_eq!(preview.masks[0].id, "mask-linear-1");
    assert_eq!(preview.masks[0].exposure, 0.75);
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert_original_hash(&jpeg_file, &original_hash, "manual mask preview");
    assert_eq!(
        durable_catalog_counts(&created.catalog_path),
        before_preview,
        "manual mask preview must not write durable catalog state"
    );
    let unmasked_followup_preview =
        preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.0, 0.0)
            .expect("preview without committed masks")
            .expect("unmasked preview result");
    assert!(unmasked_followup_preview.masks.is_empty());
    let unmasked_followup_bytes = unmasked_followup_preview
        .develop_preview_bytes
        .expect("unmasked preview bytes");

    let committed = commit_manual_linear_gradient_mask(
        &created.root_path,
        &photo_id,
        "mask-linear-1",
        "Top burn",
        100.0,
        0.0,
        false,
        0.0,
        0.0,
        1.0,
        1.0,
        Some(0.75),
        Some(0.0),
    )
    .expect("commit manual mask")
    .expect("commit result");
    assert!(committed.persisted);
    assert_eq!(committed.masks.len(), 1);
    assert_eq!(committed.masks[0].name, "Top burn");

    let restored = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read manual mask state")
        .expect("edit state");
    assert_eq!(restored.masks.len(), 1);
    assert_eq!(
        restored.masks[0].geometry,
        Some(PhotoManualMaskGeometryState::LinearGradient {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 1.0,
            end_y: 1.0,
        })
    );
    let masked_followup_preview =
        preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.0, 0.0)
            .expect("preview with committed mask")
            .expect("masked preview result");
    assert_eq!(masked_followup_preview.masks.len(), 1);
    assert_ne!(
        masked_followup_preview
            .develop_preview_bytes
            .expect("masked preview bytes"),
        unmasked_followup_bytes,
        "committed mask must affect later Develop previews"
    );

    let undo = undo_last_history_action(&created.root_path, &photo_id).expect("undo");
    assert!(undo.applied);
    let undone = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read undone state")
        .expect("edit state");
    assert!(undone.masks.is_empty());

    let redo = redo_last_history_action(&created.root_path, &photo_id).expect("redo");
    assert!(redo.applied);
    let redone = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read redone state")
        .expect("edit state");
    assert_eq!(redone.masks.len(), 1);
    assert_original_hash(&jpeg_file, &original_hash, "manual mask commit");

    remove_library_root(&workspace);
}

#[test]
fn previews_brush_mask_cache_without_durable_edit_writes_and_commits_strokes() {
    let workspace = unique_library_root("core-brush-mask-flow");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let stroke = PhotoManualBrushStrokeInput {
        id: "stroke-1".to_string(),
        radius: 0.20,
        points: vec![PhotoManualBrushPointInput { x: 0.5, y: 0.5 }],
    };
    let before_preview = durable_catalog_counts(&created.catalog_path);
    let preview = preview_manual_brush_mask(
        &created.root_path,
        &photo_id,
        "mask-brush-1",
        "Center dodge",
        100.0,
        0.0,
        false,
        vec![stroke.clone()],
        Some(0.75),
        Some(0.0),
    )
    .expect("preview brush mask")
    .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.masks.len(), 1);
    assert_eq!(preview.masks[0].kind, "brush");
    assert!(preview.masks[0].geometry.is_none());
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert_original_hash(&jpeg_file, &original_hash, "brush mask preview");
    let after_preview = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after_preview.edit_states, before_preview.edit_states);
    assert_eq!(after_preview.edit_history, before_preview.edit_history);
    assert_eq!(after_preview.action_log, before_preview.action_log);
    assert_eq!(after_preview.exports, before_preview.exports);
    assert_eq!(
        after_preview.cache_records,
        before_preview.cache_records + 1
    );
    let mask_cache = silica_storage::get_photo_cache_record(
        &created.root_path,
        &photo_id,
        silica_storage::MASK_RASTER_CACHE_TYPE,
    )
    .expect("read mask raster cache")
    .expect("mask raster cache row");
    assert!(mask_cache.path.contains("render-cache/masks"));
    assert!(Path::new(&mask_cache.path).is_file());

    silica_storage::clear_disposable_cache(&created.root_path).expect("clear cache");
    assert!(!Path::new(&mask_cache.path).exists());
    assert!(silica_storage::get_photo_cache_record(
        &created.root_path,
        &photo_id,
        silica_storage::MASK_RASTER_CACHE_TYPE,
    )
    .expect("read mask raster cache after clear")
    .is_none());

    let committed = commit_manual_brush_mask(
        &created.root_path,
        &photo_id,
        "mask-brush-1",
        "Center dodge",
        100.0,
        0.0,
        false,
        vec![stroke],
        Some(0.75),
        Some(0.0),
    )
    .expect("commit brush mask")
    .expect("commit result");
    assert!(committed.persisted);
    assert_eq!(committed.masks.len(), 1);
    assert_eq!(committed.masks[0].kind, "brush");
    assert!(committed.masks[0].geometry.is_none());

    let restored = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read brush mask state")
        .expect("edit state");
    assert_eq!(restored.masks.len(), 1);
    assert_eq!(restored.masks[0].kind, "brush");
    assert!(restored.masks[0].geometry.is_none());

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export brush masked photo")
        .expect("export result");
    assert!(exported.bytes_written > 0);
    assert!(output_path.exists());
    let export_cache = silica_storage::get_photo_cache_record(
        &created.root_path,
        &photo_id,
        silica_storage::MASK_RASTER_CACHE_TYPE,
    )
    .expect("read mask raster cache after export")
    .expect("mask raster cache row after export");
    assert!(Path::new(&export_cache.path).is_file());
    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest brush masked export")
        .expect("latest brush masked export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["masks"][0]["kind"], "brush");
    assert_eq!(settings["masks"][0]["geometry"]["kind"], "brush_raster");
    assert!(settings["masks"][0]["geometry"]["cache_key"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_original_hash(&jpeg_file, &original_hash, "brush mask export");

    remove_library_root(&workspace);
}

#[test]
fn exports_committed_manual_mask_and_records_mask_evidence() {
    let workspace = unique_library_root("core-manual-mask-export");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let neutral_output_path = export_root.join("sample-neutral.jpg");
    let masked_output_path = export_root.join("sample-masked.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let neutral_export =
        export_photo_jpeg_srgb(&created.root_path, &photo_id, &neutral_output_path)
            .expect("export neutral photo")
            .expect("neutral export result");
    assert!(neutral_output_path.exists());
    assert_original_hash(&jpeg_file, &original_hash, "neutral export before mask");

    commit_manual_linear_gradient_mask(
        &created.root_path,
        &photo_id,
        "mask-linear-1",
        "Diagonal lift",
        100.0,
        0.0,
        false,
        0.0,
        0.0,
        1.0,
        1.0,
        Some(1.0),
        Some(0.0),
    )
    .expect("commit linear mask")
    .expect("commit result");

    let masked_preview = preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.0, 0.0)
        .expect("preview committed mask")
        .expect("masked preview result");
    assert_eq!(masked_preview.status, PhotoPreviewStatus::Ready);
    assert!(masked_preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    let masked_export = export_photo_jpeg_srgb(&created.root_path, &photo_id, &masked_output_path)
        .expect("export masked photo")
        .expect("masked export result");
    assert!(masked_output_path.exists());
    assert_ne!(neutral_export.output_sha256, masked_export.output_sha256);
    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest masked export")
        .expect("latest masked export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["masks"][0]["kind"], "linear_gradient");
    assert_eq!(settings["masks"][0]["geometry"]["kind"], "linear_gradient");
    assert_eq!(settings["masks"][0]["geometry"]["start_x"], 0.0);
    assert_eq!(settings["masks"][0]["exposure"], 1.0);
    assert_original_hash(&jpeg_file, &original_hash, "masked export");

    remove_library_root(&workspace);
}

#[test]
fn undo_and_redo_edit_history_through_core() {
    let workspace = unique_library_root("core-undo-redo");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&supported_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("commit result");

    let undo = undo_last_history_action(&created.root_path, &photo_id).expect("undo");
    assert!(undo.applied);
    let undone = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read undone edit")
        .expect("edit state");
    assert_eq!(undone.exposure, 0.0);
    assert_eq!(undone.contrast, 0.0);

    let redo = redo_last_history_action(&created.root_path, &photo_id).expect("redo");
    assert!(redo.applied);
    let redone = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read redone edit")
        .expect("edit state");
    assert_eq!(redone.exposure, 0.5);
    assert_eq!(redone.contrast, -8.0);

    remove_library_root(&workspace);
}

#[test]
fn photo_history_through_core_lists_real_checkpoints() {
    let workspace = unique_library_root("core-history-panel");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&supported_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("commit result");

    let history = list_photo_history(&created.root_path, &photo_id).expect("read history");
    assert_eq!(history.photo_id, photo_id);
    assert_eq!(history.status, "ready");
    assert!(history.can_undo);
    assert!(!history.can_redo);
    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].action_kind, "edit_commit");
    assert_eq!(history.items[0].label, "Exposure / contrast");
    assert_eq!(history.items[0].history_state, "applied");

    remove_library_root(&workspace);
}

#[test]
fn exports_edited_photo_to_jpeg_srgb_and_records_catalog_row() {
    let workspace = unique_library_root("core-export");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("edit commit");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");

    assert_eq!(exported.photo_id, photo_id);
    assert_eq!(exported.output_path, output_path);
    assert_eq!(exported.format, "jpeg");
    assert_eq!(exported.color_profile, "srgb");
    assert!(exported.bytes_written > 0);
    assert_eq!(
        exported
            .source_sha256
            .as_deref()
            .expect("source SHA-256 evidence")
            .len(),
        64
    );
    assert_eq!(exported.output_sha256.len(), 64);
    assert!(exported.icc_profile_embedded);
    assert_eq!(
        exported.icc_profile_sha256,
        "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e"
    );
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );

    let decoded = image::ImageReader::open(&exported.output_path)
        .expect("open exported jpeg")
        .with_guessed_format()
        .expect("guess exported format")
        .decode()
        .expect("decode exported jpeg");
    assert_eq!(decoded.width(), 2);
    assert_eq!(decoded.height(), 2);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
        .expect("read latest export")
        .expect("latest export");
    assert_eq!(latest.id, exported.export_record_id);
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["color_profile"], "srgb");
    assert_eq!(settings["icc_profile_embedded"], true);
    assert_eq!(
        settings["icc_profile_sha256"],
        "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e"
    );
    assert_eq!(
        settings["output_sha256"]
            .as_str()
            .expect("output hash string")
            .len(),
        64
    );
    assert_eq!(
        settings["source_sha256"].as_str(),
        exported.source_sha256.as_deref()
    );
    assert_eq!(
        settings["source_sha256_after_export"].as_str(),
        exported.source_sha256.as_deref()
    );
    assert_eq!(settings["source_original_hash_unchanged"], true);

    let flags = get_photo_flags(&created.root_path, &exported.photo_id)
        .expect("read flags")
        .expect("flags row");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let exported_flag: i64 = connection
        .query_row(
            "SELECT exported FROM photo_flags WHERE photo_id = ?1",
            [&flags.photo_id],
            |row| row.get(0),
        )
        .expect("exported flag");
    assert_eq!(exported_flag, 1);

    remove_library_root(&workspace);
}

#[test]
fn exports_edited_photo_to_png_and_records_catalog_row() {
    let workspace = unique_library_root("core-export-png");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.png");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("edit commit");

    let exported = export_photo_png(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");

    assert_eq!(exported.photo_id, photo_id);
    assert_eq!(exported.output_path, output_path);
    assert_eq!(exported.format, "png");
    assert_eq!(exported.color_profile, "srgb");
    assert!(exported.bytes_written > 0);
    assert_eq!(exported.output_sha256.len(), 64);
    assert!(!exported.icc_profile_embedded);
    assert_eq!(exported.icc_profile_sha256, "");
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );

    let decoded = image::ImageReader::open(&exported.output_path)
        .expect("open exported png")
        .with_guessed_format()
        .expect("guess exported format")
        .decode()
        .expect("decode exported png");
    assert_eq!(decoded.width(), 2);
    assert_eq!(decoded.height(), 2);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
        .expect("read latest export")
        .expect("latest export");
    assert_eq!(latest.id, exported.export_record_id);
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["format"], "png");
    assert_eq!(settings["color_profile"], "srgb");
    assert_eq!(settings["icc_profile_embedded"], false);
    assert_eq!(settings["icc_profile_sha256"], serde_json::Value::Null);
    assert_eq!(settings["output_sha256"], exported.output_sha256);

    remove_library_root(&workspace);
}

#[test]
fn exports_edited_photo_to_tiff_and_records_catalog_row() {
    let workspace = unique_library_root("core-export-tiff");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.tiff");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let exported = export_photo_tiff(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");

    assert_eq!(exported.photo_id, photo_id);
    assert_eq!(exported.output_path, output_path);
    assert_eq!(exported.format, "tiff");
    assert_eq!(exported.color_profile, "srgb");
    assert!(exported.bytes_written > 0);
    assert_eq!(exported.output_sha256.len(), 64);
    assert!(!exported.icc_profile_embedded);
    assert_eq!(exported.icc_profile_sha256, "");
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );

    let decoded = image::ImageReader::open(&exported.output_path)
        .expect("open exported tiff")
        .with_guessed_format()
        .expect("guess exported format")
        .decode()
        .expect("decode exported tiff");
    assert_eq!(decoded.width(), 2);
    assert_eq!(decoded.height(), 2);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["format"], "tiff");
    assert_eq!(settings["color_profile"], "srgb");
    assert_eq!(settings["icc_profile_embedded"], false);
    assert_eq!(settings["icc_profile_sha256"], serde_json::Value::Null);
    assert_eq!(settings["output_sha256"], exported.output_sha256);

    remove_library_root(&workspace);
}

#[test]
fn previews_commits_and_exports_white_balance_through_core() {
    let workspace = unique_library_root("core-white-balance");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph before preview")
            .is_none()
    );
    assert!(list_photo_history(&created.root_path, &photo_id)
        .expect("read history before preview")
        .items
        .is_empty());

    let preview = preview_white_balance_edit(
        &created.root_path,
        &photo_id,
        silica_edit::WhiteBalance::Custom,
        6500.0,
        20.0,
    )
    .expect("preview white balance")
    .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.white_balance, silica_edit::WhiteBalance::Custom);
    assert_eq!(preview.temperature, 6500.0);
    assert_eq!(preview.tint, 20.0);
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));

    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after preview")
            .is_none(),
        "white balance preview must not write edit state"
    );
    assert!(
        list_photo_history(&created.root_path, &photo_id)
            .expect("read history after preview")
            .items
            .is_empty(),
        "white balance preview must not write edit history"
    );

    let committed = commit_white_balance_edit(
        &created.root_path,
        &photo_id,
        silica_edit::WhiteBalance::Custom,
        6500.0,
        20.0,
    )
    .expect("commit white balance")
    .expect("commit result");
    assert_eq!(committed.white_balance, silica_edit::WhiteBalance::Custom);
    assert_eq!(committed.temperature, 6500.0);
    assert_eq!(committed.tint, 20.0);
    assert!(committed.persisted);

    let persisted =
        silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
            .expect("load active graph")
            .expect("active graph");
    assert_eq!(
        persisted.basic.white_balance,
        silica_edit::WhiteBalance::Custom
    );
    assert_eq!(persisted.basic.temperature.as_f64(), Some(6500.0));
    assert_eq!(persisted.basic.tint.as_f64(), Some(20.0));

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].label, "White balance");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    assert!(exported.bytes_written > 0);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["white_balance"], "custom");
    assert_eq!(settings["temperature"], 6500.0);
    assert_eq!(settings["tint"], 20.0);

    remove_library_root(&workspace);
}

#[test]
fn previews_commits_and_exports_tone_recovery_through_core() {
    let workspace = unique_library_root("core-tone-recovery");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let preview =
        preview_tone_recovery_edit(&created.root_path, &photo_id, -35.0, 42.0, 10.0, -12.0)
            .expect("preview tone recovery")
            .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.highlights, -35.0);
    assert_eq!(preview.shadows, 42.0);
    assert_eq!(preview.whites, 10.0);
    assert_eq!(preview.blacks, -12.0);
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after preview")
            .is_none(),
        "tone recovery preview must not write edit state"
    );

    let committed =
        commit_tone_recovery_edit(&created.root_path, &photo_id, -35.0, 42.0, 10.0, -12.0)
            .expect("commit tone recovery")
            .expect("commit result");
    assert_eq!(committed.highlights, -35.0);
    assert_eq!(committed.shadows, 42.0);
    assert_eq!(committed.whites, 10.0);
    assert_eq!(committed.blacks, -12.0);
    assert!(committed.persisted);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].label, "Tone recovery");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    assert!(exported.bytes_written > 0);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["highlights"], -35.0);
    assert_eq!(settings["shadows"], 42.0);
    assert_eq!(settings["whites"], 10.0);
    assert_eq!(settings["blacks"], -12.0);

    remove_library_root(&workspace);
}

#[test]
fn previews_commits_and_exports_tone_curve_through_core() {
    let workspace = unique_library_root("core-tone-curve");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    let rgb_curve = [(0.0, 0.0), (0.5, 0.28), (1.0, 1.0)];

    let preview = preview_tone_curve_edit(&created.root_path, &photo_id, &rgb_curve, &[], &[], &[])
        .expect("preview tone curve")
        .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.tone_curve.curve_mode, silica_edit::CurveMode::Point);
    assert_eq!(preview.tone_curve.rgb_curve.len(), 3);
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after preview")
            .is_none(),
        "tone curve preview must not write edit state"
    );

    let committed =
        commit_tone_curve_edit(&created.root_path, &photo_id, &rgb_curve, &[], &[], &[])
            .expect("commit tone curve")
            .expect("commit result");
    assert_eq!(
        committed.tone_curve.curve_mode,
        silica_edit::CurveMode::Point
    );
    assert_eq!(committed.tone_curve.rgb_curve[1].y, 0.28);
    assert!(committed.persisted);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].label, "Tone curve");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    assert!(exported.bytes_written > 0);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["tone_curve"]["curve_mode"], "point");
    assert_eq!(settings["tone_curve"]["rgb_curve"][1]["x"], 0.5);
    assert_eq!(settings["tone_curve"]["rgb_curve"][1]["y"], 0.28);

    remove_library_root(&workspace);
}

#[test]
fn previews_commits_and_exports_hsl_color_mixer_through_core() {
    let workspace = unique_library_root("core-hsl-color-mixer");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let preview = preview_hsl_color_mixer_edit(
        &created.root_path,
        &photo_id,
        silica_edit::HslColorChannel::Blue,
        -12.0,
        24.0,
        -8.5,
    )
    .expect("preview hsl color mixer")
    .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.hsl_color_mixer.blue.hue, -12.0);
    assert_eq!(preview.hsl_color_mixer.blue.saturation, 24.0);
    assert_eq!(preview.hsl_color_mixer.blue.luminance, -8.5);
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after preview")
            .is_none(),
        "HSL color mixer preview must not write edit state"
    );

    let committed = commit_hsl_color_mixer_edit(
        &created.root_path,
        &photo_id,
        silica_edit::HslColorChannel::Blue,
        -12.0,
        24.0,
        -8.5,
    )
    .expect("commit hsl color mixer")
    .expect("commit result");
    assert_eq!(committed.hsl_color_mixer.blue.hue, -12.0);
    assert_eq!(committed.hsl_color_mixer.blue.saturation, 24.0);
    assert_eq!(committed.hsl_color_mixer.blue.luminance, -8.5);
    assert!(committed.persisted);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].label, "HSL color mixer");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    assert!(exported.bytes_written > 0);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["hsl_color_mixer"]["blue"]["hue"], -12.0);
    assert_eq!(settings["hsl_color_mixer"]["blue"]["saturation"], 24.0);
    assert_eq!(settings["hsl_color_mixer"]["blue"]["luminance"], -8.5);

    remove_library_root(&workspace);
}

#[test]
fn blocks_detail_preview_commit_and_export_until_renderer_support_exists() {
    let workspace = unique_library_root("core-detail-boundary");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let preview =
        preview_detail_sharpening_edit(&created.root_path, &photo_id, 42.0, 1.2, 35.0, 10.0)
            .expect("preview detail sharpening")
            .expect("preview result");
    assert_eq!(preview.status, PhotoPreviewStatus::Unsupported);
    assert_eq!(preview.detail.sharpening.amount, 42.0);
    assert!(preview.develop_preview_bytes.is_none());
    assert!(preview.message.contains("Detail"));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after blocked detail preview")
            .is_none(),
        "blocked detail preview must not write edit state"
    );

    let commit_error =
        commit_detail_sharpening_edit(&created.root_path, &photo_id, 42.0, 1.2, 35.0, 10.0)
            .expect_err("detail commit unsupported");
    assert!(matches!(commit_error, CoreError::UnsupportedEdit(_)));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after blocked detail commit")
            .is_none(),
        "blocked detail commit must not persist an edit graph"
    );

    let graph = silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
        .expect("load default graph")
        .expect("default graph");
    let detail_graph =
        silica_edit::apply_detail_sharpening(&graph, 42.0, 1.2, 35.0, 10.0, "unix:detail")
            .expect("build detail graph");
    silica_storage::commit_edit_graph(&created.root_path, detail_graph)
        .expect("seed unsupported committed detail state");
    let export_error = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect_err("active detail export unsupported");
    assert!(matches!(export_error, CoreError::ExportBlocked(_)));
    assert!(!output_path.exists());

    remove_library_root(&workspace);
}

#[test]
fn previews_commits_and_exports_geometry_through_core() {
    let workspace = unique_library_root("core-geometry");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_geometry_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    let orientation_before = get_photo_metadata(&created.root_path, &photo_id)
        .expect("read metadata before")
        .expect("metadata before")
        .orientation;

    let preview =
        preview_geometry_crop_edit(&created.root_path, &photo_id, 0.0, 0.0, 0.5, 1.0, 0.0, None)
            .expect("preview geometry crop")
            .expect("preview result");
    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(
        preview.geometry.crop.as_ref().map(|crop| crop.width),
        Some(0.5)
    );
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after preview")
            .is_none(),
        "geometry preview must not write edit state"
    );

    let crop_commit =
        commit_geometry_crop_edit(&created.root_path, &photo_id, 0.0, 0.0, 0.5, 1.0, 0.0, None)
            .expect("commit geometry crop")
            .expect("crop commit");
    assert!(crop_commit.persisted);
    assert_eq!(
        crop_commit.geometry.crop.as_ref().map(|crop| crop.height),
        Some(1.0)
    );

    let orientation_preview =
        preview_geometry_orientation_edit(&created.root_path, &photo_id, 90.0, true, false)
            .expect("preview geometry orientation")
            .expect("orientation preview");
    assert_eq!(orientation_preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(orientation_preview.geometry.rotation, 90.0);
    assert!(orientation_preview.geometry.flip_horizontal);

    let orientation_commit =
        commit_geometry_orientation_edit(&created.root_path, &photo_id, 90.0, true, false)
            .expect("commit geometry orientation")
            .expect("orientation commit");
    assert_eq!(orientation_commit.geometry.rotation, 90.0);
    assert!(orientation_commit.geometry.flip_horizontal);
    assert!(orientation_commit.persisted);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 2);
    assert_eq!(history.items[0].label, "Geometry orientation");
    assert_eq!(history.items[1].label, "Geometry crop");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export geometry photo")
        .expect("export result");
    let decoded = image::ImageReader::open(&exported.output_path)
        .expect("open geometry export")
        .with_guessed_format()
        .expect("guess geometry export")
        .decode()
        .expect("decode geometry export");
    assert_eq!(decoded.width(), 3);
    assert_eq!(decoded.height(), 2);
    assert_original_hash(&jpeg_file, &original_hash, "geometry preview/export");
    let orientation_after = get_photo_metadata(&created.root_path, &photo_id)
        .expect("read metadata after")
        .expect("metadata after")
        .orientation;
    assert_eq!(orientation_after, orientation_before);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["geometry"]["crop"]["width"], 0.5);
    assert_eq!(settings["geometry"]["rotation"], 90.0);
    assert_eq!(settings["geometry"]["flip_horizontal"], true);

    remove_library_root(&workspace);
}

#[test]
fn blocks_unsupported_lens_and_geometry_export_states() {
    let workspace = unique_library_root("core-unsupported-geometry");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let lens_output_path = export_root.join("lens-export.jpg");
    let transform_output_path = export_root.join("transform-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_geometry_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let graph = silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
        .expect("load default graph")
        .expect("default graph");
    let lens_graph =
        silica_edit::apply_lens_adjustments(&graph, true, false, 0.0, 0.0, "unix:lens")
            .expect("build unsupported lens graph");
    silica_storage::commit_edit_graph(&created.root_path, lens_graph)
        .expect("seed unsupported lens state");
    let lens_error = export_photo_jpeg_srgb(&created.root_path, &photo_id, &lens_output_path)
        .expect_err("active lens export unsupported");
    assert!(matches!(lens_error, CoreError::ExportBlocked(_)));
    assert!(!lens_output_path.exists());

    let transform_graph = silica_edit::apply_geometry_transform(
        &graph,
        0.0,
        0.0,
        0.0,
        125.0,
        0.0,
        0.0,
        "unix:transform",
    )
    .expect("build unsupported transform graph");
    silica_storage::commit_edit_graph(&created.root_path, transform_graph)
        .expect("seed unsupported transform state");
    let transform_error =
        export_photo_jpeg_srgb(&created.root_path, &photo_id, &transform_output_path)
            .expect_err("active transform export unsupported");
    assert!(matches!(transform_error, CoreError::ExportBlocked(_)));
    assert!(!transform_output_path.exists());

    remove_library_root(&workspace);
}

#[test]
fn previews_commits_and_exports_color_presence_through_core() {
    let workspace = unique_library_root("core-color-presence");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let preview = preview_color_presence_edit(&created.root_path, &photo_id, 24.0, -8.5)
        .expect("preview color presence")
        .expect("preview result");

    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_eq!(preview.vibrance, 24.0);
    assert_eq!(preview.saturation, -8.5);
    assert!(preview
        .develop_preview_bytes
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2));
    assert!(
        silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
            .expect("load active graph after preview")
            .is_none(),
        "color presence preview must not write edit state"
    );

    let committed = commit_color_presence_edit(&created.root_path, &photo_id, 24.0, -8.5)
        .expect("commit color presence")
        .expect("commit result");
    assert_eq!(committed.vibrance, 24.0);
    assert_eq!(committed.saturation, -8.5);
    assert!(committed.persisted);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].label, "Color presence");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    assert!(exported.bytes_written > 0);

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["vibrance"], 24.0);
    assert_eq!(settings["saturation"], -8.5);

    remove_library_root(&workspace);
}

#[test]
fn reset_and_basic_preset_commits_are_undoable_without_mutating_original() {
    let workspace = unique_library_root("core-basic-preset-reset");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let preset = commit_basic_preset_edit(
        &created.root_path,
        &photo_id,
        silica_edit::BasicPreset::WarmContrast,
    )
    .expect("commit preset")
    .expect("preset result");
    assert_eq!(preset.white_balance, silica_edit::WhiteBalance::Custom);
    assert_eq!(preset.temperature, 6200.0);
    assert_eq!(preset.contrast, 18.0);
    assert_eq!(preset.vibrance, 12.0);
    assert!(preset.persisted);

    let reset = commit_p0_basic_reset(&created.root_path, &photo_id)
        .expect("commit reset")
        .expect("reset result");
    assert_eq!(reset.white_balance, silica_edit::WhiteBalance::AsShot);
    assert_eq!(reset.temperature, 5200.0);
    assert_eq!(reset.tint, 0.0);
    assert_eq!(reset.exposure, 0.0);
    assert_eq!(reset.contrast, 0.0);
    assert_eq!(reset.highlights, 0.0);
    assert_eq!(reset.shadows, 0.0);
    assert_eq!(reset.whites, 0.0);
    assert_eq!(reset.blacks, 0.0);
    assert_eq!(reset.vibrance, 0.0);
    assert_eq!(reset.saturation, 0.0);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 2);
    assert!(history.can_undo);

    undo_last_history_action(&created.root_path, &photo_id).expect("undo reset");
    let restored = get_photo_edit_state(&created.root_path, &photo_id)
        .expect("read restored preset")
        .expect("edit state");
    assert_eq!(restored.temperature, 6200.0);
    assert_eq!(restored.contrast, 18.0);
    assert_eq!(restored.vibrance, 12.0);
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );

    remove_library_root(&workspace);
}

#[test]
fn sensitive_core_actions_append_action_log_entries() {
    let workspace = unique_library_root("core-action-log");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    write_photo_sidecar(&created.root_path, &photo_id, "test")
        .expect("write sidecar")
        .expect("sidecar result");
    export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    clear_library_cache(&created.root_path).expect("clear cache");

    let entries = list_action_log_entries(&created.root_path, 20).expect("list action log");
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "import_reference"
            && entry.side_effect_category == "catalog_reference"));
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "sidecar_write"
            && entry.side_effect_category == "sidecar_write"
            && entry.subject_id.as_deref() == Some(photo_id.as_str())));
    assert!(entries.iter().any(|entry| entry.action_type == "export"
        && entry.side_effect_category == "file_write"
        && entry.subject_id.as_deref() == Some(photo_id.as_str())));
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "cache_clear"
            && entry.side_effect_category == "cache_delete"));

    remove_library_root(&workspace);
}

#[test]
fn permissioned_extension_actions_append_action_log_entries_without_state_mutation() {
    let workspace = unique_library_root("core-permission-action-log");
    let library_root = workspace.join("SilicaRAW Library");
    let created = create_library(&library_root).expect("create library");
    let before = durable_catalog_counts(&created.catalog_path);

    record_permission_decision(
        &created.root_path,
        "plugin",
        Some("preset-pack"),
        ExtensionPermission::MetadataRead,
        true,
        "prompt-approved",
    )
    .expect("record permission grant");
    record_permission_decision(
        &created.root_path,
        "plugin",
        Some("preset-pack"),
        ExtensionPermission::FilesystemLimitedWrite,
        false,
        "prompt-denied",
    )
    .expect("record permission denial");
    record_plugin_apply_attempt(
        &created.root_path,
        "preset-pack",
        Some("photo-1"),
        ExtensionPermission::EditSuggestionApply,
    )
    .expect("record plugin apply");
    record_ai_approval(
        &created.root_path,
        "quality-model",
        Some("photo-1"),
        ExtensionPermission::AiResultPropose,
    )
    .expect("record ai approval");
    record_mcp_read(
        &created.root_path,
        "session-1",
        "photo",
        Some("photo-1"),
        ExtensionPermission::McpReadOnly,
    )
    .expect("record mcp read");
    record_permissioned_export_attempt(
        &created.root_path,
        "mcp",
        Some("session-1"),
        Some("photo-1"),
        ExtensionPermission::ExportLocal,
        "/tmp/silicaraw-export.jpg",
    )
    .expect("record export attempt");

    let after = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after.edit_states, before.edit_states);
    assert_eq!(after.edit_history, before.edit_history);
    assert_eq!(after.exports, before.exports);
    assert_eq!(after.cache_records, before.cache_records);
    assert_eq!(after.action_log, before.action_log + 6);

    let entries = list_action_log_entries(&created.root_path, 20).expect("list action log");
    for action_type in [
        "permission_grant",
        "permission_denial",
        "plugin_apply",
        "ai_approval",
        "mcp_read",
        "export_attempt",
    ] {
        assert!(
            entries.iter().any(|entry| entry.action_type == action_type),
            "missing permissioned action log entry {action_type}"
        );
    }
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "permission_denial"
            && entry.side_effect_category == "permission_decision"
            && entry.payload_json.contains("filesystem:limited_write")
            && entry.payload_json.contains("prompt-denied")));
    assert!(entries.iter().any(|entry| entry.action_type == "mcp_read"
        && entry.actor_type == "mcp"
        && entry.side_effect_category == "catalog_read"));

    remove_library_root(&workspace);
}

#[test]
fn plugin_preset_approval_commits_history_and_logs_apply_without_original_mutation() {
    let workspace = unique_library_root("core-plugin-preset-approval");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let approval = approve_plugin_preset(
        &created.root_path,
        &photo_id,
        r#"{
              "schema":"silica.plugin",
              "version":1,
              "plugin_id":"silicaraw.test.preset_pack",
              "name":"Silica Test Presets",
              "description":"Data-only preset pack manifest.",
              "author":"SilicaRAW",
              "license":"MIT",
              "plugin_version":"0.1.0",
              "minimum_silica_version":"0.1.0",
              "type":"preset_pack",
              "permissions":["edit_suggestion:apply"]
            }"#,
        r#"{
              "schema":"silica.plugin_preset_pack",
              "version":1,
              "plugin_id":"silicaraw.test.preset_pack",
              "presets":[
                {
                  "preset_id":"warm_skin",
                  "name":"Warm Skin",
                  "description":"Warm basic adjustments.",
                  "basic":{
                    "white_balance":"custom",
                    "temperature":6100,
                    "tint":4,
                    "exposure":0.35,
                    "contrast":12,
                    "highlights":-18,
                    "shadows":14,
                    "whites":8,
                    "blacks":-6,
                    "vibrance":10,
                    "saturation":3
                  }
                }
              ]
            }"#,
        "warm_skin",
    )
    .expect("approve plugin preset")
    .expect("plugin preset approval result");

    assert_eq!(approval.plugin_id, "silicaraw.test.preset_pack");
    assert_eq!(approval.preset_id, "warm_skin");
    assert_eq!(approval.commit.photo_id, photo_id);
    assert_eq!(approval.commit.exposure, 0.35);
    assert_eq!(approval.commit.contrast, 12.0);
    assert!(approval.writes_edit_graph);
    assert!(!approval.writes_photo_flags);
    assert!(!approval.writes_original);

    let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
    assert_eq!(history.items.len(), 1);
    assert!(history.can_undo);

    let entries = list_action_log_entries(&created.root_path, 20).expect("list action log");
    let entry = entries
        .iter()
        .find(|entry| entry.action_type == "plugin_apply")
        .expect("plugin apply action log entry");
    assert_eq!(entry.actor_type, "plugin");
    assert_eq!(
        entry.actor_id.as_deref(),
        Some("silicaraw.test.preset_pack")
    );
    assert!(entry.payload_json.contains("\"granted\":true"));
    assert!(entry.payload_json.contains("edit_suggestion:apply"));
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );

    remove_library_root(&workspace);
}

#[test]
fn plugin_permission_review_logs_grants_and_denials_without_state_mutation() {
    let workspace = unique_library_root("core-plugin-permission-review");
    let library_root = workspace.join("SilicaRAW Library");
    let created = create_library(&library_root).expect("create library");
    let before = durable_catalog_counts(&created.catalog_path);
    let manifest_json = r#"{
          "schema":"silica.plugin",
          "version":1,
          "plugin_id":"silicaraw.test.preset_pack",
          "name":"Silica Test Presets",
          "description":"Data-only preset pack manifest.",
          "author":"SilicaRAW",
          "license":"MIT",
          "plugin_version":"0.1.0",
          "minimum_silica_version":"0.1.0",
          "type":"preset_pack",
          "permissions":["edit_suggestion:apply"]
        }"#;

    let enable_review = review_plugin_enable_permission(
        &created.root_path,
        manifest_json,
        ExtensionPermission::EditSuggestionApply,
        true,
        "user-approved-enable-review",
    )
    .expect("review plugin enable permission");
    let apply_denial = review_plugin_apply_permission(
        &created.root_path,
        manifest_json,
        Some("photo-1"),
        Some("warm_skin"),
        false,
        "user-denied-apply-review",
    )
    .expect("review plugin apply denial");

    assert_eq!(enable_review.plugin_id, "silicaraw.test.preset_pack");
    assert!(enable_review.granted);
    assert!(!enable_review.runtime_started);
    assert!(!enable_review.permission_persisted);
    assert!(!apply_denial.granted);
    assert!(!apply_denial.writes_edit_graph);
    assert!(!apply_denial.writes_original);

    let after = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after.edit_states, before.edit_states);
    assert_eq!(after.edit_history, before.edit_history);
    assert_eq!(after.exports, before.exports);
    assert_eq!(after.cache_records, before.cache_records);
    assert_eq!(after.action_log, before.action_log + 2);

    let entries = list_action_log_entries(&created.root_path, 20).expect("list action log");
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "permission_grant"
            && entry.actor_type == "plugin"
            && entry.payload_json.contains("plugin_enable")
            && entry.payload_json.contains("user-approved-enable-review")));
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "plugin_apply"
            && entry.actor_type == "plugin"
            && entry.payload_json.contains("\"granted\":false")
            && entry.payload_json.contains("user-denied-apply-review")));

    remove_library_root(&workspace);
}

#[test]
fn ai_result_store_flows_through_core_without_edit_or_flag_mutation() {
    let workspace = unique_library_root("core-ai-result-store");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    let flags_before: i64 = connection
        .query_row("SELECT COUNT(*) FROM photo_flags", [], |row| row.get(0))
        .expect("count flags");
    drop(connection);

    let before = durable_catalog_counts(&created.catalog_path);
    let result = store_ai_result(
        &created.root_path,
        &photo_id,
        "blur_score",
        "silicaraw.blur-review.test",
        r#"{"review":{"score":0.25,"label":"usable"}}"#,
    )
    .expect("store ai result");

    assert!(!result.approved);
    let payload: serde_json::Value =
        serde_json::from_str(&result.result_json).expect("parse ai result payload");
    assert_eq!(payload["local_only"], true);
    assert_eq!(payload["permission_id"], "ai_result:propose");

    let results =
        list_ai_results_for_photo(&created.root_path, &photo_id, 10).expect("list ai results");
    assert_eq!(results, vec![result]);

    let after = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after.edit_states, before.edit_states);
    assert_eq!(after.edit_history, before.edit_history);
    assert_eq!(after.exports, before.exports);
    assert_eq!(after.cache_records, before.cache_records);
    assert_eq!(after.ai_results, before.ai_results + 1);

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let flags_after: i64 = connection
        .query_row("SELECT COUNT(*) FROM photo_flags", [], |row| row.get(0))
        .expect("count flags after");
    assert_eq!(flags_after, flags_before);

    remove_library_root(&workspace);
}

#[test]
fn ai_blur_review_panel_reads_result_without_edit_or_original_mutation() {
    let workspace = unique_library_root("core-ai-review-panel");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let before = durable_catalog_counts(&created.catalog_path);
    store_ai_result(
        &created.root_path,
        &photo_id,
        "blur_score",
        "silicaraw.blur-review.test",
        r#"{"review":{"label":"Motion blur likely","recommendation":"review","confidence":0.91}}"#,
    )
    .expect("store blur review result");

    let panel = get_ai_review_panel(&created.root_path, &photo_id).expect("read ai review");

    assert_eq!(panel.status, AiReviewPanelStatus::ReviewAvailable);
    assert_eq!(panel.task_type, "blur_score");
    assert!(!panel.writes_edit_graph);
    assert!(!panel.writes_photo_flags);
    assert!(panel.requires_explicit_approval);
    assert_eq!(panel.items.len(), 1);
    assert_eq!(panel.items[0].label, "Motion blur likely");
    assert_eq!(panel.items[0].recommendation, "review");
    assert_eq!(panel.items[0].confidence_percent, Some(91));
    assert!(!panel.items[0].approved);

    let after = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after.edit_states, before.edit_states);
    assert_eq!(after.edit_history, before.edit_history);
    assert_eq!(after.action_log, before.action_log);
    assert_eq!(after.exports, before.exports);
    assert_eq!(after.cache_records, before.cache_records);
    assert_eq!(after.ai_results, before.ai_results + 1);
    assert_original_hash(&jpeg_file, &original_hash, "AI blur review panel read");

    remove_library_root(&workspace);
}

#[test]
fn ai_blur_review_panel_is_unavailable_but_editor_usable_when_model_missing() {
    let workspace = unique_library_root("core-ai-review-missing-model");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let before = durable_catalog_counts(&created.catalog_path);
    let panel =
        get_ai_review_panel(&created.root_path, &photo_id).expect("read missing review panel");

    assert_eq!(panel.status, AiReviewPanelStatus::ModelUnavailable);
    assert!(panel.items.is_empty());
    assert!(panel.editor_remains_usable);
    assert_eq!(
        panel.message,
        "No local blur review model or stored result is available."
    );
    assert!(!panel.writes_edit_graph);
    assert!(!panel.writes_photo_flags);
    assert_eq!(durable_catalog_counts(&created.catalog_path), before);
    assert_original_hash(&jpeg_file, &original_hash, "missing AI review model");

    remove_library_root(&workspace);
}

#[test]
fn ai_suggestion_approval_commits_exposure_contrast_with_provenance() {
    let workspace = unique_library_root("core-ai-suggestion-approval");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let before = durable_catalog_counts(&created.catalog_path);
    let result = store_ai_result(
        &created.root_path,
        &photo_id,
        "blur_score",
        "silicaraw.blur-review.test",
        r#"{"review":{"label":"Usable detail","recommendation":"keep","confidence":0.74},"approval_suggestion":{"kind":"basic_exposure_contrast","exposure":0.35,"contrast":9.0,"summary":"Apply conservative clarity compensation."}}"#,
    )
    .expect("store approvable ai result");

    let approval = approve_ai_suggestion(&created.root_path, &photo_id, &result.id)
        .expect("approve ai suggestion")
        .expect("approval outcome");

    assert_eq!(approval.photo_id, photo_id);
    assert_eq!(approval.result_id, result.id);
    assert_eq!(approval.model_id, "silicaraw.blur-review.test");
    assert!(approval.commit.persisted);
    assert_eq!(approval.commit.exposure, 0.35);
    assert_eq!(approval.commit.contrast, 9.0);

    let after = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after.edit_states, before.edit_states + 1);
    assert_eq!(after.edit_history, before.edit_history + 1);
    assert_eq!(after.action_log, before.action_log + 1);
    assert_eq!(after.ai_results, before.ai_results + 1);

    let listed =
        list_ai_results_for_photo(&created.root_path, &photo_id, 10).expect("list ai results");
    assert!(listed[0].approved);

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let action_json: String = connection
        .query_row(
            "SELECT action_json FROM edit_history WHERE photo_id = ?1 ORDER BY sequence DESC LIMIT 1",
            [&photo_id],
            |row| row.get(0),
        )
        .expect("read history action");
    let action: serde_json::Value =
        serde_json::from_str(&action_json).expect("parse history action");
    assert_eq!(
        action["after"]["edit_graph"]["extensions"]["silica.ai_provenance"]["result_id"],
        result.id
    );
    assert_eq!(
        action["after"]["edit_graph"]["extensions"]["silica.ai_provenance"]["task_type"],
        "blur_score"
    );
    drop(connection);

    let entries = list_action_log_entries(&created.root_path, 10).expect("list action log");
    assert!(entries
        .iter()
        .any(|entry| entry.action_type == "ai_approval"
            && entry.subject_id.as_deref() == Some(photo_id.as_str())
            && entry.payload_json.contains(&result.id)
            && entry.payload_json.contains("edit_suggestion:apply")));

    undo_last_history_action(&created.root_path, &photo_id).expect("undo ai approval");
    let undone_graph =
        silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
            .expect("load undone graph")
            .expect("active graph");
    assert_eq!(undone_graph.basic.exposure.as_f64().unwrap_or(0.0), 0.0);
    assert_eq!(undone_graph.basic.contrast.as_f64().unwrap_or(0.0), 0.0);
    assert_original_hash(&jpeg_file, &original_hash, "AI suggestion approval");

    remove_library_root(&workspace);
}

#[test]
fn ai_suggestion_rejection_leaves_edit_state_unchanged() {
    let workspace = unique_library_root("core-ai-suggestion-rejection");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let result = store_ai_result(
        &created.root_path,
        &photo_id,
        "blur_score",
        "silicaraw.blur-review.test",
        r#"{"review":{"label":"Motion blur likely","recommendation":"review","confidence":0.91},"approval_suggestion":{"kind":"basic_exposure_contrast","exposure":0.2,"contrast":6.0}}"#,
    )
    .expect("store rejectable ai result");
    let before = durable_catalog_counts(&created.catalog_path);

    let rejection = reject_ai_suggestion(&created.root_path, &photo_id, &result.id)
        .expect("reject ai suggestion")
        .expect("rejection outcome");

    assert_eq!(rejection.photo_id, photo_id);
    assert_eq!(rejection.result_id, result.id);
    let after = durable_catalog_counts(&created.catalog_path);
    assert_eq!(after.edit_states, before.edit_states);
    assert_eq!(after.edit_history, before.edit_history);
    assert_eq!(after.ai_results, before.ai_results);
    assert_eq!(after.action_log, before.action_log + 1);

    let listed =
        list_ai_results_for_photo(&created.root_path, &photo_id, 10).expect("list ai results");
    assert!(!listed[0].approved);
    let graph = silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
        .expect("load graph")
        .expect("default graph");
    assert_eq!(graph.basic.exposure.as_f64().unwrap_or(0.0), 0.0);
    assert_eq!(graph.basic.contrast.as_f64().unwrap_or(0.0), 0.0);
    assert_original_hash(&jpeg_file, &original_hash, "AI suggestion rejection");

    remove_library_root(&workspace);
}

#[test]
fn raw_derived_jpeg_srgb_export_rejects_original_overwrite_before_decode() {
    let workspace = unique_library_root("core-raw-export-overwrite");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let raw_file = import_root.join("sample.cr2");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&raw_file, b"raw placeholder").expect("write raw placeholder");
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.cr2'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    let probe = successful_raw_probe(&raw_file.display().to_string(), Some(5184), Some(3456));

    let error = export_raw_photo_jpeg_srgb_from_probe(
        &created.root_path,
        &photo_id,
        "A",
        &probe,
        &raw_file,
    )
    .expect_err("RAW export cannot overwrite original");

    assert!(matches!(
        error,
        CoreError::RawExport(
            silica_decode::RawFullResolutionExportSourceError::OutputMatchesSource(_)
        )
    ));
    assert_original_hash(&raw_file, &file_hash(&raw_file), "RAW overwrite rejection");

    remove_library_root(&workspace);
}

#[cfg(unix)]
#[test]
fn raw_derived_jpeg_srgb_export_rejects_original_hard_link_before_decode() {
    let workspace = unique_library_root("core-raw-export-hard-link");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let raw_file = import_root.join("sample.cr2");
    let output_path = export_root.join("sample-hard-link.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    std::fs::write(&raw_file, b"raw placeholder").expect("write raw placeholder");
    let original_hash = file_hash(&raw_file);
    std::fs::hard_link(&raw_file, &output_path).expect("create raw source hard link");
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.cr2'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    let probe = successful_raw_probe(&raw_file.display().to_string(), Some(5184), Some(3456));

    let error = export_raw_photo_jpeg_srgb_from_probe(
        &created.root_path,
        &photo_id,
        "A",
        &probe,
        &output_path,
    )
    .expect_err("RAW export cannot overwrite hard-linked original");

    assert!(matches!(
        error,
        CoreError::RawExport(
            silica_decode::RawFullResolutionExportSourceError::OutputMatchesSource(_)
        )
    ));
    assert_original_hash(
        &raw_file,
        &original_hash,
        "RAW hard-link overwrite rejection",
    );

    remove_library_root(&workspace);
}

#[test]
fn raw_derived_jpeg_srgb_export_blocks_committed_manual_masks_before_output() {
    let workspace = unique_library_root("core-raw-export-mask-block");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let raw_file = import_root.join("sample.cr2");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    std::fs::write(&raw_file, b"raw placeholder").expect("write raw placeholder");
    let original_hash = file_hash(&raw_file);
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.cr2'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let draft = silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
        .expect("load RAW proof draft")
        .expect("RAW proof draft");
    let mask = silica_edit::manual_linear_gradient_mask(
        "mask-linear-1",
        "Diagonal lift",
        100.0,
        0.0,
        false,
        0.0,
        0.0,
        1.0,
        1.0,
        silica_edit::ManualMaskLocalAdjustments {
            exposure: Some(1.0),
            contrast: Some(0.0),
        },
    )
    .expect("build RAW proof mask");
    let edited = silica_edit::append_manual_mask(&draft, mask, "unix:raw-proof-mask")
        .expect("append RAW proof mask");
    silica_storage::commit_edit_graph(&created.root_path, edited)
        .expect("commit RAW proof mask graph");
    let probe = successful_raw_probe(&raw_file.display().to_string(), Some(5184), Some(3456));

    let error = export_raw_photo_jpeg_srgb_from_probe(
        &created.root_path,
        &photo_id,
        "A",
        &probe,
        &output_path,
    )
    .expect_err("RAW-derived masked export should block before output");

    assert!(matches!(error, CoreError::ExportBlocked(_)));
    assert!(error.to_string().contains("RAW-derived export"));
    assert!(!output_path.exists());
    assert!(!created
        .root_path
        .join("render-cache")
        .join("raw-export-sources")
        .exists());
    assert_original_hash(&raw_file, &original_hash, "blocked RAW mask export");

    remove_library_root(&workspace);
}

#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
#[test]
#[ignore]
fn raw_derived_jpeg_srgb_export_from_fixture_records_evidence_without_preview_cache() {
    let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
        .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
    let report = silica_decode::probe_raw_fixture_manifest(manifest).expect("probe RAW fixtures");
    let fixture = report
        .results
        .iter()
        .find(|result| result.fixture_class == "A")
        .expect("Class A fixture evidence");
    let raw_path = PathBuf::from(&fixture.probe.source_path);
    let import_root = raw_path.parent().expect("fixture parent");
    let workspace = unique_library_root("core-raw-export-fixture");
    let library_root = workspace.join("SilicaRAW Library");
    let export_root = workspace.join("Exports");
    let baseline_output = export_root.join("baseline.jpg");
    let adjusted_output = export_root.join("adjusted.jpg");

    std::fs::create_dir_all(&export_root).expect("create export directory");
    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, import_root).expect("import RAW fixture folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE path = ?1",
            [&fixture.probe.source_path],
            |row| row.get(0),
        )
        .expect("fixture photo id");
    drop(connection);

    let baseline = export_raw_photo_jpeg_srgb_from_probe(
        &created.root_path,
        &photo_id,
        &fixture.fixture_class,
        &fixture.probe,
        &baseline_output,
    )
    .expect("export baseline RAW photo")
    .expect("baseline export result");
    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit exposure/contrast")
        .expect("commit result");
    let adjusted = export_raw_photo_jpeg_srgb_from_probe(
        &created.root_path,
        &photo_id,
        &fixture.fixture_class,
        &fixture.probe,
        &adjusted_output,
    )
    .expect("export adjusted RAW photo")
    .expect("adjusted export result");

    assert_eq!(
        adjusted.source_sha256.as_deref(),
        fixture.probe.source_sha256.as_deref()
    );
    assert_ne!(baseline.output_sha256, adjusted.output_sha256);
    assert!(adjusted.icc_profile_embedded);
    assert_eq!(adjusted.decoder_backend.as_deref(), Some("core_image_raw"));
    assert_eq!(adjusted.input_profile.as_deref(), Some("core_image_raw"));
    assert_eq!(adjusted.working_space.as_deref(), Some("srgb"));
    assert!(adjusted.output_path.is_file());
    assert_ne!(adjusted.output_path, raw_path);
    assert!(silica_storage::get_photo_cache_record(
        &created.root_path,
        &photo_id,
        silica_storage::PREVIEW_CACHE_TYPE,
    )
    .expect("preview cache lookup")
    .is_none());

    let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(
        settings["source_sha256"],
        fixture.probe.source_sha256.clone().unwrap()
    );
    assert_eq!(settings["output_sha256"], adjusted.output_sha256);
    assert_eq!(settings["icc_profile_embedded"], true);
    assert_eq!(settings["icc_profile_sha256"], adjusted.icc_profile_sha256);
    assert_eq!(settings["decoder_backend"], "core_image_raw");
    assert_eq!(settings["input_profile"], "core_image_raw");
    assert_eq!(settings["working_space"], "srgb");
    assert_eq!(settings["profile_metadata_source"], "silica-export");
    assert_eq!(
        settings["export_source_kind"],
        "raw_full_resolution_artifact"
    );
    assert_eq!(settings["viewer_texture_cache_source"], false);
    assert_eq!(settings["raw_source_original_hash_unchanged"], true);
    let artifact_path = settings["raw_export_source_artifact_path"]
        .as_str()
        .expect("artifact path");
    assert!(artifact_path.contains("render-cache/raw-export-sources"));
    assert!(!artifact_path.contains("/previews/"));

    if let Ok(qa_dir) = std::env::var("SILICARAW_RAW_EXPORT_QA_DIR") {
        let qa_dir = PathBuf::from(qa_dir);
        std::fs::create_dir_all(&qa_dir).expect("create RAW export QA directory");
        let qa_output = qa_dir.join(format!("{}-adjusted-srgb.jpg", fixture.fixture_id));
        std::fs::copy(&adjusted.output_path, &qa_output).expect("copy adjusted QA export");
        let qa_evidence = serde_json::json!({
            "task": "15.6",
            "fixture_id": fixture.fixture_id,
            "fixture_class": fixture.fixture_class,
            "source_path": fixture.probe.source_path,
            "source_sha256": fixture.probe.source_sha256,
            "output_path": qa_output.display().to_string(),
            "output_sha256": adjusted.output_sha256,
            "icc_profile_embedded": adjusted.icc_profile_embedded,
            "icc_profile_sha256": adjusted.icc_profile_sha256,
            "decoder_backend": adjusted.decoder_backend,
            "input_profile": adjusted.input_profile,
            "working_space": adjusted.working_space,
            "export_settings": settings,
        })
        .to_string();
        std::fs::write(qa_dir.join("raw-export-qa-evidence.json"), qa_evidence)
            .expect("write RAW export QA evidence");
    }

    remove_library_root(&workspace);
}

#[test]
fn exports_edited_photo_to_jpeg_display_p3_when_explicit() {
    let workspace = unique_library_root("core-export-display-p3");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-display-p3.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let exported = export_photo_jpeg(
        &created.root_path,
        &photo_id,
        &output_path,
        PhotoExportColorProfile::DisplayP3,
    )
    .expect("export photo")
    .expect("export result");

    assert_eq!(exported.output_path, output_path);
    assert_eq!(exported.format, "jpeg");
    assert_eq!(exported.color_profile, "display_p3");
    assert!(exported.bytes_written > 0);
    assert_eq!(exported.icc_profile_sha256.len(), 64);
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );
    let latest = silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["color_profile"], "display_p3");
    assert_eq!(settings["icc_profile_embedded"], true);
    assert_eq!(settings["icc_profile_sha256"], exported.icc_profile_sha256);

    remove_library_root(&workspace);
}

#[test]
fn export_metadata_policy_removes_gps_and_records_evidence() {
    let workspace = unique_library_root("core-export-metadata-policy");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-remove-gps.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg_with_exif(&jpeg_file);
    let original_before = std::fs::read(&jpeg_file).expect("read original before");

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let exported = export_photo_jpeg_with_metadata_policy(
        &created.root_path,
        &photo_id,
        &output_path,
        PhotoExportColorProfile::Srgb,
        PhotoExportMetadataPolicy::RemoveGps,
    )
    .expect("export photo")
    .expect("export result");

    assert_eq!(exported.format, "jpeg");
    assert_eq!(
        std::fs::read(&jpeg_file).expect("read original after"),
        original_before
    );
    assert!(jpeg_contains_exif_make(&exported.output_path));
    assert!(!jpeg_has_exif_gps_ifd(&exported.output_path));

    let latest = silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
        .expect("read latest export")
        .expect("latest export");
    let settings: serde_json::Value =
        serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
    assert_eq!(settings["metadata_policy"], "remove_gps");
    assert_eq!(settings["source_metadata_segments"], 1);
    assert_eq!(settings["output_metadata_segments"], 1);
    assert_eq!(settings["source_metadata_copied"], true);
    assert_eq!(settings["gps_metadata_removed"], true);

    remove_library_root(&workspace);
}

#[test]
fn recent_exports_report_missing_output_evidence() {
    let workspace = unique_library_root("core-recent-exports");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let existing_output = export_root.join("sample-export.jpg");
    let missing_output = export_root.join("missing-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    std::fs::write(&existing_output, b"export bytes").expect("write export output");

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    silica_storage::record_export(
        &created.root_path,
        &photo_id,
        &existing_output,
        r#"{"format":"jpeg"}"#,
    )
    .expect("record existing export");
    silica_storage::record_export(
        &created.root_path,
        &photo_id,
        &missing_output,
        r#"{"format":"png"}"#,
    )
    .expect("record missing export");

    let recent = list_recent_exports(&created.root_path, 2).expect("list recent exports");

    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].output_path, missing_output.display().to_string());
    assert!(!recent[0].output_exists);
    assert_eq!(recent[1].output_path, existing_output.display().to_string());
    assert!(recent[1].output_exists);
    assert!(!recent[0].created_at.is_empty());

    remove_library_root(&workspace);
}

#[test]
fn export_settings_defaults_and_presets_flow_through_core_without_edit_history() {
    let workspace = unique_library_root("core-export-settings");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");

    let initial_catalog =
        get_export_settings_catalog(&created.root_path).expect("read export settings");
    assert_eq!(
        initial_catalog.default_settings,
        ExportSettings::jpeg_srgb_default()
    );

    let display_p3_settings = ExportSettings {
        color_profile: "display_p3".to_string(),
        ..ExportSettings::jpeg_srgb_default()
    };
    let preset = upsert_export_preset(
        &created.root_path,
        "Core Display P3 Review",
        display_p3_settings.clone(),
    )
    .expect("upsert preset through core");
    let updated_catalog = set_default_export_settings(
        &created.root_path,
        Some(&preset.id),
        display_p3_settings.clone(),
    )
    .expect("set default export settings through core");
    assert_eq!(updated_catalog.default_settings, display_p3_settings);
    assert_eq!(
        updated_catalog.default_preset_id.as_deref(),
        Some(preset.id.as_str())
    );

    let counts = durable_catalog_counts(&created.catalog_path);
    assert_eq!(counts.edit_states, 0);
    assert_eq!(counts.edit_history, 0);

    remove_library_root(&workspace);
}

#[test]
fn writes_and_reads_photo_sidecar_through_core() {
    let workspace = unique_library_root("core-sidecar");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    set_photo_flags(
        &created.root_path,
        photo_id.clone(),
        2,
        true,
        false,
        Some("blue".to_string()),
    )
    .expect("set flags");

    let written = write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write sidecar")
        .expect("sidecar write result");
    assert_eq!(written.photo_id, photo_id);
    assert!(written.sidecar_path.is_file());
    assert_original_hash(&jpeg_file, &original_hash, "core sidecar write");

    let read = read_photo_sidecar(&created.root_path, &photo_id)
        .expect("read sidecar")
        .expect("sidecar exists");
    assert_eq!(read.photo_id, photo_id);
    assert_eq!(read.flags.rating, 2);
    assert_eq!(read.flags.color_label.as_deref(), Some("blue"));
    assert_original_hash(&jpeg_file, &original_hash, "core sidecar read");

    remove_library_root(&workspace);
}

#[test]
fn sidecar_status_after_history_is_exposed_through_core() {
    let workspace = unique_library_root("core-sidecar-status-history");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write sidecar")
        .expect("sidecar result");
    let clean_status = get_photo_sidecar_status(&created.root_path, &photo_id)
        .expect("read clean status")
        .expect("clean status");
    assert_eq!(clean_status.conflict_state, "clean");

    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("commit result");
    let stale_status = get_photo_sidecar_status(&created.root_path, &photo_id)
        .expect("read stale status")
        .expect("stale status");
    assert_eq!(stale_status.conflict_state, "catalog_newer");

    remove_library_root(&workspace);
}

#[test]
fn dry_runs_sidecar_rebuild_through_core_without_mutating_flags() {
    let workspace = unique_library_root("core-sidecar-rebuild");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    set_photo_flags(
        &created.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set sidecar flags");
    write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write sidecar")
        .expect("sidecar write result");
    set_photo_flags(&created.root_path, photo_id.clone(), 1, false, true, None)
        .expect("change live catalog flags");

    let report =
        dry_run_catalog_rebuild_from_sidecars(&created.root_path).expect("dry-run rebuild");

    assert_eq!(report.sidecars_scanned, 1);
    assert!(report.issues.is_empty());
    assert_eq!(report.entries.len(), 1);
    assert_eq!(
        report.entries[0].action,
        CatalogRebuildDryRunAction::UpdatePhotoFlags
    );
    assert_eq!(
        report.entries[0].flag_source,
        CatalogRebuildFlagSource::SidecarFlags
    );
    assert_eq!(report.entries[0].resolved_flags.rating, 5);

    let live_flags = get_photo_flags(&created.root_path, &photo_id)
        .expect("read live flags")
        .expect("live flags");
    assert_eq!(live_flags.rating, 1);
    assert!(!live_flags.picked);
    assert!(live_flags.rejected);

    remove_library_root(&workspace);
}

#[test]
fn local_alpha_workflow_preserves_original_file_hash() {
    let workspace = unique_library_root("core-original-safety");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("Exports");
    let jpeg_file = import_root.join("sample.jpg");
    let output_path = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library through core");
    import_folder(&created.root_path, &import_root).expect("import through core");
    assert_original_hash(&jpeg_file, &original_hash, "import by reference");

    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    set_photo_flags(
        &created.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set flags through core");
    assert_original_hash(&jpeg_file, &original_hash, "rating and pick update");

    let preview = open_photo_preview(&created.root_path, &photo_id)
        .expect("open preview")
        .expect("preview session");
    assert_eq!(preview.status, PhotoPreviewStatus::Ready);
    assert_original_hash(&jpeg_file, &original_hash, "preview open");

    preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("preview edit")
        .expect("preview edit request");
    assert_original_hash(&jpeg_file, &original_hash, "draft edit preview");

    commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
        .expect("commit edit")
        .expect("edit commit");
    assert_original_hash(&jpeg_file, &original_hash, "edit commit");

    let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
        .expect("export photo")
        .expect("export result");
    assert_eq!(exported.source_path, jpeg_file.display().to_string());
    assert_eq!(exported.output_path, output_path);
    assert!(exported.output_path.is_file());
    assert_ne!(exported.output_path, jpeg_file);
    assert_original_hash(&jpeg_file, &original_hash, "JPEG sRGB export");

    let cache_clear = clear_library_cache(&created.root_path).expect("clear library cache");
    assert_eq!(cache_clear.removed_cache_records, 1);
    assert_eq!(
        cache_clear.cleared_directories,
        vec!["thumbnails", "previews", "render-cache", "ai-cache"]
    );
    for directory in &cache_clear.recreated_directories {
        assert!(created.root_path.join(directory).is_dir());
    }
    assert_original_hash(&jpeg_file, &original_hash, "cache directory clear");

    let reopened = open_library(&library_root).expect("reopen library through core");
    assert_original_hash(&jpeg_file, &original_hash, "library restart and reopen");

    let flags = get_photo_flags(&reopened.root_path, &photo_id)
        .expect("read flags")
        .expect("flags row");
    assert_eq!(flags.rating, 5);
    assert!(flags.picked);
    assert!(!flags.rejected);

    let persisted =
        silica_storage::load_active_edit_graph_or_default(&reopened.root_path, &photo_id)
            .expect("load active graph")
            .expect("active graph");
    assert_eq!(persisted.basic.exposure.as_f64(), Some(0.5));
    assert_eq!(persisted.basic.contrast.as_f64(), Some(-8.0));

    let latest = silica_storage::get_latest_export_record(&reopened.root_path, &photo_id)
        .expect("read latest export")
        .expect("latest export");
    assert_eq!(
        latest.output_path,
        exported.output_path.display().to_string()
    );

    remove_library_root(&workspace);
}

fn unique_library_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "silicaraw-core-library-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn remove_library_root(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}

fn file_hash(path: &Path) -> String {
    let bytes = std::fs::read(path).expect("read file for hash");
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn assert_original_hash(path: &Path, expected_hash: &str, stage: &str) {
    assert_eq!(
        file_hash(path),
        expected_hash,
        "original file hash changed after {stage}"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DurableCatalogCounts {
    edit_states: i64,
    edit_history: i64,
    action_log: i64,
    exports: i64,
    cache_records: i64,
    ai_results: i64,
}

fn durable_catalog_counts(catalog_path: &Path) -> DurableCatalogCounts {
    let connection = silica_storage::open_catalog(catalog_path).expect("open catalog");
    DurableCatalogCounts {
        edit_states: connection
            .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
            .expect("count edit states"),
        edit_history: connection
            .query_row("SELECT COUNT(*) FROM edit_history", [], |row| row.get(0))
            .expect("count edit history"),
        action_log: connection
            .query_row("SELECT COUNT(*) FROM action_log", [], |row| row.get(0))
            .expect("count action log"),
        exports: connection
            .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
            .expect("count exports"),
        cache_records: connection
            .query_row("SELECT COUNT(*) FROM cache_records", [], |row| row.get(0))
            .expect("count cache records"),
        ai_results: connection
            .query_row("SELECT COUNT(*) FROM ai_results", [], |row| row.get(0))
            .expect("count ai results"),
    }
}

fn write_source_jpeg(path: &Path) {
    write_source_image(path, image::ImageFormat::Jpeg);
}

fn write_source_image(path: &Path, format: image::ImageFormat) {
    let image = image::RgbImage::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            image::Rgb([64, 128, 192])
        } else {
            image::Rgb([192, 128, 64])
        }
    });
    image
        .save_with_format(path, format)
        .expect("write source image");
}

fn write_source_jpeg_with_exif(path: &Path) {
    write_source_jpeg(path);
    let bytes = std::fs::read(path).expect("read source jpeg");
    let with_exif = insert_app1_exif_segment(&bytes, &minimal_exif_with_gps());
    std::fs::write(path, with_exif).expect("write source jpeg exif");
}

fn insert_app1_exif_segment(jpeg: &[u8], exif: &[u8]) -> Vec<u8> {
    assert!(jpeg.starts_with(&[0xFF, 0xD8]));
    let segment_len = exif.len() + 2;
    let mut output = Vec::with_capacity(jpeg.len() + segment_len + 2);
    output.extend_from_slice(&jpeg[..2]);
    output.extend_from_slice(&[0xFF, 0xE1]);
    output.extend_from_slice(&(segment_len as u16).to_be_bytes());
    output.extend_from_slice(exif);
    output.extend_from_slice(&jpeg[2..]);
    output
}

fn minimal_exif_with_gps() -> Vec<u8> {
    let make_offset = 38_u32;
    let gps_offset = 48_u32;
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II");
    tiff.extend_from_slice(&42_u16.to_le_bytes());
    tiff.extend_from_slice(&8_u32.to_le_bytes());
    tiff.extend_from_slice(&2_u16.to_le_bytes());
    tiff.extend_from_slice(&0x010F_u16.to_le_bytes());
    tiff.extend_from_slice(&2_u16.to_le_bytes());
    tiff.extend_from_slice(&10_u32.to_le_bytes());
    tiff.extend_from_slice(&make_offset.to_le_bytes());
    tiff.extend_from_slice(&0x8825_u16.to_le_bytes());
    tiff.extend_from_slice(&4_u16.to_le_bytes());
    tiff.extend_from_slice(&1_u32.to_le_bytes());
    tiff.extend_from_slice(&gps_offset.to_le_bytes());
    tiff.extend_from_slice(&0_u32.to_le_bytes());
    tiff.extend_from_slice(b"SilicaCam\0");
    tiff.extend_from_slice(&1_u16.to_le_bytes());
    tiff.extend_from_slice(&1_u16.to_le_bytes());
    tiff.extend_from_slice(&2_u16.to_le_bytes());
    tiff.extend_from_slice(&2_u32.to_le_bytes());
    tiff.extend_from_slice(b"N\0\0\0");
    tiff.extend_from_slice(&0_u32.to_le_bytes());

    let mut exif = b"Exif\0\0".to_vec();
    exif.extend_from_slice(&tiff);
    exif
}

fn jpeg_contains_exif_make(path: &Path) -> bool {
    std::fs::read(path)
        .expect("read jpeg")
        .windows(b"SilicaCam".len())
        .any(|window| window == b"SilicaCam")
}

fn jpeg_has_exif_gps_ifd(path: &Path) -> bool {
    let bytes = std::fs::read(path).expect("read jpeg");
    bytes.windows(2).any(|window| window == [0x25, 0x88])
        || bytes.windows(2).any(|window| window == [0x88, 0x25])
}

fn write_geometry_source_jpeg(path: &Path) {
    let image = image::RgbImage::from_fn(4, 3, |x, y| {
        image::Rgb([
            (32 + (x * 40)) as u8,
            (48 + (y * 50)) as u8,
            (96 + ((x + y) * 10)) as u8,
        ])
    });
    image
        .save_with_format(path, image::ImageFormat::Jpeg)
        .expect("write geometry source jpeg");
}

fn successful_raw_probe(
    source_path: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> silica_decode::RawProbeResult {
    silica_decode::RawProbeResult {
        backend: silica_decode::RawProbeBackend::CoreImageRaw,
        platform: silica_decode::RawProbePlatform::Macos,
        macos_version: Some("26.4".to_string()),
        source_path: source_path.to_string(),
        source_sha256: Some(file_hash(Path::new(source_path))),
        original_file_size: Some(1024),
        original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
        status: silica_decode::RawProbeStatus::Success,
        width,
        height,
        orientation: None,
        error_category: None,
        message: "Core Image opened the RAW source.".to_string(),
    }
}
