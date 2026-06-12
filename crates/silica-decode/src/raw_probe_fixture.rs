#[cfg(feature = "core-image-raw-probe")]
use std::fs;
#[cfg(feature = "core-image-raw-probe")]
use std::io::{self, Read};
#[cfg(feature = "core-image-raw-probe")]
use std::path::{Component, Path, PathBuf};

#[cfg(feature = "core-image-raw-probe")]
use serde_json::Value;

#[cfg(feature = "core-image-raw-probe")]
pub fn probe_raw_fixture_manifest(
    manifest_path: &str,
) -> Result<crate::RawFixtureProbeReport, crate::RawFixtureProbeError> {
    let manifest_path_buf = PathBuf::from(manifest_path);
    let bytes = fs::read(&manifest_path_buf).map_err(|error| {
        crate::RawFixtureProbeError::ReadManifest {
            path: manifest_path.to_string(),
            message: error.to_string(),
        }
    })?;
    let manifest: Value = serde_json::from_slice(&bytes).map_err(|error| {
        crate::RawFixtureProbeError::InvalidManifest {
            path: manifest_path.to_string(),
            message: error.to_string(),
        }
    })?;

    validate_manifest_header(manifest_path, &manifest)?;

    let expected_hashes = manifest
        .get("expected_source_hashes")
        .and_then(Value::as_object)
        .ok_or_else(|| crate::RawFixtureProbeError::InvalidManifest {
            path: manifest_path.to_string(),
            message: "fixture manifest missing expected_source_hashes object".to_string(),
        })?;
    let fixtures = manifest
        .get("fixtures")
        .and_then(Value::as_array)
        .filter(|fixtures| !fixtures.is_empty())
        .ok_or_else(|| crate::RawFixtureProbeError::InvalidManifest {
            path: manifest_path.to_string(),
            message: "fixture manifest must contain at least one fixture".to_string(),
        })?;

    let base_dir = manifest_path_buf.parent().unwrap_or_else(|| Path::new("."));
    let mut results = Vec::with_capacity(fixtures.len());

    for fixture in fixtures {
        let fixture_id = required_string(fixture, "id", "<unknown>")?;
        let fixture_class = required_string(fixture, "class", &fixture_id)?;
        let kind = required_string(fixture, "kind", &fixture_id)?;
        if kind != "raw" {
            return Err(crate::RawFixtureProbeError::InvalidFixture {
                fixture_id,
                message: "RAW probe manifest entries must use kind \"raw\"".to_string(),
            });
        }

        let relative_path = required_string(fixture, "relative_path", &fixture_id)?;
        validate_relative_path(&fixture_id, &relative_path)?;

        if fixture.get("raw").and_then(Value::as_object).is_none() {
            return Err(crate::RawFixtureProbeError::InvalidFixture {
                fixture_id,
                message: "RAW probe fixture missing raw metadata object".to_string(),
            });
        }

        let expected_hash = expected_hashes
            .get(&relative_path)
            .and_then(Value::as_str)
            .ok_or_else(|| crate::RawFixtureProbeError::InvalidFixture {
                fixture_id: fixture_id.clone(),
                message: "fixture missing expected_source_hashes entry".to_string(),
            })?;
        validate_sha256(&fixture_id, expected_hash)?;
        let integrity_hash = fixture
            .get("integrity")
            .and_then(Value::as_object)
            .and_then(|integrity| integrity.get("sha256"))
            .and_then(Value::as_str)
            .ok_or_else(|| crate::RawFixtureProbeError::InvalidFixture {
                fixture_id: fixture_id.clone(),
                message: "fixture missing integrity.sha256".to_string(),
            })?;
        if integrity_hash != expected_hash {
            return Err(crate::RawFixtureProbeError::InvalidFixture {
                fixture_id,
                message: "expected_source_hashes entry must match integrity.sha256".to_string(),
            });
        }

        let source_path = base_dir.join(&relative_path);
        let before_hash = sha256_file(&source_path).map_err(|error| {
            crate::RawFixtureProbeError::InvalidFixture {
                fixture_id: fixture_id.clone(),
                message: format!("fixture source could not be hashed before probe: {error}"),
            }
        })?;
        let probe = crate::probe_core_image_raw(crate::RawProbeRequest {
            source_path: source_path.to_string_lossy().to_string(),
            expected_sha256: Some(expected_hash.to_string()),
        });
        let after_hash = sha256_file(&source_path).map_err(|error| {
            crate::RawFixtureProbeError::InvalidFixture {
                fixture_id: fixture_id.clone(),
                message: format!("fixture source could not be hashed after probe: {error}"),
            }
        })?;

        results.push(crate::RawFixtureProbeResult {
            fixture_id,
            fixture_class,
            relative_path,
            probe,
            original_hash_unchanged: before_hash == after_hash,
        });
    }

    Ok(crate::RawFixtureProbeReport {
        manifest_path: manifest_path.to_string(),
        results,
    })
}

#[cfg(not(feature = "core-image-raw-probe"))]
pub fn probe_raw_fixture_manifest(
    _manifest_path: &str,
) -> Result<crate::RawFixtureProbeReport, crate::RawFixtureProbeError> {
    Err(crate::RawFixtureProbeError::FeatureDisabled)
}

#[cfg(feature = "core-image-raw-probe")]
fn validate_manifest_header(
    manifest_path: &str,
    manifest: &Value,
) -> Result<(), crate::RawFixtureProbeError> {
    if manifest.get("schema").and_then(Value::as_str) != Some("silica.fixture_manifest") {
        return Err(crate::RawFixtureProbeError::InvalidManifest {
            path: manifest_path.to_string(),
            message: "fixture manifest schema must be silica.fixture_manifest".to_string(),
        });
    }
    if manifest.get("version").and_then(Value::as_i64) != Some(1) {
        return Err(crate::RawFixtureProbeError::InvalidManifest {
            path: manifest_path.to_string(),
            message: "fixture manifest version must be 1".to_string(),
        });
    }
    Ok(())
}

#[cfg(feature = "core-image-raw-probe")]
fn required_string(
    value: &Value,
    key: &str,
    fixture_id: &str,
) -> Result<String, crate::RawFixtureProbeError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| crate::RawFixtureProbeError::InvalidFixture {
            fixture_id: fixture_id.to_string(),
            message: format!("fixture missing string field {key}"),
        })
}

#[cfg(feature = "core-image-raw-probe")]
fn validate_relative_path(
    fixture_id: &str,
    relative_path: &str,
) -> Result<(), crate::RawFixtureProbeError> {
    let path = Path::new(relative_path);
    let normal_components = path
        .components()
        .all(|component| matches!(component, Component::Normal(_)));
    let safe_path_parts = !relative_path.is_empty()
        && !relative_path.contains('\\')
        && !relative_path.contains("//")
        && relative_path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");

    if path.is_absolute() || !normal_components || !safe_path_parts {
        return Err(crate::RawFixtureProbeError::InvalidFixture {
            fixture_id: fixture_id.to_string(),
            message: "fixture relative_path must be relative and contain no parent/current directory parts"
                .to_string(),
        });
    }

    Ok(())
}

#[cfg(feature = "core-image-raw-probe")]
fn validate_sha256(fixture_id: &str, hash: &str) -> Result<(), crate::RawFixtureProbeError> {
    if hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }

    Err(crate::RawFixtureProbeError::InvalidFixture {
        fixture_id: fixture_id.to_string(),
        message: "fixture expected SHA-256 must be 64 lowercase hex characters".to_string(),
    })
}

#[cfg(feature = "core-image-raw-probe")]
fn sha256_file(path: &Path) -> Result<String, io::Error> {
    use sha2::{Digest, Sha256};

    let mut file = fs::File::open(path)?;
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
