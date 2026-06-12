//! Product native Metal viewer module shell.
//!
//! This is not the Spike 001 proof module. Task 14.2 only establishes the
//! feature-gated product boundary; later Phase 14 tasks add layout, lifecycle,
//! input, render request, texture, and QA behavior.

/// Compile-time contract for the product native viewer shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMetalViewerModuleContract {
    pub module_name: &'static str,
    pub feature_name: &'static str,
    pub phase_task: &'static str,
    pub product_module: bool,
    pub uses_spike_module: bool,
    pub installs_in_default_build: bool,
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
    }
}
