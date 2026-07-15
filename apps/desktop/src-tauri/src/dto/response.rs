use serde::Serialize;

use super::DesktopCommandData;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopCommandResponse {
    pub(crate) ok: bool,
    pub(crate) command: &'static str,
    pub(crate) message: String,
    pub(crate) data: Option<DesktopCommandData>,
    pub(crate) error: Option<DesktopCommandError>,
}

impl DesktopCommandResponse {
    pub(crate) fn ok(
        command: &'static str,
        message: impl Into<String>,
        data: DesktopCommandData,
    ) -> Self {
        Self {
            ok: true,
            command,
            message: message.into(),
            data: Some(data),
            error: None,
        }
    }

    pub(crate) fn empty(command: &'static str, message: impl Into<String>) -> Self {
        Self {
            ok: true,
            command,
            message: message.into(),
            data: None,
            error: None,
        }
    }

    pub(crate) fn error(
        command: &'static str,
        error: silica_core::CoreError,
        context: DesktopCommandContext,
    ) -> Self {
        let kind = core_error_kind(&error).to_string();
        let message = error.to_string();
        Self {
            ok: false,
            command,
            message: message.clone(),
            data: None,
            error: Some(DesktopCommandError {
                kind,
                message,
                context,
            }),
        }
    }

    pub(crate) fn error_message(
        command: &'static str,
        message: impl Into<String>,
        kind: impl Into<String>,
        context: DesktopCommandContext,
    ) -> Self {
        let message = message.into();
        Self {
            ok: false,
            command,
            message: message.clone(),
            data: None,
            error: Some(DesktopCommandError {
                kind: kind.into(),
                message,
                context,
            }),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopCommandContext {
    pub(crate) library_path: Option<String>,
    pub(crate) folder_path: Option<String>,
    pub(crate) output_path: Option<String>,
    pub(crate) photo_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopCommandError {
    pub(crate) kind: String,
    pub(crate) message: String,
    pub(crate) context: DesktopCommandContext,
}

pub(crate) fn core_error_kind(error: &silica_core::CoreError) -> &'static str {
    match error {
        silica_core::CoreError::Storage(_) => "storage",
        silica_core::CoreError::Decode(_) => "decode",
        silica_core::CoreError::RawExport(_) => "decode",
        silica_core::CoreError::EditGraph(_) => "editGraph",
        silica_core::CoreError::EditClipboard(_) => "editClipboard",
        silica_core::CoreError::UnsupportedEdit(_) => "unsupportedEdit",
        silica_core::CoreError::Export(_) => "export",
        silica_core::CoreError::ExportBlocked(_) => "exportBlocked",
        silica_core::CoreError::AppSession(_) => "appSession",
        silica_core::CoreError::AiReview(_) => "aiReview",
        silica_core::CoreError::Plugin(_) => "plugin",
    }
}
