//! Export coordination boundary for SilicaRAW.
//!
//! The local alpha export path accepts already-rendered raster inputs and writes
//! a separate JPEG file. RAW decoding and final color-management fixtures remain
//! outside this crate.

use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-export";

/// File format written by the local alpha export path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportImageFormat {
    Jpeg,
}

/// Output color profile target recorded by the export contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportColorProfile {
    Srgb,
}

/// Request to export an already-rendered raster source as JPEG sRGB.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegSrgbExportRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub exposure: f64,
    pub contrast: f64,
    pub quality: u8,
}

/// Result returned after a JPEG sRGB export is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegSrgbExportResult {
    pub output_path: PathBuf,
    pub format: ExportImageFormat,
    pub color_profile: ExportColorProfile,
    pub bytes_written: u64,
}

/// Request to create a disposable JPEG thumbnail for a raster source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegThumbnailRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub max_edge: u32,
    pub quality: u8,
}

/// Result returned after a JPEG thumbnail is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegThumbnailResult {
    pub output_path: PathBuf,
    pub format: ExportImageFormat,
    pub bytes_written: u64,
}

/// Errors returned by the export crate.
#[derive(Debug)]
pub enum ExportError {
    Io(std::io::Error),
    Image(image::ImageError),
    InvalidQuality(u8),
    InvalidThumbnailEdge(u32),
    NonFiniteAdjustment,
    SameSourceAndOutput(PathBuf),
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "filesystem error: {error}"),
            Self::Image(error) => write!(formatter, "image error: {error}"),
            Self::InvalidQuality(quality) => {
                write!(
                    formatter,
                    "jpeg quality must be between 1 and 100, got {quality}"
                )
            }
            Self::InvalidThumbnailEdge(edge) => {
                write!(
                    formatter,
                    "thumbnail max edge must be greater than 0, got {edge}"
                )
            }
            Self::NonFiniteAdjustment => {
                write!(
                    formatter,
                    "exposure and contrast adjustments must be finite"
                )
            }
            Self::SameSourceAndOutput(path) => {
                write!(
                    formatter,
                    "output path must differ from original source path: {}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Image(error) => Some(error),
            Self::InvalidQuality(_)
            | Self::InvalidThumbnailEdge(_)
            | Self::NonFiniteAdjustment
            | Self::SameSourceAndOutput(_) => None,
        }
    }
}

impl From<std::io::Error> for ExportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<image::ImageError> for ExportError {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

/// Export an already-rendered raster source as a separate JPEG sRGB file.
pub fn export_jpeg_srgb(
    request: JpegSrgbExportRequest,
) -> Result<JpegSrgbExportResult, ExportError> {
    if paths_match(&request.source_path, &request.output_path)? {
        return Err(ExportError::SameSourceAndOutput(request.output_path));
    }
    if !(1..=100).contains(&request.quality) {
        return Err(ExportError::InvalidQuality(request.quality));
    }
    if !request.exposure.is_finite() || !request.contrast.is_finite() {
        return Err(ExportError::NonFiniteAdjustment);
    }

    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded.to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);

    let mut output = File::create(&request.output_path)?;
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, request.quality);
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    drop(output);

    Ok(JpegSrgbExportResult {
        bytes_written: fs::metadata(&request.output_path)?.len(),
        output_path: request.output_path,
        format: ExportImageFormat::Jpeg,
        color_profile: ExportColorProfile::Srgb,
    })
}

/// Write a disposable JPEG thumbnail for a raster source.
pub fn write_jpeg_thumbnail(
    request: JpegThumbnailRequest,
) -> Result<JpegThumbnailResult, ExportError> {
    if paths_match(&request.source_path, &request.output_path)? {
        return Err(ExportError::SameSourceAndOutput(request.output_path));
    }
    if !(1..=100).contains(&request.quality) {
        return Err(ExportError::InvalidQuality(request.quality));
    }
    if request.max_edge == 0 {
        return Err(ExportError::InvalidThumbnailEdge(request.max_edge));
    }

    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let rgb = decoded
        .thumbnail(request.max_edge, request.max_edge)
        .to_rgb8();

    let mut output = File::create(&request.output_path)?;
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, request.quality);
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    drop(output);

    Ok(JpegThumbnailResult {
        bytes_written: fs::metadata(&request.output_path)?.len(),
        output_path: request.output_path,
        format: ExportImageFormat::Jpeg,
    })
}

fn paths_match(source_path: &PathBuf, output_path: &PathBuf) -> Result<bool, ExportError> {
    if source_path == output_path {
        return Ok(true);
    }
    if !output_path.exists() {
        return Ok(false);
    }

    Ok(fs::canonicalize(source_path)? == fs::canonicalize(output_path)?)
}

fn apply_exposure_contrast(image: &mut image::RgbImage, exposure: f64, contrast: f64) {
    let exposure_scale = 2.0_f32.powf(exposure as f32);
    let contrast_scale = ((100.0 + contrast as f32) / 100.0).max(0.0);

    for pixel in image.pixels_mut() {
        for channel in &mut pixel.0 {
            let normalized = f32::from(*channel) / 255.0;
            let adjusted =
                ((normalized * exposure_scale - 0.5) * contrast_scale + 0.5).clamp(0.0, 1.0);
            *channel = (adjusted * 255.0).round() as u8;
        }
    }
}

#[cfg(test)]
mod tests {
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
            quality: 90,
        })
        .expect("export jpeg srgb");

        assert_eq!(result.output_path, output_path);
        assert_eq!(result.format, super::ExportImageFormat::Jpeg);
        assert_eq!(result.color_profile, super::ExportColorProfile::Srgb);
        assert!(result.bytes_written > 0);
        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
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
            quality: 90,
        })
        .expect_err("same source/output path should fail");

        assert!(error.to_string().contains("output path must differ"));
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
}
