use rusqlite::{params, Connection};
use serde_json::{json, Value};
use silica_storage::{
    create_local_library, open_catalog, query_library_photos, LibraryQueryFileType,
    LibraryQueryFilters, LibraryQueryMetadataFilter, LibraryQueryRequest, LibraryQuerySort,
    LOCAL_LIBRARY_ID,
};
use std::cmp::Ordering;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

type BenchResult<T> = Result<T, Box<dyn Error>>;

const DATASET_SIZES: &[u64] = &[1_000, 10_000, 50_000];
const QUERY_LIMIT: u16 = 100;
const QUERY_RUNS: usize = 3;

fn main() -> BenchResult<()> {
    let workdir = env::var("SILICARAW_BENCHMARK_WORKDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".tmp/library-scale-benchmark"));
    if workdir.exists() {
        fs::remove_dir_all(&workdir)?;
    }
    fs::create_dir_all(&workdir)?;

    let mut datasets = Vec::new();
    for size in DATASET_SIZES {
        let library_root = workdir.join(format!("library-{size}"));
        let seed_started = Instant::now();
        let shape = seed_catalog(&library_root, *size)?;
        let seed_catalog_ms = elapsed_ms(seed_started);
        let timings = benchmark_queries(&library_root)?;
        datasets.push(json!({
            "photo_count": size,
            "shape": shape,
            "seed_catalog_ms": round3(seed_catalog_ms),
            "query_runs": QUERY_RUNS,
            "query_limit": QUERY_LIMIT,
            "timings": timings,
        }));
    }

    let report = json!({
        "schema_version": 1,
        "generated_unix_seconds": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        "scope_note": "Local benchmark evidence for this machine only; results are not universal performance guarantees.",
        "workdir": workdir.to_string_lossy(),
        "machine": machine_metadata(),
        "datasets": datasets,
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn seed_catalog(library_root: &Path, photo_count: u64) -> BenchResult<Value> {
    let library = create_local_library(library_root)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    connection.pragma_update(None, "synchronous", "OFF")?;
    seed_catalog_rows(&mut connection, photo_count)
}

fn seed_catalog_rows(connection: &mut Connection, photo_count: u64) -> BenchResult<Value> {
    let transaction = connection.transaction()?;
    let folder_id = "bench-folder";
    let folder_path = "/benchmark/source";
    transaction.execute(
        r#"
        INSERT INTO folders(id, library_id, path, scanned_at, missing)
        VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, 0)
        "#,
        params![folder_id, LOCAL_LIBRARY_ID, folder_path],
    )?;

    let mut jpeg_count = 0_u64;
    let mut raw_count = 0_u64;
    let mut unsupported_count = 0_u64;
    let mut picked_count = 0_u64;
    let mut rejected_count = 0_u64;
    let mut metadata_dimensions_count = 0_u64;

    {
        let mut photo_statement = transaction.prepare(
            r#"
            INSERT INTO photos(
              id,
              library_id,
              folder_id,
              file_name,
              path,
              file_size,
              modified_at,
              imported_at,
              missing,
              unsupported,
              partial_hash,
              file_type
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10, ?11)
            "#,
        )?;
        let mut flags_statement = transaction.prepare(
            r#"
            INSERT INTO photo_flags(photo_id, rating, picked, rejected, color_label)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )?;
        let mut metadata_statement = transaction.prepare(
            r#"
            INSERT INTO photo_metadata(
              photo_id,
              camera_make,
              camera_model,
              lens_model,
              capture_time,
              raw_json,
              width,
              height,
              orientation
            )
            VALUES (?1, ?2, ?3, ?4, ?5, '{}', ?6, ?7, ?8)
            "#,
        )?;

        for index in 0..photo_count {
            let class = photo_class(index);
            let photo_id = format!("bench-photo-{index:06}");
            let file_name = format!("photo-{index:06}.{}", class.extension);
            let path = format!("/benchmark/source/roll-{:03}/{file_name}", index / 1_000);
            let imported_at = imported_at(index);
            let unsupported = i64::from(class.file_type == "unsupported");
            let rating = (index % 6) as i64;
            let picked = i64::from(index % 7 == 0);
            let rejected = i64::from(index % 17 == 0);
            if picked == 1 {
                picked_count += 1;
            }
            if rejected == 1 {
                rejected_count += 1;
            }

            match class.file_type {
                "jpeg" => jpeg_count += 1,
                "raw" => raw_count += 1,
                "unsupported" => unsupported_count += 1,
                _ => {}
            }

            photo_statement.execute(params![
                photo_id,
                LOCAL_LIBRARY_ID,
                folder_id,
                file_name,
                path,
                128_000_i64 + i64::try_from(index % 65_536)?,
                "2026-06-17T00:00:00Z",
                imported_at,
                unsupported,
                format!("bench-{index:016x}"),
                class.file_type,
            ])?;

            flags_statement.execute(params![
                format!("bench-photo-{index:06}"),
                rating,
                picked,
                rejected,
                if index % 11 == 0 { Some("blue") } else { None },
            ])?;

            if class.file_type == "jpeg" && index % 2 == 0 {
                metadata_dimensions_count += 1;
                metadata_statement.execute(params![
                    format!("bench-photo-{index:06}"),
                    "SilicaRAW Benchmark",
                    "Synthetic Catalog",
                    "Synthetic 50mm",
                    imported_at,
                    6000_i64,
                    4000_i64,
                    "landscape",
                ])?;
            } else if class.file_type == "raw" {
                metadata_statement.execute(params![
                    format!("bench-photo-{index:06}"),
                    Option::<String>::None,
                    Option::<String>::None,
                    Option::<String>::None,
                    Option::<String>::None,
                    Option::<i64>::None,
                    Option::<i64>::None,
                    Option::<String>::None,
                ])?;
            }
        }
    }

    transaction.commit()?;

    Ok(json!({
        "jpeg_count": jpeg_count,
        "raw_count": raw_count,
        "unsupported_count": unsupported_count,
        "picked_count": picked_count,
        "rejected_count": rejected_count,
        "metadata_dimensions_count": metadata_dimensions_count,
        "folders": 1,
        "query_limit": QUERY_LIMIT,
        "sorts": ["imported_at_desc", "file_name_asc", "rating_desc"],
        "filters": ["file_type=jpeg", "metadata=has_dimensions", "search=photo-000"],
    }))
}

fn benchmark_queries(library_root: &Path) -> BenchResult<Value> {
    let imported = benchmark_operation(|| {
        let page = query_library_photos(
            library_root,
            LibraryQueryRequest::new(
                0,
                QUERY_LIMIT,
                LibraryQuerySort::ImportedAtDesc,
                LibraryQueryFilters::default(),
            ),
        )?;
        Ok(page.items.len())
    })?;

    let filtered_jpeg = benchmark_operation(|| {
        let page = query_library_photos(
            library_root,
            LibraryQueryRequest::new(
                0,
                QUERY_LIMIT,
                LibraryQuerySort::RatingDesc,
                LibraryQueryFilters {
                    min_rating: Some(3),
                    file_type: Some(LibraryQueryFileType::Jpeg),
                    ..LibraryQueryFilters::default()
                },
            ),
        )?;
        Ok(page.items.len())
    })?;

    let metadata_dimensions = benchmark_operation(|| {
        let page = query_library_photos(
            library_root,
            LibraryQueryRequest::new(
                0,
                QUERY_LIMIT,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters {
                    metadata: Some(LibraryQueryMetadataFilter::HasDimensions),
                    ..LibraryQueryFilters::default()
                },
            ),
        )?;
        Ok(page.items.len())
    })?;

    let search = benchmark_operation(|| {
        let page = query_library_photos(
            library_root,
            LibraryQueryRequest::new(
                0,
                QUERY_LIMIT,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters {
                    search: "photo-000".to_string(),
                    ..LibraryQueryFilters::default()
                },
            ),
        )?;
        Ok(page.items.len())
    })?;

    let render_adjacent = benchmark_operation(|| {
        let page = query_library_photos(
            library_root,
            LibraryQueryRequest::new(
                0,
                QUERY_LIMIT,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters::default(),
            ),
        )?;
        let page_model = page
            .items
            .iter()
            .map(|item| {
                format!(
                    "{}|{}|{}|{}|{}|{}",
                    item.photo_id,
                    item.file_name,
                    item.file_type,
                    item.rating,
                    item.picked,
                    item.thumbnail_path.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>();
        Ok(page_model.len())
    })?;

    Ok(json!({
        "query_imported_page_ms": imported.median_ms,
        "query_imported_page_rows": imported.rows,
        "query_filtered_jpeg_ms": filtered_jpeg.median_ms,
        "query_filtered_jpeg_rows": filtered_jpeg.rows,
        "query_metadata_dimensions_ms": metadata_dimensions.median_ms,
        "query_metadata_dimensions_rows": metadata_dimensions.rows,
        "query_search_ms": search.median_ms,
        "query_search_rows": search.rows,
        "render_adjacent_page_model_ms": render_adjacent.median_ms,
        "render_adjacent_page_model_rows": render_adjacent.rows,
    }))
}

struct Timing {
    median_ms: f64,
    rows: usize,
}

fn benchmark_operation<F>(mut operation: F) -> BenchResult<Timing>
where
    F: FnMut() -> BenchResult<usize>,
{
    let mut durations = Vec::with_capacity(QUERY_RUNS);
    let mut rows = 0;
    for _ in 0..QUERY_RUNS {
        let started = Instant::now();
        rows = operation()?;
        durations.push(elapsed_ms(started));
    }
    durations.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    Ok(Timing {
        median_ms: round3(durations[durations.len() / 2]),
        rows,
    })
}

struct PhotoClass {
    extension: &'static str,
    file_type: &'static str,
}

fn photo_class(index: u64) -> PhotoClass {
    if index % 20 == 0 {
        PhotoClass {
            extension: "txt",
            file_type: "unsupported",
        }
    } else if index % 5 == 0 {
        PhotoClass {
            extension: "dng",
            file_type: "raw",
        }
    } else {
        PhotoClass {
            extension: "jpg",
            file_type: "jpeg",
        }
    }
}

fn imported_at(index: u64) -> String {
    let seconds = index % 60;
    let minutes = (index / 60) % 60;
    let hours = (index / 3_600) % 24;
    let days = 1 + ((index / 86_400) % 28);
    format!("2026-06-{days:02}T{hours:02}:{minutes:02}:{seconds:02}.{index:06}Z")
}

fn machine_metadata() -> Value {
    json!({
        "os": env::consts::OS,
        "arch": env::consts::ARCH,
        "cpu_count": thread::available_parallelism().map(usize::from).unwrap_or(0),
        "rustc": rustc_version(),
    })
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unavailable".to_string())
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1_000.0
}

fn round3(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}
