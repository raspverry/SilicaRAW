//! Product native Metal viewer module shell.
//!
//! This is not the Spike 001 proof module. Task 14.2 established the
//! feature-gated product boundary, Task 14.3 established reserved host geometry,
//! and Task 14.4 adds neutral lifecycle proof state. Later Phase 14 tasks add
//! input, render request, texture, and QA behavior.

use std::error::Error;
use std::fmt;
use std::time::Duration;

/// Compile-time contract for the product native viewer shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMetalViewerModuleContract {
    pub module_name: &'static str,
    pub feature_name: &'static str,
    pub phase_task: &'static str,
    pub product_module: bool,
    pub uses_spike_module: bool,
    pub installs_in_default_build: bool,
    pub reserved_surfaces: [&'static str; 2],
    pub consumes_web_host_geometry: bool,
    pub controls_must_be_external: bool,
}

/// Returns the current product viewer module contract.
pub fn module_contract() -> NativeMetalViewerModuleContract {
    NativeMetalViewerModuleContract {
        module_name: "native_metal_viewer",
        feature_name: "native-metal-viewer",
        phase_task: "14.2",
        product_module: true,
        uses_spike_module: false,
        installs_in_default_build: false,
        reserved_surfaces: ["loupe", "develop"],
        consumes_web_host_geometry: true,
        controls_must_be_external: true,
    }
}

/// Drawable pixel dimensions derived from a logical viewer host and backing scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawableSize {
    pub width_px: u32,
    pub height_px: u32,
}

/// Logical viewer host geometry reported by the web shell.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeViewerHostGeometry {
    surface: String,
    width_points: f64,
    height_points: f64,
    backing_scale_factor: f64,
}

impl NativeViewerHostGeometry {
    pub fn new(
        surface: impl Into<String>,
        width_points: f64,
        height_points: f64,
        backing_scale_factor: f64,
    ) -> Result<Self, NativeViewerLifecycleError> {
        let surface = surface.into();
        if surface != "loupe" && surface != "develop" {
            return Err(NativeViewerLifecycleError::new(format!(
                "unsupported native viewer surface: {surface}"
            )));
        }
        if !width_points.is_finite() || width_points <= 0.0 {
            return Err(NativeViewerLifecycleError::new(format!(
                "invalid native viewer width: {width_points}"
            )));
        }
        if !height_points.is_finite() || height_points <= 0.0 {
            return Err(NativeViewerLifecycleError::new(format!(
                "invalid native viewer height: {height_points}"
            )));
        }
        if !backing_scale_factor.is_finite() || backing_scale_factor <= 0.0 {
            return Err(NativeViewerLifecycleError::new(format!(
                "invalid native viewer backing scale: {backing_scale_factor}"
            )));
        }

        Ok(Self {
            surface,
            width_points,
            height_points,
            backing_scale_factor,
        })
    }

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn backing_scale_factor(&self) -> f64 {
        self.backing_scale_factor
    }

    pub fn drawable_size(&self) -> DrawableSize {
        DrawableSize {
            width_px: round_pixels(self.width_points * self.backing_scale_factor),
            height_px: round_pixels(self.height_points * self.backing_scale_factor),
        }
    }
}

/// Lifecycle state for the product native viewer proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeViewerLifecycleState {
    Installed,
    Uninstalled,
}

impl NativeViewerLifecycleState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Uninstalled => "uninstalled",
        }
    }
}

/// Reason the product native viewer proof released its host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeViewerCleanupReason {
    AppClosed,
    WindowClosed,
}

impl NativeViewerCleanupReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::AppClosed => "app-closed",
            Self::WindowClosed => "window-closed",
        }
    }
}

/// Observable render timing for neutral proof frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeViewerFrameTiming {
    pub frames_drawn: u64,
    pub elapsed_ms: f64,
}

/// Feature-gated lifecycle proof state for the product native viewer bridge.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeViewerLifecycleProof {
    host: NativeViewerHostGeometry,
    state: NativeViewerLifecycleState,
    preferred_frames_per_second: u16,
    resize_events: u64,
    frames_drawn: u64,
    last_frame_elapsed_ms: Option<f64>,
    cleanup_reason: Option<NativeViewerCleanupReason>,
    neutral_clear_only: bool,
}

impl NativeViewerLifecycleProof {
    pub fn install(
        host: NativeViewerHostGeometry,
        preferred_frames_per_second: u16,
    ) -> Result<Self, NativeViewerLifecycleError> {
        if preferred_frames_per_second == 0 {
            return Err(NativeViewerLifecycleError::new(
                "preferred frame rate must be greater than zero",
            ));
        }

        Ok(Self {
            host,
            state: NativeViewerLifecycleState::Installed,
            preferred_frames_per_second,
            resize_events: 0,
            frames_drawn: 0,
            last_frame_elapsed_ms: None,
            cleanup_reason: None,
            neutral_clear_only: true,
        })
    }

    pub fn record_resize(
        &mut self,
        host: NativeViewerHostGeometry,
    ) -> Result<(), NativeViewerLifecycleError> {
        self.ensure_installed("record resize")?;
        if host.surface() != self.host.surface() {
            return Err(NativeViewerLifecycleError::new(format!(
                "cannot resize surface {} with geometry for {}",
                self.host.surface(),
                host.surface()
            )));
        }

        self.host = host;
        self.resize_events += 1;
        Ok(())
    }

    pub fn record_render_frame(
        &mut self,
        elapsed: Duration,
    ) -> Result<NativeViewerFrameTiming, NativeViewerLifecycleError> {
        self.ensure_installed("record render frame")?;
        if elapsed.is_zero() {
            return Err(NativeViewerLifecycleError::new(
                "render frame timing must be observable",
            ));
        }

        self.frames_drawn += 1;
        let timing = NativeViewerFrameTiming {
            frames_drawn: self.frames_drawn,
            elapsed_ms: elapsed.as_secs_f64() * 1000.0,
        };
        self.last_frame_elapsed_ms = Some(timing.elapsed_ms);
        Ok(timing)
    }

    pub fn uninstall(
        &mut self,
        reason: NativeViewerCleanupReason,
    ) -> Result<(), NativeViewerLifecycleError> {
        self.ensure_installed("uninstall")?;
        self.state = NativeViewerLifecycleState::Uninstalled;
        self.cleanup_reason = Some(reason);
        Ok(())
    }

    pub fn state(&self) -> NativeViewerLifecycleState {
        self.state
    }

    pub fn host_surface(&self) -> &str {
        self.host.surface()
    }

    pub fn current_drawable_size(&self) -> DrawableSize {
        self.host.drawable_size()
    }

    pub fn backing_scale_factor(&self) -> f64 {
        self.host.backing_scale_factor()
    }

    pub fn resize_event_count(&self) -> u64 {
        self.resize_events
    }

    pub fn render_timing_observable(&self) -> bool {
        self.frames_drawn > 0 && self.last_frame_elapsed_ms.is_some()
    }

    pub fn cleanup_reason(&self) -> Option<NativeViewerCleanupReason> {
        self.cleanup_reason
    }

    pub fn neutral_clear_only(&self) -> bool {
        self.neutral_clear_only
    }

    pub fn evidence_summary(&self) -> String {
        let drawable = self.current_drawable_size();
        let cleanup = self
            .cleanup_reason
            .map(NativeViewerCleanupReason::as_str)
            .unwrap_or("none");
        let last_frame_ms = self
            .last_frame_elapsed_ms
            .map(|elapsed| format!("{elapsed:.3}"))
            .unwrap_or_else(|| "none".to_string());

        format!(
            "surface={} state={} drawable={}x{}px backing_scale={:.2} resize_events={} frames={} last_frame_ms={} preferred_fps={} cleanup={} neutral_clear_only={}",
            self.host_surface(),
            self.state.as_str(),
            drawable.width_px,
            drawable.height_px,
            self.backing_scale_factor(),
            self.resize_events,
            self.frames_drawn,
            last_frame_ms,
            self.preferred_frames_per_second,
            cleanup,
            self.neutral_clear_only
        )
    }

    fn ensure_installed(&self, action: &'static str) -> Result<(), NativeViewerLifecycleError> {
        if self.state == NativeViewerLifecycleState::Installed {
            Ok(())
        } else {
            Err(NativeViewerLifecycleError::new(format!(
                "cannot {action} after native viewer uninstall"
            )))
        }
    }
}

/// Error returned by the feature-gated native viewer proof recorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeViewerLifecycleError {
    message: String,
}

impl NativeViewerLifecycleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for NativeViewerLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for NativeViewerLifecycleError {}

fn round_pixels(value: f64) -> u32 {
    value.round().clamp(1.0, u32::MAX as f64) as u32
}

/// Returns neutral, reviewable lifecycle evidence for feature-gated manual smoke runs.
pub fn lifecycle_smoke_evidence() -> Result<String, NativeViewerLifecycleError> {
    let host = NativeViewerHostGeometry::new("develop", 960.0, 540.0, 2.0)?;
    let mut proof = NativeViewerLifecycleProof::install(host, 60)?;
    let resized = NativeViewerHostGeometry::new("develop", 800.25, 450.25, 1.5)?;

    proof.record_resize(resized)?;
    let timing = proof.record_render_frame(Duration::from_micros(16_667))?;
    proof.uninstall(NativeViewerCleanupReason::WindowClosed)?;

    debug_assert_eq!(proof.state(), NativeViewerLifecycleState::Uninstalled);
    debug_assert_eq!(proof.resize_event_count(), 1);
    debug_assert!(proof.render_timing_observable());
    debug_assert_eq!(
        proof.cleanup_reason(),
        Some(NativeViewerCleanupReason::WindowClosed)
    );
    debug_assert!(proof.neutral_clear_only());

    Ok(format!(
        "{} frame_timing_ms={:.3} cleanup_supported={}",
        proof.evidence_summary(),
        timing.elapsed_ms,
        cleanup_reason_labels().join(",")
    ))
}

fn cleanup_reason_labels() -> [&'static str; 2] {
    [
        NativeViewerCleanupReason::AppClosed.as_str(),
        NativeViewerCleanupReason::WindowClosed.as_str(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        DrawableSize, NativeViewerCleanupReason, NativeViewerHostGeometry,
        NativeViewerLifecycleProof, NativeViewerLifecycleState,
    };
    use std::time::Duration;

    #[test]
    fn lifecycle_proof_records_resize_retina_render_timing_and_cleanup() {
        let host = NativeViewerHostGeometry::new("develop", 960.0, 540.0, 2.0).unwrap();
        let mut proof = NativeViewerLifecycleProof::install(host, 60).unwrap();

        assert_eq!(proof.state(), NativeViewerLifecycleState::Installed);
        assert_eq!(proof.host_surface(), "develop");
        assert_eq!(
            proof.current_drawable_size(),
            DrawableSize {
                width_px: 1920,
                height_px: 1080
            }
        );
        assert_eq!(proof.backing_scale_factor(), 2.0);
        assert!(proof.neutral_clear_only());

        let resized = NativeViewerHostGeometry::new("develop", 800.25, 450.25, 1.5).unwrap();
        proof.record_resize(resized).unwrap();

        assert_eq!(proof.resize_event_count(), 1);
        assert_eq!(proof.backing_scale_factor(), 1.5);
        assert_eq!(
            proof.current_drawable_size(),
            DrawableSize {
                width_px: 1200,
                height_px: 675
            }
        );

        let timing = proof
            .record_render_frame(Duration::from_micros(16_667))
            .unwrap();
        assert_eq!(timing.frames_drawn, 1);
        assert!((timing.elapsed_ms - 16.667).abs() < 0.001);
        assert!(proof.render_timing_observable());

        proof
            .uninstall(NativeViewerCleanupReason::WindowClosed)
            .unwrap();
        assert_eq!(proof.state(), NativeViewerLifecycleState::Uninstalled);
        assert_eq!(
            proof.cleanup_reason(),
            Some(NativeViewerCleanupReason::WindowClosed)
        );
        let evidence = proof.evidence_summary();
        assert!(evidence.contains("surface=develop"));
        assert!(evidence.contains("state=uninstalled"));
        assert!(evidence.contains("drawable=1200x675px"));
        assert!(evidence.contains("backing_scale=1.50"));
        assert!(evidence.contains("frames=1"));
        assert!(evidence.contains("neutral_clear_only=true"));
    }

    #[test]
    fn lifecycle_smoke_evidence_is_neutral_and_reviewable() {
        let evidence = super::lifecycle_smoke_evidence().unwrap();
        println!("[SilicaRAW Native Viewer] {evidence}");

        assert!(evidence.contains("surface=develop"));
        assert!(evidence.contains("state=uninstalled"));
        assert!(evidence.contains("drawable=1200x675px"));
        assert!(evidence.contains("backing_scale=1.50"));
        assert!(evidence.contains("frames=1"));
        assert!(evidence.contains("cleanup_supported=app-closed,window-closed"));
        assert!(evidence.contains("neutral_clear_only=true"));
    }
}
