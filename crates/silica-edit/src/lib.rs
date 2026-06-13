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
    for (key, value) in &mask.local_adjustments {
        validate_number(format!("{prefix}.local_adjustments.{key}"), value)?;
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
