use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    RawPreviewArtifactError, RawProbeBackend, RawProbeErrorCategory, RawProbePlatform,
    RawProbeRequest, RawProbeResult, RawProbeStatus,
};

use super::CoreImageRawPreviewArtifact;

pub fn probe_core_image_raw(request: RawProbeRequest) -> RawProbeResult {
    let source_path = request.source_path.clone();
    let path = PathBuf::from(&request.source_path);
    let macos_version = macos_version();

    let metadata = match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => {
            return failed_result(
                macos_version,
                source_path,
                None,
                None,
                None,
                RawProbeErrorCategory::InvalidFixture,
                "RAW probe source path is not a regular file.",
            );
        }
        Err(error) => {
            return failed_result(
                macos_version,
                source_path,
                None,
                None,
                None,
                io_error_category(&error),
                "RAW probe source file could not be inspected.",
            );
        }
    };

    let original_file_size = Some(metadata.len());
    let original_modified_at = metadata.modified().ok().and_then(format_system_time);
    let source_sha256 = match sha256_file(&path) {
        Ok(hash) => hash,
        Err(error) => {
            return failed_result(
                macos_version,
                source_path,
                None,
                original_file_size,
                original_modified_at,
                io_error_category(&error),
                "RAW probe source file could not be hashed.",
            );
        }
    };

    if let Some(expected_sha256) = request.expected_sha256.as_deref() {
        if !expected_sha256.eq_ignore_ascii_case(&source_sha256) {
            return failed_result(
                macos_version,
                source_path,
                Some(source_sha256),
                original_file_size,
                original_modified_at,
                RawProbeErrorCategory::SourceHashMismatch,
                "RAW probe source SHA-256 did not match expected fixture hash.",
            );
        }
    }

    match probe_core_image_dimensions(&path) {
        Ok((width, height)) => RawProbeResult {
            backend: RawProbeBackend::CoreImageRaw,
            platform: RawProbePlatform::Macos,
            macos_version,
            source_path,
            source_sha256: Some(source_sha256),
            original_file_size,
            original_modified_at,
            status: RawProbeStatus::Success,
            width: Some(width),
            height: Some(height),
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source and reported image dimensions.".to_string(),
        },
        Err((category, message)) => failed_result(
            macos_version,
            source_path,
            Some(source_sha256),
            original_file_size,
            original_modified_at,
            category,
            message,
        ),
    }
}

fn failed_result(
    macos_version: Option<String>,
    source_path: String,
    source_sha256: Option<String>,
    original_file_size: Option<u64>,
    original_modified_at: Option<String>,
    error_category: RawProbeErrorCategory,
    message: impl Into<String>,
) -> RawProbeResult {
    RawProbeResult {
        backend: RawProbeBackend::CoreImageRaw,
        platform: RawProbePlatform::Macos,
        macos_version,
        source_path,
        source_sha256,
        original_file_size,
        original_modified_at,
        status: RawProbeStatus::Failed,
        width: None,
        height: None,
        orientation: None,
        error_category: Some(error_category),
        message: message.into(),
    }
}

fn macos_version() -> Option<String> {
    let output = Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8(output.stdout).ok()?;
    let trimmed = version.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn io_error_category(error: &io::Error) -> RawProbeErrorCategory {
    match error.kind() {
        io::ErrorKind::NotFound => RawProbeErrorCategory::MissingFile,
        io::ErrorKind::PermissionDenied => RawProbeErrorCategory::PermissionDenied,
        _ => RawProbeErrorCategory::Unknown,
    }
}

fn sha256_file(path: &Path) -> Result<String, io::Error> {
    use sha2::{Digest, Sha256};

    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }

    Ok(format!("{:x}", digest.finalize()))
}

fn format_system_time(time: SystemTime) -> Option<String> {
    let since_epoch = time.duration_since(UNIX_EPOCH).ok()?;
    Some(format!(
        "unix:{}.{:09}",
        since_epoch.as_secs(),
        since_epoch.subsec_nanos()
    ))
}

fn probe_core_image_dimensions(path: &Path) -> Result<(u32, u32), (RawProbeErrorCategory, String)> {
    use objc2_core_image::CIImage;
    use objc2_foundation::{NSString, NSURL};

    let Some(path) = path.to_str() else {
        return Err((
            RawProbeErrorCategory::InvalidFixture,
            "RAW probe source path is not valid UTF-8 for NSURL creation.".to_string(),
        ));
    };

    let ns_path = NSString::from_str(path);
    let url = NSURL::fileURLWithPath(&ns_path);
    let Some(image) = (unsafe { CIImage::imageWithContentsOfURL(&url) }) else {
        return Err((
            RawProbeErrorCategory::CoreImageOpenFailed,
            "Core Image could not open the RAW source.".to_string(),
        ));
    };

    let extent = unsafe { image.extent() };
    let width = dimension_to_u32(extent.size.width as f64);
    let height = dimension_to_u32(extent.size.height as f64);

    match (width, height) {
        (Some(width), Some(height)) => Ok((width, height)),
        _ => Err((
            RawProbeErrorCategory::CoreImageMetadataMissing,
            "Core Image opened the source but did not report finite positive dimensions."
                .to_string(),
        )),
    }
}

pub fn write_core_image_raw_preview_artifact(
    probe: &RawProbeResult,
    output_path: &Path,
    max_edge: u32,
) -> Result<CoreImageRawPreviewArtifact, RawPreviewArtifactError> {
    let source_path = PathBuf::from(&probe.source_path);
    let before_hash = sha256_file(&source_path)?;
    if let Some(expected_hash) = probe.source_sha256.as_deref() {
        if !expected_hash.eq_ignore_ascii_case(&before_hash) {
            return Err(RawPreviewArtifactError::SourceHashMismatch {
                expected: expected_hash.to_string(),
                actual: before_hash,
            });
        }
    }

    write_core_image_jpeg(&source_path, output_path, max_edge)?;

    let after_hash = sha256_file(&source_path)?;
    let bytes_written = fs::metadata(output_path)?.len();

    Ok(CoreImageRawPreviewArtifact {
        output_path: output_path.to_path_buf(),
        bytes_written,
        original_hash_unchanged: before_hash == after_hash,
    })
}

fn write_core_image_jpeg(
    source_path: &Path,
    output_path: &Path,
    max_edge: u32,
) -> Result<(), RawPreviewArtifactError> {
    use objc2::runtime::AnyObject;
    use objc2_core_graphics::{kCGColorSpaceSRGB, CGAffineTransformMakeScale, CGColorSpace};
    use objc2_core_image::{CIContext, CIImage, CIImageRepresentationOption};
    use objc2_foundation::NSDictionary;

    let source_url = file_url(source_path)?;
    let output_url = file_url(output_path)?;
    let Some(parent) = output_path.parent() else {
        return Err(RawPreviewArtifactError::InvalidRequest(
            "RAW preview output path must have a parent directory.".to_string(),
        ));
    };
    fs::create_dir_all(parent)?;

    let Some(image) = (unsafe { CIImage::imageWithContentsOfURL(&source_url) }) else {
        return Err(RawPreviewArtifactError::CoreImageWrite(
            "Core Image could not open RAW source for preview artifact.".to_string(),
        ));
    };
    let extent = unsafe { image.extent() };
    let max_dimension = extent.size.width.max(extent.size.height) as f64;
    if !max_dimension.is_finite() || max_dimension <= 0.0 {
        return Err(RawPreviewArtifactError::CoreImageWrite(
            "Core Image opened the RAW source but reported invalid dimensions.".to_string(),
        ));
    }

    let scale = (max_edge as f64 / max_dimension).min(1.0);
    let image = if scale < 1.0 {
        let transform = CGAffineTransformMakeScale(scale, scale);
        unsafe { image.imageByApplyingTransform_highQualityDownsample(transform, true) }
    } else {
        image
    };

    let Some(color_space) = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB })) else {
        return Err(RawPreviewArtifactError::CoreImageWrite(
            "Core Graphics could not create an sRGB color space.".to_string(),
        ));
    };
    let options = NSDictionary::<CIImageRepresentationOption, AnyObject>::new();
    let context = unsafe { CIContext::context() };

    unsafe {
        context
            .writeJPEGRepresentationOfImage_toURL_colorSpace_options_error(
                &image,
                &output_url,
                &color_space,
                &options,
            )
            .map_err(|error| {
                RawPreviewArtifactError::CoreImageWrite(format!(
                    "Core Image JPEG representation failed: {error:?}"
                ))
            })
    }
}

fn file_url(
    path: &Path,
) -> Result<objc2::rc::Retained<objc2_foundation::NSURL>, RawPreviewArtifactError> {
    let Some(path) = path.to_str() else {
        return Err(RawPreviewArtifactError::InvalidRequest(format!(
            "path is not valid UTF-8 for NSURL creation: {}",
            path.display()
        )));
    };

    let ns_path = objc2_foundation::NSString::from_str(path);
    Ok(objc2_foundation::NSURL::fileURLWithPath(&ns_path))
}

fn dimension_to_u32(value: f64) -> Option<u32> {
    if value.is_finite() && value > 0.0 && value <= u32::MAX as f64 {
        Some(value.round() as u32)
    } else {
        None
    }
}
