//! Export coordination boundary for SilicaRAW.
//!
//! The local alpha export path accepts already-rendered raster inputs and writes
//! a separate JPEG file. RAW decoding and final color-management fixtures remain
//! outside this crate.

mod metadata;
mod model;
mod ops;
mod pixels;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-export";

pub use metadata::{inspect_jpeg_icc_profile, sha256_file};
pub use model::*;
pub use ops::{
    compute_jpeg_develop_histogram, export_jpeg_srgb, export_jpeg_with_color_profile,
    export_jpeg_with_metadata_policy, export_raster_srgb, read_raster_dimensions,
    write_jpeg_develop_preview, write_jpeg_thumbnail,
};

#[cfg(test)]
use metadata::{classify_icc_profile, portable_icc_profile, sha256_hex};

#[cfg(all(test, target_os = "macos"))]
use metadata::{export_icc_profile, export_icc_profile_path, system_or_portable_icc_profile};

#[cfg(test)]
mod tests;
