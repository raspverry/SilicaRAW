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

/// Bounds of the reserved viewer input surface in AppKit point space.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeViewerInputBounds {
    surface: String,
    x_points: f64,
    y_points: f64,
    width_points: f64,
    height_points: f64,
}

impl NativeViewerInputBounds {
    pub fn new(
        surface: impl Into<String>,
        x_points: f64,
        y_points: f64,
        width_points: f64,
        height_points: f64,
    ) -> Result<Self, NativeViewerLifecycleError> {
        let surface = surface.into();
        if surface != "loupe" && surface != "develop" {
            return Err(NativeViewerLifecycleError::new(format!(
                "unsupported native viewer input surface: {surface}"
            )));
        }
        for (name, value) in [
            ("x", x_points),
            ("y", y_points),
            ("width", width_points),
            ("height", height_points),
        ] {
            if !value.is_finite() {
                return Err(NativeViewerLifecycleError::new(format!(
                    "invalid native viewer input {name}: {value}"
                )));
            }
        }
        if width_points <= 0.0 || height_points <= 0.0 {
            return Err(NativeViewerLifecycleError::new(
                "native viewer input bounds must have positive size",
            ));
        }

        Ok(Self {
            surface,
            x_points,
            y_points,
            width_points,
            height_points,
        })
    }

    fn surface(&self) -> &str {
        &self.surface
    }

    fn contains(&self, event: &NativeViewerInputEvent) -> bool {
        event.x_points >= self.x_points
            && event.y_points >= self.y_points
            && event.x_points <= self.x_points + self.width_points
            && event.y_points <= self.y_points + self.height_points
    }
}

/// Input event kinds covered by the Phase 14.5 proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeViewerInputKind {
    MouseDown,
    MouseDrag,
    Scroll,
    Magnify,
}

/// Ownership decision for one input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeViewerInputOwnership {
    NativeViewer,
    WebUi,
}

/// Input event sample for feature-gated ownership proof.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeViewerInputEvent {
    kind: NativeViewerInputKind,
    x_points: f64,
    y_points: f64,
    delta_x_points: f64,
    delta_y_points: f64,
    magnification: f64,
}

impl NativeViewerInputEvent {
    pub fn mouse_down(x_points: f64, y_points: f64) -> Self {
        Self {
            kind: NativeViewerInputKind::MouseDown,
            x_points,
            y_points,
            delta_x_points: 0.0,
            delta_y_points: 0.0,
            magnification: 0.0,
        }
    }

    pub fn mouse_drag(
        x_points: f64,
        y_points: f64,
        delta_x_points: f64,
        delta_y_points: f64,
    ) -> Self {
        Self {
            kind: NativeViewerInputKind::MouseDrag,
            x_points,
            y_points,
            delta_x_points,
            delta_y_points,
            magnification: 0.0,
        }
    }

    pub fn scroll(x_points: f64, y_points: f64, delta_x_points: f64, delta_y_points: f64) -> Self {
        Self {
            kind: NativeViewerInputKind::Scroll,
            x_points,
            y_points,
            delta_x_points,
            delta_y_points,
            magnification: 0.0,
        }
    }

    pub fn magnify(x_points: f64, y_points: f64, magnification: f64) -> Self {
        Self {
            kind: NativeViewerInputKind::Magnify,
            x_points,
            y_points,
            delta_x_points: 0.0,
            delta_y_points: 0.0,
            magnification,
        }
    }

    fn validate(&self) -> Result<(), NativeViewerLifecycleError> {
        for (name, value) in [
            ("x", self.x_points),
            ("y", self.y_points),
            ("delta_x", self.delta_x_points),
            ("delta_y", self.delta_y_points),
            ("magnification", self.magnification),
        ] {
            if !value.is_finite() {
                return Err(NativeViewerLifecycleError::new(format!(
                    "invalid native viewer input {name}: {value}"
                )));
            }
        }
        Ok(())
    }
}

/// Ownership record returned by the input proof recorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeViewerInputRecord {
    pub ownership: NativeViewerInputOwnership,
}

/// Feature-gated input ownership proof for the product native viewer bridge.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeViewerInputProof {
    bounds: NativeViewerInputBounds,
    native_events: u64,
    web_events: u64,
    mouse_down_seen: bool,
    mouse_drag_seen: bool,
    scroll_seen: bool,
    magnify_seen: bool,
    web_controls_external: bool,
    remote_reporting_enabled: bool,
    persistent_input_log_enabled: bool,
}

impl NativeViewerInputProof {
    pub fn new(bounds: NativeViewerInputBounds) -> Self {
        Self {
            bounds,
            native_events: 0,
            web_events: 0,
            mouse_down_seen: false,
            mouse_drag_seen: false,
            scroll_seen: false,
            magnify_seen: false,
            web_controls_external: true,
            remote_reporting_enabled: false,
            persistent_input_log_enabled: false,
        }
    }

    pub fn record_event(
        &mut self,
        event: NativeViewerInputEvent,
    ) -> Result<NativeViewerInputRecord, NativeViewerLifecycleError> {
        event.validate()?;
        let ownership = if self.bounds.contains(&event) {
            self.native_events += 1;
            match event.kind {
                NativeViewerInputKind::MouseDown => self.mouse_down_seen = true,
                NativeViewerInputKind::MouseDrag => self.mouse_drag_seen = true,
                NativeViewerInputKind::Scroll => self.scroll_seen = true,
                NativeViewerInputKind::Magnify => self.magnify_seen = true,
            }
            NativeViewerInputOwnership::NativeViewer
        } else {
            self.web_events += 1;
            NativeViewerInputOwnership::WebUi
        };

        Ok(NativeViewerInputRecord { ownership })
    }

    pub fn native_event_count(&self) -> u64 {
        self.native_events
    }

    pub fn web_event_count(&self) -> u64 {
        self.web_events
    }

    pub fn web_controls_remain_external(&self) -> bool {
        self.web_controls_external
    }

    pub fn remote_reporting_enabled(&self) -> bool {
        self.remote_reporting_enabled
    }

    pub fn persistent_input_log_enabled(&self) -> bool {
        self.persistent_input_log_enabled
    }

    pub fn evidence_summary(&self) -> String {
        format!(
            "surface={} native_events={} web_events={} mouse_down={} mouse_drag={} scroll={} magnify={} web_controls_external={} remote_reporting={} persistent_input_log={}",
            self.bounds.surface(),
            self.native_events,
            self.web_events,
            self.mouse_down_seen,
            self.mouse_drag_seen,
            self.scroll_seen,
            self.magnify_seen,
            self.web_controls_external,
            self.remote_reporting_enabled,
            self.persistent_input_log_enabled
        )
    }
}

/// Returns neutral, reviewable input ownership evidence for manual smoke runs.
pub fn input_smoke_evidence() -> Result<String, NativeViewerLifecycleError> {
    let bounds = NativeViewerInputBounds::new("develop", 100.0, 80.0, 800.0, 450.0)?;
    let mut proof = NativeViewerInputProof::new(bounds);

    proof.record_event(NativeViewerInputEvent::mouse_down(120.0, 96.0))?;
    proof.record_event(NativeViewerInputEvent::mouse_drag(200.0, 180.0, 8.0, -2.0))?;
    proof.record_event(NativeViewerInputEvent::scroll(320.0, 240.0, 0.0, -12.5))?;
    proof.record_event(NativeViewerInputEvent::magnify(420.0, 280.0, 0.08))?;
    proof.record_event(NativeViewerInputEvent::mouse_down(24.0, 36.0))?;

    debug_assert_eq!(proof.native_event_count(), 4);
    debug_assert_eq!(proof.web_event_count(), 1);
    debug_assert!(proof.web_controls_remain_external());
    debug_assert!(!proof.remote_reporting_enabled());
    debug_assert!(!proof.persistent_input_log_enabled());

    Ok(proof.evidence_summary())
}

/// Feature-gated bridge between typed render requests and the native viewer scheduler.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeViewerRenderBridge {
    scheduler: silica_render::ViewerPreviewRenderScheduler,
    catalog_write_requested: bool,
}

impl NativeViewerRenderBridge {
    pub fn schedule_render_request(
        &mut self,
        request: silica_render::ViewerPreviewRenderRequest,
    ) -> silica_render::ViewerPreviewScheduleResult {
        debug_assert!(!request.writes_catalog_state());
        self.catalog_write_requested = false;
        self.scheduler.schedule(request)
    }

    pub fn latest_request_id(&self) -> Option<silica_render::ViewerPreviewRenderRequestId> {
        self.scheduler.latest_request_id()
    }

    pub fn latest_request(&self) -> Option<&silica_render::ViewerPreviewRenderRequest> {
        self.scheduler.latest_request()
    }

    pub fn catalog_write_requested(&self) -> bool {
        self.catalog_write_requested
    }
}

/// Returns neutral, reviewable render-request boundary evidence for manual smoke runs.
pub fn render_request_smoke_evidence() -> String {
    let first = silica_render::ViewerPreviewRenderRequest::new(
        silica_render::ViewerPreviewRenderRequestId(11),
        "photo-11",
        "/tmp/source.raw",
        silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
        silica_render::ViewerPreviewInput::no_pixels_yet(silica_render::PreviewRenderStatus::Ready),
        1,
    );
    let second = silica_render::ViewerPreviewRenderRequest::new(
        silica_render::ViewerPreviewRenderRequestId(12),
        "photo-11",
        "/tmp/source.raw",
        silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
        silica_render::ViewerPreviewInput::future_texture(
            "decode-cache/photo-11/request-12",
            4032,
            3024,
            silica_render::ViewerPreviewPixelFormat::Bgra8Unorm,
        ),
        2,
    );
    let mut bridge = NativeViewerRenderBridge::default();
    let _first_result = bridge.schedule_render_request(first);
    let second_result = bridge.schedule_render_request(second);
    let latest_request = bridge
        .latest_request()
        .expect("render request smoke must schedule a latest request");
    let latest_request_id = bridge
        .latest_request_id()
        .expect("render request smoke must record latest request");
    let latest_wins = latest_request_id == second_result.accepted_request_id;
    let replaced_request_id = second_result
        .replaced_request_id
        .map(|request_id| request_id.0)
        .unwrap_or_default();
    let future_texture_identity = matches!(
        latest_request.input,
        silica_render::ViewerPreviewInput::FutureTexture { .. }
    );

    format!(
        "latest_request={} replaced_request={} latest_wins={} catalog_write_requested={} contains_image_pixels={} future_texture_identity={}",
        latest_request_id.0,
        replaced_request_id,
        latest_wins,
        bridge.catalog_write_requested(),
        latest_request.contains_image_pixels(),
        future_texture_identity
    )
}

/// Feature-gated disposable texture lifecycle boundary for the native viewer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeViewerTextureBoundary {
    lifecycle: silica_render::ViewerTextureLifecycle,
}

impl NativeViewerTextureBoundary {
    pub fn bind_disposable_texture(&mut self, identity: silica_render::ViewerTextureIdentity) {
        self.lifecycle.bind_texture(identity);
    }

    pub fn cleanup_for_photo_change(&mut self) {
        self.lifecycle
            .release(silica_render::ViewerTextureReleaseReason::PhotoChanged);
    }

    pub fn cleanup_for_library_close(&mut self) {
        self.lifecycle
            .release(silica_render::ViewerTextureReleaseReason::LibraryClosed);
    }

    pub fn cleanup_for_app_close(&mut self) {
        self.lifecycle
            .release(silica_render::ViewerTextureReleaseReason::AppClosed);
    }

    pub fn cleanup_for_drawable_resize(&mut self) {
        self.lifecycle
            .release(silica_render::ViewerTextureReleaseReason::DrawableResized);
    }

    pub fn state(&self) -> silica_render::ViewerTextureLifecycleState {
        self.lifecycle.state()
    }

    pub fn current_texture(&self) -> Option<&silica_render::ViewerTextureIdentity> {
        self.lifecycle.current_texture()
    }

    pub fn last_release_reason(&self) -> Option<silica_render::ViewerTextureReleaseReason> {
        self.lifecycle.last_release_reason()
    }

    pub fn release_count(&self) -> u64 {
        self.lifecycle.release_count()
    }

    pub fn writes_catalog_state(&self) -> bool {
        self.lifecycle.writes_catalog_state()
    }

    pub fn writes_sidecar_state(&self) -> bool {
        self.lifecycle.writes_sidecar_state()
    }

    pub fn uses_original_path_as_write_destination(&self) -> bool {
        self.lifecycle.uses_original_path_as_write_destination()
    }

    pub fn persistent_gpu_cache_enabled(&self) -> bool {
        self.lifecycle.persistent_gpu_cache_enabled()
    }

    pub fn is_rebuildable(&self) -> bool {
        self.lifecycle.is_rebuildable()
    }
}

/// Returns neutral, reviewable texture lifecycle evidence for manual smoke runs.
pub fn texture_lifecycle_smoke_evidence() -> String {
    let mut boundary = NativeViewerTextureBoundary::default();
    boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
        silica_render::ViewerPreviewRenderRequestId(21),
        "texture/request-21",
        silica_render::ViewerTextureDrawableSize::new(1200, 675),
    ));
    boundary.cleanup_for_photo_change();
    boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
        silica_render::ViewerPreviewRenderRequestId(22),
        "texture/request-22",
        silica_render::ViewerTextureDrawableSize::new(1600, 900),
    ));
    boundary.cleanup_for_drawable_resize();
    boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
        silica_render::ViewerPreviewRenderRequestId(23),
        "texture/request-23",
        silica_render::ViewerTextureDrawableSize::new(1600, 900),
    ));
    boundary.cleanup_for_library_close();
    boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
        silica_render::ViewerPreviewRenderRequestId(24),
        "texture/request-24",
        silica_render::ViewerTextureDrawableSize::new(1600, 900),
    ));
    boundary.cleanup_for_app_close();

    debug_assert_eq!(boundary.release_count(), 4);
    debug_assert_eq!(boundary.current_texture(), None);
    debug_assert!(!boundary.writes_catalog_state());
    debug_assert!(!boundary.writes_sidecar_state());
    debug_assert!(!boundary.uses_original_path_as_write_destination());
    debug_assert!(!boundary.persistent_gpu_cache_enabled());
    debug_assert!(boundary.is_rebuildable());

    format!(
        "state={} release_count={} last_release={} catalog_write={} sidecar_write={} original_write_destination={} persistent_gpu_cache={} rebuildable={}",
        texture_state_label(boundary.state()),
        boundary.release_count(),
        texture_release_reason_label(boundary.last_release_reason()),
        boundary.writes_catalog_state(),
        boundary.writes_sidecar_state(),
        boundary.uses_original_path_as_write_destination(),
        boundary.persistent_gpu_cache_enabled(),
        boundary.is_rebuildable()
    )
}

fn texture_state_label(state: silica_render::ViewerTextureLifecycleState) -> &'static str {
    match state {
        silica_render::ViewerTextureLifecycleState::Empty => "empty",
        silica_render::ViewerTextureLifecycleState::Bound => "bound",
        silica_render::ViewerTextureLifecycleState::Released => "released",
    }
}

fn texture_release_reason_label(
    reason: Option<silica_render::ViewerTextureReleaseReason>,
) -> &'static str {
    match reason {
        Some(silica_render::ViewerTextureReleaseReason::PhotoChanged) => "photo-changed",
        Some(silica_render::ViewerTextureReleaseReason::LibraryClosed) => "library-closed",
        Some(silica_render::ViewerTextureReleaseReason::AppClosed) => "app-closed",
        Some(silica_render::ViewerTextureReleaseReason::DrawableResized) => "drawable-resized",
        None => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DrawableSize, NativeViewerCleanupReason, NativeViewerHostGeometry, NativeViewerInputBounds,
        NativeViewerInputEvent, NativeViewerInputOwnership, NativeViewerInputProof,
        NativeViewerLifecycleProof, NativeViewerLifecycleState, NativeViewerRenderBridge,
        NativeViewerTextureBoundary,
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

    #[test]
    fn input_ownership_proof_records_viewer_events_without_claiming_web_controls() {
        let bounds = NativeViewerInputBounds::new("develop", 100.0, 80.0, 800.0, 450.0).unwrap();
        let mut proof = NativeViewerInputProof::new(bounds);

        assert_eq!(
            proof
                .record_event(NativeViewerInputEvent::mouse_down(120.0, 96.0))
                .unwrap()
                .ownership,
            NativeViewerInputOwnership::NativeViewer
        );
        assert_eq!(
            proof
                .record_event(NativeViewerInputEvent::mouse_drag(200.0, 180.0, 8.0, -2.0))
                .unwrap()
                .ownership,
            NativeViewerInputOwnership::NativeViewer
        );
        assert_eq!(
            proof
                .record_event(NativeViewerInputEvent::scroll(320.0, 240.0, 0.0, -12.5))
                .unwrap()
                .ownership,
            NativeViewerInputOwnership::NativeViewer
        );
        assert_eq!(
            proof
                .record_event(NativeViewerInputEvent::magnify(420.0, 280.0, 0.08))
                .unwrap()
                .ownership,
            NativeViewerInputOwnership::NativeViewer
        );
        assert_eq!(
            proof
                .record_event(NativeViewerInputEvent::mouse_down(24.0, 36.0))
                .unwrap()
                .ownership,
            NativeViewerInputOwnership::WebUi
        );

        assert_eq!(proof.native_event_count(), 4);
        assert_eq!(proof.web_event_count(), 1);
        assert!(proof.web_controls_remain_external());
        assert!(!proof.remote_reporting_enabled());
        assert!(!proof.persistent_input_log_enabled());

        let evidence = proof.evidence_summary();
        assert!(evidence.contains("surface=develop"));
        assert!(evidence.contains("native_events=4"));
        assert!(evidence.contains("web_events=1"));
        assert!(evidence.contains("mouse_down=true"));
        assert!(evidence.contains("mouse_drag=true"));
        assert!(evidence.contains("scroll=true"));
        assert!(evidence.contains("magnify=true"));
        assert!(evidence.contains("web_controls_external=true"));
        assert!(evidence.contains("remote_reporting=false"));
        assert!(evidence.contains("persistent_input_log=false"));
    }

    #[test]
    fn input_smoke_evidence_is_manual_review_ready() {
        let evidence = super::input_smoke_evidence().unwrap();
        println!("[SilicaRAW Native Viewer] {evidence}");

        assert!(evidence.contains("surface=develop"));
        assert!(evidence.contains("native_events=4"));
        assert!(evidence.contains("web_events=1"));
        assert!(evidence.contains("mouse_down=true"));
        assert!(evidence.contains("mouse_drag=true"));
        assert!(evidence.contains("scroll=true"));
        assert!(evidence.contains("magnify=true"));
        assert!(evidence.contains("web_controls_external=true"));
        assert!(evidence.contains("remote_reporting=false"));
        assert!(evidence.contains("persistent_input_log=false"));
    }

    #[test]
    fn render_bridge_schedules_latest_request_without_catalog_writes() {
        let first = silica_render::ViewerPreviewRenderRequest::new(
            silica_render::ViewerPreviewRenderRequestId(11),
            "photo-11",
            "/tmp/source.raw",
            silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
            silica_render::ViewerPreviewInput::no_pixels_yet(
                silica_render::PreviewRenderStatus::Ready,
            ),
            1,
        );
        let second = silica_render::ViewerPreviewRenderRequest::new(
            silica_render::ViewerPreviewRenderRequestId(12),
            "photo-11",
            "/tmp/source.raw",
            silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
            silica_render::ViewerPreviewInput::future_texture(
                "decode-cache/photo-11/request-12",
                4032,
                3024,
                silica_render::ViewerPreviewPixelFormat::Bgra8Unorm,
            ),
            2,
        );
        let mut bridge = NativeViewerRenderBridge::default();

        let first_result = bridge.schedule_render_request(first);
        assert_eq!(
            first_result.accepted_request_id,
            silica_render::ViewerPreviewRenderRequestId(11)
        );
        assert_eq!(first_result.replaced_request_id, None);

        let second_result = bridge.schedule_render_request(second);
        assert_eq!(
            second_result.accepted_request_id,
            silica_render::ViewerPreviewRenderRequestId(12)
        );
        assert_eq!(
            second_result.replaced_request_id,
            Some(silica_render::ViewerPreviewRenderRequestId(11))
        );
        assert_eq!(
            bridge.latest_request_id(),
            Some(silica_render::ViewerPreviewRenderRequestId(12))
        );
        assert!(!bridge.catalog_write_requested());
        assert!(!bridge.latest_request().unwrap().contains_image_pixels());
    }

    #[test]
    fn render_request_smoke_evidence_is_reviewable() {
        let evidence = super::render_request_smoke_evidence();
        println!("[SilicaRAW Native Viewer] {evidence}");

        assert!(evidence.contains("latest_request=12"));
        assert!(evidence.contains("replaced_request=11"));
        assert!(evidence.contains("latest_wins=true"));
        assert!(evidence.contains("catalog_write_requested=false"));
        assert!(evidence.contains("contains_image_pixels=false"));
        assert!(evidence.contains("future_texture_identity=true"));
    }

    #[test]
    fn texture_boundary_releases_disposable_state_without_catalog_or_sidecar_writes() {
        let mut boundary = NativeViewerTextureBoundary::default();
        let identity = silica_render::ViewerTextureIdentity::new(
            silica_render::ViewerPreviewRenderRequestId(21),
            "texture/request-21",
            silica_render::ViewerTextureDrawableSize::new(1200, 675),
        );

        boundary.bind_disposable_texture(identity.clone());
        assert_eq!(
            boundary.state(),
            silica_render::ViewerTextureLifecycleState::Bound
        );
        assert_eq!(boundary.current_texture(), Some(&identity));

        boundary.cleanup_for_photo_change();
        assert_eq!(
            boundary.last_release_reason(),
            Some(silica_render::ViewerTextureReleaseReason::PhotoChanged)
        );
        assert_eq!(boundary.current_texture(), None);
        assert_eq!(boundary.release_count(), 1);

        boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
            silica_render::ViewerPreviewRenderRequestId(22),
            "texture/request-22",
            silica_render::ViewerTextureDrawableSize::new(1600, 900),
        ));
        boundary.cleanup_for_drawable_resize();
        boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
            silica_render::ViewerPreviewRenderRequestId(23),
            "texture/request-23",
            silica_render::ViewerTextureDrawableSize::new(1600, 900),
        ));
        boundary.cleanup_for_library_close();
        boundary.bind_disposable_texture(silica_render::ViewerTextureIdentity::new(
            silica_render::ViewerPreviewRenderRequestId(24),
            "texture/request-24",
            silica_render::ViewerTextureDrawableSize::new(1600, 900),
        ));
        boundary.cleanup_for_app_close();

        assert_eq!(boundary.release_count(), 4);
        assert!(!boundary.writes_catalog_state());
        assert!(!boundary.writes_sidecar_state());
        assert!(!boundary.uses_original_path_as_write_destination());
        assert!(!boundary.persistent_gpu_cache_enabled());
        assert!(boundary.is_rebuildable());
    }

    #[test]
    fn texture_lifecycle_smoke_evidence_is_reviewable() {
        let evidence = super::texture_lifecycle_smoke_evidence();
        println!("[SilicaRAW Native Viewer] {evidence}");

        assert!(evidence.contains("state=released"));
        assert!(evidence.contains("release_count=4"));
        assert!(evidence.contains("last_release=app-closed"));
        assert!(evidence.contains("catalog_write=false"));
        assert!(evidence.contains("sidecar_write=false"));
        assert!(evidence.contains("original_write_destination=false"));
        assert!(evidence.contains("persistent_gpu_cache=false"));
        assert!(evidence.contains("rebuildable=true"));
    }
}
