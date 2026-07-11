use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exposes_crate_name() {
    assert_eq!(super::CRATE_NAME, "silica-export");
}

#[test]
fn exports_jpeg_srgb_without_mutating_original() {
    let root = unique_export_root("jpeg");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("edited.jpg");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        exposure: 0.5,
        contrast: -8.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
    })
    .expect("export jpeg srgb");

    assert_eq!(result.output_path, output_path);
    assert_eq!(result.format, super::ExportImageFormat::Jpeg);
    assert_eq!(result.color_profile, super::ExportColorProfile::Srgb);
    assert!(result.bytes_written > 0);
    assert!(result.icc_profile_embedded);
    assert_eq!(
        result.output_sha256,
        super::sha256_file(&result.output_path).expect("hash exported jpeg")
    );
    assert_eq!(
        result.source_sha256,
        super::sha256_file(&source_path).expect("hash source jpeg")
    );
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    let inspection =
        super::inspect_jpeg_icc_profile(&result.output_path).expect("inspect exported ICC");
    assert!(inspection.embedded);
    assert_eq!(
        inspection.color_profile,
        Some(super::ExportColorProfile::Srgb)
    );
    assert_eq!(
        inspection.icc_profile_sha256.as_deref(),
        Some(result.icc_profile_sha256.as_str())
    );

    let exported = image::ImageReader::open(&result.output_path)
        .expect("open exported jpeg")
        .with_guessed_format()
        .expect("guess jpeg format")
        .decode()
        .expect("decode exported jpeg");
    assert_eq!(exported.width(), 2);
    assert_eq!(exported.height(), 2);

    remove_export_root(&root);
}

#[test]
fn exports_display_p3_jpeg_only_when_explicitly_requested() {
    let root = unique_export_root("display-p3");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("display-p3.jpg");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::DisplayP3,
    })
    .expect("export display p3 jpeg");

    assert_eq!(result.output_path, output_path);
    assert_eq!(result.format, super::ExportImageFormat::Jpeg);
    assert_eq!(result.color_profile, super::ExportColorProfile::DisplayP3);
    assert!(result.icc_profile_embedded);
    assert_eq!(
        result.output_sha256,
        super::sha256_file(&result.output_path).expect("hash exported jpeg")
    );
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    let inspection =
        super::inspect_jpeg_icc_profile(&result.output_path).expect("inspect exported ICC");
    assert!(inspection.embedded);
    assert_eq!(
        inspection.color_profile,
        Some(super::ExportColorProfile::DisplayP3)
    );
    assert_eq!(
        inspection.icc_profile_sha256.as_deref(),
        Some(result.icc_profile_sha256.as_str())
    );

    remove_export_root(&root);
}

#[test]
fn exports_jpeg_preserves_source_exif_when_requested() {
    let root = unique_export_root("metadata-preserve");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("preserve.jpg");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg_with_exif(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::export_jpeg_with_metadata_policy(
        jpeg_metadata_request(source_path.clone(), output_path.clone()),
        super::ExportMetadataPolicy::Preserve,
    )
    .expect("export jpeg with preserved metadata");

    assert_eq!(result.output_path, output_path);
    assert_eq!(
        result.metadata_policy,
        super::ExportMetadataPolicy::Preserve
    );
    assert!(result.source_metadata_copied);
    assert_eq!(result.source_metadata_segments, 1);
    assert!(jpeg_contains_exif_make(&result.output_path));
    assert!(jpeg_has_exif_gps_ifd(&result.output_path));
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn exports_jpeg_removes_gps_metadata_when_requested() {
    let root = unique_export_root("metadata-remove-gps");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("remove-gps.jpg");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg_with_exif(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::export_jpeg_with_metadata_policy(
        jpeg_metadata_request(source_path.clone(), output_path.clone()),
        super::ExportMetadataPolicy::RemoveGps,
    )
    .expect("export jpeg without gps metadata");

    assert_eq!(
        result.metadata_policy,
        super::ExportMetadataPolicy::RemoveGps
    );
    assert!(result.source_metadata_copied);
    assert!(result.gps_metadata_removed);
    assert!(jpeg_contains_exif_make(&result.output_path));
    assert!(!jpeg_has_exif_gps_ifd(&result.output_path));
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn exports_jpeg_removes_source_metadata_when_requested() {
    let root = unique_export_root("metadata-remove-all");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("remove-all.jpg");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg_with_exif(&source_path);

    let result = super::export_jpeg_with_metadata_policy(
        jpeg_metadata_request(source_path, output_path),
        super::ExportMetadataPolicy::RemoveAll,
    )
    .expect("export jpeg without source metadata");

    assert_eq!(
        result.metadata_policy,
        super::ExportMetadataPolicy::RemoveAll
    );
    assert!(!result.source_metadata_copied);
    assert_eq!(result.source_metadata_segments, 1);
    assert!(!jpeg_contains_exif_make(&result.output_path));
    assert!(!jpeg_has_exif_gps_ifd(&result.output_path));

    remove_export_root(&root);
}

#[test]
fn exports_png_without_mutating_original() {
    let root = unique_export_root("png");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("edited.png");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::export_raster_srgb(super::RasterSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        format: super::ExportImageFormat::Png,
        exposure: 0.5,
        contrast: -8.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("export png srgb");

    assert_eq!(result.output_path, output_path);
    assert_eq!(result.format, super::ExportImageFormat::Png);
    assert_eq!(result.color_profile, super::ExportColorProfile::Srgb);
    assert!(result.bytes_written > 0);
    assert!(!result.icc_profile_embedded);
    assert_eq!(result.icc_profile_sha256, None);
    assert_eq!(
        result.output_sha256,
        super::sha256_file(&result.output_path).expect("hash exported png")
    );
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    let exported = image::ImageReader::open(&result.output_path)
        .expect("open exported png")
        .with_guessed_format()
        .expect("guess png format")
        .decode()
        .expect("decode exported png");
    assert_eq!(exported.width(), 2);
    assert_eq!(exported.height(), 2);

    remove_export_root(&root);
}

#[test]
fn exports_tiff_without_mutating_original() {
    let root = unique_export_root("tiff");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("edited.tiff");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::export_raster_srgb(super::RasterSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        format: super::ExportImageFormat::Tiff,
        exposure: 0.25,
        contrast: 4.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("export tiff srgb");

    assert_eq!(result.output_path, output_path);
    assert_eq!(result.format, super::ExportImageFormat::Tiff);
    assert_eq!(result.color_profile, super::ExportColorProfile::Srgb);
    assert!(result.bytes_written > 0);
    assert!(!result.icc_profile_embedded);
    assert_eq!(result.icc_profile_sha256, None);
    assert_eq!(
        result.output_sha256,
        super::sha256_file(&result.output_path).expect("hash exported tiff")
    );
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    let exported = image::ImageReader::open(&result.output_path)
        .expect("open exported tiff")
        .with_guessed_format()
        .expect("guess tiff format")
        .decode()
        .expect("decode exported tiff");
    assert_eq!(exported.width(), 2);
    assert_eq!(exported.height(), 2);

    remove_export_root(&root);
}

#[test]
fn classifies_display_p3_icc_profile_by_xyz_tags_when_hash_differs() {
    let profile = synthetic_rgb_icc_profile([
        [0.5151214599609375, 0.2411956787109375, -0.0010528564453125],
        [0.2919769287109375, 0.6922454833984375, 0.0418853759765625],
        [0.1571044921875, 0.0665740966796875, 0.7840728759765625],
    ]);

    assert_eq!(
        super::classify_icc_profile(&profile),
        Some(super::ExportColorProfile::DisplayP3)
    );
}

#[test]
fn portable_icc_profiles_have_pinned_hashes_and_classification() {
    for (profile, expected_sha256) in [
        (
            super::ExportColorProfile::Srgb,
            "c56e1685d888f5edb92fe07f2750f387f8fe8e91b32ff8fb0b56bfbbb9458353",
        ),
        (
            super::ExportColorProfile::DisplayP3,
            "231752984cd4a5278e1b8d2390fe496767d4511fc81f54e1a5c69ae9ab4c42b5",
        ),
    ] {
        let bytes = super::portable_icc_profile(profile);

        assert_eq!(super::sha256_hex(bytes), expected_sha256);
        assert_eq!(super::classify_icc_profile(bytes), Some(profile));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_export_profile_matches_system_bytes_or_valid_portable_fallback() {
    for profile in [
        super::ExportColorProfile::Srgb,
        super::ExportColorProfile::DisplayP3,
    ] {
        let path = super::export_icc_profile_path(profile);
        let exported = super::export_icc_profile(profile).expect("load export ICC profile");

        match std::fs::read(path) {
            Ok(system_bytes) => assert_eq!(exported, system_bytes),
            Err(_) => {
                assert_eq!(exported, super::portable_icc_profile(profile));
                assert_eq!(super::classify_icc_profile(&exported), Some(profile));
            }
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_profile_loader_preserves_exact_readable_bytes() {
    let root = unique_export_root("readable-system-icc");
    let path = root.join("Readable Profile.icc");
    std::fs::create_dir_all(&root).expect("create profile root");
    let system_bytes = b"exact readable system profile bytes";
    std::fs::write(&path, system_bytes).expect("write readable profile");

    assert_eq!(
        super::system_or_portable_icc_profile(super::ExportColorProfile::Srgb, &path),
        system_bytes
    );

    remove_export_root(&root);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_missing_profile_path_uses_valid_portable_fallback() {
    let path = unique_export_root("missing-system-icc").join("Missing Profile.icc");

    for profile in [
        super::ExportColorProfile::Srgb,
        super::ExportColorProfile::DisplayP3,
    ] {
        let fallback = super::system_or_portable_icc_profile(profile, &path);

        assert_eq!(fallback, super::portable_icc_profile(profile));
        assert_eq!(super::classify_icc_profile(&fallback), Some(profile));
    }
}

#[test]
fn refuses_to_export_over_original_path() {
    let root = unique_export_root("same-path");
    let source_path = root.join("source.jpg");
    std::fs::create_dir_all(&root).expect("create export root");
    write_source_jpeg(&source_path);

    let error = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: source_path.clone(),
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
    })
    .expect_err("same source/output path should fail");

    assert!(error.to_string().contains("output path must differ"));
    remove_export_root(&root);
}

#[cfg(unix)]
#[test]
fn refuses_to_export_over_original_hard_link() {
    let root = unique_export_root("same-hard-link");
    let source_path = root.join("source.jpg");
    let output_path = root.join("source-hard-link.jpg");
    std::fs::create_dir_all(&root).expect("create export root");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    std::fs::hard_link(&source_path, &output_path).expect("create source hard link");

    let error = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
    })
    .expect_err("hard-linked source/output path should fail");

    assert!(error.to_string().contains("output path must differ"));
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    remove_export_root(&root);
}

#[test]
fn writes_jpeg_thumbnail_without_mutating_original() {
    let root = unique_export_root("thumbnail");
    let source_path = root.join("source.jpg");
    let output_path = root.join("thumbs").join("source-thumb.jpg");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let result = super::write_jpeg_thumbnail(super::JpegThumbnailRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        max_edge: 2,
        quality: 80,
    })
    .expect("write thumbnail");

    assert_eq!(result.output_path, output_path);
    assert_eq!(result.format, super::ExportImageFormat::Jpeg);
    assert!(result.bytes_written > 0);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    let decoded = image::ImageReader::open(&result.output_path)
        .expect("open thumbnail")
        .with_guessed_format()
        .expect("guess thumbnail format")
        .decode()
        .expect("decode thumbnail");
    assert!(decoded.width() <= 2);
    assert!(decoded.height() <= 2);

    remove_export_root(&root);
}

#[test]
fn writes_adjusted_jpeg_preview_without_mutating_original() {
    let root = unique_export_root("develop-preview");
    let source_path = root.join("source.jpg");
    let neutral_path = root.join("previews").join("source-neutral.jpg");
    let adjusted_path = root.join("previews").join("source-adjusted.jpg");
    std::fs::create_dir_all(neutral_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: adjusted_path,
        max_edge: 2,
        quality: 82,
        exposure: 1.0,
        contrast: 20.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write adjusted preview");

    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(adjusted.output_path).expect("read adjusted preview")
    );

    remove_export_root(&root);
}

#[test]
fn writes_masked_jpeg_preview_without_mutating_original() {
    let root = unique_export_root("masked-develop-preview");
    let source_path = root.join("source.jpg");
    let neutral_path = root.join("previews").join("source-neutral.jpg");
    let masked_path = root.join("previews").join("source-masked.jpg");
    std::fs::create_dir_all(neutral_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let mask = super::ManualMaskAdjustment {
        id: "mask-linear-1".to_string(),
        enabled: true,
        invert: false,
        opacity: 100.0,
        feather: 0.0,
        geometry: super::ManualMaskGeometry::LinearGradient {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 1.0,
            end_y: 1.0,
        },
        exposure: 1.0,
        contrast: 0.0,
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_path,
        max_edge: 2,
        quality: 95,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let masked = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: masked_path,
        max_edge: 2,
        quality: 95,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: vec![mask],
    })
    .expect("write masked preview");

    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(masked.output_path).expect("read masked preview")
    );

    remove_export_root(&root);
}

#[test]
fn exports_masked_jpeg_srgb_without_mutating_original() {
    let root = unique_export_root("masked-export");
    let source_path = root.join("source.jpg");
    let neutral_path = root.join("export").join("neutral.jpg");
    let masked_path = root.join("export").join("masked.jpg");
    std::fs::create_dir_all(neutral_path.parent().expect("output parent"))
        .expect("create output directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let mask = super::ManualMaskAdjustment {
        id: "mask-linear-1".to_string(),
        enabled: true,
        invert: false,
        opacity: 100.0,
        feather: 0.0,
        geometry: super::ManualMaskGeometry::LinearGradient {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 1.0,
            end_y: 1.0,
        },
        exposure: 1.0,
        contrast: 0.0,
    };

    let neutral = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: neutral_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 95,
    })
    .expect("export neutral jpeg");
    let masked = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
        source_path: source_path.clone(),
        output_path: masked_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: vec![mask],
        quality: 95,
    })
    .expect("export masked jpeg");

    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    assert_ne!(neutral.output_sha256, masked.output_sha256);
    assert_eq!(masked.color_profile, super::ExportColorProfile::Srgb);

    remove_export_root(&root);
}

#[test]
fn writes_brush_masked_jpeg_preview_from_alpha_plane_without_mutating_original() {
    let root = unique_export_root("brush-masked-develop-preview");
    let source_path = root.join("source.jpg");
    let neutral_path = root.join("previews").join("source-neutral.jpg");
    let masked_path = root.join("previews").join("source-brush-masked.jpg");
    std::fs::create_dir_all(neutral_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let mask = super::ManualMaskAdjustment {
        id: "mask-brush-1".to_string(),
        enabled: true,
        invert: false,
        opacity: 100.0,
        feather: 0.0,
        geometry: super::ManualMaskGeometry::RasterAlphaPlane {
            width: 2,
            height: 2,
            alpha: vec![255, 0, 0, 0],
        },
        exposure: 1.0,
        contrast: 0.0,
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_path,
        max_edge: 2,
        quality: 95,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let masked = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: masked_path,
        max_edge: 2,
        quality: 95,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: vec![mask],
    })
    .expect("write brush masked preview");

    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(masked.output_path).expect("read masked preview")
    );

    remove_export_root(&root);
}

#[test]
fn writes_white_balance_adjusted_preview_and_export_without_mutating_original() {
    let root = unique_export_root("white-balance");
    let source_path = root.join("source.jpg");
    let neutral_preview_path = root.join("previews").join("neutral.jpg");
    let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
    let adjusted_export_path = root.join("export").join("adjusted.jpg");
    std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
        .expect("create export directory");
    std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let white_balance = super::WhiteBalanceAdjustment {
        mode: super::WhiteBalanceMode::Custom,
        temperature: 6500.0,
        tint: 20.0,
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: adjusted_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance,
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write white balance preview");
    let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: adjusted_export_path,
        exposure: 0.0,
        contrast: 0.0,
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
        white_balance,
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("export white balance jpeg");

    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(adjusted.output_path).expect("read adjusted preview")
    );
    assert!(exported.bytes_written > 0);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn applies_supported_geometry_preview_and_export_without_mutating_original() {
    let root = unique_export_root("geometry");
    let source_path = root.join("source.jpg");
    let preview_path = root.join("previews").join("geometry.jpg");
    let export_path = root.join("export").join("geometry.jpg");
    std::fs::create_dir_all(export_path.parent().expect("export parent"))
        .expect("create export directory");
    std::fs::create_dir_all(preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_geometry_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let geometry = super::GeometryAdjustment {
        crop: Some(super::GeometryCropAdjustment {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 1.0,
            angle: 0.0,
            aspect: None,
        }),
        rotation: 90.0,
        flip_horizontal: true,
        flip_vertical: false,
        ..super::GeometryAdjustment::neutral()
    };

    let preview = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: preview_path,
        max_edge: 4,
        quality: 95,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: geometry.clone(),
        masks: Vec::new(),
    })
    .expect("write geometry preview");
    let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: export_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry,
        masks: Vec::new(),
        quality: 95,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect("export geometry jpeg");

    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );
    for path in [&preview.output_path, &exported.output_path] {
        let decoded = image::ImageReader::open(path)
            .expect("open geometry output")
            .with_guessed_format()
            .expect("guess geometry format")
            .decode()
            .expect("decode geometry output");
        assert_eq!(decoded.width(), 3);
        assert_eq!(decoded.height(), 2);
    }

    remove_export_root(&root);
}

#[test]
fn blocks_unsupported_geometry_without_writing_output() {
    let root = unique_export_root("unsupported-geometry");
    let source_path = root.join("source.jpg");
    let output_path = root.join("export").join("unsupported.jpg");
    std::fs::create_dir_all(output_path.parent().expect("export parent"))
        .expect("create export directory");
    write_geometry_source_jpeg(&source_path);
    let geometry = super::GeometryAdjustment {
        rotation: 13.0,
        ..super::GeometryAdjustment::neutral()
    };

    let error = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path,
        output_path: output_path.clone(),
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry,
        masks: Vec::new(),
        quality: 95,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect_err("unsupported arbitrary rotation should fail");

    assert!(matches!(
        error,
        super::ExportError::UnsupportedGeometryAdjustment(_)
    ));
    assert!(!output_path.exists());
    remove_export_root(&root);
}

#[test]
fn writes_tone_recovery_adjusted_preview_and_export_without_mutating_original() {
    let root = unique_export_root("tone-recovery");
    let source_path = root.join("source.jpg");
    let neutral_preview_path = root.join("previews").join("neutral.jpg");
    let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
    let adjusted_export_path = root.join("export").join("adjusted.jpg");
    std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
        .expect("create export directory");
    std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let tone_recovery = super::ToneRecoveryAdjustment {
        highlights: -35.0,
        shadows: 42.0,
        whites: 10.0,
        blacks: -12.0,
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: adjusted_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery,
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write tone recovery preview");
    let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: adjusted_export_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery,
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect("export tone recovery jpeg");

    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(adjusted.output_path).expect("read adjusted preview")
    );
    assert!(exported.bytes_written > 0);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn writes_tone_curve_adjusted_preview_and_export_without_mutating_original() {
    let root = unique_export_root("tone-curve");
    let source_path = root.join("source.jpg");
    let neutral_preview_path = root.join("previews").join("neutral.jpg");
    let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
    let adjusted_export_path = root.join("export").join("adjusted.jpg");
    std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
        .expect("create export directory");
    std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let tone_curve = super::ToneCurveAdjustment {
        mode: super::ToneCurveMode::Point,
        rgb_curve: vec![
            super::ToneCurvePoint { x: 0.0, y: 0.0 },
            super::ToneCurvePoint { x: 0.5, y: 0.28 },
            super::ToneCurvePoint { x: 1.0, y: 1.0 },
        ],
        red_curve: Vec::new(),
        green_curve: Vec::new(),
        blue_curve: Vec::new(),
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: adjusted_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: tone_curve.clone(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write tone curve preview");
    let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: adjusted_export_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve,
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect("export tone curve jpeg");

    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(adjusted.output_path).expect("read adjusted preview")
    );
    assert!(exported.bytes_written > 0);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn writes_color_presence_adjusted_preview_and_export_without_mutating_original() {
    let root = unique_export_root("color-presence");
    let source_path = root.join("source.jpg");
    let neutral_preview_path = root.join("previews").join("neutral.jpg");
    let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
    let adjusted_export_path = root.join("export").join("adjusted.jpg");
    std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
        .expect("create export directory");
    std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let color_presence = super::ColorPresenceAdjustment {
        vibrance: 24.0,
        saturation: -8.5,
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: adjusted_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence,
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write color presence preview");
    let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: adjusted_export_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence,
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect("export color presence jpeg");

    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(adjusted.output_path).expect("read adjusted preview")
    );
    assert!(exported.bytes_written > 0);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn writes_hsl_color_mixer_preview_and_export_without_mutating_original() {
    let root = unique_export_root("hsl-color-mixer");
    let source_path = root.join("source.jpg");
    let neutral_preview_path = root.join("previews").join("neutral.jpg");
    let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
    let adjusted_export_path = root.join("export").join("adjusted.jpg");
    std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
        .expect("create export directory");
    std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");
    let hsl_color_mixer = super::HslColorMixerAdjustment {
        blue: super::HslColorChannelAdjustment {
            hue: -12.0,
            saturation: 24.0,
            luminance: -8.5,
        },
        ..super::HslColorMixerAdjustment::neutral()
    };

    let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: neutral_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write neutral preview");
    let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: adjusted_preview_path,
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer,
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect("write hsl preview");
    let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: adjusted_export_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer,
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect("export hsl jpeg");

    assert_ne!(
        std::fs::read(neutral.output_path).expect("read neutral preview"),
        std::fs::read(adjusted.output_path).expect("read adjusted preview")
    );
    assert!(exported.bytes_written > 0);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

#[test]
fn rejects_non_neutral_detail_preview_and_export_until_renderer_support_exists() {
    let root = unique_export_root("detail-boundary");
    let source_path = root.join("source.jpg");
    let preview_path = root.join("previews").join("detail.jpg");
    let export_path = root.join("export").join("detail.jpg");
    std::fs::create_dir_all(preview_path.parent().expect("preview parent"))
        .expect("create preview directory");
    std::fs::create_dir_all(export_path.parent().expect("export parent"))
        .expect("create export directory");
    write_source_jpeg(&source_path);
    let detail = super::DetailAdjustment {
        sharpening: super::DetailSharpeningAdjustment {
            amount: 42.0,
            radius: 1.2,
            detail: 35.0,
            masking: 10.0,
        },
        ..super::DetailAdjustment::neutral()
    };

    let preview_error = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
        source_path: source_path.clone(),
        output_path: preview_path.clone(),
        max_edge: 2,
        quality: 82,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail,
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
    })
    .expect_err("detail preview unsupported");
    let export_error = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
        source_path: source_path.clone(),
        output_path: export_path.clone(),
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail,
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
    })
    .expect_err("detail export unsupported");

    assert!(matches!(
        preview_error,
        super::ExportError::UnsupportedDetailAdjustment(_)
    ));
    assert!(matches!(
        export_error,
        super::ExportError::UnsupportedDetailAdjustment(_)
    ));
    assert!(!preview_path.exists());
    assert!(!export_path.exists());

    remove_export_root(&root);
}

#[test]
fn computes_adjusted_jpeg_histogram_without_mutating_original() {
    let root = unique_export_root("histogram");
    let source_path = root.join("source.jpg");
    std::fs::create_dir_all(&root).expect("create histogram root");
    write_source_jpeg(&source_path);
    let original_before = std::fs::read(&source_path).expect("read original before");

    let histogram = super::compute_jpeg_develop_histogram(super::JpegHistogramRequest {
        source_path: source_path.clone(),
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment {
            vibrance: 24.0,
            saturation: -8.5,
        },
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
    })
    .expect("compute histogram");

    assert_eq!(histogram.pixel_count, 4);
    assert_eq!(histogram.red.len(), 256);
    assert_eq!(histogram.green.len(), 256);
    assert_eq!(histogram.blue.len(), 256);
    assert_eq!(histogram.luminance.len(), 256);
    assert_eq!(
        std::fs::read(&source_path).expect("read original after"),
        original_before
    );

    remove_export_root(&root);
}

fn write_source_jpeg(path: &Path) {
    let image = image::RgbImage::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            image::Rgb([64, 128, 192])
        } else {
            image::Rgb([192, 128, 64])
        }
    });
    image
        .save_with_format(path, image::ImageFormat::Jpeg)
        .expect("write source jpeg");
}

fn write_source_jpeg_with_exif(path: &Path) {
    write_source_jpeg(path);
    let bytes = std::fs::read(path).expect("read source jpeg");
    let with_exif = insert_app1_exif_segment(&bytes, &minimal_exif_with_gps());
    std::fs::write(path, with_exif).expect("write source jpeg exif");
}

fn jpeg_metadata_request(
    source_path: PathBuf,
    output_path: PathBuf,
) -> super::JpegColorExportRequest {
    super::JpegColorExportRequest {
        source_path,
        output_path,
        exposure: 0.0,
        contrast: 0.0,
        white_balance: super::WhiteBalanceAdjustment::neutral(),
        tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        color_presence: super::ColorPresenceAdjustment::neutral(),
        tone_curve: super::ToneCurveAdjustment::neutral(),
        hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
        detail: super::DetailAdjustment::neutral(),
        geometry: super::GeometryAdjustment::neutral(),
        masks: Vec::new(),
        quality: 90,
        color_profile: super::ExportColorProfile::Srgb,
    }
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

fn synthetic_rgb_icc_profile(primaries: [[f64; 3]; 3]) -> Vec<u8> {
    let tag_table_start = 128;
    let tag_count_size = 4;
    let tag_record_size = 12;
    let tag_data_start = tag_table_start + tag_count_size + (3 * tag_record_size);
    let tag_data_size = 20;
    let profile_size = tag_data_start + (3 * tag_data_size);
    let mut profile = vec![0_u8; profile_size];
    profile[0..4].copy_from_slice(&(profile_size as u32).to_be_bytes());
    profile[36..40].copy_from_slice(b"acsp");
    profile[128..132].copy_from_slice(&3_u32.to_be_bytes());

    for (index, (signature, values)) in [
        (b"rXYZ", primaries[0]),
        (b"gXYZ", primaries[1]),
        (b"bXYZ", primaries[2]),
    ]
    .into_iter()
    .enumerate()
    {
        let record_offset = 132 + (index * tag_record_size);
        let data_offset = tag_data_start + (index * tag_data_size);
        profile[record_offset..record_offset + 4].copy_from_slice(signature);
        profile[record_offset + 4..record_offset + 8]
            .copy_from_slice(&(data_offset as u32).to_be_bytes());
        profile[record_offset + 8..record_offset + 12]
            .copy_from_slice(&(tag_data_size as u32).to_be_bytes());
        profile[data_offset..data_offset + 4].copy_from_slice(b"XYZ ");
        for (component, value) in values.iter().enumerate() {
            let fixed = (value * 65536.0).round() as i32;
            let component_offset = data_offset + 8 + (component * 4);
            profile[component_offset..component_offset + 4].copy_from_slice(&fixed.to_be_bytes());
        }
    }

    profile
}

fn unique_export_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "silicaraw-export-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn remove_export_root(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}
