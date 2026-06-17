#!/usr/bin/env python3
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "apps/desktop/static/index.html"
BASE_CSS = ROOT / "apps/desktop/static/styles/base.css"
APP_FRAME_CSS = ROOT / "apps/desktop/static/styles/app-frame.css"
FINAL_VISUAL_QA = ROOT / "scripts/harness/run-final-visual-qa.py"


class StaticUiParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.ids = {}
        self.styles = 0
        self.mode_buttons = {}
        self.native_viewer_hosts = {}
        self.native_viewer_descendant_ids = {}
        self._element_stack = []
        self._current_button_mode = None

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        current_native_host = next(
            (entry["native_host_id"] for entry in reversed(self._element_stack) if entry["native_host_id"]),
            None,
        )
        if tag == "style":
            self.styles += 1
        if tag == "link" and attrs.get("rel") == "stylesheet":
            self.links.append(attrs.get("href", ""))
        if "id" in attrs:
            self.ids[attrs["id"]] = attrs
            if current_native_host:
                self.native_viewer_descendant_ids[attrs["id"]] = current_native_host
        native_host_id = attrs.get("id") if attrs.get("data-native-viewer-host") == "reserved" else None
        if native_host_id:
            self.native_viewer_hosts[native_host_id] = attrs
        if tag == "button" and "data-mode" in attrs:
            self._current_button_mode = attrs["data-mode"]
            self.mode_buttons[attrs["data-mode"]] = {"attrs": attrs, "text": ""}
        self._element_stack.append({"tag": tag, "native_host_id": native_host_id})

    def handle_data(self, data):
        if self._current_button_mode:
            self.mode_buttons[self._current_button_mode]["text"] += data

    def handle_endtag(self, tag):
        if tag == "button":
            self._current_button_mode = None
        for index in range(len(self._element_stack) - 1, -1, -1):
            if self._element_stack[index]["tag"] == tag:
                del self._element_stack[index:]
                break


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def main():
    source = INDEX.read_text(encoding="utf-8")
    parser = StaticUiParser()
    parser.feed(source)
    failures = []

    require(parser.styles == 0, "index.html must not use inline <style> blocks", failures)
    require(
        "function countImportErrorIssues" in source,
        "index.html must separate error issue counting from unsupported/skipped issue counting",
        failures,
    )
    require(
        "response.data?.issues?.length" not in source,
        "#importErrorCount must not count every structured import issue as an error",
        failures,
    )
    require(
        "Unsupported and failed files will be reviewable after import." not in source,
        "import issue review must not be placeholder status text",
        failures,
    )
    for href in [
        "./styles/tokens.css",
        "./styles/base.css",
        "./styles/app-frame.css",
    ]:
        require(href in parser.links, f"index.html must link {href}", failures)

    required_ids = [
        "appFrame",
        "appToolbar",
        "modeNavigation",
        "modeLibrary",
        "modeDevelop",
        "modeExport",
        "toggleSidebar",
        "toggleInspector",
        "leftSidebar",
        "mainSurface",
        "rightInspector",
        "bottomStatus",
        "appStatus",
        "welcomeScreen",
        "welcomeStatus",
        "libraryPath",
        "openLibrary",
        "createLibrary",
        "openRecent",
        "recentEmptyState",
        "recentLibraryList",
        "importPanel",
        "importFolderPath",
        "startImport",
        "importStatus",
        "importProgress",
        "importSummary",
        "unsupportedCount",
        "importRecursive",
        "importIssueReview",
        "importIssueReviewTitle",
        "importIssueReviewSummary",
        "importIssueList",
        "closeImportIssues",
        "viewImportErrors",
        "openAiReview",
        "aiReviewSurface",
        "aiReviewTitle",
        "aiReviewStatus",
        "aiReviewSelectedPhoto",
        "aiReviewBlurTab",
        "aiReviewQualityTab",
        "aiReviewDuplicateTab",
        "aiReviewList",
        "aiReviewEmptyState",
        "aiReviewSummary",
        "aiReviewActionPreview",
        "aiReviewApprovalDeferred",
        "aiReviewBackToGrid",
        "libraryGrid",
        "gridEmptyState",
        "gridLoadingState",
        "gridStateNote",
        "libraryPhotoCount",
        "selectionSummary",
        "primarySelectedPhotoName",
        "multiSelectionCount",
        "clearMultiSelection",
        "selectedPhotoName",
        "selectedPhotoRating",
        "metadataFileType",
        "metadataDimensions",
        "metadataCaptureTime",
        "metadataCamera",
        "metadataLens",
        "metadataModifiedAt",
        "metadataFileSize",
        "metadataStatus",
        "ratingControlGroup",
        "clearCullingFlags",
        "pickSelectedPhoto",
        "rejectSelectedPhoto",
        "cullingStatus",
        "thumbnailSize",
        "toggleFilmstrip",
        "resetLayout",
        "librarySearch",
        "fileTypeFilter",
        "metadataFilter",
        "minRatingFilter",
        "cullingFilter",
        "librarySort",
        "openLoupe",
        "closeLoupe",
        "loupeSurface",
        "loupeViewer",
        "loupePreviewStatus",
        "loupePhotoName",
        "loupePhotoRating",
        "loupePreviewMessage",
        "loupeFitMode",
        "loupeFilmstrip",
        "developPhotoName",
        "developPreviewSurface",
        "developPreviewStatus",
        "developExposureSlider",
        "developExposureValue",
        "developExposureReset",
        "developContrastSlider",
        "developContrastValue",
        "developContrastReset",
        "developMaskPanel",
        "developMaskSelectedPhoto",
        "developMaskSupportStatus",
        "developMaskBoundaryStatus",
        "developMaskRawBoundaryStatus",
        "developMaskToolManual",
        "developMaskToolAI",
        "developMaskToolMLX",
        "developMaskAddMask",
        "developMaskList",
        "developMaskBrushRow",
        "developMaskBrushName",
        "developMaskBrushSummary",
        "developMaskBrushState",
        "developMaskLinearRow",
        "developMaskLinearName",
        "developMaskLinearSummary",
        "developMaskLinearState",
        "developMaskRadialRow",
        "developMaskRadialName",
        "developMaskRadialSummary",
        "developMaskRadialState",
        "developMaskSubjectUnavailable",
        "developMaskSkyUnavailable",
        "developMaskActiveState",
        "developMaskActiveGeometry",
        "developMaskOverlayToggle",
        "developMaskOverlayColor",
        "developMaskExposureSlider",
        "developMaskExposureValue",
        "developMaskContrastSlider",
        "developMaskContrastValue",
        "developMaskOpacitySlider",
        "developMaskOpacityValue",
        "developMaskFeatherSlider",
        "developMaskFeatherValue",
        "developToneCurvePanel",
        "developToneCurveMidpointSlider",
        "developToneCurveMidpointValue",
        "developToneCurveReset",
        "developToneCurveSupportStatus",
        "developToneCurveChannelRed",
        "developToneCurveChannelGreen",
        "developToneCurveChannelBlue",
        "developToneCurveParametric",
        "developHslPanel",
        "developHslSupportStatus",
        "developHslChannelRed",
        "developHslChannelOrange",
        "developHslChannelYellow",
        "developHslChannelGreen",
        "developHslChannelAqua",
        "developHslChannelBlue",
        "developHslChannelPurple",
        "developHslChannelMagenta",
        "developHslHueSlider",
        "developHslHueValue",
        "developHslHueReset",
        "developHslSaturationSlider",
        "developHslSaturationValue",
        "developHslSaturationReset",
        "developHslLuminanceSlider",
        "developHslLuminanceValue",
        "developHslLuminanceReset",
        "developDetailPanel",
        "developDetailSupportStatus",
        "developDetailBoundaryStatus",
        "developDetailSharpeningAmountSlider",
        "developDetailSharpeningAmountValue",
        "developDetailSharpeningRadiusSlider",
        "developDetailSharpeningRadiusValue",
        "developDetailSharpeningDetailSlider",
        "developDetailSharpeningDetailValue",
        "developDetailSharpeningMaskingSlider",
        "developDetailSharpeningMaskingValue",
        "developDetailNoiseLuminanceSlider",
        "developDetailNoiseLuminanceValue",
        "developDetailNoiseDetailSlider",
        "developDetailNoiseDetailValue",
        "developDetailNoiseContrastSlider",
        "developDetailNoiseContrastValue",
        "developDetailNoiseColorSlider",
        "developDetailNoiseColorValue",
        "developDetailNoiseColorDetailSlider",
        "developDetailNoiseColorDetailValue",
        "developDetailMlxDenoise",
        "developLensGeometryPanel",
        "developGeometrySupportStatus",
        "developGeometryCropStatus",
        "developLensSupportStatus",
        "developLensProfileCorrection",
        "developLensChromaticAberration",
        "developLensDistortionSlider",
        "developLensVignettingSlider",
        "developGeometryCropXSlider",
        "developGeometryCropXValue",
        "developGeometryCropYSlider",
        "developGeometryCropYValue",
        "developGeometryCropWidthSlider",
        "developGeometryCropWidthValue",
        "developGeometryCropHeightSlider",
        "developGeometryCropHeightValue",
        "developGeometryCropClear",
        "developGeometryRotateLeft",
        "developGeometryRotateRight",
        "developGeometryOrientationReset",
        "developGeometryFlipHorizontal",
        "developGeometryFlipVertical",
        "developGeometryTransformStatus",
        "developGeometryTransformScaleSlider",
        "developGeometryTransformVerticalSlider",
        "developGeometryTransformHorizontalSlider",
        "developCommitEdit",
        "developRevertEdit",
        "developClipboardPanel",
        "developClipboardSource",
        "developClipboardSelectionCount",
        "clipboardSubsetBasic",
        "clipboardSubsetTone",
        "clipboardSubsetColor",
        "clipboardSubsetDetail",
        "clipboardSubsetLens",
        "clipboardSubsetGeometry",
        "copyEditClipboard",
        "pasteEditClipboard",
        "syncEditClipboard",
        "editClipboardStatus",
        "editClipboardPlanList",
        "developHistoryPanel",
        "developHistoryStatus",
        "developHistoryList",
        "developUndoHistory",
        "developRedoHistory",
        "developFilmstrip",
        "developEditState",
        "openPreferences",
        "preferencesDialog",
        "preferencesDialogTitle",
        "closePreferencesDialog",
        "preferencesSectionList",
        "preferencesSectionAppearance",
        "preferencesSectionLibrary",
        "preferencesSectionCache",
        "preferencesSectionColor",
        "preferencesSectionExport",
        "preferencesSectionAdvanced",
        "preferencesPanelAppearance",
        "preferencesPanelLibrary",
        "preferencesPanelCache",
        "preferencesPanelColor",
        "preferencesPanelExport",
        "preferencesPanelAdvanced",
        "preferencesAppearanceTheme",
        "preferencesAppearanceDensity",
        "preferencesAppearanceUiScale",
        "preferencesAppearanceScaleValue",
        "preferencesAppearanceReset",
        "preferencesLibraryDefaultPath",
        "preferencesLibraryUseCurrent",
        "preferencesLibraryReset",
        "preferencesLibraryStartupAction",
        "preferencesCacheClear",
        "preferencesCacheRefresh",
        "preferencesColorDefaultSpace",
        "preferencesColorProfilePolicy",
        "preferencesExportDefaultFormat",
        "preferencesExportDefaultQuality",
        "preferencesAdvancedAgentAccess",
        "preferencesAdvancedMcpAccess",
        "preferencesAdvancedPluginRuntime",
        "permissionPromptContract",
        "permissionPromptActor",
        "permissionPromptPermission",
        "permissionPromptSideEffects",
        "permissionPromptConfirmation",
        "permissionPromptUndo",
        "permissionPromptDenyAction",
        "permissionPromptDangerousUnavailable",
        "preferencesStatus",
        "openExportDialog",
        "exportDialog",
        "closeExportDialog",
        "exportSelectedPhotoName",
        "exportOutputPath",
        "runJpegExport",
        "cancelExport",
        "exportStatus",
        "exportSafetyNote",
        "exportBatchProgress",
        "exportBatchProgressLabel",
        "exportBatchFailureList",
        "recentExportsList",
        "recentExportsEmpty",
        "exportPreset",
        "exportPresetName",
        "saveExportPreset",
        "exportFormat",
        "exportColorSpace",
        "exportQuality",
        "exportSummaryPreset",
        "exportSummaryFormat",
        "exportSummaryColor",
        "exportSummaryQuality",
        "exportSummaryMetadata",
        "exportSummaryBatch",
        "exportSummaryFile",
    ]
    for element_id in required_ids:
        require(element_id in parser.ids, f"missing #{element_id}", failures)
    for rating in range(6):
        require(f"ratePhoto{rating}" in parser.ids, f"missing #ratePhoto{rating}", failures)

    require(
        '<ol class="sr-history-list" id="developHistoryList" aria-live="polite"></ol>' in source,
        "#developHistoryList must be empty static markup backed only by runtime checkpoints",
        failures,
    )
    for command in [
        "get_photo_history",
        "undo_last_history_action",
        "redo_last_history_action",
        "get_ai_review_panel",
    ]:
        require(command in source, f"index.html must wire {command}", failures)
    require(
        "disabled" in parser.ids.get("aiReviewApprovalDeferred", {}),
        "#aiReviewApprovalDeferred must stay disabled until explicit AI approval task",
        failures,
    )
    require(
        "Review information only" in source and "does not write edits, flags, or originals" in source,
        "AI review surface must state review-only non-mutating behavior",
        failures,
    )

    require(
        "disabled" in parser.ids.get("openRecent", {}),
        "#openRecent must be disabled until real recents exist",
        failures,
    )
    for forbidden in [
        "demo-",
        "/Volumes/Photography",
        "Tokyo Street Walk",
        "Cafe Reviews",
        "Portrait Session",
        "Open Sample Project",
        "Rate 5",
    ]:
        require(forbidden not in source, f"static UI must not contain fake demo marker: {forbidden}", failures)

    app_frame = parser.ids.get("appFrame", {})
    require(
        app_frame.get("data-active-mode") == "library",
        "#appFrame must default to data-active-mode=\"library\"",
        failures,
    )
    require(
        app_frame.get("data-library-state") == "welcome",
        "#appFrame must default to data-library-state=\"welcome\"",
        failures,
    )

    expected_modes = {"library": "Library", "develop": "Develop", "export": "Export"}
    for mode, label in expected_modes.items():
        button = parser.mode_buttons.get(mode)
        require(button is not None, f"missing mode button for {mode}", failures)
        if button:
            require(
                button["attrs"].get("aria-pressed") in {"true", "false"},
                f"{mode} mode button needs aria-pressed",
                failures,
            )
            require(label in button["text"], f"{mode} mode button label must include {label}", failures)

    require(APP_FRAME_CSS.is_file(), "missing styles/app-frame.css", failures)
    if APP_FRAME_CSS.is_file():
        css = APP_FRAME_CSS.read_text(encoding="utf-8")
        for selector in [
            ".app-frame",
            ".sr-toolbar",
            ".sr-mode-switcher",
            ".sr-sidebar",
            ".sr-main-surface",
            ".sr-inspector",
            ".sr-statusbar",
            ".sr-develop-workbench",
            ".sr-adjustment-slider",
            ".sr-tone-curve-panel",
            ".sr-mask-panel",
            ".sr-mask-row",
            ".sr-detail-panel",
            ".sr-geometry-panel",
            ".sr-edit-clipboard-panel",
            ".sr-preferences-dialog",
            ".sr-preferences-dialog-panel",
            ".sr-preferences-section-list",
            ".sr-preferences-pane",
            ".sr-export-dialog",
            ".sr-export-dialog-panel",
            ".sr-ai-review-surface",
            ".sr-ai-review-layout",
            ".sr-ai-review-card",
            ".sr-ai-review-summary",
        ]:
            require(selector in css, f"app-frame.css missing {selector}", failures)
        require("@media" in css, "app-frame.css must define responsive behavior", failures)
        require(
            re.search(r"#[0-9a-fA-F]{3,8}|rgba?\(", css) is None,
            "app-frame.css must consume color tokens instead of raw color literals",
            failures,
        )

    require(FINAL_VISUAL_QA.is_file(), "missing scripts/harness/run-final-visual-qa.py", failures)
    if FINAL_VISUAL_QA.is_file():
        visual_qa_source = FINAL_VISUAL_QA.read_text(encoding="utf-8")
        for surface_name in [
            "M015-library-filters",
            "M016-library-metadata",
            "M017-develop-history",
            "M018-develop-expanded",
            "M019-mask-editor",
            "M020-preferences-appearance",
            "M021-preferences-advanced",
            "M022-export-workflow",
            "M023-ai-review",
        ]:
            require(
                surface_name in visual_qa_source,
                f"final visual QA must include expanded Phase 22 surface {surface_name}",
                failures,
            )

    preferences_dialog = parser.ids.get("preferencesDialog", {})
    require(
        preferences_dialog.get("role") == "dialog"
        and preferences_dialog.get("aria-modal") == "true"
        and "hidden" in preferences_dialog,
        "#preferencesDialog must be a hidden modal dialog by default",
        failures,
    )
    for section in ["Appearance", "Library", "Cache", "Color", "Export", "Advanced"]:
        require(section in source, f"preferences IA must expose {section}", failures)
    for marker in [
        "function openPreferencesDialog",
        "function closePreferencesDialog",
        "function setPreferencesSection",
        "function normalizeAppearancePreferences",
        "function applyAppearancePreferences",
        "function recordAppearancePreferences",
        "function resetAppearancePreferences",
    ]:
        require(marker in source, f"preferences IA marker missing: {marker}", failures)
    for control_id in [
        "preferencesAppearanceTheme",
        "preferencesAppearanceDensity",
        "preferencesAppearanceUiScale",
        "preferencesAppearanceReset",
    ]:
        require(
            "disabled" not in parser.ids.get(control_id, {}),
            f"#{control_id} must be enabled by Task 21.2",
            failures,
        )
    require(
        parser.ids.get("preferencesAppearanceTheme", {}).get("name") == "preferencesAppearanceTheme"
        and '<option value="dark">Dark</option>' in source
        and '<option value="light">Light</option>' in source,
        "appearance theme preference must expose persisted dark/light choices",
        failures,
    )
    require(
        parser.ids.get("preferencesAppearanceDensity", {}).get("name") == "preferencesAppearanceDensity"
        and '<option value="compact">Compact</option>' in source
        and '<option value="comfortable">Comfortable</option>' in source,
        "appearance density preference must expose compact/comfortable choices",
        failures,
    )
    ui_scale = parser.ids.get("preferencesAppearanceUiScale", {})
    require(
        ui_scale.get("type") == "range"
        and ui_scale.get("min") == "90"
        and ui_scale.get("max") == "120"
        and ui_scale.get("step") == "5",
        "#preferencesAppearanceUiScale must be a bounded range control",
        failures,
    )
    for marker in [
        "record_app_session_appearance",
        "reset_app_session_appearance",
        "data-density",
        "--sr-ui-scale",
    ]:
        require(marker in source or (APP_FRAME_CSS.is_file() and marker in APP_FRAME_CSS.read_text(encoding="utf-8")), f"appearance preference marker missing: {marker}", failures)
    for control_id in [
        "preferencesLibraryDefaultPath",
        "preferencesLibraryUseCurrent",
        "preferencesLibraryReset",
        "preferencesCacheClear",
        "preferencesCacheRefresh",
    ]:
        require(
            "disabled" not in parser.ids.get(control_id, {}),
            f"#{control_id} must be enabled by Task 21.3",
            failures,
        )
    require(
        "disabled" in parser.ids.get("preferencesLibraryStartupAction", {}),
        "#preferencesLibraryStartupAction must stay disabled because Task 21.3 does not change launch behavior",
        failures,
    )
    for marker in [
        "function refreshPreferencesCacheStatus",
        "function runPreferencesCacheClearCommand",
        "function currentLibraryPreferences",
        "function recordLibraryPreferences",
        "function resetLibraryPreferences",
        "get_library_cache_status",
        "clear_library_cache",
        "record_app_session_library_preferences",
        "reset_app_session_library_preferences",
        "response.data.totalBytes",
        "response.data.cacheRecordCount",
    ]:
        require(marker in source, f"library/cache preference marker missing: {marker}", failures)
    for control_id in [
        "preferencesColorDefaultSpace",
        "preferencesExportDefaultFormat",
        "preferencesExportDefaultQuality",
    ]:
        require(
            "disabled" not in parser.ids.get(control_id, {}),
            f"#{control_id} must be enabled by Task 21.4",
            failures,
        )
    require(
        '<option value="srgb">sRGB</option>' in source
        and '<option value="display_p3">Display P3</option>' in source
        and '<option value="jpeg">JPEG</option>' in source
        and '<option value="png">PNG</option>' in source
        and '<option value="tiff">TIFF</option>' in source,
        "Task 21.4 preferences must expose supported export default choices",
        failures,
    )
    for marker in [
        "function applyPreferencesExportSettings",
        "function currentPreferencesExportSettings",
        "function savePreferencesExportDefaults",
        "save_export_settings",
        "get_export_settings",
        "Display P3 is JPEG-only",
    ]:
        require(marker in source, f"color/export preference marker missing: {marker}", failures)
    for control_id in [
        "preferencesColorProfilePolicy",
        "preferencesAdvancedAgentAccess",
        "preferencesAdvancedMcpAccess",
        "preferencesAdvancedPluginRuntime",
    ]:
        require(
            "disabled" in parser.ids.get(control_id, {}),
            f"#{control_id} must stay disabled until its scoped implementation task",
            failures,
        )
    for control_id in [
        "preferencesAdvancedAgentAccess",
        "preferencesAdvancedMcpAccess",
        "preferencesAdvancedPluginRuntime",
    ]:
        attrs = parser.ids.get(control_id, {})
        require(
            attrs.get("type") == "checkbox" and "checked" not in attrs,
            f"#{control_id} must default off",
            failures,
        )
    for marker in [
        "No plugin runtime, MCP server, or agent bridge starts from Preferences.",
        "Permission prompts, Core API boundaries, and action-log evidence are required before future access.",
        "Direct SQLite writes stay forbidden.",
        "Local library, catalog, sidecar, edit, and export actions require explicit permission.",
    ]:
        require(marker in source, f"advanced access preference explanation missing: {marker}", failures)
    for marker in [
        "Permission Prompt Contract",
        "Actor",
        "Requested permission",
        "Side effects",
        "Confirmation",
        "Undo availability",
        "Deny keeps the requested extension disabled and records no grant.",
        "Dangerous permissions are unavailable unless a future ADR approves them.",
        "Not available",
    ]:
        require(marker in source, f"permission prompt contract marker missing: {marker}", failures)
    for promotional_marker in [
        "unlock",
        "supercharge",
        "magic",
        "boost",
    ]:
        require(
            promotional_marker not in source.lower(),
            f"permission prompt copy must stay non-promotional: {promotional_marker}",
            failures,
        )
    for forbidden_marker in [
        'invoke("start_mcp',
        'invoke("start_plugin',
        'invoke("start_agent',
        "create_mcp_server",
        "load_plugin_runtime",
    ]:
        require(forbidden_marker not in source, f"advanced preferences must not start runtime path: {forbidden_marker}", failures)

    exposure_slider = parser.ids.get("developExposureSlider", {})
    require(exposure_slider.get("type") == "range", "#developExposureSlider must be a range input", failures)
    require(exposure_slider.get("min") == "-5", "#developExposureSlider min must match edit graph exposure", failures)
    require(exposure_slider.get("max") == "5", "#developExposureSlider max must match edit graph exposure", failures)
    require(exposure_slider.get("step") == "0.05", "#developExposureSlider step must support precise exposure edits", failures)

    import_recursive = parser.ids.get("importRecursive", {})
    require(
        import_recursive.get("type") == "checkbox",
        "#importRecursive must be a checkbox",
        failures,
    )
    require(
        "checked" not in import_recursive,
        "#importRecursive must default off",
        failures,
    )

    contrast_slider = parser.ids.get("developContrastSlider", {})
    require(contrast_slider.get("type") == "range", "#developContrastSlider must be a range input", failures)
    require(contrast_slider.get("min") == "-100", "#developContrastSlider min must match edit graph contrast", failures)
    require(contrast_slider.get("max") == "100", "#developContrastSlider max must match edit graph contrast", failures)
    require(contrast_slider.get("step") == "1", "#developContrastSlider step must support integer contrast edits", failures)

    mask_panel = parser.ids.get("developMaskPanel", {})
    require(
        mask_panel.get("data-mask-state") == "empty",
        "#developMaskPanel must default to an honest empty mask state",
        failures,
    )
    for control_id in [
        "developMaskToolManual",
        "developMaskToolAI",
        "developMaskToolMLX",
        "developMaskAddMask",
        "developMaskSubjectUnavailable",
        "developMaskSkyUnavailable",
    ]:
        require(
            "disabled" in parser.ids.get(control_id, {}),
            f"#{control_id} must stay disabled until desktop mask creation support exists",
            failures,
        )
    for control_id, (minimum, maximum, step) in {
        "developMaskExposureSlider": ("-5", "5", "0.05"),
        "developMaskContrastSlider": ("-100", "100", "1"),
        "developMaskOpacitySlider": ("0", "100", "1"),
        "developMaskFeatherSlider": ("0", "100", "1"),
    }.items():
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "range", f"#{control_id} must be a range input", failures)
        require("disabled" in attrs, f"#{control_id} must be readback-only in Task 19.5", failures)
        require(attrs.get("min") == minimum, f"#{control_id} min must match mask bounds", failures)
        require(attrs.get("max") == maximum, f"#{control_id} max must match mask bounds", failures)
        require(attrs.get("step") == step, f"#{control_id} step must match mask precision", failures)
    for marker in [
        "Manual",
        "AI unavailable",
        "MLX unavailable",
        "Subject Mask",
        "Sky Mask",
        "RAW masked export blocks before output until RAW decode is implemented.",
    ]:
        require(marker in source, f"Mask editor marker missing: {marker}", failures)

    tone_curve_slider = parser.ids.get("developToneCurveMidpointSlider", {})
    require(tone_curve_slider.get("type") == "range", "#developToneCurveMidpointSlider must be a range input", failures)
    require(tone_curve_slider.get("min") == "0", "#developToneCurveMidpointSlider min must use normalized tone values", failures)
    require(tone_curve_slider.get("max") == "1", "#developToneCurveMidpointSlider max must use normalized tone values", failures)
    require(tone_curve_slider.get("step") == "0.01", "#developToneCurveMidpointSlider step must support point curve edits", failures)
    for control_id in [
        "developToneCurveChannelRed",
        "developToneCurveChannelGreen",
        "developToneCurveChannelBlue",
        "developToneCurveParametric",
    ]:
        require(
            "disabled" in parser.ids.get(control_id, {}),
            f"#{control_id} must remain disabled until runtime support exists",
            failures,
        )

    hsl_channels = {
        "developHslChannelRed": "red",
        "developHslChannelOrange": "orange",
        "developHslChannelYellow": "yellow",
        "developHslChannelGreen": "green",
        "developHslChannelAqua": "aqua",
        "developHslChannelBlue": "blue",
        "developHslChannelPurple": "purple",
        "developHslChannelMagenta": "magenta",
    }
    for control_id, channel in hsl_channels.items():
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("data-hsl-channel") == channel, f"#{control_id} must declare data-hsl-channel={channel}", failures)
        require(attrs.get("type") == "button", f"#{control_id} must be a button", failures)
    for control_id in [
        "developHslHueSlider",
        "developHslSaturationSlider",
        "developHslLuminanceSlider",
    ]:
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "range", f"#{control_id} must be a range input", failures)
        require(attrs.get("min") == "-100", f"#{control_id} min must match HSL edit graph bounds", failures)
        require(attrs.get("max") == "100", f"#{control_id} max must match HSL edit graph bounds", failures)
        require(attrs.get("step") == "1", f"#{control_id} step must support integer HSL edits", failures)

    detail_controls = {
        "developDetailSharpeningAmountSlider": ("0", "150", "1"),
        "developDetailSharpeningRadiusSlider": ("0.1", "5", "0.1"),
        "developDetailSharpeningDetailSlider": ("0", "100", "1"),
        "developDetailSharpeningMaskingSlider": ("0", "100", "1"),
        "developDetailNoiseLuminanceSlider": ("0", "100", "1"),
        "developDetailNoiseDetailSlider": ("0", "100", "1"),
        "developDetailNoiseContrastSlider": ("0", "100", "1"),
        "developDetailNoiseColorSlider": ("0", "100", "1"),
        "developDetailNoiseColorDetailSlider": ("0", "100", "1"),
    }
    for control_id, (minimum, maximum, step) in detail_controls.items():
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "range", f"#{control_id} must be a range input", failures)
        require("disabled" in attrs, f"#{control_id} must stay disabled until renderer support exists", failures)
        require(attrs.get("min") == minimum, f"#{control_id} min must match Detail edit graph bounds", failures)
        require(attrs.get("max") == maximum, f"#{control_id} max must match Detail edit graph bounds", failures)
        require(attrs.get("step") == step, f"#{control_id} step must match Detail edit graph precision", failures)
    for control_id in [
        "developDetailSharpeningAmountValue",
        "developDetailSharpeningRadiusValue",
        "developDetailSharpeningDetailValue",
        "developDetailSharpeningMaskingValue",
        "developDetailNoiseLuminanceValue",
        "developDetailNoiseDetailValue",
        "developDetailNoiseContrastValue",
        "developDetailNoiseColorValue",
        "developDetailNoiseColorDetailValue",
    ]:
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "number", f"#{control_id} must be a numeric readback field", failures)
        require("disabled" in attrs, f"#{control_id} must stay disabled until renderer support exists", failures)
    require(
        "disabled" in parser.ids.get("developDetailMlxDenoise", {}),
        "#developDetailMlxDenoise must stay disabled while MLX denoise is deferred",
        failures,
    )
    for marker in [
        "Detail preview/export is unsupported until renderer support exists.",
        "No Detail pixel effect is enabled in this build.",
        "MLX Denoise",
    ]:
        require(marker in source, f"Detail blocked-state marker missing: {marker}", failures)

    clipboard_controls = {
        "clipboardSubsetBasic": "basic",
        "clipboardSubsetTone": "tone",
        "clipboardSubsetColor": "color",
        "clipboardSubsetDetail": "detail",
        "clipboardSubsetLens": "lens",
        "clipboardSubsetGeometry": "geometry",
    }
    for control_id, section in clipboard_controls.items():
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "checkbox", f"#{control_id} must be a checkbox", failures)
        require(
            attrs.get("data-edit-clipboard-section") == section,
            f"#{control_id} must declare clipboard section {section}",
            failures,
        )
    require(
        "checked" in parser.ids.get("clipboardSubsetBasic", {}),
        "#clipboardSubsetBasic must default on for explicit minimal copy scope",
        failures,
    )
    for control_id in ["clipboardSubsetDetail", "clipboardSubsetLens"]:
        require(
            "disabled" in parser.ids.get(control_id, {}),
            f"#{control_id} must stay disabled until runtime support exists",
            failures,
        )
    for control_id in ["pasteEditClipboard", "syncEditClipboard"]:
        require(
            "disabled" in parser.ids.get(control_id, {}),
            f"#{control_id} must default disabled before a payload exists",
            failures,
        )
    for command in [
        "copy_edit_clipboard_payload",
        "plan_edit_clipboard_sync",
        "apply_edit_clipboard_sync",
    ]:
        require(command in source, f"index.html must wire {command}", failures)
    require(
        "function isJpegDevelopFileType" in source,
        "Develop clipboard UI must gate copy/paste/sync to JPEG/JPG file types",
        failures,
    )
    require(
        "isJpegDevelopFileType(photo?.fileType)" in source,
        "isDevelopable must use the JPEG/JPG file-type gate",
        failures,
    )
    for marker in [
        "Copy &amp; Sync",
        "Edit subsets",
        "selected on this page",
        "Copy reads committed edit state",
        "Batch sync requires at least two selected photos on this page.",
        "Nothing was written.",
    ]:
        require(marker in source, f"Edit clipboard UI marker missing: {marker}", failures)

    geometry_crop_controls = {
        "developGeometryCropXSlider": ("0", "1", "0.01"),
        "developGeometryCropYSlider": ("0", "1", "0.01"),
        "developGeometryCropWidthSlider": ("0.01", "1", "0.01"),
        "developGeometryCropHeightSlider": ("0.01", "1", "0.01"),
    }
    for control_id, (minimum, maximum, step) in geometry_crop_controls.items():
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "range", f"#{control_id} must be a range input", failures)
        require(attrs.get("min") == minimum, f"#{control_id} min must match normalized crop bounds", failures)
        require(attrs.get("max") == maximum, f"#{control_id} max must match normalized crop bounds", failures)
        require(attrs.get("step") == step, f"#{control_id} step must support normalized crop precision", failures)
    for control_id in [
        "developGeometryCropXValue",
        "developGeometryCropYValue",
        "developGeometryCropWidthValue",
        "developGeometryCropHeightValue",
    ]:
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "number", f"#{control_id} must be a numeric crop input", failures)
    for control_id in [
        "developGeometryCropClear",
        "developGeometryRotateLeft",
        "developGeometryRotateRight",
        "developGeometryOrientationReset",
        "developGeometryFlipHorizontal",
        "developGeometryFlipVertical",
    ]:
        attrs = parser.ids.get(control_id, {})
        require(attrs.get("type") == "button", f"#{control_id} must be a button", failures)
    require(
        parser.ids.get("developGeometryRotateLeft", {}).get("data-geometry-rotate") == "-90",
        "#developGeometryRotateLeft must declare a supported quarter-turn rotation",
        failures,
    )
    require(
        parser.ids.get("developGeometryRotateRight", {}).get("data-geometry-rotate") == "90",
        "#developGeometryRotateRight must declare a supported quarter-turn rotation",
        failures,
    )
    for control_id in [
        "developLensProfileCorrection",
        "developLensChromaticAberration",
        "developLensDistortionSlider",
        "developLensVignettingSlider",
        "developGeometryTransformScaleSlider",
        "developGeometryTransformVerticalSlider",
        "developGeometryTransformHorizontalSlider",
    ]:
        attrs = parser.ids.get(control_id, {})
        require("disabled" in attrs, f"#{control_id} must stay disabled until runtime support exists", failures)
    for marker in [
        "Lens &amp; Geometry",
        "Geometry Ready",
        "Lens correction unavailable.",
        "Transform unsupported.",
        "Crop",
        "Rotate",
        "Flip",
    ]:
        require(marker in source, f"Lens/Geometry panel marker missing: {marker}", failures)

    expected_native_hosts = {
        "loupeViewer": "loupe",
        "developPreviewSurface": "develop",
    }
    require(
        set(parser.native_viewer_hosts) == set(expected_native_hosts),
        "reserved native viewer hosts must be exactly Loupe and Develop viewer surfaces",
        failures,
    )
    for host_id, surface in expected_native_hosts.items():
        host_attrs = parser.native_viewer_hosts.get(host_id, {})
        require(
            host_attrs.get("data-native-viewer-surface") == surface,
            f"#{host_id} must report native viewer surface {surface}",
            failures,
        )
        require(
            host_attrs.get("data-native-viewer-state") == "web-fallback",
            f"#{host_id} must default to web-fallback while native viewer is feature-gated",
            failures,
        )
        require(
            host_attrs.get("data-native-viewer-controls") == "external",
            f"#{host_id} must declare controls outside the reserved native viewer host",
            failures,
        )

    interactive_viewer_controls = {
        "closeLoupe",
        "loupeFitMode",
        "developExposureSlider",
        "developExposureValue",
        "developExposureReset",
        "developContrastSlider",
        "developContrastValue",
        "developContrastReset",
        "developToneCurveMidpointSlider",
        "developToneCurveMidpointValue",
        "developToneCurveReset",
        "developHslHueSlider",
        "developHslHueValue",
        "developHslHueReset",
        "developHslSaturationSlider",
        "developHslSaturationValue",
        "developHslSaturationReset",
        "developHslLuminanceSlider",
        "developHslLuminanceValue",
        "developHslLuminanceReset",
        "developGeometryCropXSlider",
        "developGeometryCropXValue",
        "developGeometryCropYSlider",
        "developGeometryCropYValue",
        "developGeometryCropWidthSlider",
        "developGeometryCropWidthValue",
        "developGeometryCropHeightSlider",
        "developGeometryCropHeightValue",
        "developGeometryCropClear",
        "developGeometryRotateLeft",
        "developGeometryRotateRight",
        "developGeometryOrientationReset",
        "developGeometryFlipHorizontal",
        "developGeometryFlipVertical",
        "developCommitEdit",
        "developRevertEdit",
        "openExportDialog",
        "exportDialog",
    }
    for control_id in interactive_viewer_controls:
        require(
            control_id not in parser.native_viewer_descendant_ids,
            f"#{control_id} must remain outside reserved native viewer hosts",
            failures,
        )
    require(
        "function viewerHostGeometry" in source,
        "index.html must expose viewerHostGeometry for native bridge geometry reporting",
        failures,
    )
    require(
        "function currentNativeViewerHostGeometry" in source,
        "index.html must expose currentNativeViewerHostGeometry",
        failures,
    )
    require(
        "window.SilicaRAWViewerHost" in source,
        "index.html must expose inert feature-off native viewer host API",
        failures,
    )

    require(
        parser.ids.get("developExposureValue", {}).get("type") == "number",
        "#developExposureValue must be a manual numeric input",
        failures,
    )
    require(
        parser.ids.get("developContrastValue", {}).get("type") == "number",
        "#developContrastValue must be a manual numeric input",
        failures,
    )

    require(
        parser.ids.get("exportOutputPath", {}).get("type") == "text",
        "#exportOutputPath must be a text input for local alpha output paths",
        failures,
    )
    require(
        parser.ids.get("exportFormat", {}).get("name") == "exportFormat"
        and '<option value="jpeg">JPEG</option>' in source
        and '<option value="png">PNG</option>' in source
        and '<option value="tiff">TIFF</option>' in source,
        "#exportFormat must expose JPEG, PNG, and TIFF export options",
        failures,
    )
    require(
        parser.ids.get("exportColorSpace", {}).get("value") == "sRGB",
        "#exportColorSpace must lock the MVP to sRGB",
        failures,
    )
    require(
        parser.ids.get("exportQuality", {}).get("value") == "90",
        "#exportQuality must expose local-alpha JPEG default quality",
        failures,
    )
    require(
        parser.ids.get("exportMetadataPolicy", {}).get("name") == "exportMetadataPolicy"
        and '<option value="minimal">Minimal</option>' in source
        and '<option value="preserve">Preserve</option>' in source
        and '<option value="remove_gps">Remove GPS</option>' in source
        and '<option value="remove_all">Remove All</option>' in source,
        "#exportMetadataPolicy must expose explicit metadata policy options",
        failures,
    )
    require(
        parser.ids.get("exportPreset", {}).get("name") == "exportPreset",
        "#exportPreset must expose the current export preset",
        failures,
    )
    require(
        parser.ids.get("exportBatchProgress", {}).get("max") == "100"
        and parser.ids.get("exportBatchProgress", {}).get("value") == "0"
        and "function runBatchExport" in source
        and "exportBatchFailures" in source
        and "function loadRecentExports" in source,
        "export workflow must expose batch progress, failures, and recent export loading",
        failures,
    )
    require(
        parser.ids.get("exportPresetName", {}).get("type") == "text",
        "#exportPresetName must allow editing a local preset name",
        failures,
    )

    require(BASE_CSS.is_file(), "missing styles/base.css", failures)
    if BASE_CSS.is_file():
        css = BASE_CSS.read_text(encoding="utf-8")
        require("[hidden]" in css, "base.css must preserve native hidden behavior", failures)

    if failures:
        for failure in failures:
            print(f"static-ui check failed: {failure}", file=sys.stderr)
        return 1

    print("static UI contract ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
