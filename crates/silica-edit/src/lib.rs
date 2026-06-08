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
            input_profile: "camera_default".to_string(),
            working_space: "linear_display_p3".to_string(),
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
    for (name, curve) in [
        ("tone.rgb_curve", &tone.rgb_curve),
        ("tone.red_curve", &tone.red_curve),
        ("tone.green_curve", &tone.green_curve),
        ("tone.blue_curve", &tone.blue_curve),
    ] {
        for (index, point) in curve.iter().enumerate() {
            validate_range(format!("{name}.{index}.x"), &point.x, 0.0, 1.0)?;
            validate_range(format!("{name}.{index}.y"), &point.y, 0.0, 1.0)?;
        }
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
        super::validate_edit_graph(&graph).expect("default edit graph validates");

        let edited = super::apply_exposure_contrast(&graph, 0.75, -12.0, "unix:3")
            .expect("apply exposure and contrast");

        assert_eq!(edited.basic.exposure.as_f64(), Some(0.75));
        assert_eq!(edited.basic.contrast.as_f64(), Some(-12.0));
        assert_eq!(edited.updated_at, "unix:3");
        super::validate_edit_graph(&edited).expect("edited graph validates");
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
}
