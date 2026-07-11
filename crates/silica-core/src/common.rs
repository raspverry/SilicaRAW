use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(super) const LOCAL_ALPHA_JPEG_QUALITY: u8 = 90;
pub(super) const LOCAL_ALPHA_THUMBNAIL_QUALITY: u8 = 82;
pub(super) const LOCAL_ALPHA_THUMBNAIL_MAX_EDGE: u32 = 320;
pub(super) const LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY: u8 = 88;
pub(super) const LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE: u32 = 2048;
pub(super) const LOCAL_ALPHA_DEVELOP_PREVIEW_QUALITY: u8 = 86;
pub(super) const LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE: u32 = 512;

pub(super) fn current_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("unix:{}", duration.as_secs()))
        .unwrap_or_else(|_| "unix:0".to_string())
}
