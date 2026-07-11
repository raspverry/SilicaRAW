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
    pub tone_recovery: ToneRecoveryRenderAdjustment,
    pub color_presence: ColorPresenceRenderAdjustment,
    pub tone_curve: ToneCurveRenderAdjustment,
    pub hsl_color_mixer: HslColorMixerRenderAdjustment,
    pub detail: DetailRenderAdjustment,
    pub geometry: GeometryRenderAdjustment,
    pub masks: Vec<ManualMaskRenderAdjustment>,
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
    pub tone_recovery: ToneRecoveryRenderAdjustment,
    pub color_presence: ColorPresenceRenderAdjustment,
    pub tone_curve: ToneCurveRenderAdjustment,
    pub hsl_color_mixer: HslColorMixerRenderAdjustment,
    pub detail: DetailRenderAdjustment,
    pub geometry: GeometryRenderAdjustment,
    pub quality: u8,
    pub message: String,
}

/// Manual mask geometry carried by preview render requests.
#[derive(Debug, Clone, PartialEq)]
pub enum ManualMaskRenderGeometry {
    LinearGradient {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    },
    RadialGradient {
        center_x: f64,
        center_y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
    },
    BrushRaster {
        width: u32,
        height: u32,
        alpha: Vec<u8>,
        cache_key: String,
    },
}

/// Manual mask adjustment carried by preview render requests.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMaskRenderAdjustment {
    pub id: String,
    pub enabled: bool,
    pub invert: bool,
    pub opacity: f64,
    pub feather: f64,
    pub geometry: ManualMaskRenderGeometry,
    pub exposure: f64,
    pub contrast: f64,
}

/// Normalized brush point used by the pure CPU reference rasterizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushMaskRasterPoint {
    pub x: f64,
    pub y: f64,
}

/// Normalized brush stroke used by the pure CPU reference rasterizer.
#[derive(Debug, Clone, PartialEq)]
pub struct BrushMaskRasterStroke {
    pub id: String,
    pub radius: f64,
    pub points: Vec<BrushMaskRasterPoint>,
}

/// Disposable alpha plane generated from durable brush strokes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushMaskRaster {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<u8>,
    pub cache_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushMaskRasterError {
    path: String,
    message: String,
}

impl BrushMaskRasterError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BrushMaskRasterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for BrushMaskRasterError {}

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

/// Tone recovery values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneRecoveryRenderAdjustment {
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
}

impl ToneRecoveryRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
        }
    }
}

/// Tone curve mode carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneCurveRenderMode {
    None,
    Parametric,
    Point,
}

/// One normalized point in a render-side tone curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneCurveRenderPoint {
    pub x: f64,
    pub y: f64,
}

/// Tone curve values carried by preview/export render requests.
#[derive(Debug, Clone, PartialEq)]
pub struct ToneCurveRenderAdjustment {
    pub mode: ToneCurveRenderMode,
    pub rgb_curve: Vec<ToneCurveRenderPoint>,
    pub red_curve: Vec<ToneCurveRenderPoint>,
    pub green_curve: Vec<ToneCurveRenderPoint>,
    pub blue_curve: Vec<ToneCurveRenderPoint>,
}

impl ToneCurveRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            mode: ToneCurveRenderMode::None,
            rgb_curve: Vec::new(),
            red_curve: Vec::new(),
            green_curve: Vec::new(),
            blue_curve: Vec::new(),
        }
    }
}

/// One HSL color mixer channel carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HslColorChannelRenderAdjustment {
    pub hue: f64,
    pub saturation: f64,
    pub luminance: f64,
}

impl HslColorChannelRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            luminance: 0.0,
        }
    }
}

/// HSL color mixer values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HslColorMixerRenderAdjustment {
    pub red: HslColorChannelRenderAdjustment,
    pub orange: HslColorChannelRenderAdjustment,
    pub yellow: HslColorChannelRenderAdjustment,
    pub green: HslColorChannelRenderAdjustment,
    pub aqua: HslColorChannelRenderAdjustment,
    pub blue: HslColorChannelRenderAdjustment,
    pub purple: HslColorChannelRenderAdjustment,
    pub magenta: HslColorChannelRenderAdjustment,
}

impl HslColorMixerRenderAdjustment {
    pub fn neutral() -> Self {
        let neutral = HslColorChannelRenderAdjustment::neutral();
        Self {
            red: neutral,
            orange: neutral,
            yellow: neutral,
            green: neutral,
            aqua: neutral,
            blue: neutral,
            purple: neutral,
            magenta: neutral,
        }
    }
}

/// Sharpening values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailSharpeningRenderAdjustment {
    pub amount: f64,
    pub radius: f64,
    pub detail: f64,
    pub masking: f64,
}

impl DetailSharpeningRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            amount: 0.0,
            radius: 1.0,
            detail: 25.0,
            masking: 0.0,
        }
    }

    pub fn is_neutral(self) -> bool {
        self == Self::neutral()
    }
}

/// Non-MLX noise reduction values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailNoiseReductionRenderAdjustment {
    pub luminance: f64,
    pub detail: f64,
    pub contrast: f64,
    pub color: f64,
    pub color_detail: f64,
}

impl DetailNoiseReductionRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            luminance: 0.0,
            detail: 50.0,
            contrast: 0.0,
            color: 25.0,
            color_detail: 50.0,
        }
    }

    pub fn is_neutral(self) -> bool {
        self == Self::neutral()
    }
}

/// Detail values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailRenderAdjustment {
    pub sharpening: DetailSharpeningRenderAdjustment,
    pub noise_reduction: DetailNoiseReductionRenderAdjustment,
}

impl DetailRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            sharpening: DetailSharpeningRenderAdjustment::neutral(),
            noise_reduction: DetailNoiseReductionRenderAdjustment::neutral(),
        }
    }

    pub fn is_neutral(self) -> bool {
        self.sharpening.is_neutral() && self.noise_reduction.is_neutral()
    }
}

/// Normalized crop rectangle carried by preview/export render requests.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryCropRenderAdjustment {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub aspect: Option<String>,
}

/// Perspective/scale transform controls carried for explicit unsupported-state handling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryTransformRenderAdjustment {
    pub vertical: f64,
    pub horizontal: f64,
    pub aspect: f64,
    pub scale: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

impl GeometryTransformRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            vertical: 0.0,
            horizontal: 0.0,
            aspect: 0.0,
            scale: 100.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    pub fn is_neutral(self) -> bool {
        self == Self::neutral()
    }
}

/// Supported non-destructive geometry carried through preview/export contracts.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryRenderAdjustment {
    pub crop: Option<GeometryCropRenderAdjustment>,
    pub rotation: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub transform: GeometryTransformRenderAdjustment,
}

impl GeometryRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            crop: None,
            rotation: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
            transform: GeometryTransformRenderAdjustment::neutral(),
        }
    }

    pub fn is_neutral(&self) -> bool {
        self.crop.is_none()
            && self.rotation == 0.0
            && !self.flip_horizontal
            && !self.flip_vertical
            && self.transform.is_neutral()
    }
}

/// Color presence values carried by preview/export render requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPresenceRenderAdjustment {
    pub vibrance: f64,
    pub saturation: f64,
}

impl ColorPresenceRenderAdjustment {
    pub fn neutral() -> Self {
        Self {
            vibrance: 0.0,
            saturation: 0.0,
        }
    }
}

/// Histogram data for a real RGB pixel buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbHistogram {
    pub red: Vec<u32>,
    pub green: Vec<u32>,
    pub blue: Vec<u32>,
    pub luminance: Vec<u32>,
    pub pixel_count: u64,
}

/// Histogram computation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistogramError {
    RgbByteLengthNotMultipleOfThree { byte_len: usize },
}

impl std::fmt::Display for HistogramError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RgbByteLengthNotMultipleOfThree { byte_len } => {
                write!(
                    formatter,
                    "RGB byte length must be divisible by 3: {byte_len}"
                )
            }
        }
    }
}

impl std::error::Error for HistogramError {}

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
        _ if icc_rgb_primaries_match(profile, DISPLAY_P3_D50_XYZ) => {
            ColorProbeInputProfile::DisplayP3
        }
        _ if icc_rgb_primaries_match(profile, SRGB_D50_XYZ) => ColorProbeInputProfile::Srgb,
        _ if profile_contains_ascii(profile, b"sRGB") => ColorProbeInputProfile::Srgb,
        _ => ColorProbeInputProfile::Unknown,
    }
}

#[cfg(feature = "color-probe")]
const DISPLAY_P3_D50_XYZ: [[f64; 3]; 3] = [
    [0.5151214599609375, 0.2411956787109375, -0.0010528564453125],
    [0.2919769287109375, 0.6922454833984375, 0.0418853759765625],
    [0.1571044921875, 0.0665740966796875, 0.7840728759765625],
];

#[cfg(feature = "color-probe")]
const SRGB_D50_XYZ: [[f64; 3]; 3] = [
    [0.436065673828125, 0.2224884033203125, 0.013916015625],
    [0.3851470947265625, 0.7168731689453125, 0.097076416015625],
    [0.14306640625, 0.06060791015625, 0.7140960693359375],
];

#[cfg(feature = "color-probe")]
fn icc_rgb_primaries_match(profile: &[u8], expected: [[f64; 3]; 3]) -> bool {
    let Some(actual) = icc_rgb_primaries(profile) else {
        return false;
    };
    actual
        .iter()
        .flatten()
        .zip(expected.iter().flatten())
        .all(|(actual, expected)| (actual - expected).abs() <= 0.02)
}

#[cfg(feature = "color-probe")]
fn icc_rgb_primaries(profile: &[u8]) -> Option<[[f64; 3]; 3]> {
    if profile.len() < 132 || &profile[36..40] != b"acsp" {
        return None;
    }
    let tag_count = read_u32_be(profile, 128)? as usize;
    let tag_table_end = 132_usize.checked_add(tag_count.checked_mul(12)?)?;
    if tag_table_end > profile.len() {
        return None;
    }

    Some([
        icc_xyz_tag(profile, tag_count, b"rXYZ")?,
        icc_xyz_tag(profile, tag_count, b"gXYZ")?,
        icc_xyz_tag(profile, tag_count, b"bXYZ")?,
    ])
}

#[cfg(feature = "color-probe")]
fn icc_xyz_tag(profile: &[u8], tag_count: usize, signature: &[u8; 4]) -> Option<[f64; 3]> {
    for index in 0..tag_count {
        let record_offset = 132 + (index * 12);
        if &profile[record_offset..record_offset + 4] != signature {
            continue;
        }
        let tag_offset = read_u32_be(profile, record_offset + 4)? as usize;
        let tag_size = read_u32_be(profile, record_offset + 8)? as usize;
        if tag_size < 20 || tag_offset.checked_add(20)? > profile.len() {
            return None;
        }
        if &profile[tag_offset..tag_offset + 4] != b"XYZ " {
            return None;
        }
        return Some([
            read_s15_fixed_16(profile, tag_offset + 8)?,
            read_s15_fixed_16(profile, tag_offset + 12)?,
            read_s15_fixed_16(profile, tag_offset + 16)?,
        ]);
    }
    None
}

#[cfg(feature = "color-probe")]
fn read_u32_be(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

#[cfg(feature = "color-probe")]
fn read_s15_fixed_16(bytes: &[u8], offset: usize) -> Option<f64> {
    let raw = i32::from_be_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
    Some(f64::from(raw) / 65536.0)
}

#[cfg(feature = "color-probe")]
fn profile_contains_ascii(profile: &[u8], needle: &[u8]) -> bool {
    profile
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

#[cfg(feature = "color-probe")]
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Compute 8-bit RGB and luminance histograms from real RGB pixels.
pub fn compute_rgb_histogram(rgb_bytes: &[u8]) -> Result<RgbHistogram, HistogramError> {
    if rgb_bytes.len() % 3 != 0 {
        return Err(HistogramError::RgbByteLengthNotMultipleOfThree {
            byte_len: rgb_bytes.len(),
        });
    }

    let mut histogram = RgbHistogram {
        red: vec![0; 256],
        green: vec![0; 256],
        blue: vec![0; 256],
        luminance: vec![0; 256],
        pixel_count: (rgb_bytes.len() / 3) as u64,
    };

    for pixel in rgb_bytes.chunks_exact(3) {
        let red = pixel[0] as usize;
        let green = pixel[1] as usize;
        let blue = pixel[2] as usize;
        let luminance = (f32::from(pixel[0]) * 0.2126
            + f32::from(pixel[1]) * 0.7152
            + f32::from(pixel[2]) * 0.0722)
            .round()
            .clamp(0.0, 255.0) as usize;

        histogram.red[red] += 1;
        histogram.green[green] += 1;
        histogram.blue[blue] += 1;
        histogram.luminance[luminance] += 1;
    }

    Ok(histogram)
}

/// Rasterize durable normalized brush strokes into a disposable 8-bit alpha plane.
pub fn rasterize_brush_mask(
    mask_id: &str,
    strokes: &[BrushMaskRasterStroke],
    width: u32,
    height: u32,
) -> Result<BrushMaskRaster, BrushMaskRasterError> {
    if mask_id.trim().is_empty() {
        return Err(BrushMaskRasterError::new("mask_id", "must not be empty"));
    }
    if width == 0 {
        return Err(BrushMaskRasterError::new(
            "width",
            "must be greater than zero",
        ));
    }
    if height == 0 {
        return Err(BrushMaskRasterError::new(
            "height",
            "must be greater than zero",
        ));
    }

    for (stroke_index, stroke) in strokes.iter().enumerate() {
        validate_brush_raster_stroke(stroke_index, stroke)?;
    }

    let mut alpha = vec![0_u8; (width as usize) * (height as usize)];
    let radius_scale = f64::from(width.min(height).max(1));
    let x_scale = f64::from(width.saturating_sub(1));
    let y_scale = f64::from(height.saturating_sub(1));

    for stroke in strokes {
        let radius_pixels = stroke.radius * radius_scale;
        for point in &stroke.points {
            let center_x = point.x * x_scale;
            let center_y = point.y * y_scale;
            let min_x = (center_x - radius_pixels).floor().max(0.0) as u32;
            let max_x = (center_x + radius_pixels)
                .ceil()
                .min(f64::from(width.saturating_sub(1))) as u32;
            let min_y = (center_y - radius_pixels).floor().max(0.0) as u32;
            let max_y = (center_y + radius_pixels)
                .ceil()
                .min(f64::from(height.saturating_sub(1))) as u32;
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let dx = f64::from(x) - center_x;
                    let dy = f64::from(y) - center_y;
                    if (dx * dx + dy * dy).sqrt() <= radius_pixels {
                        let index = (y as usize) * (width as usize) + (x as usize);
                        alpha[index] = 255;
                    }
                }
            }
        }
    }

    Ok(BrushMaskRaster {
        width,
        height,
        alpha,
        cache_key: brush_raster_cache_key(mask_id, strokes, width, height),
    })
}

fn validate_brush_raster_stroke(
    stroke_index: usize,
    stroke: &BrushMaskRasterStroke,
) -> Result<(), BrushMaskRasterError> {
    let prefix = format!("strokes.{stroke_index}");
    if stroke.id.trim().is_empty() {
        return Err(BrushMaskRasterError::new(
            format!("{prefix}.id"),
            "must not be empty",
        ));
    }
    if !stroke.radius.is_finite() || stroke.radius <= 0.0 || stroke.radius > 1.0 {
        return Err(BrushMaskRasterError::new(
            format!("{prefix}.radius"),
            "must be finite and in the range (0, 1]",
        ));
    }
    if stroke.points.is_empty() {
        return Err(BrushMaskRasterError::new(
            format!("{prefix}.points"),
            "must contain at least one point",
        ));
    }
    for (point_index, point) in stroke.points.iter().enumerate() {
        validate_brush_raster_coordinate(&format!("{prefix}.points.{point_index}.x"), point.x)?;
        validate_brush_raster_coordinate(&format!("{prefix}.points.{point_index}.y"), point.y)?;
    }
    Ok(())
}

fn validate_brush_raster_coordinate(path: &str, value: f64) -> Result<(), BrushMaskRasterError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(BrushMaskRasterError::new(
            path,
            "must be finite and in the range [0, 1]",
        ));
    }
    Ok(())
}

fn brush_raster_cache_key(
    mask_id: &str,
    strokes: &[BrushMaskRasterStroke],
    width: u32,
    height: u32,
) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    fn write_hash(hash: &mut u64, value: &str) {
        for byte in value.as_bytes() {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    write_hash(&mut hash, "brush-mask-v1");
    write_hash(&mut hash, mask_id);
    write_hash(&mut hash, &format!("{width}x{height}"));
    for stroke in strokes {
        write_hash(&mut hash, &stroke.id);
        write_hash(&mut hash, &format!("{:.6}", stroke.radius));
        for point in &stroke.points {
            write_hash(&mut hash, &format!("{:.6},{:.6}", point.x, point.y));
        }
    }
    format!("brush-mask-v1-{hash:016x}")
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
    let mut request = plan_tone_recovery_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        ToneRecoveryRenderAdjustment::neutral(),
    );
    if request.status == PreviewRenderStatus::Ready {
        request.message = "White balance preview request is ready.".to_string();
    }
    request
}

/// Build a render request for a draft tone-recovery preview update.
pub fn plan_tone_recovery_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_color_presence_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        ColorPresenceRenderAdjustment::neutral(),
    );
    if request.status == PreviewRenderStatus::Ready {
        request.message = "Tone recovery preview request is ready.".to_string();
    }
    request
}

/// Build a render request for a draft color-presence preview update.
pub fn plan_color_presence_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let message = match preview_plan.status {
        PreviewRenderStatus::Ready => "Color presence preview request is ready.".to_string(),
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
        tone_recovery,
        color_presence,
        tone_curve: ToneCurveRenderAdjustment::neutral(),
        hsl_color_mixer: HslColorMixerRenderAdjustment::neutral(),
        detail: DetailRenderAdjustment::neutral(),
        geometry: GeometryRenderAdjustment::neutral(),
        masks: Vec::new(),
        message,
    }
}

/// Build a render request for a draft tone-curve preview update.
pub fn plan_tone_curve_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_color_presence_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
    );
    request.tone_curve = tone_curve;
    if request.status == PreviewRenderStatus::Ready {
        request.message = "Tone curve preview request is ready.".to_string();
    }
    request
}

/// Build a render request for a draft HSL color mixer preview update.
pub fn plan_hsl_color_mixer_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_tone_curve_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
    );
    request.hsl_color_mixer = hsl_color_mixer;
    if request.status == PreviewRenderStatus::Ready {
        request.message = "HSL color mixer preview request is ready.".to_string();
    }
    request
}

/// Build a render request for a draft Detail preview update.
///
/// The current local alpha renderer has no honest Detail implementation. Non-neutral
/// detail values are carried for UI/readback but marked unsupported instead of no-oping.
pub fn plan_detail_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    detail: DetailRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_hsl_color_mixer_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        hsl_color_mixer,
    );
    request.detail = detail;
    if request.status == PreviewRenderStatus::Ready {
        if detail.is_neutral() {
            request.message = "Detail preview request is neutral.".to_string();
        } else {
            request.status = PreviewRenderStatus::Unsupported;
            request.message =
                "Detail preview/export is unsupported until renderer support exists.".to_string();
        }
    }
    request
}

/// Build a render request for a draft Geometry preview update.
///
/// The local alpha supports rectangular crop, quarter-turn rotation, and flips.
/// Perspective transform, crop angle, and arbitrary rotation remain explicit
/// unsupported states until the renderer has real implementations.
pub fn plan_geometry_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    detail: DetailRenderAdjustment,
    geometry: GeometryRenderAdjustment,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_detail_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        hsl_color_mixer,
        detail,
    );
    request.geometry = geometry;
    if request.status == PreviewRenderStatus::Ready {
        if let Some(message) = unsupported_geometry_message(&request.geometry) {
            request.status = PreviewRenderStatus::Unsupported;
            request.message = message;
        } else {
            request.message = "Geometry preview request is ready.".to_string();
        }
    }
    request
}

/// Build a render request for a draft manual gradient mask preview update.
pub fn plan_manual_mask_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    detail: DetailRenderAdjustment,
    geometry: GeometryRenderAdjustment,
    masks: Vec<ManualMaskRenderAdjustment>,
) -> ExposureContrastPreviewRequest {
    let mut request = plan_geometry_preview(
        preview_plan,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        hsl_color_mixer,
        detail,
        geometry,
    );
    request.masks = masks;
    if request.status == PreviewRenderStatus::Ready {
        request.message = "Manual mask preview request is ready.".to_string();
    }
    request
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
    plan_jpeg_srgb_export_with_tone_recovery(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        ToneRecoveryRenderAdjustment::neutral(),
        quality,
    )
}

/// Build a render-side request for exporting an edited raster source with tone recovery.
pub fn plan_jpeg_srgb_export_with_tone_recovery(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    plan_jpeg_srgb_export_with_color_presence(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        ColorPresenceRenderAdjustment::neutral(),
        quality,
    )
}

/// Build a render-side request for exporting an edited raster source with color presence.
pub fn plan_jpeg_srgb_export_with_color_presence(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
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
        tone_recovery,
        color_presence,
        tone_curve: ToneCurveRenderAdjustment::neutral(),
        hsl_color_mixer: HslColorMixerRenderAdjustment::neutral(),
        detail: DetailRenderAdjustment::neutral(),
        geometry: GeometryRenderAdjustment::neutral(),
        quality,
        message: "JPEG sRGB export request is ready.".to_string(),
    }
}

/// Build a render-side request for exporting an edited raster source with tone curve.
pub fn plan_jpeg_srgb_export_with_tone_curve(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    let mut request = plan_jpeg_srgb_export_with_color_presence(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        quality,
    );
    request.tone_curve = tone_curve;
    request
}

/// Build a render-side request for exporting an edited raster source with HSL color mixer.
pub fn plan_jpeg_srgb_export_with_hsl_color_mixer(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    let mut request = plan_jpeg_srgb_export_with_tone_curve(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        quality,
    );
    request.hsl_color_mixer = hsl_color_mixer;
    request
}

/// Build a render-side request for exporting a raster source with Detail values.
///
/// Non-neutral Detail remains unsupported and must be blocked by callers before
/// pixel export. The values are still carried so the boundary is explicit.
pub fn plan_jpeg_srgb_export_with_detail(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    detail: DetailRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    let mut request = plan_jpeg_srgb_export_with_hsl_color_mixer(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        hsl_color_mixer,
        quality,
    );
    request.detail = detail;
    if !detail.is_neutral() {
        request.message = "Detail export unsupported until renderer support exists.".to_string();
    }
    request
}

/// Build a render-side request for exporting a raster source with supported Geometry values.
#[allow(clippy::too_many_arguments)]
pub fn plan_jpeg_srgb_export_with_geometry(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    detail: DetailRenderAdjustment,
    geometry: GeometryRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    let mut request = plan_jpeg_srgb_export_with_detail(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        hsl_color_mixer,
        detail,
        quality,
    );
    request.geometry = geometry;
    if let Some(message) = unsupported_geometry_message(&request.geometry) {
        request.message = message;
    } else if request.detail.is_neutral() && !request.geometry.is_neutral() {
        request.message = "Geometry JPEG sRGB export request is ready.".to_string();
    }
    request
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
    plan_raw_derived_jpeg_srgb_export_with_tone_recovery(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        ToneRecoveryRenderAdjustment::neutral(),
        quality,
    )
}

/// Build a render-side request for exporting a RAW-derived source artifact with tone recovery.
pub fn plan_raw_derived_jpeg_srgb_export_with_tone_recovery(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    plan_raw_derived_jpeg_srgb_export_with_color_presence(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        ColorPresenceRenderAdjustment::neutral(),
        quality,
    )
}

/// Build a render-side request for exporting a RAW-derived source artifact with color presence.
pub fn plan_raw_derived_jpeg_srgb_export_with_color_presence(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
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
        tone_recovery,
        color_presence,
        tone_curve: ToneCurveRenderAdjustment::neutral(),
        hsl_color_mixer: HslColorMixerRenderAdjustment::neutral(),
        detail: DetailRenderAdjustment::neutral(),
        geometry: GeometryRenderAdjustment::neutral(),
        quality,
        message: "RAW-derived JPEG sRGB export request is ready.".to_string(),
    }
}

/// Build a render-side request for exporting a RAW-derived source artifact with tone curve.
pub fn plan_raw_derived_jpeg_srgb_export_with_tone_curve(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    let mut request = plan_raw_derived_jpeg_srgb_export_with_color_presence(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        quality,
    );
    request.tone_curve = tone_curve;
    request
}

/// Build a render-side request for exporting a RAW-derived source artifact with HSL color mixer.
pub fn plan_raw_derived_jpeg_srgb_export_with_hsl_color_mixer(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceRenderAdjustment,
    tone_recovery: ToneRecoveryRenderAdjustment,
    color_presence: ColorPresenceRenderAdjustment,
    tone_curve: ToneCurveRenderAdjustment,
    hsl_color_mixer: HslColorMixerRenderAdjustment,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    let mut request = plan_raw_derived_jpeg_srgb_export_with_tone_curve(
        source_path,
        output_path,
        exposure,
        contrast,
        white_balance,
        tone_recovery,
        color_presence,
        tone_curve,
        quality,
    );
    request.hsl_color_mixer = hsl_color_mixer;
    request
}

fn unsupported_geometry_message(geometry: &GeometryRenderAdjustment) -> Option<String> {
    if !geometry.transform.is_neutral() {
        return Some(
            "Geometry transform preview/export is unsupported until renderer support exists."
                .to_string(),
        );
    }
    if let Some(crop) = &geometry.crop {
        if crop.angle != 0.0 {
            return Some(
                "Angled crop preview/export is unsupported until renderer support exists."
                    .to_string(),
            );
        }
    }
    if !is_supported_quarter_turn(geometry.rotation) {
        return Some(
            "Arbitrary rotation preview/export is unsupported until renderer support exists."
                .to_string(),
        );
    }
    None
}

fn is_supported_quarter_turn(rotation: f64) -> bool {
    [0.0, 90.0, -90.0, 180.0, -180.0]
        .iter()
        .any(|supported| (rotation - supported).abs() <= f64::EPSILON)
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
    fn computes_rgb_histogram_from_real_pixels() {
        let histogram = super::compute_rgb_histogram(&[
            0, 0, 0, //
            255, 128, 64,
        ])
        .expect("compute histogram");

        assert_eq!(histogram.pixel_count, 2);
        assert_eq!(histogram.red[0], 1);
        assert_eq!(histogram.red[255], 1);
        assert_eq!(histogram.green[0], 1);
        assert_eq!(histogram.green[128], 1);
        assert_eq!(histogram.blue[0], 1);
        assert_eq!(histogram.blue[64], 1);
        assert_eq!(histogram.luminance[0], 1);
        assert_eq!(histogram.luminance[150], 1);
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
    fn plans_tone_recovery_preview_and_export_requests() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let tone_recovery = super::ToneRecoveryRenderAdjustment {
            highlights: -35.0,
            shadows: 42.0,
            whites: 10.0,
            blacks: -12.0,
        };

        let preview = super::plan_tone_recovery_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            tone_recovery,
        );
        let export = super::plan_jpeg_srgb_export_with_tone_recovery(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            tone_recovery,
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.tone_recovery, tone_recovery);
        assert!(preview.message.contains("Tone recovery"));
        assert_eq!(export.tone_recovery, tone_recovery);
        assert_eq!(export.exposure, 0.25);
        assert_eq!(export.contrast, -3.0);
    }

    #[test]
    fn plans_tone_curve_preview_and_export_requests() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let tone_curve = super::ToneCurveRenderAdjustment {
            mode: super::ToneCurveRenderMode::Point,
            rgb_curve: vec![
                super::ToneCurveRenderPoint { x: 0.0, y: 0.0 },
                super::ToneCurveRenderPoint { x: 0.5, y: 0.35 },
                super::ToneCurveRenderPoint { x: 1.0, y: 1.0 },
            ],
            red_curve: Vec::new(),
            green_curve: Vec::new(),
            blue_curve: Vec::new(),
        };

        let preview = super::plan_tone_curve_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            tone_curve.clone(),
        );
        let export = super::plan_jpeg_srgb_export_with_tone_curve(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            tone_curve.clone(),
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.tone_curve, tone_curve);
        assert!(preview.message.contains("Tone curve"));
        assert_eq!(export.tone_curve, tone_curve);
        assert_eq!(export.exposure, 0.25);
        assert_eq!(export.contrast, -3.0);
    }

    #[test]
    fn plans_hsl_color_mixer_preview_and_export_requests() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let hsl = super::HslColorMixerRenderAdjustment {
            blue: super::HslColorChannelRenderAdjustment {
                hue: -12.0,
                saturation: 24.0,
                luminance: -8.5,
            },
            ..super::HslColorMixerRenderAdjustment::neutral()
        };

        let preview = super::plan_hsl_color_mixer_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            hsl,
        );
        let export = super::plan_jpeg_srgb_export_with_hsl_color_mixer(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            hsl,
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.hsl_color_mixer, hsl);
        assert!(preview.message.contains("HSL"));
        assert_eq!(export.hsl_color_mixer, hsl);
        assert_eq!(
            export.tone_curve,
            super::ToneCurveRenderAdjustment::neutral()
        );
    }

    #[test]
    fn blocks_non_neutral_detail_preview_and_marks_export_boundary() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let detail = super::DetailRenderAdjustment {
            sharpening: super::DetailSharpeningRenderAdjustment {
                amount: 42.0,
                radius: 1.2,
                detail: 35.0,
                masking: 10.0,
            },
            ..super::DetailRenderAdjustment::neutral()
        };

        let preview = super::plan_detail_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            super::HslColorMixerRenderAdjustment::neutral(),
            detail,
        );
        let export = super::plan_jpeg_srgb_export_with_detail(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            super::HslColorMixerRenderAdjustment::neutral(),
            detail,
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Unsupported);
        assert_eq!(preview.detail, detail);
        assert!(preview.message.contains("Detail"));
        assert_eq!(export.detail, detail);
        assert!(export.message.contains("Detail export unsupported"));
    }

    #[test]
    fn plans_geometry_preview_and_export_requests() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let geometry = super::GeometryRenderAdjustment {
            crop: Some(super::GeometryCropRenderAdjustment {
                x: 0.0,
                y: 0.0,
                width: 0.75,
                height: 1.0,
                angle: 0.0,
                aspect: None,
            }),
            rotation: 90.0,
            flip_horizontal: true,
            flip_vertical: false,
            ..super::GeometryRenderAdjustment::neutral()
        };

        let preview = super::plan_geometry_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            super::HslColorMixerRenderAdjustment::neutral(),
            super::DetailRenderAdjustment::neutral(),
            geometry.clone(),
        );
        let export = super::plan_jpeg_srgb_export_with_geometry(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            super::HslColorMixerRenderAdjustment::neutral(),
            super::DetailRenderAdjustment::neutral(),
            geometry.clone(),
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.geometry, geometry);
        assert!(preview.message.contains("Geometry"));
        assert_eq!(export.geometry, geometry);
        assert!(export.message.contains("Geometry"));
    }

    #[test]
    fn plans_manual_mask_preview_request() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let mask = super::ManualMaskRenderAdjustment {
            id: "mask-linear-1".to_string(),
            enabled: true,
            invert: false,
            opacity: 85.0,
            feather: 35.0,
            geometry: super::ManualMaskRenderGeometry::LinearGradient {
                start_x: 0.2,
                start_y: 0.0,
                end_x: 0.8,
                end_y: 1.0,
            },
            exposure: -0.45,
            contrast: 12.0,
        };

        let preview = super::plan_manual_mask_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            super::ColorPresenceRenderAdjustment::neutral(),
            super::ToneCurveRenderAdjustment::neutral(),
            super::HslColorMixerRenderAdjustment::neutral(),
            super::DetailRenderAdjustment::neutral(),
            super::GeometryRenderAdjustment::neutral(),
            vec![mask.clone()],
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.masks, vec![mask]);
        assert!(preview.message.contains("Manual mask"));
    }

    #[test]
    fn rasterizes_manual_brush_mask_to_deterministic_alpha_plane() {
        let stroke = super::BrushMaskRasterStroke {
            id: "stroke-1".to_string(),
            radius: 0.20,
            points: vec![super::BrushMaskRasterPoint { x: 0.5, y: 0.5 }],
        };

        let raster =
            super::rasterize_brush_mask("mask-brush-1", std::slice::from_ref(&stroke), 5, 5)
                .expect("rasterize brush");
        let repeated = super::rasterize_brush_mask("mask-brush-1", &[stroke], 5, 5)
            .expect("repeat rasterize brush");

        assert_eq!(raster.width, 5);
        assert_eq!(raster.height, 5);
        assert_eq!(
            raster.alpha,
            vec![
                0, 0, 0, 0, 0, //
                0, 0, 255, 0, 0, //
                0, 255, 255, 255, 0, //
                0, 0, 255, 0, 0, //
                0, 0, 0, 0, 0,
            ]
        );
        assert!(raster.cache_key.starts_with("brush-mask-v1-"));
        assert_eq!(raster.cache_key, repeated.cache_key);
    }

    #[test]
    fn rejects_invalid_manual_brush_raster_inputs() {
        let missing_size =
            super::rasterize_brush_mask("mask-brush-1", &[], 0, 5).expect_err("zero width");
        assert!(missing_size.to_string().contains("width"));

        let bad_radius = super::BrushMaskRasterStroke {
            id: "stroke-1".to_string(),
            radius: 0.0,
            points: vec![super::BrushMaskRasterPoint { x: 0.5, y: 0.5 }],
        };
        let radius_error = super::rasterize_brush_mask("mask-brush-1", &[bad_radius], 5, 5)
            .expect_err("zero radius");
        assert!(radius_error.to_string().contains("radius"));

        let bad_point = super::BrushMaskRasterStroke {
            id: "stroke-1".to_string(),
            radius: 0.2,
            points: vec![super::BrushMaskRasterPoint { x: 1.5, y: 0.5 }],
        };
        let point_error = super::rasterize_brush_mask("mask-brush-1", &[bad_point], 5, 5)
            .expect_err("invalid point");
        assert!(point_error.to_string().contains("points.0.x"));
    }

    #[test]
    fn plans_color_presence_preview_and_export_requests() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));
        let color_presence = super::ColorPresenceRenderAdjustment {
            vibrance: 24.0,
            saturation: -8.5,
        };

        let preview = super::plan_color_presence_preview(
            preview_plan,
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            color_presence,
        );
        let export = super::plan_jpeg_srgb_export_with_color_presence(
            "/tmp/original.jpg",
            "/tmp/exported.jpg",
            0.25,
            -3.0,
            super::WhiteBalanceRenderAdjustment::neutral(),
            super::ToneRecoveryRenderAdjustment::neutral(),
            color_presence,
            90,
        );

        assert_eq!(preview.status, super::PreviewRenderStatus::Ready);
        assert_eq!(preview.color_presence, color_presence);
        assert!(preview.message.contains("Color presence"));
        assert_eq!(export.color_presence, color_presence);
        assert_eq!(export.exposure, 0.25);
        assert_eq!(export.contrast, -3.0);
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

    #[cfg(feature = "color-probe")]
    #[test]
    fn color_probe_classifies_embedded_srgb_profile() {
        let profile = include_bytes!("../../../assets/color-profiles/sRGB-v4.icc");
        assert_eq!(
            super::sha256_hex(profile),
            "c56e1685d888f5edb92fe07f2750f387f8fe8e91b32ff8fb0b56bfbbb9458353"
        );
        let jpeg = jpeg_with_icc_profile(profile);
        assert_embedded_profile_classification(&jpeg, profile, super::ColorProbeInputProfile::Srgb);
        let path = write_color_probe_fixture("srgb", jpeg);

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.source_path, path.to_string_lossy());
        assert_color_probe_outcome(
            &result,
            super::ColorProbeInputProfile::Srgb,
            true,
            super::ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb,
        );
    }

    #[cfg(feature = "color-probe")]
    #[test]
    fn color_probe_classifies_portable_display_p3_profile() {
        let profile = include_bytes!("../../../assets/color-profiles/DisplayP3Compat-v4.icc");
        assert_eq!(
            super::sha256_hex(profile),
            "231752984cd4a5278e1b8d2390fe496767d4511fc81f54e1a5c69ae9ab4c42b5"
        );
        let jpeg = jpeg_with_icc_profile(profile);
        assert_embedded_profile_classification(
            &jpeg,
            profile,
            super::ColorProbeInputProfile::DisplayP3,
        );
        let path = write_color_probe_fixture("display-p3", jpeg);

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_color_probe_outcome(
            &result,
            super::ColorProbeInputProfile::DisplayP3,
            true,
            super::ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb,
        );
    }

    #[cfg(feature = "color-probe")]
    #[test]
    fn color_probe_classifies_display_p3_profile_by_xyz_tags_when_hash_differs() {
        let profile = synthetic_rgb_icc_profile([
            [0.5151214599609375, 0.2411956787109375, -0.0010528564453125],
            [0.2919769287109375, 0.6922454833984375, 0.0418853759765625],
            [0.1571044921875, 0.0665740966796875, 0.7840728759765625],
        ]);
        let jpeg = jpeg_with_icc_profile(&profile);
        assert_embedded_profile_classification(
            &jpeg,
            &profile,
            super::ColorProbeInputProfile::DisplayP3,
        );
        let path = write_color_probe_fixture("display-p3-synthetic", jpeg);

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_color_probe_outcome(
            &result,
            super::ColorProbeInputProfile::DisplayP3,
            true,
            super::ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb,
        );
    }

    #[cfg(feature = "color-probe")]
    #[test]
    fn color_probe_records_untagged_raster_as_assume_srgb() {
        let jpeg = minimal_jpeg_without_icc();
        assert_eq!(super::first_icc_profile(&jpeg).expect("parse JPEG"), None);
        let path = write_color_probe_fixture("untagged", jpeg);

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_color_probe_outcome(
            &result,
            super::ColorProbeInputProfile::None,
            false,
            super::ColorProbeTransformPath::AssumeSrgbToLinearDisplayP3ToSrgb,
        );
    }

    #[cfg(feature = "color-probe")]
    #[test]
    fn color_probe_reports_missing_file_without_panicking() {
        let path = std::env::temp_dir().join(unique_color_probe_name("missing"));
        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));

        assert_eq!(result.status, super::ColorProbeStatus::Failed);
        assert_eq!(
            result.error_category,
            Some(match result.platform {
                super::ColorProbePlatform::Macos => super::ColorProbeErrorCategory::MissingFile,
                super::ColorProbePlatform::UnsupportedPlatform => {
                    super::ColorProbeErrorCategory::UnsupportedPlatform
                }
            })
        );
        assert_eq!(result.source_sha256, None);
    }

    #[cfg(feature = "color-probe")]
    fn assert_embedded_profile_classification(
        jpeg: &[u8],
        expected_profile: &[u8],
        expected_classification: super::ColorProbeInputProfile,
    ) {
        let embedded_profile = super::first_icc_profile(jpeg)
            .expect("parse JPEG")
            .expect("embedded ICC profile");

        assert_eq!(embedded_profile.as_slice(), expected_profile);
        assert_eq!(
            super::classify_icc_profile(&embedded_profile),
            expected_classification
        );
    }

    #[cfg(feature = "color-probe")]
    fn assert_color_probe_outcome(
        result: &super::ColorProbeResult,
        expected_input_profile: super::ColorProbeInputProfile,
        expected_embedded_icc: bool,
        expected_transform_path: super::ColorProbeTransformPath,
    ) {
        assert_eq!(
            result.working_space,
            super::WorkingColorSpace::LinearDisplayP3
        );
        assert_eq!(result.output_profile, super::ColorProbeOutputProfile::Srgb);

        match result.platform {
            super::ColorProbePlatform::Macos => {
                assert_eq!(result.status, super::ColorProbeStatus::Success);
                assert_eq!(result.input_profile, expected_input_profile);
                assert_eq!(result.embedded_icc, expected_embedded_icc);
                assert!(result.source_sha256.is_some());
                assert_eq!(result.transform_path, expected_transform_path);
                assert_eq!(result.error_category, None);
            }
            super::ColorProbePlatform::UnsupportedPlatform => {
                assert_eq!(result.status, super::ColorProbeStatus::Failed);
                assert_eq!(result.input_profile, super::ColorProbeInputProfile::Unknown);
                assert!(!result.embedded_icc);
                assert_eq!(result.source_sha256, None);
                assert_eq!(
                    result.transform_path,
                    super::ColorProbeTransformPath::Unavailable
                );
                assert_eq!(
                    result.error_category,
                    Some(super::ColorProbeErrorCategory::UnsupportedPlatform)
                );
            }
        }
    }

    #[cfg(feature = "color-probe")]
    fn write_color_probe_fixture(name: &str, bytes: Vec<u8>) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(unique_color_probe_name(name));
        std::fs::write(&path, bytes).expect("write color probe fixture");
        path
    }

    #[cfg(feature = "color-probe")]
    fn unique_color_probe_name(name: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        format!("silicaraw-color-probe-{name}-{nanos}.jpg")
    }

    #[cfg(feature = "color-probe")]
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

    #[cfg(feature = "color-probe")]
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
                profile[component_offset..component_offset + 4]
                    .copy_from_slice(&fixed.to_be_bytes());
            }
        }

        profile
    }

    #[cfg(feature = "color-probe")]
    fn minimal_jpeg_without_icc() -> Vec<u8> {
        vec![0xff, 0xd8, 0xff, 0xd9]
    }
}
