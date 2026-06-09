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

#[tauri::command]
fn import_folder(library_path: String, folder_path: String) -> Result<String, String> {
    silica_core::import_folder(PathBuf::from(library_path), PathBuf::from(folder_path))
        .map(|summary| {
            format!(
                "Imported folder: {}\nScanned: {}\nSupported: {}\nUnsupported: {}\nOriginal files unchanged: true",
                summary.folder_path.display(),
                summary.scanned_files,
                summary.supported_files,
                summary.unsupported_files
            )
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_photo_flags(
    library_path: String,
    photo_id: String,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
) -> Result<String, String> {
    silica_core::set_photo_flags(
        PathBuf::from(library_path),
        photo_id,
        rating,
        picked,
        rejected,
        color_label,
    )
    .map(|flags| {
        photo_flags_status_text(
            &flags.photo_id,
            flags.rating,
            flags.picked,
            flags.rejected,
            flags.color_label.as_deref(),
        )
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_photo_flags(library_path: String, photo_id: String) -> Result<Option<String>, String> {
    silica_core::get_photo_flags(PathBuf::from(library_path), &photo_id)
        .map(|flags| {
            flags.map(|flags| {
                photo_flags_status_text(
                    &flags.photo_id,
                    flags.rating,
                    flags.picked,
                    flags.rejected,
                    flags.color_label.as_deref(),
                )
            })
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_photo_preview(library_path: String, photo_id: String) -> Result<Option<String>, String> {
    silica_core::open_photo_preview(PathBuf::from(library_path), &photo_id)
        .map(|preview| preview.map(|preview| preview.status_text()))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn preview_exposure_contrast_edit(
    library_path: String,
    photo_id: String,
    exposure: f64,
    contrast: f64,
) -> Result<Option<String>, String> {
    silica_core::preview_exposure_contrast_edit(
        PathBuf::from(library_path),
        &photo_id,
        exposure,
        contrast,
    )
    .map(|preview| preview.map(|preview| preview.status_text()))
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn commit_exposure_contrast_edit(
    library_path: String,
    photo_id: String,
    exposure: f64,
    contrast: f64,
) -> Result<Option<String>, String> {
    silica_core::commit_exposure_contrast_edit(
        PathBuf::from(library_path),
        &photo_id,
        exposure,
        contrast,
    )
    .map(|commit| commit.map(|commit| commit.status_text()))
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_photo_jpeg_srgb(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> Result<Option<String>, String> {
    silica_core::export_photo_jpeg_srgb(
        PathBuf::from(library_path),
        &photo_id,
        PathBuf::from(output_path),
    )
    .map(|export| export.map(|export| export.status_text()))
    .map_err(|error| error.to_string())
}

fn photo_flags_status_text(
    photo_id: &str,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<&str>,
) -> String {
    format!(
        "Photo: {photo_id}\nRating: {rating}\nPicked: {picked}\nRejected: {rejected}\nColor label: {}",
        color_label.unwrap_or("none")
    )
}

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
    let builder = builder.setup(metal_host_spike::install);

    builder
        .invoke_handler(tauri::generate_handler![
            create_library,
            open_library,
            import_folder,
            set_photo_flags,
            get_photo_flags,
            open_photo_preview,
            preview_exposure_contrast_edit,
            commit_exposure_contrast_edit,
            export_photo_jpeg_srgb
        ])
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

    #[test]
    fn desktop_commands_set_and_get_photo_flags() {
        let workspace = unique_library_root("desktop-flags");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let updated = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            2,
            true,
            false,
            Some("blue".to_string()),
        )
        .expect("set flags command");
        assert!(updated.contains("Rating: 2"));
        assert!(updated.contains("Picked: true"));
        assert!(updated.contains("Color label: blue"));

        let reopened = super::get_photo_flags(library_root.display().to_string(), photo_id)
            .expect("get flags command")
            .expect("flags row");
        assert_eq!(reopened, updated);

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_imports_folder_by_reference() {
        let workspace = unique_library_root("desktop-import");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
        std::fs::write(&unsupported_file, b"not a photo").expect("write unsupported");

        silica_core::create_library(&library_root).expect("create library");

        let imported = super::import_folder(
            library_root.display().to_string(),
            import_root.display().to_string(),
        )
        .expect("import folder command");

        assert!(imported.contains("Imported folder:"));
        assert!(imported.contains("Scanned: 2"));
        assert!(imported.contains("Supported: 1"));
        assert!(imported.contains("Unsupported: 1"));
        assert!(supported_file.is_file());
        assert!(unsupported_file.is_file());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_opens_photo_preview_status() {
        let workspace = unique_library_root("desktop-preview");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::open_photo_preview(library_root.display().to_string(), photo_id)
            .expect("open preview command")
            .expect("preview session");

        assert!(preview.contains("File: sample.jpg"));
        assert!(preview.contains("Preview: Ready"));
        assert!(preview.contains("display-profile-aware"));

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_exposure_contrast_edit() {
        let workspace = unique_library_root("desktop-edit-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.5,
            -8.0,
        )
        .expect("preview edit command")
        .expect("preview edit request");
        assert!(preview.contains("Preview: Ready"));
        assert!(preview.contains("Exposure: 0.5"));
        assert!(preview.contains("Contrast: -8"));

        let committed = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id,
            0.5,
            -8.0,
        )
        .expect("commit edit command")
        .expect("committed edit");
        assert!(committed.contains("Persisted: true"));
        assert!(committed.contains("Exposure: 0.5"));
        assert!(committed.contains("Contrast: -8"));

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_exports_photo_jpeg_srgb() {
        let workspace = unique_library_root("desktop-export");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let supported_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        silica_core::commit_exposure_contrast_edit(&library_root, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("committed edit");

        let export = super::export_photo_jpeg_srgb(
            library_root.display().to_string(),
            photo_id,
            output_path.display().to_string(),
        )
        .expect("export command")
        .expect("export status");

        assert!(export.contains("Format: jpeg"));
        assert!(export.contains("Color: srgb"));
        assert!(output_path.is_file());

        remove_library_root(&workspace);
    }

    fn stable_catalog_id(prefix: &str, value: &str) -> String {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in value.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{prefix}-{hash:016x}")
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

    fn write_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgb([64, 128, 192])
            } else {
                image::Rgb([192, 128, 64])
            }
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write source jpeg");
    }
}
