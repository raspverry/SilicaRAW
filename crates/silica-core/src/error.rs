use std::error::Error;
use std::fmt;

/// Errors returned by core command APIs.
#[derive(Debug)]
pub enum CoreError {
    Storage(silica_storage::LibraryStorageError),
    Decode(silica_decode::RawPreviewArtifactError),
    RawExport(silica_decode::RawFullResolutionExportSourceError),
    EditGraph(silica_edit::EditGraphValidationError),
    EditClipboard(silica_edit::EditClipboardError),
    Export(silica_export::ExportError),
    ExportBlocked(String),
    UnsupportedEdit(String),
    AppSession(String),
    AiReview(String),
    Plugin(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "{error}"),
            Self::Decode(error) => write!(formatter, "{error}"),
            Self::RawExport(error) => write!(formatter, "{error}"),
            Self::EditGraph(error) => write!(formatter, "{error}"),
            Self::EditClipboard(error) => write!(formatter, "{error}"),
            Self::Export(error) => write!(formatter, "{error}"),
            Self::ExportBlocked(message) => write!(formatter, "export blocked: {message}"),
            Self::UnsupportedEdit(message) => write!(formatter, "unsupported edit: {message}"),
            Self::AppSession(message) => write!(formatter, "app session error: {message}"),
            Self::AiReview(message) => write!(formatter, "AI review error: {message}"),
            Self::Plugin(message) => write!(formatter, "plugin error: {message}"),
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::RawExport(error) => Some(error),
            Self::EditGraph(error) => Some(error),
            Self::EditClipboard(error) => Some(error),
            Self::Export(error) => Some(error),
            Self::ExportBlocked(_) => None,
            Self::UnsupportedEdit(_) => None,
            Self::AppSession(_) => None,
            Self::AiReview(_) => None,
            Self::Plugin(_) => None,
        }
    }
}

impl From<silica_storage::LibraryStorageError> for CoreError {
    fn from(error: silica_storage::LibraryStorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<silica_decode::RawPreviewArtifactError> for CoreError {
    fn from(error: silica_decode::RawPreviewArtifactError) -> Self {
        Self::Decode(error)
    }
}

impl From<silica_decode::RawFullResolutionExportSourceError> for CoreError {
    fn from(error: silica_decode::RawFullResolutionExportSourceError) -> Self {
        Self::RawExport(error)
    }
}

impl From<silica_edit::EditGraphValidationError> for CoreError {
    fn from(error: silica_edit::EditGraphValidationError) -> Self {
        Self::EditGraph(error)
    }
}

impl From<silica_edit::EditClipboardError> for CoreError {
    fn from(error: silica_edit::EditClipboardError) -> Self {
        Self::EditClipboard(error)
    }
}

impl From<silica_export::ExportError> for CoreError {
    fn from(error: silica_export::ExportError) -> Self {
        Self::Export(error)
    }
}

impl From<silica_plugin::PluginManifestError> for CoreError {
    fn from(error: silica_plugin::PluginManifestError) -> Self {
        Self::Plugin(error.to_string())
    }
}
