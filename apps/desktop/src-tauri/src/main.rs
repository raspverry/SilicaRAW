#[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
mod metal_host_spike;

use std::path::PathBuf;

#[tauri::command]
fn create_library(path: String) -> Result<String, String> {
    silica_core::create_library(PathBuf::from(path))
        .map(|session| session.status_text())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_library(path: String) -> Result<String, String> {
    silica_core::open_library(PathBuf::from(path))
        .map(|session| session.status_text())
        .map_err(|error| error.to_string())
}

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
    let builder = builder.setup(metal_host_spike::install);

    builder
        .invoke_handler(tauri::generate_handler![create_library, open_library])
        .run(tauri::generate_context!())
        .expect("failed to run SilicaRAW desktop shell");
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn desktop_commands_create_and_open_library() {
        let root = unique_library_root("desktop");

        let created = super::create_library(root.display().to_string()).expect("create library");
        let opened = super::open_library(root.display().to_string()).expect("open library");

        assert!(created.contains("Library:"));
        assert!(created.contains("catalog.db"));
        assert_eq!(opened, created);

        remove_library_root(&root);
    }

    fn unique_library_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-desktop-library-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn remove_library_root(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }
}
