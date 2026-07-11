use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::model::{
    ExportColorProfile, ExportError, ExportMetadataPolicy, JpegIccProfileInspection,
};

const PORTABLE_SRGB_ICC_PROFILE: &[u8] =
    include_bytes!("../../../assets/color-profiles/sRGB-v4.icc");
const PORTABLE_DISPLAY_P3_ICC_PROFILE: &[u8] =
    include_bytes!("../../../assets/color-profiles/DisplayP3Compat-v4.icc");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedJpegMetadata {
    pub(crate) source_segment_count: usize,
    pub(crate) output_segments: Vec<Vec<u8>>,
    pub(crate) gps_removed: bool,
}

pub(crate) fn prepare_jpeg_source_metadata(
    path: &Path,
    policy: ExportMetadataPolicy,
) -> Result<PreparedJpegMetadata, ExportError> {
    let bytes = fs::read(path)?;
    let source_segments = jpeg_metadata_segments(&bytes);
    let source_segment_count = source_segments.len();
    if matches!(
        policy,
        ExportMetadataPolicy::Minimal | ExportMetadataPolicy::RemoveAll
    ) {
        return Ok(PreparedJpegMetadata {
            source_segment_count,
            output_segments: Vec::new(),
            gps_removed: false,
        });
    }

    let mut gps_removed = false;
    let output_segments = source_segments
        .into_iter()
        .map(|segment| {
            if policy == ExportMetadataPolicy::RemoveGps && segment.payload.starts_with(b"Exif\0\0")
            {
                let (payload, removed) = strip_exif_gps_ifd(segment.payload);
                gps_removed |= removed;
                jpeg_segment_bytes(segment.marker, &payload)
            } else {
                jpeg_segment_bytes(segment.marker, segment.payload)
            }
        })
        .collect();

    Ok(PreparedJpegMetadata {
        source_segment_count,
        output_segments,
        gps_removed,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JpegMetadataSegment<'a> {
    marker: u8,
    payload: &'a [u8],
}

fn jpeg_metadata_segments(bytes: &[u8]) -> Vec<JpegMetadataSegment<'_>> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Vec::new();
    }

    let mut cursor = 2;
    let mut segments = Vec::new();
    while cursor + 4 <= bytes.len() {
        if bytes[cursor] != 0xFF {
            break;
        }
        let marker = bytes[cursor + 1];
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        let length = u16::from_be_bytes([bytes[cursor + 2], bytes[cursor + 3]]) as usize;
        if length < 2 || cursor + 2 + length > bytes.len() {
            break;
        }
        let payload_start = cursor + 4;
        let payload_end = cursor + 2 + length;
        if matches!(marker, 0xE1 | 0xED) {
            segments.push(JpegMetadataSegment {
                marker,
                payload: &bytes[payload_start..payload_end],
            });
        }
        cursor = payload_end;
    }
    segments
}

fn jpeg_segment_bytes(marker: u8, payload: &[u8]) -> Vec<u8> {
    let segment_length = payload.len() + 2;
    let mut segment = Vec::with_capacity(segment_length + 2);
    segment.push(0xFF);
    segment.push(marker);
    segment.extend_from_slice(&(segment_length as u16).to_be_bytes());
    segment.extend_from_slice(payload);
    segment
}

pub(crate) fn insert_jpeg_metadata_segments(
    path: &Path,
    segments: &[Vec<u8>],
) -> Result<(), ExportError> {
    if segments.is_empty() {
        return Ok(());
    }
    let bytes = fs::read(path)?;
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Ok(());
    }
    let mut output =
        Vec::with_capacity(bytes.len() + segments.iter().map(std::vec::Vec::len).sum::<usize>());
    output.extend_from_slice(&bytes[..2]);
    for segment in segments {
        output.extend_from_slice(segment);
    }
    output.extend_from_slice(&bytes[2..]);
    fs::write(path, output)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExifEntry {
    tag: u16,
    field_type: u16,
    count: u32,
    value: [u8; 4],
}

fn strip_exif_gps_ifd(payload: &[u8]) -> (Vec<u8>, bool) {
    if !payload.starts_with(b"Exif\0\0") || payload.len() < 14 {
        return (payload.to_vec(), false);
    }
    let tiff = &payload[6..];
    let endian = match &tiff[0..2] {
        b"II" => ExifEndian::Little,
        b"MM" => ExifEndian::Big,
        _ => return (payload.to_vec(), false),
    };
    if read_u16(tiff, 2, endian) != Some(42) {
        return (payload.to_vec(), false);
    }
    let Some(ifd_offset) = read_u32(tiff, 4, endian).map(|value| value as usize) else {
        return (payload.to_vec(), false);
    };
    let Some(entry_count) = read_u16(tiff, ifd_offset, endian).map(usize::from) else {
        return (payload.to_vec(), false);
    };
    let entries_start = ifd_offset + 2;
    let entries_end = entries_start + (entry_count * 12);
    if entries_end + 4 > tiff.len() {
        return (payload.to_vec(), false);
    }

    let mut kept_entries = Vec::new();
    let mut removed = false;
    for index in 0..entry_count {
        let offset = entries_start + (index * 12);
        let Some(tag) = read_u16(tiff, offset, endian) else {
            return (payload.to_vec(), false);
        };
        if tag == 0x8825 {
            removed = true;
            continue;
        }
        let Some(field_type) = read_u16(tiff, offset + 2, endian) else {
            return (payload.to_vec(), false);
        };
        let Some(count) = read_u32(tiff, offset + 4, endian) else {
            return (payload.to_vec(), false);
        };
        let mut value = [0_u8; 4];
        value.copy_from_slice(&tiff[offset + 8..offset + 12]);
        kept_entries.push(ExifEntry {
            tag,
            field_type,
            count,
            value,
        });
    }
    if !removed {
        return (payload.to_vec(), false);
    }

    let mut rebuilt_tiff = tiff[..ifd_offset].to_vec();
    write_u16(&mut rebuilt_tiff, kept_entries.len() as u16, endian);
    let entry_offsets_start = rebuilt_tiff.len();
    for entry in &kept_entries {
        write_u16(&mut rebuilt_tiff, entry.tag, endian);
        write_u16(&mut rebuilt_tiff, entry.field_type, endian);
        write_u32(&mut rebuilt_tiff, entry.count, endian);
        rebuilt_tiff.extend_from_slice(&entry.value);
    }
    write_u32(&mut rebuilt_tiff, 0, endian);

    for (index, entry) in kept_entries.iter().enumerate() {
        let data_len = exif_type_size(entry.field_type)
            .and_then(|size| size.checked_mul(entry.count as usize))
            .unwrap_or(0);
        if data_len <= 4 {
            continue;
        }
        let Some(old_data_offset) = read_u32(&entry.value, 0, endian).map(|value| value as usize)
        else {
            continue;
        };
        if old_data_offset + data_len > tiff.len() {
            continue;
        }
        let new_data_offset = rebuilt_tiff.len() as u32;
        rebuilt_tiff.extend_from_slice(&tiff[old_data_offset..old_data_offset + data_len]);
        let patch_offset = entry_offsets_start + (index * 12) + 8;
        write_u32_at(&mut rebuilt_tiff, patch_offset, new_data_offset, endian);
    }

    let mut output = b"Exif\0\0".to_vec();
    output.extend_from_slice(&rebuilt_tiff);
    (output, true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExifEndian {
    Little,
    Big,
}

fn read_u16(bytes: &[u8], offset: usize, endian: ExifEndian) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(match endian {
        ExifEndian::Little => u16::from_le_bytes([slice[0], slice[1]]),
        ExifEndian::Big => u16::from_be_bytes([slice[0], slice[1]]),
    })
}

fn read_u32(bytes: &[u8], offset: usize, endian: ExifEndian) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(match endian {
        ExifEndian::Little => u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]),
        ExifEndian::Big => u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]),
    })
}

fn write_u16(output: &mut Vec<u8>, value: u16, endian: ExifEndian) {
    match endian {
        ExifEndian::Little => output.extend_from_slice(&value.to_le_bytes()),
        ExifEndian::Big => output.extend_from_slice(&value.to_be_bytes()),
    }
}

fn write_u32(output: &mut Vec<u8>, value: u32, endian: ExifEndian) {
    match endian {
        ExifEndian::Little => output.extend_from_slice(&value.to_le_bytes()),
        ExifEndian::Big => output.extend_from_slice(&value.to_be_bytes()),
    }
}

fn write_u32_at(output: &mut [u8], offset: usize, value: u32, endian: ExifEndian) {
    let bytes = match endian {
        ExifEndian::Little => value.to_le_bytes(),
        ExifEndian::Big => value.to_be_bytes(),
    };
    output[offset..offset + 4].copy_from_slice(&bytes);
}

fn exif_type_size(field_type: u16) -> Option<usize> {
    match field_type {
        1 | 2 | 6 | 7 => Some(1),
        3 | 8 => Some(2),
        4 | 9 | 11 => Some(4),
        5 | 10 | 12 => Some(8),
        _ => None,
    }
}

/// Inspect the first embedded ICC profile in a JPEG file.
pub fn inspect_jpeg_icc_profile(
    path: impl AsRef<Path>,
) -> Result<JpegIccProfileInspection, ExportError> {
    let path = path.as_ref();
    let bytes = fs::read(path)?;
    let profile = first_icc_profile(&bytes)
        .map_err(|_| ExportError::InvalidJpegIccProfile(path.to_path_buf()))?;
    let Some(profile) = profile else {
        return Ok(JpegIccProfileInspection {
            embedded: false,
            color_profile: None,
            icc_profile_sha256: None,
        });
    };

    let icc_profile_sha256 = sha256_hex(&profile);
    Ok(JpegIccProfileInspection {
        embedded: true,
        color_profile: classify_icc_profile(&profile),
        icc_profile_sha256: Some(icc_profile_sha256),
    })
}

pub(crate) fn export_icc_profile(profile: ExportColorProfile) -> Result<Vec<u8>, ExportError> {
    #[cfg(target_os = "macos")]
    {
        let path = export_icc_profile_path(profile);
        return Ok(system_or_portable_icc_profile(profile, &path));
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(portable_icc_profile(profile).to_vec())
    }
}

pub(super) fn portable_icc_profile(profile: ExportColorProfile) -> &'static [u8] {
    match profile {
        ExportColorProfile::Srgb => PORTABLE_SRGB_ICC_PROFILE,
        ExportColorProfile::DisplayP3 => PORTABLE_DISPLAY_P3_ICC_PROFILE,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn system_or_portable_icc_profile(profile: ExportColorProfile, path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|_| portable_icc_profile(profile).to_vec())
}

#[cfg(target_os = "macos")]
pub(super) fn export_icc_profile_path(profile: ExportColorProfile) -> PathBuf {
    match profile {
        ExportColorProfile::Srgb => {
            PathBuf::from("/System/Library/ColorSync/Profiles/sRGB Profile.icc")
        }
        ExportColorProfile::DisplayP3 => {
            PathBuf::from("/System/Library/ColorSync/Profiles/Display P3.icc")
        }
    }
}

fn first_icc_profile(bytes: &[u8]) -> Result<Option<Vec<u8>>, ()> {
    if bytes.len() < 2 || bytes[0..2] != [0xff, 0xd8] {
        return Err(());
    }

    let mut index = 2;
    while index + 4 <= bytes.len() {
        if bytes[index] != 0xff {
            return Err(());
        }

        let marker = bytes[index + 1];
        if marker == 0xd9 || marker == 0xda {
            return Ok(None);
        }

        let length = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;
        if length < 2 || index + 2 + length > bytes.len() {
            return Err(());
        }

        let marker_payload = &bytes[index + 4..index + 2 + length];
        if marker == 0xe2
            && marker_payload.starts_with(b"ICC_PROFILE\0")
            && marker_payload.len() >= 14
        {
            return Ok(Some(marker_payload[14..].to_vec()));
        }

        index += 2 + length;
    }

    Ok(None)
}

pub(super) fn classify_icc_profile(profile: &[u8]) -> Option<ExportColorProfile> {
    match sha256_hex(profile).as_str() {
        "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e" => {
            Some(ExportColorProfile::Srgb)
        }
        "0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0" => {
            Some(ExportColorProfile::DisplayP3)
        }
        _ if icc_rgb_primaries_match(profile, DISPLAY_P3_D50_XYZ) => {
            Some(ExportColorProfile::DisplayP3)
        }
        _ if icc_rgb_primaries_match(profile, SRGB_D50_XYZ) => Some(ExportColorProfile::Srgb),
        _ if profile_contains_ascii(profile, b"sRGB") => Some(ExportColorProfile::Srgb),
        _ => None,
    }
}

const DISPLAY_P3_D50_XYZ: [[f64; 3]; 3] = [
    [0.5151214599609375, 0.2411956787109375, -0.0010528564453125],
    [0.2919769287109375, 0.6922454833984375, 0.0418853759765625],
    [0.1571044921875, 0.0665740966796875, 0.7840728759765625],
];

const SRGB_D50_XYZ: [[f64; 3]; 3] = [
    [0.436065673828125, 0.2224884033203125, 0.013916015625],
    [0.3851470947265625, 0.7168731689453125, 0.097076416015625],
    [0.14306640625, 0.06060791015625, 0.7140960693359375],
];

fn icc_rgb_primaries_match(profile: &[u8], expected: [[f64; 3]; 3]) -> bool {
    let Some(actual) = icc_rgb_primaries(profile) else {
        return false;
    };
    actual
        .iter()
        .flatten()
        .zip(expected.iter().flatten())
        .all(|(actual, expected)| (actual - expected).abs() <= 0.02)
}

fn icc_rgb_primaries(profile: &[u8]) -> Option<[[f64; 3]; 3]> {
    if profile.len() < 132 || &profile[36..40] != b"acsp" {
        return None;
    }
    let tag_count = read_u32_be(profile, 128)? as usize;
    let tag_table_end = 132_usize.checked_add(tag_count.checked_mul(12)?)?;
    if tag_table_end > profile.len() {
        return None;
    }

    Some([
        icc_xyz_tag(profile, tag_count, b"rXYZ")?,
        icc_xyz_tag(profile, tag_count, b"gXYZ")?,
        icc_xyz_tag(profile, tag_count, b"bXYZ")?,
    ])
}

fn icc_xyz_tag(profile: &[u8], tag_count: usize, signature: &[u8; 4]) -> Option<[f64; 3]> {
    for index in 0..tag_count {
        let record_offset = 132 + (index * 12);
        if &profile[record_offset..record_offset + 4] != signature {
            continue;
        }
        let tag_offset = read_u32_be(profile, record_offset + 4)? as usize;
        let tag_size = read_u32_be(profile, record_offset + 8)? as usize;
        if tag_size < 20 || tag_offset.checked_add(20)? > profile.len() {
            return None;
        }
        if &profile[tag_offset..tag_offset + 4] != b"XYZ " {
            return None;
        }
        return Some([
            read_s15_fixed_16(profile, tag_offset + 8)?,
            read_s15_fixed_16(profile, tag_offset + 12)?,
            read_s15_fixed_16(profile, tag_offset + 16)?,
        ]);
    }
    None
}

fn read_u32_be(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_s15_fixed_16(bytes: &[u8], offset: usize) -> Option<f64> {
    let raw = i32::from_be_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
    Some(f64::from(raw) / 65536.0)
}

fn profile_contains_ascii(profile: &[u8], needle: &[u8]) -> bool {
    profile
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

pub fn sha256_file(path: &Path) -> Result<String, io::Error> {
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

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
