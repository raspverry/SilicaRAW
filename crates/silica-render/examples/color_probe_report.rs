#[cfg(feature = "color-probe")]
fn main() {
    let mut args = std::env::args().skip(1);
    let source_path = match (args.next(), args.next()) {
        (Some(source_path), None) => source_path,
        _ => {
            eprintln!("usage: color_probe_report <source-path>");
            std::process::exit(2);
        }
    };

    let result = silica_render::probe_color_profile(silica_render::ColorProbeRequest::new(
        source_path.as_str(),
    ));

    println!("platform={}", color_probe_platform(result.platform));
    println!("source_path={}", result.source_path);
    println!(
        "source_sha256={}",
        result.source_sha256.as_deref().unwrap_or("")
    );
    println!("status={}", color_probe_status(result.status));
    println!(
        "input_profile={}",
        color_probe_input_profile(result.input_profile)
    );
    println!("embedded_icc={}", result.embedded_icc);
    println!(
        "working_space={}",
        working_color_space(result.working_space)
    );
    println!(
        "output_profile={}",
        color_probe_output_profile(result.output_profile)
    );
    println!(
        "transform_path={}",
        color_probe_transform_path(result.transform_path)
    );
    println!(
        "error_category={}",
        result
            .error_category
            .map(color_probe_error_category)
            .unwrap_or("none")
    );
    println!("message={}", result.message);
}

#[cfg(not(feature = "color-probe"))]
fn main() {
    eprintln!("color_probe_report requires the color-probe feature");
    std::process::exit(2);
}

#[cfg(feature = "color-probe")]
fn color_probe_platform(platform: silica_render::ColorProbePlatform) -> &'static str {
    match platform {
        silica_render::ColorProbePlatform::Macos => "macos",
        silica_render::ColorProbePlatform::UnsupportedPlatform => "unsupported_platform",
    }
}

#[cfg(feature = "color-probe")]
fn color_probe_status(status: silica_render::ColorProbeStatus) -> &'static str {
    match status {
        silica_render::ColorProbeStatus::Success => "success",
        silica_render::ColorProbeStatus::Failed => "failed",
    }
}

#[cfg(feature = "color-probe")]
fn color_probe_input_profile(profile: silica_render::ColorProbeInputProfile) -> &'static str {
    match profile {
        silica_render::ColorProbeInputProfile::Srgb => "srgb",
        silica_render::ColorProbeInputProfile::DisplayP3 => "display_p3",
        silica_render::ColorProbeInputProfile::None => "none",
        silica_render::ColorProbeInputProfile::Unknown => "unknown",
    }
}

#[cfg(feature = "color-probe")]
fn color_probe_output_profile(profile: silica_render::ColorProbeOutputProfile) -> &'static str {
    match profile {
        silica_render::ColorProbeOutputProfile::Srgb => "srgb",
    }
}

#[cfg(feature = "color-probe")]
fn color_probe_transform_path(path: silica_render::ColorProbeTransformPath) -> &'static str {
    match path {
        silica_render::ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb => {
            "embedded_icc_to_linear_display_p3_to_srgb"
        }
        silica_render::ColorProbeTransformPath::AssumeSrgbToLinearDisplayP3ToSrgb => {
            "assume_srgb_to_linear_display_p3_to_srgb"
        }
        silica_render::ColorProbeTransformPath::Unavailable => "unavailable",
    }
}

#[cfg(feature = "color-probe")]
fn color_probe_error_category(category: silica_render::ColorProbeErrorCategory) -> &'static str {
    match category {
        silica_render::ColorProbeErrorCategory::UnsupportedPlatform => "unsupported_platform",
        silica_render::ColorProbeErrorCategory::MissingFile => "missing_file",
        silica_render::ColorProbeErrorCategory::NotAFile => "not_a_file",
        silica_render::ColorProbeErrorCategory::ReadFailed => "read_failed",
        silica_render::ColorProbeErrorCategory::InvalidJpeg => "invalid_jpeg",
    }
}

#[cfg(feature = "color-probe")]
fn working_color_space(space: silica_render::WorkingColorSpace) -> &'static str {
    match space {
        silica_render::WorkingColorSpace::LinearDisplayP3 => "linear_display_p3",
    }
}
