use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopEditClipboardTarget {
    pub(crate) photo_id: String,
    pub(crate) status: String,
    pub(crate) code: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopEditClipboardCommit {
    pub(crate) photo_id: String,
    pub(crate) history_id: String,
    pub(crate) sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopEditClipboardFailure {
    pub(crate) photo_id: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopHistoryItem {
    pub(crate) history_id: String,
    pub(crate) photo_id: String,
    pub(crate) sequence: i64,
    pub(crate) action_kind: String,
    pub(crate) label: String,
    pub(crate) history_state: String,
    pub(crate) can_undo: bool,
    pub(crate) can_redo: bool,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopAiReviewItem {
    pub(crate) result_id: String,
    pub(crate) model_id: String,
    pub(crate) label: String,
    pub(crate) recommendation: String,
    pub(crate) approvable: bool,
    pub(crate) confidence_percent: Option<u8>,
    pub(crate) approved: bool,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopAiSuggestionCommit {
    pub(crate) photo_id: String,
    pub(crate) exposure: f64,
    pub(crate) contrast: f64,
    pub(crate) persisted: bool,
    pub(crate) message: String,
}

impl From<silica_core::PhotoHistoryItem> for DesktopHistoryItem {
    fn from(item: silica_core::PhotoHistoryItem) -> Self {
        Self {
            history_id: item.history_id,
            photo_id: item.photo_id,
            sequence: item.sequence,
            action_kind: item.action_kind,
            label: item.label,
            history_state: item.history_state,
            can_undo: item.can_undo,
            can_redo: item.can_redo,
            created_at: item.created_at,
        }
    }
}

impl From<silica_core::AiReviewItem> for DesktopAiReviewItem {
    fn from(item: silica_core::AiReviewItem) -> Self {
        Self {
            result_id: item.result_id,
            model_id: item.model_id,
            label: item.label,
            recommendation: item.recommendation,
            approvable: item.approvable,
            confidence_percent: item.confidence_percent,
            approved: item.approved,
            created_at: item.created_at,
        }
    }
}

impl From<silica_core::PhotoEditCommit> for DesktopAiSuggestionCommit {
    fn from(commit: silica_core::PhotoEditCommit) -> Self {
        Self {
            photo_id: commit.photo_id,
            exposure: commit.exposure,
            contrast: commit.contrast,
            persisted: commit.persisted,
            message: commit.message,
        }
    }
}

pub(crate) fn edit_clipboard_section_count(
    selection: &silica_core::EditClipboardSelection,
) -> usize {
    [
        selection.basic,
        selection.tone,
        selection.color,
        selection.detail,
        selection.lens,
        selection.geometry,
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count()
}

pub(crate) fn edit_clipboard_target_data(
    target: silica_core::BatchEditClipboardSyncTarget,
) -> DesktopEditClipboardTarget {
    DesktopEditClipboardTarget {
        photo_id: target.photo_id,
        status: target.status,
        code: target.code,
        message: target.message,
    }
}

pub(crate) fn ai_review_status_text(status: silica_core::AiReviewPanelStatus) -> &'static str {
    match status {
        silica_core::AiReviewPanelStatus::ModelUnavailable => "modelUnavailable",
        silica_core::AiReviewPanelStatus::ReviewAvailable => "reviewAvailable",
    }
}
