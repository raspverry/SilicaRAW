#[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
mod metal_host_spike;

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
    let builder = builder.setup(metal_host_spike::install);

    builder
        .run(tauri::generate_context!())
        .expect("failed to run SilicaRAW desktop shell");
}
