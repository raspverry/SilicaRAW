//! Render request and renderer boundary for SilicaRAW.
//!
//! Spike 003 records the color-managed preview/export gate. This crate still
//! does not render images, apply color transforms, or export files.

#[cfg(feature = "color-probe")]
use std::fs;
#[cfg(feature = "color-probe")]
use std::path::PathBuf;

#[cfg(feature = "color-probe")]
use sha2::{Digest, Sha256};

use silica_decode::{PreviewDecodePlan, PreviewDecodeStatus};

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-render";

/// Color-management path selected by the preview/export spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorManagementPath {
    /// Use Core Image/ColorSync-compatible color management first.
    CoreImageColorManagementPrimary,
}

/// Working color space selected for the first implementation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingColorSpace {
    /// Linear Display P3-compatible wide-gamut RGB.
    LinearDisplayP3,
}

/// Preview color behavior selected by the spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewColorBehavior {
    /// Convert from working space to the active display color space.
    DisplayProfileAware,
}

/// Export color behavior selected by the spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportColorBehavior {
    /// Export sRGB by default and support Display P3 when explicitly selected.
    SrgbDefaultDisplayP3Supported,
}

/// Status of color-management fixtures in the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFixtureStatus {
    /// Tagged sRGB and Display P3 raster fixtures are not committed yet.
    MissingTaggedRasterFixtures,
}

/// Recorded output of Spike 003.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorGate {
    pub path: ColorManagementPath,
    pub working_space: WorkingColorSpace,
    pub preview: PreviewColorBehavior,
    pub export: ExportColorBehavior,
    pub fixture_set: ColorFixtureStatus,
}

/// Spike 003 decision for downstream crates and tests.
pub const SPIKE_003_COLOR_GATE: ColorGate = ColorGate {
    path: ColorManagementPath::CoreImageColorManagementPrimary,
    working_space: WorkingColorSpace::LinearDisplayP3,
    preview: PreviewColorBehavior::DisplayProfileAware,
    export: ExportColorBehavior::SrgbDefaultDisplayP3Supported,
    fixture_set: ColorFixtureStatus::MissingTaggedRasterFixtures,
};

/// Tag used in docs and future issue labels for color-dependent work.
pub const COLOR_BLOCKING_TAG: &str = "color-blocking";

/// Render readiness state for the local alpha preview path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewRenderStatus {
    /// Preview source can be opened by the current minimal surface.
    Ready,
    /// Preview is blocked before rendering because decode is not ready.
    BlockedByDecode,
    /// Preview is unsupported for this catalog entry.
    Unsupported,
}

/// Render-side preview plan. This is a routing contract, not a Metal viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRenderPlan {
    pub source_path: String,
    pub status: PreviewRenderStatus,
    pub color_behavior: PreviewColorBehavior,
    pub message: String,
}

/// Render request for a draft exposure/contrast preview update.
#[derive(Debug, Clone, PartialEq)]
pub struct ExposureContrastPreviewRequest {
    pub source_path: String,
    pub status: PreviewRenderStatus,
    pub color_behavior: PreviewColorBehavior,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: WhiteBalanceRenderAdjustment,
    pub message: String,
}

/// Render-side export request for the local alpha JPEG sRGB path.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegSrgbExportRenderRequest {
    pub source_kind: ExportRenderSourceKind,
    pub source_path: String,
    pub output_path: String,
    pub color_behavior: ExportColorBehavior,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: WhiteBalanceRenderAdjustment,
    pub quality: u8,
    pub message: String,
}

/// White balance mode carried through render planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalanceRenderMode {
    AsShot,
    Auto,
    Daylight,
    Cloudy,
    Shade,
    Tungsten,
    Fluorescent,
    Flash,
    Custom,
}

/// White balance values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiteBalanceRenderAdjustment {
    pub mode: WhiteBalanceRenderMode,
    pub temperature: f64,
    pub tint: f64,
}

impl WhiteBalanceRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            mode: WhiteBalanceRenderMode::AsShot,
            temperature: 5200.0,
            tint: 0.0,
        }
    }
}

/// Source class for export rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportRenderSourceKind {
    RasterSource,
    RawFullResolutionArtifact,
}

impl JpegSrgbExportRenderRequest {
    pub fn uses_viewer_texture_cache_as_source(&self) -> bool {
        false
    }
}

/// Stable identity for one viewer preview render request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewerPreviewRenderRequestId(pub u64);

/// Drawable viewport for a viewer preview request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewerPreviewViewport {
    pub width_px: u32,
    pub height_px: u32,
    pub backing_scale_factor: f64,
}

impl ViewerPreviewViewport {
    pub fn new(width_px: u32, height_px: u32, backing_scale_factor: f64) -> Self {
        Self {
            width_px,
            height_px,
            backing_scale_factor,
        }
    }

    pub fn drawable_size(&self) -> ViewerTextureDrawableSize {
        ViewerTextureDrawableSize::new(
            scaled_dimension(self.width_px, self.backing_scale_factor),
            scaled_dimension(self.height_px, self.backing_scale_factor),
        )
    }
}

fn scaled_dimension(value: u32, scale: f64) -> u32 {
    if !scale.is_finite() || scale <= 0.0 {
        return value.max(1);
    }
    ((value as f64 * scale).round() as u32).max(1)
}

/// Future Metal texture pixel format identity. No pixel bytes are carried here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerPreviewPixelFormat {
    Bgra8Unorm,
    Rgba16Float,
    JpegSrgb8,
}

/// Input identity for a viewer preview request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerPreviewInput {
    NoPixelsYet {
        readiness: PreviewRenderStatus,
    },
    FutureTexture {
        texture_key: String,
        width_px: u32,
        height_px: u32,
        pixel_format: ViewerPreviewPixelFormat,
    },
    DecodedImageArtifact {
        cache_key: String,
        source_sha256: Option<String>,
        width_px: u32,
        height_px: u32,
        pixel_format: ViewerPreviewPixelFormat,
        decoder_backend: String,
        input_profile: String,
        working_space: String,
    },
}

impl ViewerPreviewInput {
    pub fn no_pixels_yet(readiness: PreviewRenderStatus) -> Self {
        Self::NoPixelsYet { readiness }
    }

    pub fn future_texture(
        texture_key: impl Into<String>,
        width_px: u32,
        height_px: u32,
        pixel_format: ViewerPreviewPixelFormat,
    ) -> Self {
        Self::FutureTexture {
            texture_key: texture_key.into(),
            width_px,
            height_px,
            pixel_format,
        }
    }

    pub fn from_decoded_handoff(handoff: &silica_decode::DecodedImageHandoff) -> Self {
        if handoff.status != silica_decode::DecodedImageHandoffStatus::Ready {
            return Self::NoPixelsYet {
                readiness: PreviewRenderStatus::BlockedByDecode,
            };
        }

        let Some(cache_identity) = handoff.cache_identity.as_ref() else {
            return Self::NoPixelsYet {
                readiness: PreviewRenderStatus::BlockedByDecode,
            };
        };
        if !cache_identity.disposable {
            return Self::NoPixelsYet {
                readiness: PreviewRenderStatus::BlockedByDecode,
            };
        }

        let (Some(width_px), Some(height_px), Some(pixel_format)) =
            (handoff.width, handoff.height, handoff.pixel_format)
        else {
            return Self::NoPixelsYet {
                readiness: PreviewRenderStatus::BlockedByDecode,
            };
        };

        Self::DecodedImageArtifact {
            cache_key: cache_identity.cache_key.clone(),
            source_sha256: handoff.source_sha256.clone(),
            width_px,
            height_px,
            pixel_format: viewer_pixel_format_from_decoded(pixel_format),
            decoder_backend: handoff.decoder_backend.as_str().to_string(),
            input_profile: handoff.input_profile.clone(),
            working_space: handoff.working_space.clone(),
        }
    }

    fn contains_image_pixels(&self) -> bool {
        false
    }
}

fn viewer_pixel_format_from_decoded(
    pixel_format: silica_decode::DecodedImagePixelFormat,
) -> ViewerPreviewPixelFormat {
    match pixel_format {
        silica_decode::DecodedImagePixelFormat::Rgba16Float => {
            ViewerPreviewPixelFormat::Rgba16Float
        }
        silica_decode::DecodedImagePixelFormat::JpegSrgb8 => ViewerPreviewPixelFormat::JpegSrgb8,
    }
}

/// Draft adjustment payload for interactive exposure/contrast preview requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewerExposureContrastDraft {
    pub exposure: f64,
    pub contrast: f64,
}

/// Typed render request boundary between `silica-render` and the native viewer.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerPreviewRenderRequest {
    pub request_id: ViewerPreviewRenderRequestId,
    pub photo_id: String,
    pub source_path: String,
    pub viewport: ViewerPreviewViewport,
    pub input: ViewerPreviewInput,
    pub edit_graph_revision: u64,
    pub exposure_contrast_draft: Option<ViewerExposureContrastDraft>,
}

impl ViewerPreviewRenderRequest {
    pub fn new(
        request_id: ViewerPreviewRenderRequestId,
        photo_id: impl Into<String>,
        source_path: impl Into<String>,
        viewport: ViewerPreviewViewport,
        input: ViewerPreviewInput,
        edit_graph_revision: u64,
    ) -> Self {
        Self {
            request_id,
            photo_id: photo_id.into(),
            source_path: source_path.into(),
            viewport,
            input,
            edit_graph_revision,
            exposure_contrast_draft: None,
        }
    }

    pub fn with_exposure_contrast_draft(mut self, exposure: f64, contrast: f64) -> Self {
        self.exposure_contrast_draft = Some(ViewerExposureContrastDraft { exposure, contrast });
        self
    }

    pub fn writes_catalog_state(&self) -> bool {
        false
    }

    pub fn contains_image_pixels(&self) -> bool {
        self.input.contains_image_pixels()
    }
}

/// Scheduling result for latest-request-wins viewer preview behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewerPreviewScheduleResult {
    pub accepted_request_id: ViewerPreviewRenderRequestId,
    pub replaced_request_id: Option<ViewerPreviewRenderRequestId>,
}

/// Minimal scheduler contract for interactive viewer preview requests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewerPreviewRenderScheduler {
    latest_request: Option<ViewerPreviewRenderRequest>,
}

impl ViewerPreviewRenderScheduler {
    pub fn schedule(&mut self, request: ViewerPreviewRenderRequest) -> ViewerPreviewScheduleResult {
        let replaced_request_id = self
            .latest_request
            .as_ref()
            .map(|request| request.request_id);
        let accepted_request_id = request.request_id;
        self.latest_request = Some(request);

        ViewerPreviewScheduleResult {
            accepted_request_id,
            replaced_request_id,
        }
    }

    pub fn latest_request_id(&self) -> Option<ViewerPreviewRenderRequestId> {
        self.latest_request
            .as_ref()
            .map(|request| request.request_id)
    }

    pub fn latest_request(&self) -> Option<&ViewerPreviewRenderRequest> {
        self.latest_request.as_ref()
    }
}

/// Drawable size attached to disposable viewer texture identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewerTextureDrawableSize {
    pub width_px: u32,
    pub height_px: u32,
}

impl ViewerTextureDrawableSize {
    pub fn new(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px,
            height_px,
        }
    }
}

/// Identity for a disposable viewer texture. This does not carry texture bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewerTextureIdentity {
    pub request_id: ViewerPreviewRenderRequestId,
    pub texture_key: String,
    pub drawable_size: ViewerTextureDrawableSize,
}

impl ViewerTextureIdentity {
    pub fn new(
        request_id: ViewerPreviewRenderRequestId,
        texture_key: impl Into<String>,
        drawable_size: ViewerTextureDrawableSize,
    ) -> Self {
        Self {
            request_id,
            texture_key: texture_key.into(),
            drawable_size,
        }
    }

    pub fn from_preview_request(request: &ViewerPreviewRenderRequest) -> Option<Self> {
        let texture_key = match &request.input {
            ViewerPreviewInput::DecodedImageArtifact { cache_key, .. } => cache_key.clone(),
            ViewerPreviewInput::FutureTexture { texture_key, .. } => texture_key.clone(),
            ViewerPreviewInput::NoPixelsYet { .. } => return None,
        };

        Some(Self::new(
            request.request_id,
            texture_key,
            request.viewport.drawable_size(),
        ))
    }
}

/// Current lifecycle state for disposable viewer texture identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerTextureLifecycleState {
    Empty,
    Bound,
    Released,
}

/// Reason a disposable viewer texture identity was released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerTextureReleaseReason {
    PhotoChanged,
    LibraryClosed,
    AppClosed,
    DrawableResized,
}

/// Disposable viewer texture lifecycle boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerTextureLifecycle {
    state: ViewerTextureLifecycleState,
    current_texture: Option<ViewerTextureIdentity>,
    last_release_reason: Option<ViewerTextureReleaseReason>,
    release_count: u64,
}

impl Default for ViewerTextureLifecycle {
    fn default() -> Self {
        Self {
            state: ViewerTextureLifecycleState::Empty,
            current_texture: None,
            last_release_reason: None,
            release_count: 0,
        }
    }
}

impl ViewerTextureLifecycle {
    pub fn bind_texture(&mut self, identity: ViewerTextureIdentity) {
        self.current_texture = Some(identity);
        self.state = ViewerTextureLifecycleState::Bound;
    }

    pub fn release(&mut self, reason: ViewerTextureReleaseReason) {
        self.current_texture = None;
        self.state = ViewerTextureLifecycleState::Released;
        self.last_release_reason = Some(reason);
        self.release_count += 1;
    }

    pub fn state(&self) -> ViewerTextureLifecycleState {
        self.state
    }

    pub fn current_texture(&self) -> Option<&ViewerTextureIdentity> {
        self.current_texture.as_ref()
    }

    pub fn last_release_reason(&self) -> Option<ViewerTextureReleaseReason> {
        self.last_release_reason
    }

    pub fn release_count(&self) -> u64 {
        self.release_count
    }

    pub fn writes_catalog_state(&self) -> bool {
        false
    }

    pub fn writes_sidecar_state(&self) -> bool {
        false
    }

    pub fn uses_original_path_as_write_destination(&self) -> bool {
        false
    }

    pub fn persistent_gpu_cache_enabled(&self) -> bool {
        false
    }

    pub fn is_rebuildable(&self) -> bool {
        true
    }
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProbeRequest {
    pub source_path: String,
}

#[cfg(feature = "color-probe")]
impl ColorProbeRequest {
    pub fn new(source_path: impl AsRef<str>) -> Self {
        Self {
            source_path: source_path.as_ref().to_string(),
        }
    }
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbePlatform {
    Macos,
    UnsupportedPlatform,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeStatus {
    Success,
    Failed,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeInputProfile {
    Srgb,
    DisplayP3,
    None,
    Unknown,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeOutputProfile {
    Srgb,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeTransformPath {
    EmbeddedIccToLinearDisplayP3ToSrgb,
    AssumeSrgbToLinearDisplayP3ToSrgb,
    Unavailable,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeErrorCategory {
    UnsupportedPlatform,
    MissingFile,
    NotAFile,
    ReadFailed,
    InvalidJpeg,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProbeResult {
    pub platform: ColorProbePlatform,
    pub source_path: String,
    pub source_sha256: Option<String>,
    pub status: ColorProbeStatus,
    pub input_profile: ColorProbeInputProfile,
    pub embedded_icc: bool,
    pub working_space: WorkingColorSpace,
    pub output_profile: ColorProbeOutputProfile,
    pub transform_path: ColorProbeTransformPath,
    pub error_category: Option<ColorProbeErrorCategory>,
    pub message: String,
}

#[cfg(feature = "color-probe")]
pub fn probe_color_profile(request: ColorProbeRequest) -> ColorProbeResult {
    let source_path = request.source_path;
    let path = PathBuf::from(&source_path);
    let platform = current_color_probe_platform();

    if platform == ColorProbePlatform::UnsupportedPlatform {
        return failed_color_probe(
            platform,
            source_path,
            None,
            ColorProbeErrorCategory::UnsupportedPlatform,
            "Color probe is available only on macOS for Phase 13 proof work.",
        );
    }

    let metadata = match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => {
            return failed_color_probe(
                platform,
                source_path,
                None,
                ColorProbeErrorCategory::NotAFile,
                "Color probe source is not a file.",
            );
        }
        Err(error) => {
            let category = if error.kind() == std::io::ErrorKind::NotFound {
                ColorProbeErrorCategory::MissingFile
            } else {
                ColorProbeErrorCategory::ReadFailed
            };
            return failed_color_probe(
                platform,
                source_path,
                None,
                category,
                "Color probe source could not be read.",
            );
        }
    };

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return failed_color_probe(
                platform,
                source_path,
                None,
                ColorProbeErrorCategory::ReadFailed,
                "Color probe source could not be read.",
            );
        }
    };
    let source_sha256 = sha256_hex(&bytes);

    let icc_profile = match first_icc_profile(&bytes) {
        Ok(profile) => profile,
        Err(category) => {
            return failed_color_probe(
                platform,
                source_path,
                Some(source_sha256),
                category,
                "Color probe source is not a readable JPEG marker stream.",
            );
        }
    };

    let input_profile = match icc_profile.as_deref() {
        Some(profile) => classify_icc_profile(profile),
        None => ColorProbeInputProfile::None,
    };
    let transform_path = if icc_profile.is_some() {
        ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb
    } else {
        ColorProbeTransformPath::AssumeSrgbToLinearDisplayP3ToSrgb
    };

    ColorProbeResult {
        platform,
        source_path,
        source_sha256: Some(source_sha256),
        status: ColorProbeStatus::Success,
        input_profile,
        embedded_icc: icc_profile.is_some(),
        working_space: SPIKE_003_COLOR_GATE.working_space,
        output_profile: ColorProbeOutputProfile::Srgb,
        transform_path,
        error_category: None,
        message: format!(
            "Color probe recorded profile metadata for {} bytes.",
            metadata.len()
        ),
    }
}

/// Build a local alpha render plan from a decode plan.
pub fn plan_preview_render(decode_plan: PreviewDecodePlan) -> PreviewRenderPlan {
    let status = match decode_plan.status {
        PreviewDecodeStatus::Ready => PreviewRenderStatus::Ready,
        PreviewDecodeStatus::Unsupported => PreviewRenderStatus::Unsupported,
        PreviewDecodeStatus::BlockedByMissingRawFixtureProbe => {
            PreviewRenderStatus::BlockedByDecode
        }
    };

    let message = match status {
        PreviewRenderStatus::Ready => {
            "Preview source is ready for a display-profile-aware surface.".to_string()
        }
        PreviewRenderStatus::Unsupported | PreviewRenderStatus::BlockedByDecode => {
            decode_plan.message
        }
    };

    PreviewRenderPlan {
        source_path: decode_plan.source_path,
        status,
        color_behavior: SPIKE_003_COLOR_GATE.preview,
        message,
    }
}

#[cfg(feature = "color-probe")]
fn failed_color_probe(
    platform: ColorProbePlatform,
    source_path: String,
    source_sha256: Option<String>,
    category: ColorProbeErrorCategory,
    message: &str,
) -> ColorProbeResult {
    ColorProbeResult {
        platform,
        source_path,
        source_sha256,
        status: ColorProbeStatus::Failed,
        input_profile: ColorProbeInputProfile::Unknown,
        embedded_icc: false,
        working_space: SPIKE_003_COLOR_GATE.working_space,
        output_profile: ColorProbeOutputProfile::Srgb,
        transform_path: ColorProbeTransformPath::Unavailable,
        error_category: Some(category),
        message: message.to_string(),
    }
}

#[cfg(feature = "color-probe")]
fn current_color_probe_platform() -> ColorProbePlatform {
    if cfg!(target_os = "macos") {
        ColorProbePlatform::Macos
    } else {
        ColorProbePlatform::UnsupportedPlatform
    }
}

#[cfg(feature = "color-probe")]
fn first_icc_profile(bytes: &[u8]) -> Result<Option<Vec<u8>>, ColorProbeErrorCategory> {
    if bytes.len() < 2 || bytes[0..2] != [0xff, 0xd8] {
        return Err(ColorProbeErrorCategory::InvalidJpeg);
    }

    let mut index = 2;
    while index + 4 <= bytes.len() {
        if bytes[index] != 0xff {
            return Err(ColorProbeErrorCategory::InvalidJpeg);
        }

        let marker = bytes[index + 1];
        if marker == 0xd9 || marker == 0xda {
            return Ok(None);
        }

        let length = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;
        if length < 2 || index + 2 + length > bytes.len() {
            return Err(ColorProbeErrorCategory::InvalidJpeg);
        }

        let marker_payload = &bytes[index + 4..index + 2 + length];
        if marker == 0xe2
            && marker_payload.starts_with(b"ICC_PROFILE\0")
            && marker_payload.len() >= 14
        {
            return Ok(Some(marker_payload[14..].to_vec()));
        }

        index += 2 + length;
    }

    Ok(None)
}

#[cfg(feature = "color-probe")]
fn classify_icc_profile(profile: &[u8]) -> ColorProbeInputProfile {
    match sha256_hex(profile).as_str() {
        "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e" => {
            ColorProbeInputProfile::Srgb
        }
        "0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0" => {
            ColorProbeInputProfile::DisplayP3
        }
        _ if profile
            .windows(b"sRGB".len())
            .any(|window| window.eq_ignore_ascii_case(b"sRGB")) =>
        {
            ColorProbeInputProfile::Srgb
        }
        _ => ColorProbeInputProfile::Unknown,
    }
}

#[cfg(feature = "color-probe")]
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Build a render request for a draft exposure/contrast preview update.
pub fn plan_exposure_contrast_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_white_balance_preview(
        preview_plan,
        exposure,
        contrast,
        WhiteBalanceRenderAdjustment::neutral(),
    );
    if request.status == PreviewRenderStatus::Ready {
        request.message = "Draft exposure/contrast preview request is ready.".to_string();
    }
    request
}

/// Build a render request for a draft white-balance preview update.
pub fn plan_white_balance_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let message = match preview_plan.status {
        PreviewRenderStatus::Ready => "White balance preview request is ready.".to_string(),
        PreviewRenderStatus::BlockedByDecode | PreviewRenderStatus::Unsupported => {
            preview_plan.message.clone()
        }
    };

    ExposureContrastPreviewRequest {
        source_path: preview_plan.source_path,
        status: preview_plan.status,
        color_behavior: preview_plan.color_behavior,
        exposure,
        contrast,
        white_balance,
        message,
    }
}

/// Build a render-side request for exporting an edited raster source as sRGB JPEG.
pub fn plan_jpeg_srgb_export(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    plan_jpeg_srgb_export_with_white_balance(
        source_path,
        output_path,
        exposure,
        contrast,
        WhiteBalanceRenderAdjustment::neutral(),
        quality,
    )
}

/// Build a render-side request for exporting an edited raster source with white balance.
pub fn plan_jpeg_srgb_export_with_white_balance(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    JpegSrgbExportRenderRequest {
        source_kind: ExportRenderSourceKind::RasterSource,
        source_path: source_path.into(),
        output_path: output_path.into(),
        color_behavior: SPIKE_003_COLOR_GATE.export,
        exposure,
        contrast,
        white_balance,
        quality,
        message: "JPEG sRGB export request is ready.".to_string(),
    }
}

/// Build a render-side request for exporting a full-resolution RAW-derived source artifact.
pub fn plan_raw_derived_jpeg_srgb_export(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    plan_raw_derived_jpeg_srgb_export_with_white_balance(
        source_path,
        output_path,
        exposure,
        contrast,
        WhiteBalanceRenderAdjustment::neutral(),
        quality,
    )
}

/// Build a render-side request for exporting a RAW-derived source artifact with white balance.
pub fn plan_raw_derived_jpeg_srgb_export_with_white_balance(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    JpegSrgbExportRenderRequest {
        source_kind: ExportRenderSourceKind::RawFullResolutionArtifact,
        source_path: source_path.into(),
        output_path: output_path.into(),
        color_behavior: SPIKE_003_COLOR_GATE.export,
        exposure,
        contrast,
        white_balance,
        quality,
        message: "RAW-derived JPEG sRGB export request is ready.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-render");
    }

    #[test]
    fn records_spike_003_color_gate() {
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.path,
            super::ColorManagementPath::CoreImageColorManagementPrimary
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.working_space,
            super::WorkingColorSpace::LinearDisplayP3
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.preview,
            super::PreviewColorBehavior::DisplayProfileAware
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.export,
            super::ExportColorBehavior::SrgbDefaultDisplayP3Supported
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.fixture_set,
            super::ColorFixtureStatus::MissingTaggedRasterFixtures
        );
        assert_eq!(super::COLOR_BLOCKING_TAG, "color-blocking");
    }

    #[test]
    fn plans_display_aware_preview_from_decode_plan() {
        let decode_plan = silica_decode::plan_preview_decode("/tmp/sample.jpg", false);
        let render_plan = super::plan_preview_render(decode_plan);

        assert_eq!(render_plan.status, super::PreviewRenderStatus::Ready);
        assert_eq!(render_plan.source_path, "/tmp/sample.jpg");
        assert_eq!(
            render_plan.color_behavior,
            super::PreviewColorBehavior::DisplayProfileAware
        );
        assert!(render_plan.message.contains("display-profile-aware"));

        let raw_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.dng",
            false,
        ));
        assert_eq!(raw_plan.status, super::PreviewRenderStatus::BlockedByDecode);
        assert!(raw_plan.message.contains("Core Image RAW preview"));

        let unsupported_plan =
            super::plan_preview_render(silica_decode::plan_preview_decode("/tmp/notes.txt", true));
        assert_eq!(
            unsupported_plan.status,
            super::PreviewRenderStatus::Unsupported
        );
    }

    #[test]
    fn plans_exposure_contrast_preview_request_from_ready_preview() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));

        let request = super::plan_exposure_contrast_preview(preview_plan, 0.5, -8.0);

        assert_eq!(request.status, super::PreviewRenderStatus::Ready);
        assert_eq!(request.source_path, "/tmp/sample.jpg");
        assert_eq!(request.exposure, 0.5);
        assert_eq!(request.contrast, -8.0);
        assert_eq!(
            request.color_behavior,
            super::PreviewColorBehavior::DisplayProfileAware
        );
        assert!(request.message.contains("exposure/contrast"));
    }

    #[test]
    fn plans_jpeg_srgb_export_request() {
        let request =
            super::plan_jpeg_srgb_export("/tmp/original.jpg", "/tmp/exported.jpg", 0.5, -8.0, 90);

        assert_eq!(request.source_path, "/tmp/original.jpg");
        assert_eq!(
            request.source_kind,
            super::ExportRenderSourceKind::RasterSource
        );
        assert_eq!(request.output_path, "/tmp/exported.jpg");
        assert_eq!(request.exposure, 0.5);
        assert_eq!(request.contrast, -8.0);
        assert_eq!(request.quality, 90);
        assert_eq!(
            request.color_behavior,
            super::ExportColorBehavior::SrgbDefaultDisplayP3Supported
        );
        assert!(request.message.contains("JPEG sRGB export"));
    }

    #[test]
    fn plans_white_balance_preview_and_export_requests() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let white_balance = super::WhiteBalanceRenderAdjustment {
            mode: super::WhiteBalanceRenderMode::Custom,
            temperature: 6500.0,
            tint: 20.0,
        };

        let preview = super::plan_white_balance_preview(preview_plan, 0.25, -3.0, white_balance);
        let export = super::plan_jpeg_srgb_export_with_white_balance(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            white_balance,
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.exposure, 0.25);
        assert_eq!(preview.contrast, -3.0);
        assert_eq!(preview.white_balance, white_balance);
        assert!(preview.message.contains("White balance"));
        assert_eq!(export.white_balance, white_balance);
        assert_eq!(export.exposure, 0.25);
        assert_eq!(export.contrast, -3.0);
        assert!(!export.uses_viewer_texture_cache_as_source());
    }

    #[test]
    fn plans_raw_derived_jpeg_srgb_export_from_full_resolution_artifact_not_viewer_cache() {
        let request = super::plan_raw_derived_jpeg_srgb_export(
            "/tmp/silicaraw-library/exports/raw-derived/photo-7-source.jpg",
            "/tmp/exported-photo-7.jpg",
            0.5,
            -8.0,
            90,
        );

        assert_eq!(
            request.source_kind,
            super::ExportRenderSourceKind::RawFullResolutionArtifact
        );
        assert_eq!(
            request.source_path,
            "/tmp/silicaraw-library/exports/raw-derived/photo-7-source.jpg"
        );
        assert!(!request.uses_viewer_texture_cache_as_source());
        assert!(request.message.contains("RAW-derived JPEG sRGB export"));
    }

    #[test]
    fn viewer_render_request_is_read_only_and_can_carry_future_texture_identity() {
        let request = super::ViewerPreviewRenderRequest::new(
            super::ViewerPreviewRenderRequestId(7),
            "photo-7",
            "/tmp/source.raw",
            super::ViewerPreviewViewport::new(1200, 675, 1.5),
            super::ViewerPreviewInput::future_texture(
                "decode-cache/photo-7/request-7",
                4032,
                3024,
                super::ViewerPreviewPixelFormat::Bgra8Unorm,
            ),
            3,
        );

        assert_eq!(request.request_id, super::ViewerPreviewRenderRequestId(7));
        assert_eq!(request.photo_id, "photo-7");
        assert_eq!(request.viewport.width_px, 1200);
        assert_eq!(request.viewport.height_px, 675);
        assert_eq!(request.viewport.backing_scale_factor, 1.5);
        assert!(!request.writes_catalog_state());
        assert!(!request.contains_image_pixels());
        assert_eq!(request.edit_graph_revision, 3);
        assert!(matches!(
            request.input,
            super::ViewerPreviewInput::FutureTexture {
                width_px: 4032,
                height_px: 3024,
                pixel_format: super::ViewerPreviewPixelFormat::Bgra8Unorm,
                ..
            }
        ));
    }

    #[test]
    fn viewer_input_from_decoded_handoff_carries_artifact_identity_without_pixels() {
        let handoff = silica_decode::DecodedImageHandoff {
            source_path: "/tmp/sample.cr2".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            decoder_backend: silica_decode::DecodedImageDecoderBackend::CoreImageRaw,
            status: silica_decode::DecodedImageHandoffStatus::Ready,
            width: Some(5184),
            height: Some(3456),
            orientation: None,
            input_profile: "unknown".to_string(),
            working_space: "linear_display_p3".to_string(),
            cache_identity: Some(silica_decode::DecodedImageCacheIdentity {
                cache_key: "previews/raw/photo-1".to_string(),
                disposable: true,
            }),
            pixel_format: Some(silica_decode::DecodedImagePixelFormat::Rgba16Float),
            message: "ready".to_string(),
        };

        let input = super::ViewerPreviewInput::from_decoded_handoff(&handoff);

        assert!(!input.contains_image_pixels());
        match &input {
            super::ViewerPreviewInput::DecodedImageArtifact {
                cache_key,
                source_sha256,
                width_px,
                height_px,
                pixel_format,
                decoder_backend,
                input_profile,
                working_space,
            } => {
                assert_eq!(cache_key, "previews/raw/photo-1");
                assert_eq!(source_sha256.as_deref(), Some("fixture-hash"));
                assert_eq!(*width_px, 5184);
                assert_eq!(*height_px, 3456);
                assert_eq!(*pixel_format, super::ViewerPreviewPixelFormat::Rgba16Float);
                assert_eq!(decoder_backend, "core_image_raw");
                assert_eq!(input_profile, "unknown");
                assert_eq!(working_space, "linear_display_p3");
            }
            other => panic!("expected decoded image artifact input, got {other:?}"),
        }
    }

    #[test]
    fn viewer_input_from_jpeg_srgb_handoff_marks_artifact_format() {
        let handoff = silica_decode::DecodedImageHandoff {
            source_path: "/tmp/sample.cr2".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            decoder_backend: silica_decode::DecodedImageDecoderBackend::CoreImageRaw,
            status: silica_decode::DecodedImageHandoffStatus::Ready,
            width: Some(2048),
            height: Some(1365),
            orientation: None,
            input_profile: "core_image_raw".to_string(),
            working_space: "srgb".to_string(),
            cache_identity: Some(silica_decode::DecodedImageCacheIdentity {
                cache_key: "raw-preview:test".to_string(),
                disposable: true,
            }),
            pixel_format: Some(silica_decode::DecodedImagePixelFormat::JpegSrgb8),
            message: "Core Image emitted a bounded JPEG sRGB preview artifact.".to_string(),
        };

        let input = super::ViewerPreviewInput::from_decoded_handoff(&handoff);

        match input {
            super::ViewerPreviewInput::DecodedImageArtifact { pixel_format, .. } => {
                assert_eq!(pixel_format, super::ViewerPreviewPixelFormat::JpegSrgb8);
            }
            other => panic!("expected decoded JPEG artifact input, got {other:?}"),
        }
    }

    #[test]
    fn viewer_input_from_blocked_handoff_stays_decode_blocked() {
        let handoff = silica_decode::DecodedImageHandoff {
            source_path: "/tmp/sample.raw".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            decoder_backend: silica_decode::DecodedImageDecoderBackend::CoreImageRaw,
            status: silica_decode::DecodedImageHandoffStatus::BlockedPendingEvidence,
            width: None,
            height: None,
            orientation: None,
            input_profile: "unknown".to_string(),
            working_space: "linear_display_p3".to_string(),
            cache_identity: None,
            pixel_format: None,
            message: "blocked".to_string(),
        };

        let input = super::ViewerPreviewInput::from_decoded_handoff(&handoff);

        assert_eq!(
            input,
            super::ViewerPreviewInput::NoPixelsYet {
                readiness: super::PreviewRenderStatus::BlockedByDecode,
            }
        );
        assert!(!input.contains_image_pixels());
    }

    #[test]
    fn viewer_render_scheduler_records_latest_request_wins() {
        let first = super::ViewerPreviewRenderRequest::new(
            super::ViewerPreviewRenderRequestId(1),
            "photo-1",
            "/tmp/source.jpg",
            super::ViewerPreviewViewport::new(1000, 600, 2.0),
            super::ViewerPreviewInput::no_pixels_yet(super::PreviewRenderStatus::Ready),
            1,
        );
        let second = super::ViewerPreviewRenderRequest::new(
            super::ViewerPreviewRenderRequestId(2),
            "photo-1",
            "/tmp/source.jpg",
            super::ViewerPreviewViewport::new(1000, 600, 2.0),
            super::ViewerPreviewInput::no_pixels_yet(super::PreviewRenderStatus::Ready),
            2,
        );
        let mut scheduler = super::ViewerPreviewRenderScheduler::default();

        let first_result = scheduler.schedule(first);
        assert_eq!(
            first_result.accepted_request_id,
            super::ViewerPreviewRenderRequestId(1)
        );
        assert_eq!(first_result.replaced_request_id, None);
        assert_eq!(
            scheduler.latest_request_id(),
            Some(super::ViewerPreviewRenderRequestId(1))
        );

        let second_result = scheduler.schedule(second);
        assert_eq!(
            second_result.accepted_request_id,
            super::ViewerPreviewRenderRequestId(2)
        );
        assert_eq!(
            second_result.replaced_request_id,
            Some(super::ViewerPreviewRenderRequestId(1))
        );
        assert_eq!(
            scheduler.latest_request_id(),
            Some(super::ViewerPreviewRenderRequestId(2))
        );
        assert!(!scheduler.latest_request().unwrap().writes_catalog_state());
    }

    #[test]
    fn viewer_texture_lifecycle_is_disposable_and_safe_to_clear() {
        let mut lifecycle = super::ViewerTextureLifecycle::default();
        assert_eq!(lifecycle.state(), super::ViewerTextureLifecycleState::Empty);
        assert!(!lifecycle.writes_catalog_state());
        assert!(!lifecycle.writes_sidecar_state());
        assert!(!lifecycle.uses_original_path_as_write_destination());
        assert!(!lifecycle.persistent_gpu_cache_enabled());
        assert!(lifecycle.is_rebuildable());

        let identity = super::ViewerTextureIdentity::new(
            super::ViewerPreviewRenderRequestId(12),
            "texture/request-12",
            super::ViewerTextureDrawableSize::new(1200, 675),
        );
        lifecycle.bind_texture(identity.clone());

        assert_eq!(lifecycle.state(), super::ViewerTextureLifecycleState::Bound);
        assert_eq!(lifecycle.current_texture(), Some(&identity));

        lifecycle.release(super::ViewerTextureReleaseReason::PhotoChanged);
        assert_eq!(
            lifecycle.state(),
            super::ViewerTextureLifecycleState::Released
        );
        assert_eq!(lifecycle.current_texture(), None);
        assert_eq!(
            lifecycle.last_release_reason(),
            Some(super::ViewerTextureReleaseReason::PhotoChanged)
        );
        assert_eq!(lifecycle.release_count(), 1);

        lifecycle.bind_texture(super::ViewerTextureIdentity::new(
            super::ViewerPreviewRenderRequestId(13),
            "texture/request-13",
            super::ViewerTextureDrawableSize::new(1600, 900),
        ));
        lifecycle.release(super::ViewerTextureReleaseReason::DrawableResized);
        lifecycle.bind_texture(super::ViewerTextureIdentity::new(
            super::ViewerPreviewRenderRequestId(14),
            "texture/request-14",
            super::ViewerTextureDrawableSize::new(1600, 900),
        ));
        lifecycle.release(super::ViewerTextureReleaseReason::LibraryClosed);
        lifecycle.bind_texture(super::ViewerTextureIdentity::new(
            super::ViewerPreviewRenderRequestId(15),
            "texture/request-15",
            super::ViewerTextureDrawableSize::new(1600, 900),
        ));
        lifecycle.release(super::ViewerTextureReleaseReason::AppClosed);

        assert_eq!(lifecycle.release_count(), 4);
        assert_eq!(
            lifecycle.last_release_reason(),
            Some(super::ViewerTextureReleaseReason::AppClosed)
        );
    }

    #[test]
    fn decoded_preview_artifact_can_become_disposable_texture_identity() {
        let request = super::ViewerPreviewRenderRequest::new(
            super::ViewerPreviewRenderRequestId(31),
            "photo-31",
            "/tmp/sample.cr2",
            super::ViewerPreviewViewport::new(1200, 675, 1.5),
            super::ViewerPreviewInput::DecodedImageArtifact {
                cache_key: "raw-preview:v1:photo-31".to_string(),
                source_sha256: Some("fixture-hash".to_string()),
                width_px: 2048,
                height_px: 1365,
                pixel_format: super::ViewerPreviewPixelFormat::JpegSrgb8,
                decoder_backend: "core_image_raw".to_string(),
                input_profile: "core_image_raw".to_string(),
                working_space: "srgb".to_string(),
            },
            3,
        );

        let identity = super::ViewerTextureIdentity::from_preview_request(&request)
            .expect("decoded artifact should produce texture identity");

        assert_eq!(identity.request_id, super::ViewerPreviewRenderRequestId(31));
        assert_eq!(identity.texture_key, "raw-preview:v1:photo-31");
        assert_eq!(
            identity.drawable_size,
            super::ViewerTextureDrawableSize::new(1800, 1013)
        );
        assert!(!request.writes_catalog_state());
        assert!(!request.contains_image_pixels());
    }

    #[test]
    fn viewer_preview_request_carries_exposure_contrast_draft_without_state_writes() {
        let request = super::ViewerPreviewRenderRequest::new(
            super::ViewerPreviewRenderRequestId(51),
            "photo-51",
            "/tmp/sample.cr2",
            super::ViewerPreviewViewport::new(1200, 675, 1.5),
            super::ViewerPreviewInput::DecodedImageArtifact {
                cache_key: "raw-preview:v1:photo-51".to_string(),
                source_sha256: Some("fixture-hash".to_string()),
                width_px: 2048,
                height_px: 1365,
                pixel_format: super::ViewerPreviewPixelFormat::JpegSrgb8,
                decoder_backend: "core_image_raw".to_string(),
                input_profile: "core_image_raw".to_string(),
                working_space: "srgb".to_string(),
            },
            7,
        )
        .with_exposure_contrast_draft(0.5, -8.0);

        assert_eq!(
            request.exposure_contrast_draft,
            Some(super::ViewerExposureContrastDraft {
                exposure: 0.5,
                contrast: -8.0
            })
        );
        assert_eq!(request.edit_graph_revision, 7);
        assert!(!request.writes_catalog_state());
        assert!(!request.contains_image_pixels());
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_classifies_embedded_srgb_profile() {
        let path = write_color_probe_fixture(
            "srgb",
            jpeg_with_icc_profile(b"header IEC sRGB profile bytes"),
        );

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.source_path, path.to_string_lossy());
        assert_eq!(result.status, super::ColorProbeStatus::Success);
        assert_eq!(result.input_profile, super::ColorProbeInputProfile::Srgb);
        assert!(result.embedded_icc);
        assert!(result.source_sha256.is_some());
        assert_eq!(
            result.working_space,
            super::WorkingColorSpace::LinearDisplayP3
        );
        assert_eq!(result.output_profile, super::ColorProbeOutputProfile::Srgb);
        assert_eq!(
            result.transform_path,
            super::ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb
        );
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_classifies_local_display_p3_profile() {
        let profile = std::fs::read("/System/Library/ColorSync/Profiles/Display P3.icc")
            .expect("local Display P3 profile");
        let path = write_color_probe_fixture("display-p3", jpeg_with_icc_profile(&profile));

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.status, super::ColorProbeStatus::Success);
        assert_eq!(
            result.input_profile,
            super::ColorProbeInputProfile::DisplayP3
        );
        assert!(result.embedded_icc);
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_records_untagged_raster_as_assume_srgb() {
        let path = write_color_probe_fixture("untagged", minimal_jpeg_without_icc());

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.status, super::ColorProbeStatus::Success);
        assert_eq!(result.input_profile, super::ColorProbeInputProfile::None);
        assert!(!result.embedded_icc);
        assert_eq!(
            result.transform_path,
            super::ColorProbeTransformPath::AssumeSrgbToLinearDisplayP3ToSrgb
        );
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_reports_missing_file_without_panicking() {
        let path = std::env::temp_dir().join(unique_color_probe_name("missing"));
        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));

        assert_eq!(result.status, super::ColorProbeStatus::Failed);
        assert_eq!(
            result.error_category,
            Some(super::ColorProbeErrorCategory::MissingFile)
        );
        assert_eq!(result.source_sha256, None);
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn write_color_probe_fixture(name: &str, bytes: Vec<u8>) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(unique_color_probe_name(name));
        std::fs::write(&path, bytes).expect("write color probe fixture");
        path
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn unique_color_probe_name(name: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        format!("silicaraw-color-probe-{name}-{nanos}.jpg")
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn jpeg_with_icc_profile(profile: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0xff, 0xd8];
        let mut payload = b"ICC_PROFILE\0\x01\x01".to_vec();
        payload.extend_from_slice(profile);
        bytes.extend_from_slice(&[0xff, 0xe2]);
        bytes.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&minimal_jpeg_without_icc()[2..]);
        bytes
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn minimal_jpeg_without_icc() -> Vec<u8> {
        vec![0xff, 0xd8, 0xff, 0xd9]
    }
}
