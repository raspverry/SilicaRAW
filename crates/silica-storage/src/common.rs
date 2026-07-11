use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};

use crate::LibraryStorageError;

pub(super) fn path_to_string(path: &Path) -> Result<String, LibraryStorageError> {
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LibraryStorageError::InvalidPath(path.to_path_buf()))
}

pub(super) fn validate_sidecar_photo_id(photo_id: &str) -> Result<(), LibraryStorageError> {
    if photo_id.is_empty()
        || photo_id == "."
        || photo_id == ".."
        || photo_id.contains('/')
        || photo_id.contains('\\')
        || photo_id.contains("..")
        || photo_id.chars().any(|character| {
            !(character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.')
        })
    {
        return Err(LibraryStorageError::InvalidSidecarPhotoId(
            photo_id.to_string(),
        ));
    }

    Ok(())
}

pub(super) fn modified_at_string(metadata: &fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| format!("unix:{}", duration.as_secs()))
}

pub(super) fn current_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("unix:{}", duration.as_secs()))
        .unwrap_or_else(|_| "unix:0".to_string())
}

pub(super) fn partial_file_hash(path: &Path) -> Result<String, LibraryStorageError> {
    const MAX_HASH_BYTES: usize = 64 * 1024;

    let mut file = File::open(path)?;
    let mut buffer = [0_u8; 8192];
    let mut remaining = MAX_HASH_BYTES;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    while remaining > 0 {
        let read_limit = remaining.min(buffer.len());
        let read = file.read(&mut buffer[..read_limit])?;
        if read == 0 {
            break;
        }

        for byte in &buffer[..read] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        remaining -= read;
    }

    Ok(format!("{hash:016x}"))
}

pub(super) fn full_file_sha256(path: &Path) -> Result<String, LibraryStorageError> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }

    Ok(format!("{:x}", digest.finalize()))
}

pub(super) fn stable_catalog_id(prefix: &str, value: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{prefix}-{hash:016x}")
}

pub(super) fn unique_catalog_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    stable_catalog_id(prefix, &format!("{prefix}\n{nanos}"))
}

pub(super) fn bool_to_sql(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

pub(super) fn sql_to_bool(value: i64) -> bool {
    value != 0
}
