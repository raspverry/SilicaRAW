//! Edit graph boundary for SilicaRAW.
//!
//! Phase 5.2 adds typed Rust structures for `schemas/edit_graph.schema.json`
//! and a schema-aware validator for the local alpha edit graph.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-edit";

/// Stable schema marker required by `schemas/edit_graph.schema.json`.
pub const EDIT_GRAPH_SCHEMA: &str = "silica.edit_graph";

/// Stable edit graph schema version for v0.1.
pub const EDIT_GRAPH_VERSION: i64 = 1;

/// Stable schema marker for typed edit clipboard payloads.
pub const EDIT_CLIPBOARD_SCHEMA: &str = "silica.edit_clipboard";

/// Stable edit clipboard contract version for v0.1.
pub const EDIT_CLIPBOARD_VERSION: i64 = 1;

/// Explicit input profile value when no fixture-backed profile evidence exists.
pub const INPUT_PROFILE_UNKNOWN: &str = "unknown";

/// First working space selected by the color pipeline proof plan.
pub const WORKING_SPACE_LINEAR_DISPLAY_P3: &str = "linear_display_p3";

/// Source fields needed to build a default edit graph for a catalog photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditGraphSource {
    pub photo_id: String,
    pub path: String,
    pub file_size: i64,
    pub modified_at: Option<String>,
    pub partial_hash: Option<String>,
    pub full_hash: Option<String>,
}

/// Typed representation of `schemas/edit_graph.schema.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditGraph {
    pub schema: String,
    pub version: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    pub source: Source,
    pub profile: Profile,
    pub basic: BasicAdjustments,
    pub tone: ToneAdjustments,
    pub color: ColorAdjustments,
    pub detail: DetailAdjustments,
    pub lens: LensAdjustments,
    pub geometry: GeometryAdjustments,
    pub masks: Vec<Mask>,
    pub metadata: EditMetadata,
    pub extensions: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub updated_at: String,
}

/// Schema-owned edit sections that can be copied and pasted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EditClipboardSelection {
    pub basic: bool,
    pub tone: bool,
    pub color: bool,
    pub detail: bool,
    pub lens: bool,
    pub geometry: bool,
}

/// Copyable schema-owned Detail controls, excluding model/plugin-owned payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditClipboardDetailSection {
    pub sharpening: Sharpening,
    pub noise_reduction: NoiseReduction,
}

/// Copyable schema-owned Lens controls, excluding source-specific profile identifiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditClipboardLensSection {
    pub profile_correction: bool,
    pub chromatic_aberration: bool,
    pub distortion: Number,
    pub vignetting: Number,
}

/// Typed payload for copying and pasting schema-owned edit sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditClipboardPayload {
    pub schema: String,
    pub version: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic: Option<BasicAdjustments>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<ToneAdjustments>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorAdjustments>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<EditClipboardDetailSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens: Option<EditClipboardLensSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryAdjustments>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub photo_id: String,
    pub path: String,
    pub fingerprint: SourceFingerprint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFingerprint {
    pub file_size: i64,
    pub modified_at: String,
    pub partial_hash: String,
    pub full_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub input_profile: String,
    pub working_space: String,
    pub camera_profile: Option<String>,
    pub decoder_backend: Option<DecoderBackend>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecoderBackend {
    CoreImageRaw,
    Libraw,
    EmbeddedPreview,
    Raster,
}

/// Evidence-backed color profile metadata for schema-owned edit graph fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProfileMetadata {
    pub input_profile: String,
    pub working_space: String,
    pub decoder_backend: Option<DecoderBackend>,
}

impl ColorProfileMetadata {
    pub fn unknown() -> Self {
        Self {
            input_profile: INPUT_PROFILE_UNKNOWN.to_string(),
            working_space: WORKING_SPACE_LINEAR_DISPLAY_P3.to_string(),
            decoder_backend: None,
        }
    }

    pub fn raster(input_profile: impl Into<String>) -> Self {
        Self {
            input_profile: input_profile.into(),
            working_space: WORKING_SPACE_LINEAR_DISPLAY_P3.to_string(),
            decoder_backend: Some(DecoderBackend::Raster),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicAdjustments {
    pub white_balance: WhiteBalance,
    pub temperature: Number,
    pub tint: Number,
    pub exposure: Number,
    pub contrast: Number,
    pub highlights: Number,
    pub shadows: Number,
    pub whites: Number,
    pub blacks: Number,
    pub texture: Number,
    pub clarity: Number,
    pub dehaze: Number,
    pub vibrance: Number,
    pub saturation: Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhiteBalance {
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

/// Built-in P0 Basic presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicPreset {
    SilicaNeutral,
    WarmContrast,
    SoftMatte,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToneAdjustments {
    pub curve_mode: CurveMode,
    pub rgb_curve: Vec<CurvePoint>,
    pub red_curve: Vec<CurvePoint>,
    pub green_curve: Vec<CurvePoint>,
    pub blue_curve: Vec<CurvePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveMode {
    None,
    Parametric,
    Point,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurvePoint {
    pub x: Number,
    pub y: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorAdjustments {
    pub hsl: HslAdjustments,
    pub grading: ColorGrading,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HslAdjustments {
    pub red: HslChannel,
    pub orange: HslChannel,
    pub yellow: HslChannel,
    pub green: HslChannel,
    pub aqua: HslChannel,
    pub blue: HslChannel,
    pub purple: HslChannel,
    pub magenta: HslChannel,
}

/// Schema-owned HSL color mixer channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HslColorChannel {
    Red,
    Orange,
    Yellow,
    Green,
    Aqua,
    Blue,
    Purple,
    Magenta,
}

impl TryFrom<&str> for HslColorChannel {
    type Error = EditGraphValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "red" => Ok(Self::Red),
            "orange" => Ok(Self::Orange),
            "yellow" => Ok(Self::Yellow),
            "green" => Ok(Self::Green),
            "aqua" => Ok(Self::Aqua),
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            "magenta" => Ok(Self::Magenta),
            unsupported => Err(EditGraphValidationError::new(
                "color.hsl",
                format!("unsupported HSL color channel: {unsupported}"),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HslChannel {
    pub hue: Number,
    pub saturation: Number,
    pub luminance: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorGrading {
    pub shadows: ColorWheel,
    pub midtones: ColorWheel,
    pub highlights: ColorWheel,
    pub blending: Number,
    pub balance: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorWheel {
    pub hue: Number,
    pub saturation: Number,
    pub luminance: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailAdjustments {
    pub sharpening: Sharpening,
    pub noise_reduction: NoiseReduction,
    pub mlx_denoise: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sharpening {
    pub amount: Number,
    pub radius: Number,
    pub detail: Number,
    pub masking: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoiseReduction {
    pub luminance: Number,
    pub detail: Number,
    pub contrast: Number,
    pub color: Number,
    pub color_detail: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LensAdjustments {
    pub profile_correction: bool,
    pub profile_id: Option<String>,
    pub chromatic_aberration: bool,
    pub distortion: Number,
    pub vignetting: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryAdjustments {
    pub crop: Option<Crop>,
    pub rotation: Number,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub transform: GeometryTransform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Crop {
    pub x: Number,
    pub y: Number,
    pub width: Number,
    pub height: Number,
    pub angle: Number,
    pub aspect: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryTransform {
    pub vertical: Number,
    pub horizontal: Number,
    pub aspect: Number,
    pub scale: Number,
    pub x_offset: Number,
    pub y_offset: Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mask {
    pub id: String,
    #[serde(rename = "type")]
    pub mask_type: MaskType,
    pub name: String,
    pub enabled: bool,
    pub invert: bool,
    pub opacity: Number,
    pub feather: Number,
    pub source: MaskSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<MaskGeometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brush: Option<MaskBrush>,
    pub local_adjustments: BTreeMap<String, Number>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskType {
    Brush,
    LinearGradient,
    RadialGradient,
    Subject,
    Sky,
    Background,
    ColorRange,
    LuminanceRange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskSource {
    pub kind: MaskSourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_result_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskSourceKind {
    Manual,
    Mlx,
    Procedural,
}

/// Durable normalized geometry for parametric manual masks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaskGeometry {
    LinearGradient {
        start_x: Number,
        start_y: Number,
        end_x: Number,
        end_y: Number,
    },
    RadialGradient {
        center_x: Number,
        center_y: Number,
        radius_x: Number,
        radius_y: Number,
        rotation: Number,
    },
}

/// Durable normalized sampled stroke payload for manual brush masks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskBrush {
    pub coordinate_space: BrushCoordinateSpace,
    pub strokes: Vec<MaskBrushStroke>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushCoordinateSpace {
    NormalizedImage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskBrushStroke {
    pub id: String,
    pub radius: Number,
    pub points: Vec<MaskBrushPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskBrushPoint {
    pub x: Number,
    pub y: Number,
}

/// Supported manual mask local adjustments for Phase 19.2.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ManualMaskLocalAdjustments {
    pub exposure: Option<f64>,
    pub contrast: Option<f64>,
}

pub const MAX_MANUAL_BRUSH_STROKES: usize = 16;
pub const MAX_MANUAL_BRUSH_POINTS: usize = 512;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditMetadata {
    pub rating: i64,
    pub picked: bool,
    pub rejected: bool,
    pub color_label: Option<ColorLabel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorLabel {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

/// Error returned by schema-aware edit graph validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditGraphValidationError {
    path: String,
    message: String,
}

impl EditGraphValidationError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for EditGraphValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl Error for EditGraphValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditClipboardError {
    path: String,
    message: String,
}

impl EditClipboardError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for EditClipboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl Error for EditClipboardError {}

impl From<EditGraphValidationError> for EditClipboardError {
    fn from(error: EditGraphValidationError) -> Self {
        Self::new("edit_graph", error.to_string())
    }
}

/// Build a schema-valid default edit graph without persisting it.
pub fn default_edit_graph(source: EditGraphSource, updated_at: impl Into<String>) -> EditGraph {
    let zero = || Number::from(0);
    let default_hsl = || HslChannel {
        hue: zero(),
        saturation: zero(),
        luminance: zero(),
    };
    let default_wheel = || ColorWheel {
        hue: zero(),
        saturation: zero(),
        luminance: zero(),
    };

    EditGraph {
        schema: EDIT_GRAPH_SCHEMA.to_string(),
        version: EDIT_GRAPH_VERSION,
        app_version: None,
        source: Source {
            photo_id: source.photo_id,
            path: source.path,
            fingerprint: SourceFingerprint {
                file_size: source.file_size,
                modified_at: source.modified_at.unwrap_or_else(|| "unknown".to_string()),
                partial_hash: source.partial_hash.unwrap_or_default(),
                full_hash: source.full_hash,
            },
        },
        profile: Profile {
            name: "silica_standard".to_string(),
            input_profile: INPUT_PROFILE_UNKNOWN.to_string(),
            working_space: WORKING_SPACE_LINEAR_DISPLAY_P3.to_string(),
            camera_profile: None,
            decoder_backend: None,
        },
        basic: BasicAdjustments {
            white_balance: WhiteBalance::AsShot,
            temperature: Number::from(5200),
            tint: zero(),
            exposure: zero(),
            contrast: zero(),
            highlights: zero(),
            shadows: zero(),
            whites: zero(),
            blacks: zero(),
            texture: zero(),
            clarity: zero(),
            dehaze: zero(),
            vibrance: zero(),
            saturation: zero(),
        },
        tone: ToneAdjustments {
            curve_mode: CurveMode::None,
            rgb_curve: Vec::new(),
            red_curve: Vec::new(),
            green_curve: Vec::new(),
            blue_curve: Vec::new(),
        },
        color: ColorAdjustments {
            hsl: HslAdjustments {
                red: default_hsl(),
                orange: default_hsl(),
                yellow: default_hsl(),
                green: default_hsl(),
                aqua: default_hsl(),
                blue: default_hsl(),
                purple: default_hsl(),
                magenta: default_hsl(),
            },
            grading: ColorGrading {
                shadows: default_wheel(),
                midtones: default_wheel(),
                highlights: default_wheel(),
                blending: Number::from(50),
                balance: zero(),
            },
        },
        detail: DetailAdjustments {
            sharpening: Sharpening {
                amount: zero(),
                radius: Number::from_f64(1.0).expect("finite default radius"),
                detail: Number::from(25),
                masking: zero(),
            },
            noise_reduction: NoiseReduction {
                luminance: zero(),
                detail: Number::from(50),
                contrast: zero(),
                color: Number::from(25),
                color_detail: Number::from(50),
            },
            mlx_denoise: None,
        },
        lens: LensAdjustments {
            profile_correction: false,
            profile_id: None,
            chromatic_aberration: false,
            distortion: zero(),
            vignetting: zero(),
        },
        geometry: GeometryAdjustments {
            crop: None,
            rotation: zero(),
            flip_horizontal: false,
            flip_vertical: false,
            transform: GeometryTransform {
                vertical: zero(),
                horizontal: zero(),
                aspect: zero(),
                scale: Number::from(100),
                x_offset: zero(),
                y_offset: zero(),
            },
        },
        masks: Vec::new(),
        metadata: EditMetadata {
            rating: 0,
            picked: false,
            rejected: false,
            color_label: None,
        },
        extensions: Map::new(),
        created_at: None,
        updated_at: updated_at.into(),
    }
}

/// Build the canonical provenance payload for manual masks.
pub fn manual_mask_source() -> MaskSource {
    MaskSource {
        kind: MaskSourceKind::Manual,
        ai_result_id: None,
        cache_path: None,
        model_id: None,
        model_version: None,
        extensions: Map::new(),
    }
}

/// Build a schema-valid manual linear gradient mask without persisting it.
#[allow(clippy::too_many_arguments)]
pub fn manual_linear_gradient_mask(
    id: impl Into<String>,
    name: impl Into<String>,
    opacity: f64,
    feather: f64,
    invert: bool,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    local_adjustments: ManualMaskLocalAdjustments,
) -> Result<Mask, EditGraphValidationError> {
    let mask = Mask {
        id: non_empty_mask_string("masks.id", id.into())?,
        mask_type: MaskType::LinearGradient,
        name: non_empty_mask_string("masks.name", name.into())?,
        enabled: true,
        invert,
        opacity: number_from_f64("masks.opacity", opacity)?,
        feather: number_from_f64("masks.feather", feather)?,
        source: manual_mask_source(),
        geometry: Some(MaskGeometry::LinearGradient {
            start_x: number_from_f64("masks.geometry.start_x", start_x)?,
            start_y: number_from_f64("masks.geometry.start_y", start_y)?,
            end_x: number_from_f64("masks.geometry.end_x", end_x)?,
            end_y: number_from_f64("masks.geometry.end_y", end_y)?,
        }),
        brush: None,
        local_adjustments: manual_mask_local_adjustments(local_adjustments)?,
    };
    validate_mask(0, &mask)?;
    Ok(mask)
}

/// Build a schema-valid manual radial gradient mask without persisting it.
#[allow(clippy::too_many_arguments)]
pub fn manual_radial_gradient_mask(
    id: impl Into<String>,
    name: impl Into<String>,
    opacity: f64,
    feather: f64,
    invert: bool,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    local_adjustments: ManualMaskLocalAdjustments,
) -> Result<Mask, EditGraphValidationError> {
    let mask = Mask {
        id: non_empty_mask_string("masks.id", id.into())?,
        mask_type: MaskType::RadialGradient,
        name: non_empty_mask_string("masks.name", name.into())?,
        enabled: true,
        invert,
        opacity: number_from_f64("masks.opacity", opacity)?,
        feather: number_from_f64("masks.feather", feather)?,
        source: manual_mask_source(),
        geometry: Some(MaskGeometry::RadialGradient {
            center_x: number_from_f64("masks.geometry.center_x", center_x)?,
            center_y: number_from_f64("masks.geometry.center_y", center_y)?,
            radius_x: number_from_f64("masks.geometry.radius_x", radius_x)?,
            radius_y: number_from_f64("masks.geometry.radius_y", radius_y)?,
            rotation: number_from_f64("masks.geometry.rotation", rotation)?,
        }),
        brush: None,
        local_adjustments: manual_mask_local_adjustments(local_adjustments)?,
    };
    validate_mask(0, &mask)?;
    Ok(mask)
}

/// Build a schema-valid durable manual brush stroke without raster cache data.
pub fn manual_brush_stroke(
    id: impl Into<String>,
    radius: f64,
    points: Vec<(f64, f64)>,
) -> Result<MaskBrushStroke, EditGraphValidationError> {
    let stroke = MaskBrushStroke {
        id: non_empty_mask_string("masks.brush.strokes.id", id.into())?,
        radius: number_from_f64("masks.brush.strokes.radius", radius)?,
        points: points
            .into_iter()
            .enumerate()
            .map(|(index, (x, y))| {
                Ok(MaskBrushPoint {
                    x: number_from_f64(&format!("masks.brush.strokes.points.{index}.x"), x)?,
                    y: number_from_f64(&format!("masks.brush.strokes.points.{index}.y"), y)?,
                })
            })
            .collect::<Result<Vec<_>, EditGraphValidationError>>()?,
    };
    validate_brush_stroke("masks.brush.strokes.0", &stroke)?;
    Ok(stroke)
}

/// Build a schema-valid manual brush mask without persisting it.
pub fn manual_brush_mask(
    id: impl Into<String>,
    name: impl Into<String>,
    opacity: f64,
    feather: f64,
    invert: bool,
    strokes: Vec<MaskBrushStroke>,
    local_adjustments: ManualMaskLocalAdjustments,
) -> Result<Mask, EditGraphValidationError> {
    let mask = Mask {
        id: non_empty_mask_string("masks.id", id.into())?,
        mask_type: MaskType::Brush,
        name: non_empty_mask_string("masks.name", name.into())?,
        enabled: true,
        invert,
        opacity: number_from_f64("masks.opacity", opacity)?,
        feather: number_from_f64("masks.feather", feather)?,
        source: manual_mask_source(),
        geometry: None,
        brush: Some(MaskBrush {
            coordinate_space: BrushCoordinateSpace::NormalizedImage,
            strokes,
        }),
        local_adjustments: manual_mask_local_adjustments(local_adjustments)?,
    };
    validate_mask(0, &mask)?;
    Ok(mask)
}

/// Append a manual mask to a cloned edit graph without touching external state.
pub fn append_manual_mask(
    graph: &EditGraph,
    mask: Mask,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    if graph.masks.iter().any(|existing| existing.id == mask.id) {
        return Err(EditGraphValidationError::new(
            "masks",
            format!("duplicate mask id: {}", mask.id),
        ));
    }
    validate_mask(graph.masks.len(), &mask)?;
    let mut edited = graph.clone();
    edited.masks.push(mask);
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with exposure and contrast adjusted, without persistence.
pub fn apply_exposure_contrast(
    graph: &EditGraph,
    exposure: f64,
    contrast: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.basic.exposure = number_from_f64("basic.exposure", exposure)?;
    edited.basic.contrast = number_from_f64("basic.contrast", contrast)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with white balance, temperature, and tint adjusted.
pub fn apply_white_balance_temperature_tint(
    graph: &EditGraph,
    white_balance: WhiteBalance,
    temperature: f64,
    tint: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.basic.white_balance = white_balance;
    edited.basic.temperature = number_from_f64("basic.temperature", temperature)?;
    edited.basic.tint = number_from_f64("basic.tint", tint)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with tone recovery controls adjusted.
pub fn apply_tone_recovery(
    graph: &EditGraph,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.basic.highlights = number_from_f64("basic.highlights", highlights)?;
    edited.basic.shadows = number_from_f64("basic.shadows", shadows)?;
    edited.basic.whites = number_from_f64("basic.whites", whites)?;
    edited.basic.blacks = number_from_f64("basic.blacks", blacks)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with point tone curves adjusted.
pub fn apply_tone_curve(
    graph: &EditGraph,
    curve_mode: CurveMode,
    rgb_curve: &[(f64, f64)],
    red_curve: &[(f64, f64)],
    green_curve: &[(f64, f64)],
    blue_curve: &[(f64, f64)],
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    if curve_mode == CurveMode::Parametric {
        return Err(EditGraphValidationError::new(
            "tone.curve_mode",
            "parametric curves have no schema-owned parameters yet",
        ));
    }

    let mut edited = graph.clone();
    edited.tone.curve_mode = curve_mode;
    edited.tone.rgb_curve = curve_points_from_pairs("tone.rgb_curve", rgb_curve)?;
    edited.tone.red_curve = curve_points_from_pairs("tone.red_curve", red_curve)?;
    edited.tone.green_curve = curve_points_from_pairs("tone.green_curve", green_curve)?;
    edited.tone.blue_curve = curve_points_from_pairs("tone.blue_curve", blue_curve)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with color presence controls adjusted.
pub fn apply_color_presence(
    graph: &EditGraph,
    vibrance: f64,
    saturation: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.basic.vibrance = number_from_f64("basic.vibrance", vibrance)?;
    edited.basic.saturation = number_from_f64("basic.saturation", saturation)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with one HSL color mixer channel adjusted.
pub fn apply_hsl_color_channel(
    graph: &EditGraph,
    channel: HslColorChannel,
    hue: f64,
    saturation: f64,
    luminance: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    let hsl_channel = hsl_color_channel_mut(&mut edited.color.hsl, channel);
    hsl_channel.hue = number_from_f64(&hsl_channel_path(channel, "hue"), hue)?;
    hsl_channel.saturation = number_from_f64(&hsl_channel_path(channel, "saturation"), saturation)?;
    hsl_channel.luminance = number_from_f64(&hsl_channel_path(channel, "luminance"), luminance)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with schema-owned sharpening controls adjusted.
pub fn apply_detail_sharpening(
    graph: &EditGraph,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.detail.sharpening.amount = number_from_f64("detail.sharpening.amount", amount)?;
    edited.detail.sharpening.radius = number_from_f64("detail.sharpening.radius", radius)?;
    edited.detail.sharpening.detail = number_from_f64("detail.sharpening.detail", detail)?;
    edited.detail.sharpening.masking = number_from_f64("detail.sharpening.masking", masking)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with schema-owned non-MLX noise reduction controls adjusted.
pub fn apply_detail_noise_reduction(
    graph: &EditGraph,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.detail.noise_reduction.luminance =
        number_from_f64("detail.noise_reduction.luminance", luminance)?;
    edited.detail.noise_reduction.detail =
        number_from_f64("detail.noise_reduction.detail", detail)?;
    edited.detail.noise_reduction.contrast =
        number_from_f64("detail.noise_reduction.contrast", contrast)?;
    edited.detail.noise_reduction.color = number_from_f64("detail.noise_reduction.color", color)?;
    edited.detail.noise_reduction.color_detail =
        number_from_f64("detail.noise_reduction.color_detail", color_detail)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with schema-owned lens toggles and sliders adjusted.
pub fn apply_lens_adjustments(
    graph: &EditGraph,
    profile_correction: bool,
    chromatic_aberration: bool,
    distortion: f64,
    vignetting: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.lens.profile_correction = profile_correction;
    edited.lens.chromatic_aberration = chromatic_aberration;
    edited.lens.distortion = number_from_f64("lens.distortion", distortion)?;
    edited.lens.vignetting = number_from_f64("lens.vignetting", vignetting)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with schema-owned geometry transform controls adjusted.
pub fn apply_geometry_transform(
    graph: &EditGraph,
    vertical: f64,
    horizontal: f64,
    aspect: f64,
    scale: f64,
    x_offset: f64,
    y_offset: f64,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.geometry.transform.vertical = number_from_f64("geometry.transform.vertical", vertical)?;
    edited.geometry.transform.horizontal =
        number_from_f64("geometry.transform.horizontal", horizontal)?;
    edited.geometry.transform.aspect = number_from_f64("geometry.transform.aspect", aspect)?;
    edited.geometry.transform.scale = number_from_f64("geometry.transform.scale", scale)?;
    edited.geometry.transform.x_offset = number_from_f64("geometry.transform.x_offset", x_offset)?;
    edited.geometry.transform.y_offset = number_from_f64("geometry.transform.y_offset", y_offset)?;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with schema-owned rotation and flip state adjusted.
pub fn apply_geometry_orientation(
    graph: &EditGraph,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.geometry.rotation = number_from_f64("geometry.rotation", rotation)?;
    edited.geometry.flip_horizontal = flip_horizontal;
    edited.geometry.flip_vertical = flip_vertical;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with a normalized crop rectangle.
pub fn apply_geometry_crop(
    graph: &EditGraph,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<&str>,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.geometry.crop = Some(Crop {
        x: number_from_f64("geometry.crop.x", x)?,
        y: number_from_f64("geometry.crop.y", y)?,
        width: number_from_f64("geometry.crop.width", width)?,
        height: number_from_f64("geometry.crop.height", height)?,
        angle: number_from_f64("geometry.crop.angle", angle)?,
        aspect: aspect.map(str::to_string),
    });
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a draft graph with crop cleared while preserving other geometry state.
pub fn clear_geometry_crop(
    graph: &EditGraph,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.geometry.crop = None;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Reset P0 Basic controls to schema-valid defaults.
pub fn reset_p0_basic_controls(
    graph: &EditGraph,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.basic.white_balance = WhiteBalance::AsShot;
    edited.basic.temperature = Number::from(5200);
    edited.basic.tint = Number::from(0);
    edited.basic.exposure = Number::from(0);
    edited.basic.contrast = Number::from(0);
    edited.basic.highlights = Number::from(0);
    edited.basic.shadows = Number::from(0);
    edited.basic.whites = Number::from(0);
    edited.basic.blacks = Number::from(0);
    edited.basic.vibrance = Number::from(0);
    edited.basic.saturation = Number::from(0);
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Apply one built-in P0 Basic preset through the edit graph validator.
pub fn apply_basic_preset(
    graph: &EditGraph,
    preset: BasicPreset,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    match preset {
        BasicPreset::SilicaNeutral => {
            return reset_p0_basic_controls(graph, updated_at);
        }
        BasicPreset::WarmContrast => {
            edited.basic.white_balance = WhiteBalance::Custom;
            edited.basic.temperature = Number::from(6200);
            edited.basic.tint = Number::from(4);
            edited.basic.exposure = number_from_f64("basic.exposure", 0.15)?;
            edited.basic.contrast = Number::from(18);
            edited.basic.highlights = Number::from(-20);
            edited.basic.shadows = Number::from(10);
            edited.basic.whites = Number::from(12);
            edited.basic.blacks = Number::from(-8);
            edited.basic.vibrance = Number::from(12);
            edited.basic.saturation = Number::from(4);
        }
        BasicPreset::SoftMatte => {
            edited.basic.white_balance = WhiteBalance::Custom;
            edited.basic.temperature = Number::from(5400);
            edited.basic.tint = Number::from(2);
            edited.basic.exposure = Number::from(0);
            edited.basic.contrast = Number::from(-18);
            edited.basic.highlights = Number::from(-30);
            edited.basic.shadows = Number::from(24);
            edited.basic.whites = Number::from(-12);
            edited.basic.blacks = Number::from(18);
            edited.basic.vibrance = Number::from(8);
            edited.basic.saturation = Number::from(-6);
        }
    }
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Return a graph with evidence-backed color profile metadata in schema-owned fields.
pub fn apply_color_profile_metadata(
    graph: &EditGraph,
    metadata: ColorProfileMetadata,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditGraphValidationError> {
    let mut edited = graph.clone();
    edited.profile.input_profile = metadata.input_profile;
    edited.profile.working_space = metadata.working_space;
    edited.profile.decoder_backend = metadata.decoder_backend;
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Copy selected schema-owned edit sections into a typed clipboard payload.
pub fn copy_edit_clipboard_payload(
    graph: &EditGraph,
    sections: EditClipboardSelection,
) -> Result<EditClipboardPayload, EditClipboardError> {
    validate_edit_graph(graph)?;

    let payload = EditClipboardPayload {
        schema: EDIT_CLIPBOARD_SCHEMA.to_string(),
        version: EDIT_CLIPBOARD_VERSION,
        basic: sections.basic.then(|| graph.basic.clone()),
        tone: sections.tone.then(|| graph.tone.clone()),
        color: sections.color.then(|| graph.color.clone()),
        detail: sections
            .detail
            .then(|| edit_clipboard_detail_from_graph(&graph.detail)),
        lens: sections
            .lens
            .then(|| edit_clipboard_lens_from_graph(&graph.lens)),
        geometry: sections.geometry.then(|| graph.geometry.clone()),
    };
    validate_edit_clipboard_payload(&payload)?;
    Ok(payload)
}

/// Apply a typed edit clipboard payload to a target graph without changing target identity fields.
pub fn apply_edit_clipboard_payload(
    target: &EditGraph,
    payload: &EditClipboardPayload,
    updated_at: impl Into<String>,
) -> Result<EditGraph, EditClipboardError> {
    validate_edit_clipboard_payload(payload)?;

    let mut edited = target.clone();
    if let Some(basic) = payload.basic.as_ref() {
        edited.basic = basic.clone();
    }
    if let Some(tone) = payload.tone.as_ref() {
        edited.tone = tone.clone();
    }
    if let Some(color) = payload.color.as_ref() {
        edited.color = color.clone();
    }
    if let Some(detail) = payload.detail.as_ref() {
        edited.detail.sharpening = detail.sharpening.clone();
        edited.detail.noise_reduction = detail.noise_reduction.clone();
    }
    if let Some(lens) = payload.lens.as_ref() {
        edited.lens.profile_correction = lens.profile_correction;
        edited.lens.chromatic_aberration = lens.chromatic_aberration;
        edited.lens.distortion = lens.distortion.clone();
        edited.lens.vignetting = lens.vignetting.clone();
    }
    if let Some(geometry) = payload.geometry.as_ref() {
        edited.geometry = geometry.clone();
    }
    edited.updated_at = updated_at.into();
    validate_edit_graph(&edited)?;
    Ok(edited)
}

/// Validate a typed edit clipboard payload before any paste operation uses it.
pub fn validate_edit_clipboard_payload(
    payload: &EditClipboardPayload,
) -> Result<(), EditClipboardError> {
    if payload.schema != EDIT_CLIPBOARD_SCHEMA {
        return Err(EditClipboardError::new(
            "schema",
            format!("expected {EDIT_CLIPBOARD_SCHEMA}"),
        ));
    }
    if payload.version != EDIT_CLIPBOARD_VERSION {
        return Err(EditClipboardError::new(
            "version",
            format!("expected {EDIT_CLIPBOARD_VERSION}"),
        ));
    }

    if payload.basic.is_none()
        && payload.tone.is_none()
        && payload.color.is_none()
        && payload.detail.is_none()
        && payload.lens.is_none()
        && payload.geometry.is_none()
    {
        return Err(EditClipboardError::new(
            "sections",
            "at least one edit section must be selected",
        ));
    }

    if let Some(basic) = payload.basic.as_ref() {
        validate_basic(basic)?;
    }
    if let Some(tone) = payload.tone.as_ref() {
        validate_tone(tone)?;
    }
    if let Some(color) = payload.color.as_ref() {
        validate_color(color)?;
    }
    if let Some(detail) = payload.detail.as_ref() {
        validate_edit_clipboard_detail(detail)?;
    }
    if let Some(lens) = payload.lens.as_ref() {
        validate_edit_clipboard_lens(lens)?;
    }
    if let Some(geometry) = payload.geometry.as_ref() {
        validate_geometry(geometry)?;
    }

    Ok(())
}

/// Validate JSON against the local alpha edit graph contract.
pub fn validate_edit_graph_json(value: &Value) -> Result<(), EditGraphValidationError> {
    let graph: EditGraph = serde_json::from_value(value.clone())
        .map_err(|error| EditGraphValidationError::new("root", error.to_string()))?;
    validate_edit_graph(&graph)
}

/// Validate an already typed edit graph.
pub fn validate_edit_graph(graph: &EditGraph) -> Result<(), EditGraphValidationError> {
    if graph.schema != EDIT_GRAPH_SCHEMA {
        return Err(EditGraphValidationError::new(
            "schema",
            format!("expected {EDIT_GRAPH_SCHEMA}"),
        ));
    }
    if graph.version != EDIT_GRAPH_VERSION {
        return Err(EditGraphValidationError::new(
            "version",
            format!("expected {EDIT_GRAPH_VERSION}"),
        ));
    }

    validate_source(&graph.source)?;
    validate_profile(&graph.profile)?;
    validate_basic(&graph.basic)?;
    validate_tone(&graph.tone)?;
    validate_color(&graph.color)?;
    validate_detail(&graph.detail)?;
    validate_lens(&graph.lens)?;
    validate_geometry(&graph.geometry)?;
    for (index, mask) in graph.masks.iter().enumerate() {
        validate_mask(index, mask)?;
    }
    validate_metadata(&graph.metadata)?;

    Ok(())
}

fn hsl_color_channel_mut(hsl: &mut HslAdjustments, channel: HslColorChannel) -> &mut HslChannel {
    match channel {
        HslColorChannel::Red => &mut hsl.red,
        HslColorChannel::Orange => &mut hsl.orange,
        HslColorChannel::Yellow => &mut hsl.yellow,
        HslColorChannel::Green => &mut hsl.green,
        HslColorChannel::Aqua => &mut hsl.aqua,
        HslColorChannel::Blue => &mut hsl.blue,
        HslColorChannel::Purple => &mut hsl.purple,
        HslColorChannel::Magenta => &mut hsl.magenta,
    }
}

fn edit_clipboard_detail_from_graph(detail: &DetailAdjustments) -> EditClipboardDetailSection {
    EditClipboardDetailSection {
        sharpening: detail.sharpening.clone(),
        noise_reduction: detail.noise_reduction.clone(),
    }
}

fn edit_clipboard_lens_from_graph(lens: &LensAdjustments) -> EditClipboardLensSection {
    EditClipboardLensSection {
        profile_correction: lens.profile_correction,
        chromatic_aberration: lens.chromatic_aberration,
        distortion: lens.distortion.clone(),
        vignetting: lens.vignetting.clone(),
    }
}

fn validate_edit_clipboard_detail(
    detail: &EditClipboardDetailSection,
) -> Result<(), EditClipboardError> {
    let graph_detail = DetailAdjustments {
        sharpening: detail.sharpening.clone(),
        noise_reduction: detail.noise_reduction.clone(),
        mlx_denoise: None,
    };
    validate_detail(&graph_detail)?;
    Ok(())
}

fn validate_edit_clipboard_lens(lens: &EditClipboardLensSection) -> Result<(), EditClipboardError> {
    let graph_lens = LensAdjustments {
        profile_correction: lens.profile_correction,
        profile_id: None,
        chromatic_aberration: lens.chromatic_aberration,
        distortion: lens.distortion.clone(),
        vignetting: lens.vignetting.clone(),
    };
    validate_lens(&graph_lens)?;
    Ok(())
}

fn hsl_channel_path(channel: HslColorChannel, field: &str) -> String {
    let channel_name = match channel {
        HslColorChannel::Red => "red",
        HslColorChannel::Orange => "orange",
        HslColorChannel::Yellow => "yellow",
        HslColorChannel::Green => "green",
        HslColorChannel::Aqua => "aqua",
        HslColorChannel::Blue => "blue",
        HslColorChannel::Purple => "purple",
        HslColorChannel::Magenta => "magenta",
    };
    format!("color.hsl.{channel_name}.{field}")
}

fn validate_profile(profile: &Profile) -> Result<(), EditGraphValidationError> {
    if profile.name.trim().is_empty() {
        return Err(EditGraphValidationError::new(
            "profile.name",
            "must not be empty",
        ));
    }
    if profile.input_profile.trim().is_empty() {
        return Err(EditGraphValidationError::new(
            "profile.input_profile",
            "must not be empty",
        ));
    }
    if profile.working_space.trim().is_empty() {
        return Err(EditGraphValidationError::new(
            "profile.working_space",
            "must not be empty",
        ));
    }
    if profile
        .camera_profile
        .as_ref()
        .is_some_and(|camera_profile| camera_profile.trim().is_empty())
    {
        return Err(EditGraphValidationError::new(
            "profile.camera_profile",
            "must not be empty when present",
        ));
    }

    Ok(())
}

fn validate_source(source: &Source) -> Result<(), EditGraphValidationError> {
    if source.fingerprint.file_size < 0 {
        return Err(EditGraphValidationError::new(
            "source.fingerprint.file_size",
            "must be >= 0",
        ));
    }
    Ok(())
}

fn validate_basic(basic: &BasicAdjustments) -> Result<(), EditGraphValidationError> {
    validate_range("basic.temperature", &basic.temperature, 1000.0, 50000.0)?;
    validate_range("basic.tint", &basic.tint, -150.0, 150.0)?;
    validate_range("basic.exposure", &basic.exposure, -5.0, 5.0)?;
    for (path, value) in [
        ("basic.contrast", &basic.contrast),
        ("basic.highlights", &basic.highlights),
        ("basic.shadows", &basic.shadows),
        ("basic.whites", &basic.whites),
        ("basic.blacks", &basic.blacks),
        ("basic.texture", &basic.texture),
        ("basic.clarity", &basic.clarity),
        ("basic.dehaze", &basic.dehaze),
        ("basic.vibrance", &basic.vibrance),
        ("basic.saturation", &basic.saturation),
    ] {
        validate_range(path, value, -100.0, 100.0)?;
    }
    Ok(())
}

fn validate_tone(tone: &ToneAdjustments) -> Result<(), EditGraphValidationError> {
    if tone.curve_mode == CurveMode::None
        && (!tone.rgb_curve.is_empty()
            || !tone.red_curve.is_empty()
            || !tone.green_curve.is_empty()
            || !tone.blue_curve.is_empty())
    {
        return Err(EditGraphValidationError::new(
            "tone.curve_mode",
            "none mode must not carry curve points",
        ));
    }

    for (name, curve) in [
        ("tone.rgb_curve", &tone.rgb_curve),
        ("tone.red_curve", &tone.red_curve),
        ("tone.green_curve", &tone.green_curve),
        ("tone.blue_curve", &tone.blue_curve),
    ] {
        validate_curve_points(name, curve)?;
    }
    Ok(())
}

fn curve_points_from_pairs(
    path: &str,
    points: &[(f64, f64)],
) -> Result<Vec<CurvePoint>, EditGraphValidationError> {
    points
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            Ok(CurvePoint {
                x: number_from_f64(&format!("{path}.{index}.x"), *x)?,
                y: number_from_f64(&format!("{path}.{index}.y"), *y)?,
            })
        })
        .collect()
}

fn validate_curve_points(
    path: &str,
    points: &[CurvePoint],
) -> Result<(), EditGraphValidationError> {
    if points.is_empty() {
        return Ok(());
    }
    if points.len() < 2 {
        return Err(EditGraphValidationError::new(
            path,
            "non-empty curves must include endpoints",
        ));
    }

    let mut previous_x = None;
    for (index, point) in points.iter().enumerate() {
        let point_path = format!("{path}.{index}");
        validate_range(format!("{point_path}.x"), &point.x, 0.0, 1.0)?;
        validate_range(format!("{point_path}.y"), &point.y, 0.0, 1.0)?;
        let x = number_as_f64(&format!("{point_path}.x"), &point.x)?;
        if let Some(previous_x) = previous_x {
            if x <= previous_x {
                return Err(EditGraphValidationError::new(
                    format!("{point_path}.x"),
                    "x values must be strictly increasing",
                ));
            }
        }
        previous_x = Some(x);
    }

    let first = points.first().expect("non-empty curve checked");
    let last = points.last().expect("non-empty curve checked");
    let first_x = number_as_f64(&format!("{path}.0.x"), &first.x)?;
    let first_y = number_as_f64(&format!("{path}.0.y"), &first.y)?;
    let last_index = points.len() - 1;
    let last_x = number_as_f64(&format!("{path}.{last_index}.x"), &last.x)?;
    let last_y = number_as_f64(&format!("{path}.{last_index}.y"), &last.y)?;
    if first_x != 0.0 || first_y != 0.0 || last_x != 1.0 || last_y != 1.0 {
        return Err(EditGraphValidationError::new(
            path,
            "non-empty curves must start at (0, 0) and end at (1, 1)",
        ));
    }

    Ok(())
}

fn validate_color(color: &ColorAdjustments) -> Result<(), EditGraphValidationError> {
    for (name, channel) in [
        ("color.hsl.red", &color.hsl.red),
        ("color.hsl.orange", &color.hsl.orange),
        ("color.hsl.yellow", &color.hsl.yellow),
        ("color.hsl.green", &color.hsl.green),
        ("color.hsl.aqua", &color.hsl.aqua),
        ("color.hsl.blue", &color.hsl.blue),
        ("color.hsl.purple", &color.hsl.purple),
        ("color.hsl.magenta", &color.hsl.magenta),
    ] {
        validate_hsl_channel(name, channel)?;
    }
    for (name, wheel) in [
        ("color.grading.shadows", &color.grading.shadows),
        ("color.grading.midtones", &color.grading.midtones),
        ("color.grading.highlights", &color.grading.highlights),
    ] {
        validate_color_wheel(name, wheel)?;
    }
    validate_range(
        "color.grading.blending",
        &color.grading.blending,
        0.0,
        100.0,
    )?;
    validate_range(
        "color.grading.balance",
        &color.grading.balance,
        -100.0,
        100.0,
    )?;
    Ok(())
}

fn validate_hsl_channel(path: &str, channel: &HslChannel) -> Result<(), EditGraphValidationError> {
    validate_range(format!("{path}.hue"), &channel.hue, -100.0, 100.0)?;
    validate_range(
        format!("{path}.saturation"),
        &channel.saturation,
        -100.0,
        100.0,
    )?;
    validate_range(
        format!("{path}.luminance"),
        &channel.luminance,
        -100.0,
        100.0,
    )
}

fn validate_color_wheel(path: &str, wheel: &ColorWheel) -> Result<(), EditGraphValidationError> {
    validate_range(format!("{path}.hue"), &wheel.hue, 0.0, 360.0)?;
    validate_range(format!("{path}.saturation"), &wheel.saturation, 0.0, 100.0)?;
    validate_range(format!("{path}.luminance"), &wheel.luminance, -100.0, 100.0)
}

fn validate_detail(detail: &DetailAdjustments) -> Result<(), EditGraphValidationError> {
    validate_range(
        "detail.sharpening.amount",
        &detail.sharpening.amount,
        0.0,
        150.0,
    )?;
    validate_range(
        "detail.sharpening.radius",
        &detail.sharpening.radius,
        0.1,
        5.0,
    )?;
    validate_range(
        "detail.sharpening.detail",
        &detail.sharpening.detail,
        0.0,
        100.0,
    )?;
    validate_range(
        "detail.sharpening.masking",
        &detail.sharpening.masking,
        0.0,
        100.0,
    )?;
    validate_range(
        "detail.noise_reduction.luminance",
        &detail.noise_reduction.luminance,
        0.0,
        100.0,
    )?;
    validate_range(
        "detail.noise_reduction.detail",
        &detail.noise_reduction.detail,
        0.0,
        100.0,
    )?;
    validate_range(
        "detail.noise_reduction.contrast",
        &detail.noise_reduction.contrast,
        0.0,
        100.0,
    )?;
    validate_range(
        "detail.noise_reduction.color",
        &detail.noise_reduction.color,
        0.0,
        100.0,
    )?;
    validate_range(
        "detail.noise_reduction.color_detail",
        &detail.noise_reduction.color_detail,
        0.0,
        100.0,
    )?;
    if let Some(value) = &detail.mlx_denoise {
        if !value.is_object() {
            return Err(EditGraphValidationError::new(
                "detail.mlx_denoise",
                "must be object or null",
            ));
        }
    }
    Ok(())
}

fn validate_lens(lens: &LensAdjustments) -> Result<(), EditGraphValidationError> {
    validate_range("lens.distortion", &lens.distortion, -100.0, 100.0)?;
    validate_range("lens.vignetting", &lens.vignetting, -100.0, 100.0)
}

fn validate_geometry(geometry: &GeometryAdjustments) -> Result<(), EditGraphValidationError> {
    if let Some(crop) = &geometry.crop {
        validate_range("geometry.crop.x", &crop.x, 0.0, 1.0)?;
        validate_range("geometry.crop.y", &crop.y, 0.0, 1.0)?;
        validate_exclusive_min_range("geometry.crop.width", &crop.width, 0.0, 1.0)?;
        validate_exclusive_min_range("geometry.crop.height", &crop.height, 0.0, 1.0)?;
        validate_range("geometry.crop.angle", &crop.angle, -45.0, 45.0)?;
        let x = number_as_f64("geometry.crop.x", &crop.x)?;
        let y = number_as_f64("geometry.crop.y", &crop.y)?;
        let width = number_as_f64("geometry.crop.width", &crop.width)?;
        let height = number_as_f64("geometry.crop.height", &crop.height)?;
        if x + width > 1.0 {
            return Err(EditGraphValidationError::new(
                "geometry.crop.x",
                "x plus width must be <= 1",
            ));
        }
        if y + height > 1.0 {
            return Err(EditGraphValidationError::new(
                "geometry.crop.y",
                "y plus height must be <= 1",
            ));
        }
    }
    validate_range("geometry.rotation", &geometry.rotation, -180.0, 180.0)?;
    validate_range(
        "geometry.transform.vertical",
        &geometry.transform.vertical,
        -100.0,
        100.0,
    )?;
    validate_range(
        "geometry.transform.horizontal",
        &geometry.transform.horizontal,
        -100.0,
        100.0,
    )?;
    validate_range(
        "geometry.transform.aspect",
        &geometry.transform.aspect,
        -100.0,
        100.0,
    )?;
    validate_range(
        "geometry.transform.scale",
        &geometry.transform.scale,
        1.0,
        500.0,
    )?;
    validate_range(
        "geometry.transform.x_offset",
        &geometry.transform.x_offset,
        -100.0,
        100.0,
    )?;
    validate_range(
        "geometry.transform.y_offset",
        &geometry.transform.y_offset,
        -100.0,
        100.0,
    )
}

fn validate_mask(index: usize, mask: &Mask) -> Result<(), EditGraphValidationError> {
    let prefix = format!("masks.{index}");
    validate_range(format!("{prefix}.opacity"), &mask.opacity, 0.0, 100.0)?;
    validate_range(format!("{prefix}.feather"), &mask.feather, 0.0, 100.0)?;
    validate_mask_source(index, &mask.source)?;
    validate_mask_geometry(index, mask)?;
    validate_mask_brush(index, mask)?;
    for (key, value) in &mask.local_adjustments {
        validate_number(format!("{prefix}.local_adjustments.{key}"), value)?;
    }
    Ok(())
}

fn manual_mask_local_adjustments(
    adjustments: ManualMaskLocalAdjustments,
) -> Result<BTreeMap<String, Number>, EditGraphValidationError> {
    let mut values = BTreeMap::new();
    if let Some(exposure) = adjustments.exposure {
        values.insert(
            "exposure".to_string(),
            number_from_f64("masks.local_adjustments.exposure", exposure)?,
        );
        validate_range(
            "masks.local_adjustments.exposure",
            values.get("exposure").expect("inserted exposure"),
            -5.0,
            5.0,
        )?;
    }
    if let Some(contrast) = adjustments.contrast {
        values.insert(
            "contrast".to_string(),
            number_from_f64("masks.local_adjustments.contrast", contrast)?,
        );
        validate_range(
            "masks.local_adjustments.contrast",
            values.get("contrast").expect("inserted contrast"),
            -100.0,
            100.0,
        )?;
    }
    Ok(values)
}

fn non_empty_mask_string(
    path: &'static str,
    value: String,
) -> Result<String, EditGraphValidationError> {
    if value.trim().is_empty() {
        return Err(EditGraphValidationError::new(path, "must not be empty"));
    }
    Ok(value)
}

fn validate_mask_source(index: usize, source: &MaskSource) -> Result<(), EditGraphValidationError> {
    if source.kind != MaskSourceKind::Manual {
        return Ok(());
    }

    if source.ai_result_id.is_some()
        || source.cache_path.is_some()
        || source.model_id.is_some()
        || source.model_version.is_some()
        || !source.extensions.is_empty()
    {
        return Err(EditGraphValidationError::new(
            format!("masks.{index}.source"),
            "manual mask source is provenance only",
        ));
    }
    Ok(())
}

fn validate_mask_geometry(index: usize, mask: &Mask) -> Result<(), EditGraphValidationError> {
    let prefix = format!("masks.{index}.geometry");
    match (&mask.mask_type, &mask.geometry) {
        (MaskType::LinearGradient, Some(MaskGeometry::LinearGradient { .. })) => {}
        (MaskType::RadialGradient, Some(MaskGeometry::RadialGradient { .. })) => {}
        (MaskType::LinearGradient | MaskType::RadialGradient, None) => {
            return Err(EditGraphValidationError::new(
                prefix,
                "linear and radial gradient masks require matching geometry",
            ));
        }
        (MaskType::LinearGradient | MaskType::RadialGradient, Some(_)) => {
            return Err(EditGraphValidationError::new(
                prefix,
                "geometry kind must match mask type",
            ));
        }
        (_, Some(_)) => {
            return Err(EditGraphValidationError::new(
                prefix,
                "geometry is only supported for linear and radial gradient masks",
            ));
        }
        (_, None) => return Ok(()),
    }

    match mask.geometry.as_ref().expect("validated geometry presence") {
        MaskGeometry::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => {
            validate_range(format!("{prefix}.start_x"), start_x, 0.0, 1.0)?;
            validate_range(format!("{prefix}.start_y"), start_y, 0.0, 1.0)?;
            validate_range(format!("{prefix}.end_x"), end_x, 0.0, 1.0)?;
            validate_range(format!("{prefix}.end_y"), end_y, 0.0, 1.0)?;
            let start_x = number_as_f64(&format!("{prefix}.start_x"), start_x)?;
            let start_y = number_as_f64(&format!("{prefix}.start_y"), start_y)?;
            let end_x = number_as_f64(&format!("{prefix}.end_x"), end_x)?;
            let end_y = number_as_f64(&format!("{prefix}.end_y"), end_y)?;
            if start_x == end_x && start_y == end_y {
                return Err(EditGraphValidationError::new(
                    prefix,
                    "linear gradient start and end points must differ",
                ));
            }
        }
        MaskGeometry::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => {
            validate_range(format!("{prefix}.center_x"), center_x, 0.0, 1.0)?;
            validate_range(format!("{prefix}.center_y"), center_y, 0.0, 1.0)?;
            validate_exclusive_min_range(format!("{prefix}.radius_x"), radius_x, 0.0, 1.0)?;
            validate_exclusive_min_range(format!("{prefix}.radius_y"), radius_y, 0.0, 1.0)?;
            validate_range(format!("{prefix}.rotation"), rotation, -180.0, 180.0)?;
        }
    }
    Ok(())
}

fn validate_mask_brush(index: usize, mask: &Mask) -> Result<(), EditGraphValidationError> {
    let prefix = format!("masks.{index}.brush");
    match (&mask.mask_type, &mask.brush) {
        (MaskType::Brush, Some(brush)) => validate_brush(&prefix, brush),
        (MaskType::Brush, None) => Err(EditGraphValidationError::new(
            prefix,
            "brush masks require durable brush strokes",
        )),
        (_, Some(_)) => Err(EditGraphValidationError::new(
            prefix,
            "brush payload is only supported for brush masks",
        )),
        (_, None) => Ok(()),
    }
}

fn validate_brush(prefix: &str, brush: &MaskBrush) -> Result<(), EditGraphValidationError> {
    if brush.strokes.is_empty() {
        return Err(EditGraphValidationError::new(
            format!("{prefix}.strokes"),
            "must contain at least one stroke",
        ));
    }
    if brush.strokes.len() > MAX_MANUAL_BRUSH_STROKES {
        return Err(EditGraphValidationError::new(
            format!("{prefix}.strokes"),
            format!("must contain at most {MAX_MANUAL_BRUSH_STROKES} strokes"),
        ));
    }
    let total_points = brush
        .strokes
        .iter()
        .map(|stroke| stroke.points.len())
        .sum::<usize>();
    if total_points > MAX_MANUAL_BRUSH_POINTS {
        return Err(EditGraphValidationError::new(
            format!("{prefix}.strokes.points"),
            format!("must contain at most {MAX_MANUAL_BRUSH_POINTS} total points"),
        ));
    }
    for (index, stroke) in brush.strokes.iter().enumerate() {
        validate_brush_stroke(&format!("{prefix}.strokes.{index}"), stroke)?;
    }
    Ok(())
}

fn validate_brush_stroke(
    prefix: &str,
    stroke: &MaskBrushStroke,
) -> Result<(), EditGraphValidationError> {
    if stroke.id.trim().is_empty() {
        return Err(EditGraphValidationError::new(
            format!("{prefix}.id"),
            "must not be empty",
        ));
    }
    validate_exclusive_min_range(format!("{prefix}.radius"), &stroke.radius, 0.0, 1.0)?;
    if stroke.points.is_empty() {
        return Err(EditGraphValidationError::new(
            format!("{prefix}.points"),
            "must contain at least one point",
        ));
    }
    for (index, point) in stroke.points.iter().enumerate() {
        validate_range(format!("{prefix}.points.{index}.x"), &point.x, 0.0, 1.0)?;
        validate_range(format!("{prefix}.points.{index}.y"), &point.y, 0.0, 1.0)?;
    }
    Ok(())
}

fn validate_metadata(metadata: &EditMetadata) -> Result<(), EditGraphValidationError> {
    if !(0..=5).contains(&metadata.rating) {
        return Err(EditGraphValidationError::new(
            "metadata.rating",
            "must be between 0 and 5",
        ));
    }
    Ok(())
}

fn validate_range(
    path: impl Into<String>,
    value: &Number,
    min: f64,
    max: f64,
) -> Result<(), EditGraphValidationError> {
    let path = path.into();
    let value = number_as_f64(&path, value)?;
    if !value.is_finite() || value < min || value > max {
        return Err(EditGraphValidationError::new(
            path,
            format!("must be between {min} and {max}"),
        ));
    }
    Ok(())
}

fn validate_exclusive_min_range(
    path: impl Into<String>,
    value: &Number,
    min: f64,
    max: f64,
) -> Result<(), EditGraphValidationError> {
    let path = path.into();
    let value = number_as_f64(&path, value)?;
    if !value.is_finite() || value <= min || value > max {
        return Err(EditGraphValidationError::new(
            path,
            format!("must be greater than {min} and <= {max}"),
        ));
    }
    Ok(())
}

fn validate_number(
    path: impl Into<String>,
    value: &Number,
) -> Result<(), EditGraphValidationError> {
    let path = path.into();
    number_as_f64(&path, value)?;
    Ok(())
}

fn number_as_f64(path: &str, value: &Number) -> Result<f64, EditGraphValidationError> {
    value
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| EditGraphValidationError::new(path, "must be a finite number"))
}

fn number_from_f64(path: &str, value: f64) -> Result<Number, EditGraphValidationError> {
    Number::from_f64(value)
        .filter(|_| value.is_finite())
        .ok_or_else(|| EditGraphValidationError::new(path, "must be a finite number"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-edit");
    }

    #[test]
    fn deserializes_and_serializes_edit_graph_example() {
        let example = include_str!("../../../schemas/edit_graph.example.json");
        let original: serde_json::Value =
            serde_json::from_str(example).expect("parse edit graph example");

        let graph: super::EditGraph =
            serde_json::from_value(original.clone()).expect("deserialize edit graph");
        let serialized = serde_json::to_value(&graph).expect("serialize edit graph");

        assert_eq!(serialized, original);
        super::validate_edit_graph_json(&serialized).expect("schema-aware validation");
    }

    #[test]
    fn keeps_unknown_experimental_data_under_extensions() {
        let mut graph: super::EditGraph =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("deserialize edit graph");
        graph.extensions.insert(
            "com.example.experimental".to_string(),
            json!({"preview": true, "strength": 0.25}),
        );

        let serialized = serde_json::to_value(&graph).expect("serialize edit graph");
        assert!(serialized.get("com.example.experimental").is_none());
        assert_eq!(
            serialized
                .get("extensions")
                .and_then(|extensions| extensions.get("com.example.experimental")),
            Some(&json!({"preview": true, "strength": 0.25}))
        );
        super::validate_edit_graph_json(&serialized).expect("extensions validation");
    }

    #[test]
    fn rejects_unknown_experimental_data_outside_extensions() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("parse edit graph example");
        value["com.example.experimental"] = json!({"preview": true});

        super::validate_edit_graph_json(&value).expect_err("unknown top-level field");
    }

    #[test]
    fn rejects_invalid_edit_graph_values() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("parse edit graph example");
        value["basic"]["exposure"] = json!(12.0);

        let error = super::validate_edit_graph_json(&value).expect_err("invalid exposure");
        assert!(error.to_string().contains("basic.exposure"));
    }

    #[test]
    fn validates_and_round_trips_manual_gradient_masks() {
        let mut graph: super::EditGraph =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("deserialize edit graph");

        graph.masks.push(super::Mask {
            id: "mask-linear-1".to_string(),
            mask_type: super::MaskType::LinearGradient,
            name: "Linear burn".to_string(),
            enabled: true,
            invert: false,
            opacity: serde_json::Number::from(75),
            feather: serde_json::Number::from(30),
            source: super::manual_mask_source(),
            geometry: Some(super::MaskGeometry::LinearGradient {
                start_x: serde_json::Number::from_f64(0.15).expect("finite start x"),
                start_y: serde_json::Number::from_f64(0.20).expect("finite start y"),
                end_x: serde_json::Number::from_f64(0.85).expect("finite end x"),
                end_y: serde_json::Number::from_f64(0.80).expect("finite end y"),
            }),
            brush: None,
            local_adjustments: std::collections::BTreeMap::from([(
                "exposure".to_string(),
                serde_json::Number::from_f64(-0.5).expect("finite exposure"),
            )]),
        });
        graph.masks.push(super::Mask {
            id: "mask-radial-1".to_string(),
            mask_type: super::MaskType::RadialGradient,
            name: "Face lift".to_string(),
            enabled: true,
            invert: false,
            opacity: serde_json::Number::from(60),
            feather: serde_json::Number::from(45),
            source: super::manual_mask_source(),
            geometry: Some(super::MaskGeometry::RadialGradient {
                center_x: serde_json::Number::from_f64(0.50).expect("finite center x"),
                center_y: serde_json::Number::from_f64(0.48).expect("finite center y"),
                radius_x: serde_json::Number::from_f64(0.20).expect("finite radius x"),
                radius_y: serde_json::Number::from_f64(0.28).expect("finite radius y"),
                rotation: serde_json::Number::from(0),
            }),
            brush: None,
            local_adjustments: std::collections::BTreeMap::from([(
                "contrast".to_string(),
                serde_json::Number::from(8),
            )]),
        });

        super::validate_edit_graph(&graph).expect("manual gradient masks validate");

        let serialized = serde_json::to_value(&graph).expect("serialize masked graph");
        assert_eq!(serialized["masks"][0]["source"], json!({"kind": "manual"}));
        assert_eq!(
            serialized["masks"][0]["geometry"]["kind"],
            json!("linear_gradient")
        );
        assert_eq!(
            serialized["masks"][1]["geometry"]["kind"],
            json!("radial_gradient")
        );

        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip masked graph");
        assert_eq!(round_tripped, graph);
        super::validate_edit_graph_json(&serialized).expect("manual mask JSON validates");
    }

    #[test]
    fn rejects_invalid_manual_gradient_mask_shapes() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("parse edit graph example");
        value["masks"] = json!([{
            "id": "mask-linear-1",
            "type": "linear_gradient",
            "name": "Bad linear",
            "enabled": true,
            "invert": false,
            "opacity": 125,
            "feather": 20,
            "source": {"kind": "manual"},
            "geometry": {
                "kind": "linear_gradient",
                "start_x": 0.5,
                "start_y": 0.5,
                "end_x": 0.5,
                "end_y": 0.5
            },
            "local_adjustments": {"exposure": -0.25}
        }]);

        let opacity_error =
            super::validate_edit_graph_json(&value).expect_err("opacity above range");
        assert!(opacity_error.to_string().contains("masks.0.opacity"));

        value["masks"][0]["opacity"] = json!(80);
        let geometry_error =
            super::validate_edit_graph_json(&value).expect_err("degenerate linear mask");
        assert!(geometry_error.to_string().contains("masks.0.geometry"));

        value["masks"][0]["geometry"]["end_x"] = json!(0.75);
        value["masks"][0]["type"] = json!("polygon");
        let type_error = super::validate_edit_graph_json(&value).expect_err("invalid mask type");
        assert!(type_error.to_string().contains("unknown variant"));

        value["masks"][0]["type"] = json!("linear_gradient");
        value["masks"][0]["source"] = json!({});
        let source_error =
            super::validate_edit_graph_json(&value).expect_err("missing source kind");
        assert!(source_error.to_string().contains("kind"));
    }

    #[test]
    fn rejects_manual_mask_source_runtime_or_geometry_payloads() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("parse edit graph example");
        value["masks"] = json!([{
            "id": "mask-linear-1",
            "type": "linear_gradient",
            "name": "Bad provenance",
            "enabled": true,
            "invert": false,
            "opacity": 80,
            "feather": 20,
            "source": {
                "kind": "manual",
                "cache_path": "render-cache/masks/mask-linear-1.png"
            },
            "geometry": {
                "kind": "linear_gradient",
                "start_x": 0.2,
                "start_y": 0.2,
                "end_x": 0.8,
                "end_y": 0.8
            },
            "local_adjustments": {"exposure": -0.25}
        }]);

        let cache_error =
            super::validate_edit_graph_json(&value).expect_err("manual source cache path");
        assert!(cache_error.to_string().contains("masks.0.source"));

        value["masks"][0]["source"] = json!({
            "kind": "manual",
            "model_id": "subject-model",
            "model_version": "1.0"
        });
        let model_error =
            super::validate_edit_graph_json(&value).expect_err("manual source model metadata");
        assert!(model_error.to_string().contains("masks.0.source"));

        value["masks"][0]["source"] = json!({
            "kind": "manual",
            "geometry": {"hidden": true}
        });
        let hidden_geometry_error =
            super::validate_edit_graph_json(&value).expect_err("manual source hidden geometry");
        assert!(hidden_geometry_error.to_string().contains("masks.0.source"));
    }

    #[test]
    fn builds_manual_gradient_masks_with_supported_local_adjustments() {
        let graph: super::EditGraph =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("deserialize edit graph");
        let adjustments = super::ManualMaskLocalAdjustments {
            exposure: Some(-0.75),
            contrast: Some(12.0),
        };

        let linear = super::manual_linear_gradient_mask(
            "mask-linear-1",
            "Top burn",
            80.0,
            25.0,
            false,
            0.2,
            0.0,
            0.8,
            1.0,
            adjustments,
        )
        .expect("manual linear mask");
        let edited =
            super::append_manual_mask(&graph, linear, "unix:3").expect("append linear mask");
        let radial = super::manual_radial_gradient_mask(
            "mask-radial-1",
            "Face lift",
            60.0,
            45.0,
            false,
            0.5,
            0.45,
            0.25,
            0.3,
            0.0,
            super::ManualMaskLocalAdjustments {
                exposure: Some(0.35),
                contrast: None,
            },
        )
        .expect("manual radial mask");
        let edited =
            super::append_manual_mask(&edited, radial, "unix:4").expect("append radial mask");

        assert_eq!(edited.updated_at, "unix:4");
        assert_eq!(edited.masks.len(), 2);
        assert_eq!(edited.masks[0].source, super::manual_mask_source());
        assert_eq!(
            edited.masks[0]
                .local_adjustments
                .get("exposure")
                .and_then(|value| value.as_f64()),
            Some(-0.75)
        );
        assert_eq!(
            edited.masks[0]
                .local_adjustments
                .get("contrast")
                .and_then(|value| value.as_f64()),
            Some(12.0)
        );

        let serialized = serde_json::to_value(&edited).expect("serialize manual masks");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip manual masks");
        assert_eq!(round_tripped, edited);
        super::validate_edit_graph_json(&serialized).expect("manual masks validate");
    }

    #[test]
    fn rejects_invalid_manual_gradient_mask_helpers() {
        let graph: super::EditGraph =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("deserialize edit graph");

        let bad_adjustment = super::manual_linear_gradient_mask(
            "mask-linear-1",
            "Too bright",
            80.0,
            25.0,
            false,
            0.2,
            0.0,
            0.8,
            1.0,
            super::ManualMaskLocalAdjustments {
                exposure: Some(6.0),
                contrast: Some(0.0),
            },
        )
        .expect_err("unsupported exposure range");
        assert!(bad_adjustment
            .to_string()
            .contains("local_adjustments.exposure"));

        let mask = super::manual_radial_gradient_mask(
            "mask-radial-1",
            "Face lift",
            60.0,
            45.0,
            false,
            0.5,
            0.45,
            0.25,
            0.3,
            0.0,
            super::ManualMaskLocalAdjustments {
                exposure: Some(0.35),
                contrast: None,
            },
        )
        .expect("manual radial mask");
        let edited =
            super::append_manual_mask(&graph, mask.clone(), "unix:3").expect("append mask once");
        let duplicate =
            super::append_manual_mask(&edited, mask, "unix:4").expect_err("duplicate mask id");
        assert!(duplicate.to_string().contains("masks"));
    }

    #[test]
    fn builds_manual_brush_masks_with_durable_strokes() {
        let graph: super::EditGraph =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("deserialize edit graph");
        let stroke = super::manual_brush_stroke(
            "stroke-1",
            0.08,
            vec![(0.25, 0.30), (0.50, 0.45), (0.75, 0.60)],
        )
        .expect("manual brush stroke");
        let mask = super::manual_brush_mask(
            "mask-brush-1",
            "Cheek dodge",
            70.0,
            0.0,
            false,
            vec![stroke],
            super::ManualMaskLocalAdjustments {
                exposure: Some(0.35),
                contrast: Some(5.0),
            },
        )
        .expect("manual brush mask");

        let edited = super::append_manual_mask(&graph, mask, "unix:5").expect("append brush mask");
        assert_eq!(edited.masks.len(), 1);
        assert_eq!(edited.masks[0].mask_type, super::MaskType::Brush);
        assert_eq!(edited.masks[0].source, super::manual_mask_source());
        assert!(edited.masks[0].geometry.is_none());
        assert_eq!(
            edited.masks[0]
                .brush
                .as_ref()
                .expect("brush payload")
                .coordinate_space,
            super::BrushCoordinateSpace::NormalizedImage
        );

        let serialized = serde_json::to_value(&edited).expect("serialize brush mask");
        assert_eq!(serialized["masks"][0]["source"], json!({"kind": "manual"}));
        assert!(serialized["masks"][0].get("geometry").is_none());
        assert_eq!(
            serialized["masks"][0]["brush"]["coordinate_space"],
            json!("normalized_image")
        );
        assert_eq!(
            serialized["masks"][0]["brush"]["strokes"][0]["points"][1]["x"],
            json!(0.5)
        );

        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip brush mask");
        assert_eq!(round_tripped, edited);
        super::validate_edit_graph_json(&serialized).expect("manual brush JSON validates");
    }

    #[test]
    fn rejects_invalid_manual_brush_masks() {
        let graph: super::EditGraph =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("deserialize edit graph");

        let bad_radius = super::manual_brush_stroke("stroke-1", 0.0, vec![(0.25, 0.30)])
            .expect_err("zero radius");
        assert!(bad_radius.to_string().contains("radius"));

        let bad_point = super::manual_brush_stroke("stroke-1", 0.05, vec![(1.25, 0.30)])
            .expect_err("point outside normalized range");
        assert!(bad_point.to_string().contains("points.0.x"));

        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/edit_graph.example.json"))
                .expect("parse edit graph example");
        value["masks"] = json!([{
            "id": "mask-brush-1",
            "type": "brush",
            "name": "Missing brush",
            "enabled": true,
            "invert": false,
            "opacity": 80,
            "feather": 0,
            "source": {"kind": "manual"},
            "local_adjustments": {"exposure": 0.25}
        }]);
        let missing_brush =
            super::validate_edit_graph_json(&value).expect_err("brush payload required");
        assert!(missing_brush.to_string().contains("masks.0.brush"));

        value["masks"][0]["brush"] = json!({
            "coordinate_space": "normalized_image",
            "strokes": [{
                "id": "stroke-1",
                "radius": 0.05,
                "points": [{"x": 0.25, "y": 0.30}]
            }]
        });
        value["masks"][0]["geometry"] = json!({
            "kind": "linear_gradient",
            "start_x": 0.0,
            "start_y": 0.0,
            "end_x": 1.0,
            "end_y": 1.0
        });
        let brush_geometry =
            super::validate_edit_graph_json(&value).expect_err("brush geometry rejected");
        assert!(brush_geometry.to_string().contains("masks.0.geometry"));

        let stroke =
            super::manual_brush_stroke("stroke-1", 0.05, vec![(0.25, 0.30)]).expect("valid stroke");
        let mut too_many_strokes = Vec::new();
        for index in 0..=super::MAX_MANUAL_BRUSH_STROKES {
            let mut copy = stroke.clone();
            copy.id = format!("stroke-{index}");
            too_many_strokes.push(copy);
        }
        let too_many = super::manual_brush_mask(
            "mask-brush-1",
            "Too many strokes",
            80.0,
            0.0,
            false,
            too_many_strokes,
            super::ManualMaskLocalAdjustments::default(),
        )
        .expect_err("stroke cap");
        assert!(too_many.to_string().contains("strokes"));

        let mask = super::manual_brush_mask(
            "mask-brush-1",
            "Valid brush",
            80.0,
            0.0,
            false,
            vec![stroke],
            super::ManualMaskLocalAdjustments::default(),
        )
        .expect("valid brush");
        let edited =
            super::append_manual_mask(&graph, mask.clone(), "unix:5").expect("append brush");
        let duplicate =
            super::append_manual_mask(&edited, mask, "unix:6").expect_err("duplicate brush id");
        assert!(duplicate.to_string().contains("masks"));
    }

    #[test]
    fn builds_default_graph_and_applies_exposure_contrast() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: Some("unix:1".to_string()),
                partial_hash: Some("partial-hash".to_string()),
                full_hash: None,
            },
            "unix:2",
        );

        assert_eq!(graph.basic.exposure.as_f64(), Some(0.0));
        assert_eq!(graph.basic.contrast.as_f64(), Some(0.0));
        assert_eq!(graph.source.photo_id, "photo-1");
        assert_eq!(graph.profile.input_profile, super::INPUT_PROFILE_UNKNOWN);
        assert_eq!(
            graph.profile.working_space,
            super::WORKING_SPACE_LINEAR_DISPLAY_P3
        );
        assert_eq!(graph.profile.decoder_backend, None);
        super::validate_edit_graph(&graph).expect("default edit graph validates");

        let edited = super::apply_exposure_contrast(&graph, 0.75, -12.0, "unix:3")
            .expect("apply exposure and contrast");

        assert_eq!(edited.basic.exposure.as_f64(), Some(0.75));
        assert_eq!(edited.basic.contrast.as_f64(), Some(-12.0));
        assert_eq!(edited.updated_at, "unix:3");
        super::validate_edit_graph(&edited).expect("edited graph validates");
    }

    #[test]
    fn applies_white_balance_temperature_tint_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let edited = super::apply_white_balance_temperature_tint(
            &graph,
            super::WhiteBalance::Cloudy,
            6400.0,
            12.5,
            "unix:3",
        )
        .expect("apply white balance family");
        let serialized = serde_json::to_value(&edited).expect("serialize edited graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip edited graph");

        assert_eq!(edited.basic.white_balance, super::WhiteBalance::Cloudy);
        assert_eq!(edited.basic.temperature.as_f64(), Some(6400.0));
        assert_eq!(edited.basic.tint.as_f64(), Some(12.5));
        assert_eq!(edited.updated_at, "unix:3");
        assert_eq!(
            round_tripped.basic.white_balance,
            super::WhiteBalance::Cloudy
        );
        assert_eq!(round_tripped.basic.temperature.as_f64(), Some(6400.0));
        assert_eq!(round_tripped.basic.tint.as_f64(), Some(12.5));
        assert_eq!(serialized["basic"]["white_balance"], json!("cloudy"));
        assert_eq!(serialized["basic"]["temperature"].as_f64(), Some(6400.0));
        assert_eq!(serialized["basic"]["tint"].as_f64(), Some(12.5));
        super::validate_edit_graph_json(&serialized).expect("white balance graph validates");
    }

    #[test]
    fn applies_tone_recovery_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let edited = super::apply_tone_recovery(&graph, -35.0, 42.0, 10.0, -12.5, "unix:3")
            .expect("apply tone recovery");
        let serialized = serde_json::to_value(&edited).expect("serialize edited graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip edited graph");

        assert_eq!(edited.basic.highlights.as_f64(), Some(-35.0));
        assert_eq!(edited.basic.shadows.as_f64(), Some(42.0));
        assert_eq!(edited.basic.whites.as_f64(), Some(10.0));
        assert_eq!(edited.basic.blacks.as_f64(), Some(-12.5));
        assert_eq!(edited.updated_at, "unix:3");
        assert_eq!(round_tripped.basic.highlights.as_f64(), Some(-35.0));
        assert_eq!(round_tripped.basic.shadows.as_f64(), Some(42.0));
        assert_eq!(round_tripped.basic.whites.as_f64(), Some(10.0));
        assert_eq!(round_tripped.basic.blacks.as_f64(), Some(-12.5));
        assert_eq!(serialized["basic"]["highlights"].as_f64(), Some(-35.0));
        assert_eq!(serialized["basic"]["shadows"].as_f64(), Some(42.0));
        assert_eq!(serialized["basic"]["whites"].as_f64(), Some(10.0));
        assert_eq!(serialized["basic"]["blacks"].as_f64(), Some(-12.5));
        super::validate_edit_graph_json(&serialized).expect("tone recovery graph validates");
    }

    #[test]
    fn applies_tone_curve_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let edited = super::apply_tone_curve(
            &graph,
            super::CurveMode::Point,
            &[(0.0, 0.0), (0.35, 0.28), (0.72, 0.81), (1.0, 1.0)],
            &[(0.0, 0.0), (0.5, 0.48), (1.0, 1.0)],
            &[],
            &[],
            "unix:3",
        )
        .expect("apply tone curve");
        let serialized = serde_json::to_value(&edited).expect("serialize edited graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip edited graph");

        assert_eq!(edited.tone.curve_mode, super::CurveMode::Point);
        assert_eq!(edited.tone.rgb_curve.len(), 4);
        assert_eq!(edited.tone.red_curve.len(), 3);
        assert!(edited.tone.green_curve.is_empty());
        assert!(edited.tone.blue_curve.is_empty());
        assert_eq!(edited.tone.rgb_curve[1].x.as_f64(), Some(0.35));
        assert_eq!(edited.tone.rgb_curve[1].y.as_f64(), Some(0.28));
        assert_eq!(edited.updated_at, "unix:3");
        assert_eq!(round_tripped.tone.curve_mode, super::CurveMode::Point);
        assert_eq!(serialized["tone"]["curve_mode"], json!("point"));
        assert_eq!(serialized["tone"]["rgb_curve"][2]["x"].as_f64(), Some(0.72));
        assert_eq!(serialized["tone"]["rgb_curve"][2]["y"].as_f64(), Some(0.81));
        super::validate_edit_graph_json(&serialized).expect("tone curve graph validates");
    }

    #[test]
    fn rejects_invalid_tone_curve_points() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let missing_endpoint_error = super::apply_tone_curve(
            &graph,
            super::CurveMode::Point,
            &[(0.1, 0.0), (1.0, 1.0)],
            &[],
            &[],
            &[],
            "unix:3",
        )
        .expect_err("missing curve origin");
        let duplicate_x_error = super::apply_tone_curve(
            &graph,
            super::CurveMode::Point,
            &[(0.0, 0.0), (0.5, 0.4), (0.5, 0.6), (1.0, 1.0)],
            &[],
            &[],
            &[],
            "unix:3",
        )
        .expect_err("duplicate x coordinate");
        let descending_x_error = super::apply_tone_curve(
            &graph,
            super::CurveMode::Point,
            &[(0.0, 0.0), (0.8, 0.7), (0.6, 0.65), (1.0, 1.0)],
            &[],
            &[],
            &[],
            "unix:3",
        )
        .expect_err("descending x coordinate");
        let parametric_error = super::apply_tone_curve(
            &graph,
            super::CurveMode::Parametric,
            &[],
            &[],
            &[],
            &[],
            "unix:3",
        )
        .expect_err("parametric curve has no schema-owned parameters yet");

        assert!(missing_endpoint_error
            .to_string()
            .contains("tone.rgb_curve"));
        assert!(duplicate_x_error.to_string().contains("tone.rgb_curve"));
        assert!(descending_x_error.to_string().contains("tone.rgb_curve"));
        assert!(parametric_error.to_string().contains("tone.curve_mode"));
    }

    #[test]
    fn applies_color_presence_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let edited = super::apply_color_presence(&graph, 24.0, -8.5, "unix:3")
            .expect("apply color presence");
        let serialized = serde_json::to_value(&edited).expect("serialize edited graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip edited graph");

        assert_eq!(edited.basic.vibrance.as_f64(), Some(24.0));
        assert_eq!(edited.basic.saturation.as_f64(), Some(-8.5));
        assert_eq!(edited.updated_at, "unix:3");
        assert_eq!(round_tripped.basic.vibrance.as_f64(), Some(24.0));
        assert_eq!(round_tripped.basic.saturation.as_f64(), Some(-8.5));
        assert_eq!(serialized["basic"]["vibrance"].as_f64(), Some(24.0));
        assert_eq!(serialized["basic"]["saturation"].as_f64(), Some(-8.5));
        super::validate_edit_graph_json(&serialized).expect("color presence graph validates");
    }

    #[test]
    fn applies_hsl_color_channel_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let edited = super::apply_hsl_color_channel(
            &graph,
            super::HslColorChannel::Blue,
            -12.0,
            24.0,
            -8.5,
            "unix:3",
        )
        .expect("apply HSL color channel");
        let serialized = serde_json::to_value(&edited).expect("serialize HSL graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip HSL graph");

        assert_eq!(edited.color.hsl.blue.hue.as_f64(), Some(-12.0));
        assert_eq!(edited.color.hsl.blue.saturation.as_f64(), Some(24.0));
        assert_eq!(edited.color.hsl.blue.luminance.as_f64(), Some(-8.5));
        assert_eq!(edited.color.hsl.red.hue.as_f64(), Some(0.0));
        assert_eq!(edited.basic.vibrance.as_f64(), Some(0.0));
        assert_eq!(edited.updated_at, "unix:3");
        assert_eq!(round_tripped.color.hsl.blue.saturation.as_f64(), Some(24.0));
        assert_eq!(
            serialized["color"]["hsl"]["blue"]["hue"].as_f64(),
            Some(-12.0)
        );
        assert_eq!(
            serialized["color"]["hsl"]["blue"]["luminance"].as_f64(),
            Some(-8.5)
        );
        for (name, expected) in [
            ("red", super::HslColorChannel::Red),
            ("orange", super::HslColorChannel::Orange),
            ("yellow", super::HslColorChannel::Yellow),
            ("green", super::HslColorChannel::Green),
            ("aqua", super::HslColorChannel::Aqua),
            ("blue", super::HslColorChannel::Blue),
            ("purple", super::HslColorChannel::Purple),
            ("magenta", super::HslColorChannel::Magenta),
        ] {
            assert_eq!(
                super::HslColorChannel::try_from(name).expect("parse HSL color channel"),
                expected
            );
        }
        super::validate_edit_graph_json(&serialized).expect("HSL graph validates");
    }

    #[test]
    fn rejects_invalid_hsl_color_channel_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let invalid_channel =
            super::HslColorChannel::try_from("cyan").expect_err("unsupported HSL channel name");
        let hue_error = super::apply_hsl_color_channel(
            &graph,
            super::HslColorChannel::Red,
            101.0,
            0.0,
            0.0,
            "unix:3",
        )
        .expect_err("hue above schema range");
        let saturation_error = super::apply_hsl_color_channel(
            &graph,
            super::HslColorChannel::Orange,
            0.0,
            -101.0,
            0.0,
            "unix:3",
        )
        .expect_err("saturation below schema range");
        let luminance_error = super::apply_hsl_color_channel(
            &graph,
            super::HslColorChannel::Magenta,
            0.0,
            0.0,
            101.0,
            "unix:3",
        )
        .expect_err("luminance above schema range");

        assert!(invalid_channel.to_string().contains("color.hsl"));
        assert!(hue_error.to_string().contains("color.hsl.red.hue"));
        assert!(saturation_error
            .to_string()
            .contains("color.hsl.orange.saturation"));
        assert!(luminance_error
            .to_string()
            .contains("color.hsl.magenta.luminance"));
    }

    #[test]
    fn applies_detail_sharpening_and_noise_reduction_and_round_trips_json() {
        let mut graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );
        graph.detail.mlx_denoise = Some(json!({ "status": "deferred" }));

        let sharpened = super::apply_detail_sharpening(&graph, 82.0, 1.4, 58.0, 22.0, "unix:3")
            .expect("apply detail sharpening");
        let edited =
            super::apply_detail_noise_reduction(&sharpened, 32.0, 44.0, 18.0, 26.0, 64.0, "unix:4")
                .expect("apply detail noise reduction");
        let serialized = serde_json::to_value(&edited).expect("serialize detail graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip detail graph");

        assert_eq!(edited.detail.sharpening.amount.as_f64(), Some(82.0));
        assert_eq!(edited.detail.sharpening.radius.as_f64(), Some(1.4));
        assert_eq!(edited.detail.sharpening.detail.as_f64(), Some(58.0));
        assert_eq!(edited.detail.sharpening.masking.as_f64(), Some(22.0));
        assert_eq!(edited.detail.noise_reduction.luminance.as_f64(), Some(32.0));
        assert_eq!(edited.detail.noise_reduction.detail.as_f64(), Some(44.0));
        assert_eq!(edited.detail.noise_reduction.contrast.as_f64(), Some(18.0));
        assert_eq!(edited.detail.noise_reduction.color.as_f64(), Some(26.0));
        assert_eq!(
            edited.detail.noise_reduction.color_detail.as_f64(),
            Some(64.0)
        );
        assert_eq!(
            edited.detail.mlx_denoise.as_ref(),
            Some(&json!({ "status": "deferred" }))
        );
        assert_eq!(edited.updated_at, "unix:4");
        assert_eq!(round_tripped.detail.sharpening.amount.as_f64(), Some(82.0));
        assert_eq!(
            serialized["detail"]["noise_reduction"]["color_detail"].as_f64(),
            Some(64.0)
        );
        super::validate_edit_graph_json(&serialized).expect("detail graph validates");
    }

    #[test]
    fn rejects_invalid_detail_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let amount_error = super::apply_detail_sharpening(&graph, 151.0, 1.0, 25.0, 0.0, "unix:3")
            .expect_err("sharpening amount above schema range");
        let radius_error = super::apply_detail_sharpening(&graph, 40.0, 0.0, 25.0, 0.0, "unix:3")
            .expect_err("sharpening radius below schema range");
        let luminance_error =
            super::apply_detail_noise_reduction(&graph, 101.0, 50.0, 0.0, 25.0, 50.0, "unix:3")
                .expect_err("luminance noise reduction above schema range");
        let color_detail_error =
            super::apply_detail_noise_reduction(&graph, 0.0, 50.0, 0.0, 25.0, -1.0, "unix:3")
                .expect_err("color detail below schema range");

        assert!(amount_error
            .to_string()
            .contains("detail.sharpening.amount"));
        assert!(radius_error
            .to_string()
            .contains("detail.sharpening.radius"));
        assert!(luminance_error
            .to_string()
            .contains("detail.noise_reduction.luminance"));
        assert!(color_detail_error
            .to_string()
            .contains("detail.noise_reduction.color_detail"));
    }

    #[test]
    fn applies_lens_and_geometry_mutators_and_round_trips_json() {
        let mut graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );
        graph.lens.profile_id = Some("embedded-profile-id".to_string());

        let lens = super::apply_lens_adjustments(&graph, true, true, -12.5, 18.0, "unix:3")
            .expect("apply lens adjustments");
        let transformed =
            super::apply_geometry_transform(&lens, 6.0, -5.0, 2.5, 125.0, 10.0, -8.0, "unix:4")
                .expect("apply geometry transform");
        let oriented = super::apply_geometry_orientation(&transformed, 90.0, true, false, "unix:5")
            .expect("apply geometry orientation");
        let cropped =
            super::apply_geometry_crop(&oriented, 0.1, 0.2, 0.7, 0.6, -2.5, Some("4:3"), "unix:6")
                .expect("apply geometry crop");
        let serialized = serde_json::to_value(&cropped).expect("serialize lens geometry graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip lens geometry graph");

        assert!(cropped.lens.profile_correction);
        assert_eq!(
            cropped.lens.profile_id.as_deref(),
            Some("embedded-profile-id")
        );
        assert!(cropped.lens.chromatic_aberration);
        assert_eq!(cropped.lens.distortion.as_f64(), Some(-12.5));
        assert_eq!(cropped.lens.vignetting.as_f64(), Some(18.0));
        assert_eq!(cropped.geometry.rotation.as_f64(), Some(90.0));
        assert!(cropped.geometry.flip_horizontal);
        assert!(!cropped.geometry.flip_vertical);
        assert_eq!(cropped.geometry.transform.vertical.as_f64(), Some(6.0));
        assert_eq!(cropped.geometry.transform.horizontal.as_f64(), Some(-5.0));
        assert_eq!(cropped.geometry.transform.aspect.as_f64(), Some(2.5));
        assert_eq!(cropped.geometry.transform.scale.as_f64(), Some(125.0));
        assert_eq!(cropped.geometry.transform.x_offset.as_f64(), Some(10.0));
        assert_eq!(cropped.geometry.transform.y_offset.as_f64(), Some(-8.0));
        let crop = cropped.geometry.crop.as_ref().expect("crop exists");
        assert_eq!(crop.x.as_f64(), Some(0.1));
        assert_eq!(crop.y.as_f64(), Some(0.2));
        assert_eq!(crop.width.as_f64(), Some(0.7));
        assert_eq!(crop.height.as_f64(), Some(0.6));
        assert_eq!(crop.angle.as_f64(), Some(-2.5));
        assert_eq!(crop.aspect.as_deref(), Some("4:3"));
        assert_eq!(cropped.updated_at, "unix:6");
        assert_eq!(round_tripped.geometry.rotation.as_f64(), Some(90.0));
        assert_eq!(serialized["lens"]["profile_correction"], json!(true));
        assert_eq!(
            serialized["lens"]["profile_id"],
            json!("embedded-profile-id")
        );
        assert_eq!(serialized["geometry"]["crop"]["aspect"], json!("4:3"));
        super::validate_edit_graph_json(&serialized).expect("lens geometry graph validates");

        let cleared = super::clear_geometry_crop(&cropped, "unix:7").expect("clear crop");
        assert!(cleared.geometry.crop.is_none());
        assert_eq!(cleared.updated_at, "unix:7");
        super::validate_edit_graph(&cleared).expect("cleared geometry validates");
    }

    #[test]
    fn rejects_invalid_lens_and_geometry_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let distortion_error =
            super::apply_lens_adjustments(&graph, true, false, 101.0, 0.0, "unix:3")
                .expect_err("distortion above schema range");
        let vignetting_error =
            super::apply_lens_adjustments(&graph, true, false, 0.0, -101.0, "unix:3")
                .expect_err("vignetting below schema range");
        let rotation_error =
            super::apply_geometry_orientation(&graph, 181.0, false, false, "unix:3")
                .expect_err("rotation above schema range");
        let scale_error =
            super::apply_geometry_transform(&graph, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, "unix:3")
                .expect_err("scale below schema range");
        let offset_error =
            super::apply_geometry_transform(&graph, 0.0, 0.0, 0.0, 100.0, 101.0, 0.0, "unix:3")
                .expect_err("offset above schema range");
        let crop_width_error =
            super::apply_geometry_crop(&graph, 0.0, 0.0, 0.0, 0.5, 0.0, None, "unix:3")
                .expect_err("crop width must be positive");
        let crop_x_bounds_error =
            super::apply_geometry_crop(&graph, 0.6, 0.0, 0.5, 0.5, 0.0, None, "unix:3")
                .expect_err("crop x plus width must stay in frame");
        let crop_y_bounds_error =
            super::apply_geometry_crop(&graph, 0.0, 0.7, 0.5, 0.4, 0.0, None, "unix:3")
                .expect_err("crop y plus height must stay in frame");
        let crop_angle_error =
            super::apply_geometry_crop(&graph, 0.0, 0.0, 0.5, 0.5, 46.0, None, "unix:3")
                .expect_err("crop angle above schema range");

        assert!(distortion_error.to_string().contains("lens.distortion"));
        assert!(vignetting_error.to_string().contains("lens.vignetting"));
        assert!(rotation_error.to_string().contains("geometry.rotation"));
        assert!(scale_error.to_string().contains("geometry.transform.scale"));
        assert!(offset_error
            .to_string()
            .contains("geometry.transform.x_offset"));
        assert!(crop_width_error.to_string().contains("geometry.crop.width"));
        assert!(crop_x_bounds_error.to_string().contains("geometry.crop.x"));
        assert!(crop_y_bounds_error.to_string().contains("geometry.crop.y"));
        assert!(crop_angle_error.to_string().contains("geometry.crop.angle"));
    }

    #[test]
    fn edit_clipboard_copies_selected_sections_without_identity_fields() {
        let mut source = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "source-photo".to_string(),
                path: "/tmp/source.raw".to_string(),
                file_size: 2048,
                modified_at: Some("unix:11".to_string()),
                partial_hash: Some("source-partial".to_string()),
                full_hash: Some("source-full".to_string()),
            },
            "unix:12",
        );
        source.profile.input_profile = "source-camera-profile".to_string();
        source.metadata.rating = 5;
        source.metadata.picked = true;
        source.detail.mlx_denoise = Some(json!({ "status": "source-model-payload" }));
        source.lens.profile_id = Some("source-lens-profile".to_string());
        source.extensions.insert(
            "com.example.source".to_string(),
            json!({ "owned_by": "source" }),
        );
        let source = super::apply_exposure_contrast(&source, 1.25, 18.0, "unix:13")
            .expect("apply source basic");
        let source = super::apply_tone_curve(
            &source,
            super::CurveMode::Point,
            &[(0.0, 0.0), (0.45, 0.55), (1.0, 1.0)],
            &[],
            &[],
            &[],
            "unix:14",
        )
        .expect("apply source tone");
        let source = super::apply_hsl_color_channel(
            &source,
            super::HslColorChannel::Blue,
            -10.0,
            18.0,
            -6.0,
            "unix:15",
        )
        .expect("apply source color");
        let source = super::apply_detail_sharpening(&source, 72.0, 1.4, 48.0, 20.0, "unix:16")
            .expect("apply source sharpening");
        let source =
            super::apply_detail_noise_reduction(&source, 28.0, 42.0, 14.0, 32.0, 58.0, "unix:17")
                .expect("apply source noise reduction");
        let source = super::apply_lens_adjustments(&source, true, true, -8.0, 12.0, "unix:18")
            .expect("apply source lens");
        let source = super::apply_geometry_orientation(&source, 90.0, true, false, "unix:19")
            .expect("apply source orientation");
        let source =
            super::apply_geometry_crop(&source, 0.1, 0.2, 0.7, 0.6, 0.0, Some("4:3"), "unix:20")
                .expect("apply source crop");

        let mut target = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "target-photo".to_string(),
                path: "/tmp/target.raw".to_string(),
                file_size: 4096,
                modified_at: Some("unix:21".to_string()),
                partial_hash: Some("target-partial".to_string()),
                full_hash: Some("target-full".to_string()),
            },
            "unix:22",
        );
        target.profile.input_profile = "target-camera-profile".to_string();
        target.metadata.rating = 2;
        target.metadata.rejected = true;
        target.detail.mlx_denoise = Some(json!({ "status": "target-model-payload" }));
        target.lens.profile_id = Some("target-lens-profile".to_string());
        target.extensions.insert(
            "com.example.target".to_string(),
            json!({ "owned_by": "target" }),
        );

        let payload = super::copy_edit_clipboard_payload(
            &source,
            super::EditClipboardSelection {
                basic: true,
                tone: true,
                color: true,
                detail: true,
                lens: true,
                geometry: true,
            },
        )
        .expect("copy selected edit sections");
        let serialized = serde_json::to_value(&payload).expect("serialize clipboard payload");

        assert_eq!(payload.schema, super::EDIT_CLIPBOARD_SCHEMA);
        assert_eq!(payload.version, super::EDIT_CLIPBOARD_VERSION);
        assert_eq!(payload.basic.as_ref(), Some(&source.basic));
        assert_eq!(payload.tone.as_ref(), Some(&source.tone));
        assert_eq!(payload.color.as_ref(), Some(&source.color));
        assert_eq!(
            payload.detail.as_ref().map(|detail| &detail.sharpening),
            Some(&source.detail.sharpening)
        );
        assert_eq!(
            payload
                .detail
                .as_ref()
                .map(|detail| &detail.noise_reduction),
            Some(&source.detail.noise_reduction)
        );
        assert_eq!(
            payload.lens.as_ref().map(|lens| lens.profile_correction),
            Some(source.lens.profile_correction)
        );
        assert_eq!(payload.geometry.as_ref(), Some(&source.geometry));
        assert!(serialized.get("sections").is_none());
        assert!(serialized.get("source").is_none());
        assert!(serialized.get("profile").is_none());
        assert!(serialized.get("metadata").is_none());
        assert!(serialized.get("masks").is_none());
        assert!(serialized.get("extensions").is_none());
        assert!(serialized["detail"].get("mlx_denoise").is_none());
        assert!(serialized["lens"].get("profile_id").is_none());

        let pasted = super::apply_edit_clipboard_payload(&target, &payload, "unix:30")
            .expect("paste selected edit sections");

        assert_eq!(pasted.source, target.source);
        assert_eq!(pasted.profile, target.profile);
        assert_eq!(pasted.metadata, target.metadata);
        assert_eq!(pasted.extensions, target.extensions);
        assert_eq!(pasted.masks, target.masks);
        assert_eq!(pasted.created_at, target.created_at);
        assert_eq!(pasted.app_version, target.app_version);
        assert_eq!(pasted.basic, source.basic);
        assert_eq!(pasted.tone, source.tone);
        assert_eq!(pasted.color, source.color);
        assert_eq!(pasted.detail.sharpening, source.detail.sharpening);
        assert_eq!(pasted.detail.noise_reduction, source.detail.noise_reduction);
        assert_eq!(pasted.detail.mlx_denoise, target.detail.mlx_denoise);
        assert_eq!(
            pasted.lens.profile_correction,
            source.lens.profile_correction
        );
        assert_eq!(
            pasted.lens.chromatic_aberration,
            source.lens.chromatic_aberration
        );
        assert_eq!(pasted.lens.distortion, source.lens.distortion);
        assert_eq!(pasted.lens.vignetting, source.lens.vignetting);
        assert_eq!(pasted.lens.profile_id, target.lens.profile_id);
        assert_eq!(pasted.geometry, source.geometry);
        assert_eq!(pasted.updated_at, "unix:30");
        super::validate_edit_graph(&pasted).expect("pasted graph validates");
    }

    #[test]
    fn edit_clipboard_rejects_empty_selection_and_payload() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let empty_error =
            super::copy_edit_clipboard_payload(&graph, super::EditClipboardSelection::default())
                .expect_err("empty clipboard selection is invalid");
        assert!(empty_error.to_string().contains("sections"));

        let empty_payload = super::EditClipboardPayload {
            schema: super::EDIT_CLIPBOARD_SCHEMA.to_string(),
            version: super::EDIT_CLIPBOARD_VERSION,
            basic: None,
            tone: None,
            color: None,
            detail: None,
            lens: None,
            geometry: None,
        };
        let payload_error = super::validate_edit_clipboard_payload(&empty_payload)
            .expect_err("clipboard payload must contain at least one section");
        assert!(payload_error.to_string().contains("sections"));
    }

    #[test]
    fn edit_clipboard_rejects_wrong_schema_and_version() {
        let valid_basic = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        )
        .basic;

        let wrong_schema = super::EditClipboardPayload {
            schema: "silica.edit_graph".to_string(),
            version: super::EDIT_CLIPBOARD_VERSION,
            basic: Some(valid_basic.clone()),
            tone: None,
            color: None,
            detail: None,
            lens: None,
            geometry: None,
        };
        let schema_error = super::validate_edit_clipboard_payload(&wrong_schema)
            .expect_err("clipboard payload rejects wrong schema");
        assert!(schema_error.to_string().contains("schema"));

        let wrong_version = super::EditClipboardPayload {
            schema: super::EDIT_CLIPBOARD_SCHEMA.to_string(),
            version: super::EDIT_CLIPBOARD_VERSION + 1,
            basic: Some(valid_basic),
            tone: None,
            color: None,
            detail: None,
            lens: None,
            geometry: None,
        };
        let version_error = super::validate_edit_clipboard_payload(&wrong_version)
            .expect_err("clipboard payload rejects wrong version");
        assert!(version_error.to_string().contains("version"));
    }

    #[test]
    fn edit_clipboard_rejects_unknown_json_and_invalid_adjustment_ranges() {
        let invalid_json = json!({
            "schema": "silica.edit_clipboard",
            "version": 1,
            "source": { "photo_id": "source-photo" }
        });
        let json_error = serde_json::from_value::<super::EditClipboardPayload>(invalid_json)
            .expect_err("clipboard payload rejects arbitrary source patch");
        assert!(json_error.to_string().contains("unknown field"));

        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );
        let detail_with_model_payload = json!({
            "schema": super::EDIT_CLIPBOARD_SCHEMA,
            "version": super::EDIT_CLIPBOARD_VERSION,
            "detail": {
                "sharpening": graph.detail.sharpening,
                "noise_reduction": graph.detail.noise_reduction,
                "mlx_denoise": { "status": "not-copyable" }
            }
        });
        let detail_json_error =
            serde_json::from_value::<super::EditClipboardPayload>(detail_with_model_payload)
                .expect_err("clipboard payload rejects MLX denoise data");
        assert!(detail_json_error.to_string().contains("unknown field"));

        let lens_with_profile_id = json!({
            "schema": super::EDIT_CLIPBOARD_SCHEMA,
            "version": super::EDIT_CLIPBOARD_VERSION,
            "lens": {
                "profile_correction": false,
                "profile_id": "source-specific-profile",
                "chromatic_aberration": false,
                "distortion": 0,
                "vignetting": 0
            }
        });
        let lens_json_error =
            serde_json::from_value::<super::EditClipboardPayload>(lens_with_profile_id)
                .expect_err("clipboard payload rejects lens profile identifiers");
        assert!(lens_json_error.to_string().contains("unknown field"));

        let mut invalid_basic = graph.basic.clone();
        invalid_basic.exposure = serde_json::Number::from(12);
        let invalid_payload = super::EditClipboardPayload {
            schema: super::EDIT_CLIPBOARD_SCHEMA.to_string(),
            version: super::EDIT_CLIPBOARD_VERSION,
            basic: Some(invalid_basic),
            tone: None,
            color: None,
            detail: None,
            lens: None,
            geometry: None,
        };

        let range_error = super::apply_edit_clipboard_payload(&graph, &invalid_payload, "unix:3")
            .expect_err("invalid pasted adjustment range is rejected");
        assert!(range_error.to_string().contains("basic.exposure"));
    }

    #[test]
    fn resets_p0_basic_controls_to_schema_defaults_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );
        let graph = super::apply_white_balance_temperature_tint(
            &graph,
            super::WhiteBalance::Custom,
            6400.0,
            12.0,
            "unix:3",
        )
        .expect("apply white balance");
        let graph = super::apply_tone_recovery(&graph, -20.0, 15.0, 8.0, -10.0, "unix:4")
            .expect("apply tone");
        let graph = super::apply_color_presence(&graph, 18.0, -6.0, "unix:5").expect("apply color");
        let graph =
            super::apply_exposure_contrast(&graph, 0.75, 24.0, "unix:6").expect("apply exposure");

        let reset = super::reset_p0_basic_controls(&graph, "unix:7").expect("reset P0 basic");
        let serialized = serde_json::to_value(&reset).expect("serialize reset graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip reset graph");

        assert_eq!(reset.basic.white_balance, super::WhiteBalance::AsShot);
        assert_eq!(reset.basic.temperature.as_f64(), Some(5200.0));
        assert_eq!(reset.basic.tint.as_f64(), Some(0.0));
        assert_eq!(reset.basic.exposure.as_f64(), Some(0.0));
        assert_eq!(reset.basic.contrast.as_f64(), Some(0.0));
        assert_eq!(reset.basic.highlights.as_f64(), Some(0.0));
        assert_eq!(reset.basic.shadows.as_f64(), Some(0.0));
        assert_eq!(reset.basic.whites.as_f64(), Some(0.0));
        assert_eq!(reset.basic.blacks.as_f64(), Some(0.0));
        assert_eq!(reset.basic.vibrance.as_f64(), Some(0.0));
        assert_eq!(reset.basic.saturation.as_f64(), Some(0.0));
        assert_eq!(reset.updated_at, "unix:7");
        assert_eq!(round_tripped.basic.exposure.as_f64(), Some(0.0));
        super::validate_edit_graph_json(&serialized).expect("reset graph validates");
    }

    #[test]
    fn applies_builtin_basic_presets_and_round_trips_json() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let warm = super::apply_basic_preset(&graph, super::BasicPreset::WarmContrast, "unix:3")
            .expect("apply warm contrast preset");
        let soft = super::apply_basic_preset(&graph, super::BasicPreset::SoftMatte, "unix:4")
            .expect("apply soft matte preset");
        let serialized = serde_json::to_value(&warm).expect("serialize preset graph");
        let round_tripped: super::EditGraph =
            serde_json::from_value(serialized.clone()).expect("round-trip preset graph");

        assert_eq!(warm.basic.white_balance, super::WhiteBalance::Custom);
        assert_eq!(warm.basic.temperature.as_f64(), Some(6200.0));
        assert_eq!(warm.basic.tint.as_f64(), Some(4.0));
        assert_eq!(warm.basic.exposure.as_f64(), Some(0.15));
        assert_eq!(warm.basic.contrast.as_f64(), Some(18.0));
        assert_eq!(warm.basic.highlights.as_f64(), Some(-20.0));
        assert_eq!(warm.basic.shadows.as_f64(), Some(10.0));
        assert_eq!(warm.basic.whites.as_f64(), Some(12.0));
        assert_eq!(warm.basic.blacks.as_f64(), Some(-8.0));
        assert_eq!(warm.basic.vibrance.as_f64(), Some(12.0));
        assert_eq!(warm.basic.saturation.as_f64(), Some(4.0));
        assert_eq!(soft.basic.contrast.as_f64(), Some(-18.0));
        assert_eq!(soft.basic.blacks.as_f64(), Some(18.0));
        assert_eq!(round_tripped.basic.contrast.as_f64(), Some(18.0));
        super::validate_edit_graph_json(&serialized).expect("preset graph validates");
    }

    #[test]
    fn applies_color_profile_metadata_to_schema_owned_fields() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let edited = super::apply_color_profile_metadata(
            &graph,
            super::ColorProfileMetadata::raster("srgb"),
            "unix:3",
        )
        .expect("apply raster profile metadata");
        let serialized = serde_json::to_value(&edited).expect("serialize graph");

        assert_eq!(edited.profile.input_profile, "srgb");
        assert_eq!(
            edited.profile.working_space,
            super::WORKING_SPACE_LINEAR_DISPLAY_P3
        );
        assert_eq!(
            edited.profile.decoder_backend,
            Some(super::DecoderBackend::Raster)
        );
        assert_eq!(edited.updated_at, "unix:3");
        assert_eq!(serialized["profile"]["input_profile"], json!("srgb"));
        assert_eq!(serialized["profile"]["decoder_backend"], json!("raster"));
        assert_eq!(serialized["extensions"], json!({}));
        super::validate_edit_graph_json(&serialized).expect("profile metadata validates");
    }

    #[test]
    fn rejects_out_of_range_exposure_contrast_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        assert!(super::apply_exposure_contrast(&graph, 8.0, 0.0, "unix:3").is_err());
        assert!(super::apply_exposure_contrast(&graph, 0.0, 180.0, "unix:3").is_err());
    }

    #[test]
    fn rejects_out_of_range_white_balance_temperature_tint_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let cold_error = super::apply_white_balance_temperature_tint(
            &graph,
            super::WhiteBalance::Custom,
            999.0,
            0.0,
            "unix:3",
        )
        .expect_err("temperature below schema range");
        let tint_error = super::apply_white_balance_temperature_tint(
            &graph,
            super::WhiteBalance::Custom,
            5200.0,
            151.0,
            "unix:3",
        )
        .expect_err("tint above schema range");

        assert!(cold_error.to_string().contains("basic.temperature"));
        assert!(tint_error.to_string().contains("basic.tint"));
    }

    #[test]
    fn rejects_out_of_range_tone_recovery_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let highlights_error = super::apply_tone_recovery(&graph, 101.0, 0.0, 0.0, 0.0, "unix:3")
            .expect_err("highlights above schema range");
        let blacks_error = super::apply_tone_recovery(&graph, 0.0, 0.0, 0.0, -101.0, "unix:3")
            .expect_err("blacks below schema range");

        assert!(highlights_error.to_string().contains("basic.highlights"));
        assert!(blacks_error.to_string().contains("basic.blacks"));
    }

    #[test]
    fn rejects_out_of_range_color_presence_edits() {
        let graph = super::default_edit_graph(
            super::EditGraphSource {
                photo_id: "photo-1".to_string(),
                path: "/tmp/sample.jpg".to_string(),
                file_size: 16,
                modified_at: None,
                partial_hash: None,
                full_hash: None,
            },
            "unix:2",
        );

        let vibrance_error = super::apply_color_presence(&graph, 101.0, 0.0, "unix:3")
            .expect_err("vibrance above schema range");
        let saturation_error = super::apply_color_presence(&graph, 0.0, -101.0, "unix:3")
            .expect_err("saturation below schema range");

        assert!(vibrance_error.to_string().contains("basic.vibrance"));
        assert!(saturation_error.to_string().contains("basic.saturation"));
    }
}
