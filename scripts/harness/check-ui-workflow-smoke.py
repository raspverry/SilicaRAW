#!/usr/bin/env python3
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "apps/desktop/static/index.html"

ALLOWED_COMMANDS = {
    "resolve_launch_restore",
    "record_app_session_selection",
    "record_app_session_layout",
    "reset_app_session_layout",
    "read_app_session",
    "create_library",
    "open_library",
    "import_folder",
    "list_library_photos",
    "query_library_photos",
    "set_photo_flags",
    "get_photo_metadata",
    "open_photo_preview",
    "preview_exposure_contrast_edit",
    "preview_white_balance_edit",
    "preview_tone_recovery_edit",
    "preview_color_presence_edit",
    "preview_tone_curve_edit",
    "preview_hsl_color_mixer_edit",
    "commit_exposure_contrast_edit",
    "commit_white_balance_edit",
    "commit_tone_recovery_edit",
    "commit_color_presence_edit",
    "commit_tone_curve_edit",
    "commit_hsl_color_mixer_edit",
    "commit_p0_basic_reset",
    "commit_basic_preset_edit",
    "get_photo_edit_state",
    "get_photo_histogram",
    "get_photo_history",
    "undo_last_history_action",
    "redo_last_history_action",
    "export_photo_jpeg_srgb",
    "export_photo_jpeg",
    "clear_library_cache",
}

WORKFLOW_STEPS = [
    {
        "name": "open-or-create-library",
        "ids": [
            "libraryPath",
            "chooseLibraryPath",
            "openLibrary",
            "createLibrary",
            "welcomeStatus",
        ],
        "commands": ["open_library", "create_library"],
        "text": ["Open Folder", "Create Library"],
    },
    {
        "name": "import-by-reference",
        "ids": [
            "importPanel",
            "importFolderPath",
            "chooseImportFolderPath",
            "startImport",
            "importRecursive",
            "importStatus",
            "importSafetyNote",
            "importIssueReview",
            "importIssueReviewSummary",
            "importIssueList",
            "closeImportIssues",
        ],
        "commands": ["import_folder"],
        "text": [
            "Import by Reference",
            "Include subfolders",
            "Recursive import stays off unless selected",
            "Review Issues",
            "Import issue review",
            "Unsupported file",
            "Skipped entry",
            "Failed entry",
            "Original files will stay in place",
            "records references only",
        ],
    },
    {
        "name": "browse-and-cull-grid",
        "ids": [
            "libraryGrid",
            "gridEmptyState",
            "gridLoadingState",
            "gridErrorState",
            "gridPageStatus",
            "gridPreviousPage",
            "gridNextPage",
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
            "metadataFilter",
            "ratingControlGroup",
            "ratePhoto0",
            "ratePhoto1",
            "ratePhoto2",
            "ratePhoto3",
            "ratePhoto4",
            "ratePhoto5",
            "clearCullingFlags",
            "pickSelectedPhoto",
            "rejectSelectedPhoto",
            "cullingStatus",
        ],
        "commands": ["query_library_photos", "set_photo_flags", "get_photo_metadata"],
        "text": [
            "All Photos",
            "Has Dimensions",
            "Camera/lens unavailable",
            "Dimensions",
            "Camera",
            "Unavailable",
            "Rating",
            "Pick",
            "Reject",
            "Clear",
        ],
    },
    {
        "name": "preview-loupe",
        "ids": [
            "openLoupe",
            "loupeSurface",
            "loupeViewer",
            "loupePreviewStatus",
            "loupeFilmstrip",
        ],
        "commands": ["open_photo_preview"],
        "text": ["Blocked by Decode", "Unsupported"],
    },
    {
        "name": "develop-basic-controls",
        "ids": [
            "modeDevelop",
            "developPreviewSurface",
            "developBeforeView",
            "developAfterView",
            "developExposureSlider",
            "developContrastSlider",
            "developWhiteBalanceMode",
            "developTemperatureSlider",
            "developTintSlider",
            "developHighlightsSlider",
            "developShadowsSlider",
            "developWhitesSlider",
            "developBlacksSlider",
            "developVibranceSlider",
            "developSaturationSlider",
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
            "developCommitEdit",
            "developRevertEdit",
            "developResetBasic",
            "presetSilicaNeutral",
            "presetWarmContrast",
            "presetSoftMatte",
            "developEditState",
            "developHistoryPanel",
            "developHistoryStatus",
            "developHistoryList",
            "developUndoHistory",
            "developRedoHistory",
            "photoHistogram",
            "photoHistogramBars",
            "photoHistogramStatus",
        ],
        "commands": [
            "get_photo_edit_state",
            "get_photo_histogram",
            "get_photo_history",
            "undo_last_history_action",
            "redo_last_history_action",
            "preview_exposure_contrast_edit",
            "preview_white_balance_edit",
            "preview_tone_recovery_edit",
            "preview_color_presence_edit",
            "preview_tone_curve_edit",
            "preview_hsl_color_mixer_edit",
            "commit_exposure_contrast_edit",
            "commit_white_balance_edit",
            "commit_tone_recovery_edit",
            "commit_color_presence_edit",
            "commit_tone_curve_edit",
            "commit_hsl_color_mixer_edit",
            "commit_p0_basic_reset",
            "commit_basic_preset_edit",
        ],
        "text": [
            "Draft not committed",
            "Before",
            "After",
            "Commit Edit",
            "Revert Draft",
            "Reset All",
            "Tone Curve",
            "RGB Midpoint",
            "Point RGB",
            "HSL Mixer",
            "Hue",
            "Luminance",
            "History",
            "No committed history yet.",
        ],
    },
    {
        "name": "export-jpeg-srgb",
        "ids": [
            "modeExport",
            "openExportDialog",
            "exportDialog",
            "exportOutputPath",
            "chooseExportOutputPath",
            "runJpegExport",
            "exportStatus",
            "exportSafetyNote",
            "exportFormat",
            "exportColorSpace",
            "exportColorSrgb",
            "exportColorDisplayP3",
            "exportColorProof",
            "exportQuality",
        ],
        "commands": ["export_photo_jpeg"],
        "text": [
            "JPEG",
            "sRGB",
            "Display P3",
            "explicit selection",
            "Original files will not be modified",
            "Output path must differ from the original source path.",
        ],
    },
    {
        "name": "maintenance-cache-clear",
        "ids": [
            "cacheMaintenance",
            "clearLibraryCache",
            "cacheClearScope",
            "cacheClearStatus",
        ],
        "commands": ["clear_library_cache"],
        "text": [
            "Clear Disposable Caches",
            "thumbnails, previews, render-cache, and ai-cache only",
            "Catalog, edits, exports, originals, sidecars, backups, and logs stay intact",
        ],
    },
]


class WorkflowParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = {}
        self.text_parts = []

    def handle_starttag(self, _tag, attrs):
        attrs = dict(attrs)
        if "id" in attrs:
            self.ids[attrs["id"]] = attrs
        for value in attrs.values():
            if value:
                self.text_parts.append(value)

    def handle_data(self, data):
        if data.strip():
            self.text_parts.append(data.strip())


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def command_names(source):
    direct_invokes = set(re.findall(r"""invoke\(\s*["']([^"']+)["']""", source))
    delegated_library_commands = set(
        re.findall(r"""runLibraryCommand\(\s*["']([^"']+)["']""", source)
    )
    return direct_invokes | delegated_library_commands


def has_text(text_blob, expected):
    return expected in text_blob


def main():
    failures = []

    if not INDEX.is_file():
        print("ui-workflow-smoke failed: missing apps/desktop/static/index.html", file=sys.stderr)
        return 1

    source = INDEX.read_text(encoding="utf-8")
    parser = WorkflowParser()
    parser.feed(source)
    text_blob = "\n".join(parser.text_parts)
    commands = command_names(source)

    unknown_commands = sorted(commands - ALLOWED_COMMANDS)
    require(
        not unknown_commands,
        f"unexpected Tauri command wiring: {', '.join(unknown_commands)}",
        failures,
    )

    for step in WORKFLOW_STEPS:
        for element_id in step["ids"]:
            require(element_id in parser.ids, f"{step['name']} missing #{element_id}", failures)
        for command in step["commands"]:
            require(command in commands, f"{step['name']} missing invoke(\"{command}\")", failures)
        for text in step["text"]:
            require(has_text(text_blob, text), f"{step['name']} missing text: {text}", failures)

    require(
        "await invoke(command, { path })" in source,
        "open/create library workflow must delegate selected command into Tauri invoke",
        failures,
    )
    for forbidden in [
        "parsePreviewField",
        "readImportNumber",
        "JSON.parse(message)",
        "const message = await invoke",
    ]:
        require(
            forbidden not in source,
            f"UI runtime commands must use structured response fields, not {forbidden}",
            failures,
        )
    for marker in [
        "responseMessage",
        "responseErrorMessage",
        "commandFailed",
        "response.data?.photos",
        "response.data?.supportedFiles",
        "response.data.outputPath",
        "response.data.status",
    ]:
        require(
            marker in source,
            f"structured command response marker missing: {marker}",
            failures,
        )
    require(
        'createButton.addEventListener("click", () => runLibraryCommand("create_library"))'
        in source,
        "create library button must call runLibraryCommand(\"create_library\")",
        failures,
    )
    require(
        'openButton.addEventListener("click", () => runLibraryCommand("open_library"))'
        in source,
        "open library button must call runLibraryCommand(\"open_library\")",
        failures,
        )
    for marker in [
        "const dialogApi = window.__TAURI__?.dialog",
        "async function chooseDirectoryPath",
        "async function chooseExportPath",
        "chooseDirectoryPath(\"library\")",
        "chooseDirectoryPath(\"import\")",
        "chooseExportPath()",
        "Path selection canceled.",
    ]:
        require(marker in source, f"native path picker marker missing: {marker}", failures)
    for marker in [
        "data-import-step-progress",
        "data-import-step-output",
        "setImportStepState",
        "lastImportIssues",
        "normalizeImportIssues",
        "renderImportIssueReview",
        "openImportIssueReview",
        "closeImportIssueReview",
        "countImportErrorIssues",
        "importRecursiveInput",
        "recursive: importRecursiveInput.checked",
    ]:
        require(marker in source, f"import progress step marker missing: {marker}", failures)
    require(
        "Unsupported and failed files will be reviewable after import." not in source,
        "import issue review must replace placeholder View Errors behavior",
        failures,
    )
    for marker in [
        "thumbnailPath",
        "thumbnailBytes",
        "renderThumbnailArt",
        "URL.createObjectURL",
        "sr-thumb-image",
        "data-page-scoped-thumbnail-request",
        "libraryGridPageRequest",
        "query_library_photos",
        "renderGridError",
        "updateGridPageControls",
        "gridPageStatus",
        "data-virtual-grid-window",
        "renderVirtualGridWindow",
        "releaseGridThumbnailObjectUrls",
        "data-grid-keyboard-navigation",
        "handleLibraryGridKeydown",
        "moveGridSelectionToIndex",
        "multiSelectionCount",
        "selectPhotoRange",
        "togglePhotoSelection",
        "clearMultiSelection",
    ]:
        require(marker in source, f"real thumbnail grid marker missing: {marker}", failures)
    for marker in [
        "previewBytes",
        "loupeObjectUrls",
        "renderLoupePreviewImage",
        "sr-loupe-image",
    ]:
        require(marker in source, f"real loupe preview marker missing: {marker}", failures)
    for marker in [
        "developPreviewBytes",
        "developObjectUrls",
        "renderDevelopPreviewImage",
        "sr-develop-image",
    ]:
        require(marker in source, f"real develop preview marker missing: {marker}", failures)
    for marker in [
        "get_photo_histogram",
        "renderPhotoHistogram",
        "loadPhotoHistogram",
        "sr-histogram-bar",
        "data-histogram-state",
    ]:
        require(marker in source, f"real histogram marker missing: {marker}", failures)
    for marker in [
        "renderExportPreviewImage",
        "sr-export-preview-image",
    ]:
        require(marker in source, f"real export preview marker missing: {marker}", failures)
    for marker in [
        "readPersistedDevelopState",
        "get_photo_edit_state",
        "readbackLoaded",
        "Restored committed edit state.",
    ]:
        require(marker in source, f"persisted develop readback marker missing: {marker}", failures)
    for marker in [
        "loadDevelopHistory",
        "get_photo_history",
        "runDevelopHistoryCommand",
        "undo_last_history_action",
        "redo_last_history_action",
        "developHistoryList.replaceChildren",
        "Desktop runtime is required to load committed history.",
        "Next undo",
        "Next redo",
    ]:
        require(marker in source, f"develop history marker missing: {marker}", failures)
    for marker in [
        "setDevelopBeforeAfterMode",
        "data-before-after-mode",
        "applyBasicPreset",
        "commit_basic_preset_edit",
        "resetP0BasicControls",
        "commit_p0_basic_reset",
        "Preset/reset applies one undoable edit checkpoint.",
    ]:
        require(marker in source, f"develop preset/reset/before-after marker missing: {marker}", failures)
    for marker in [
        "runCacheClearCommand",
        "clear_library_cache",
        "response.data.clearedDirectories",
        "response.data.removedCacheRecords",
        "Cache clear removed only disposable library caches.",
    ]:
        require(marker in source, f"cache clear marker missing: {marker}", failures)
    for marker in [
        "recentEmptyState",
        "recentLibraryList",
        "renderRecentLibraries",
        "openRecentLibrary",
        "resolveLaunchRestore",
        "resolve_launch_restore",
        "fallbackReason",
        "resolvedMode",
        "selectedPhotoStatus",
        "recordActiveSessionState",
        "record_app_session_selection",
        "applyLayoutPreferences",
        "recordLayoutPreferences",
        "resetLayoutPreferences",
        "record_app_session_layout",
        "reset_app_session_layout",
        "sidebarCollapsed",
        "thumbnailSize",
        "read_app_session",
        "Unavailable",
        "No recent libraries yet",
        "setSelectedRating",
        "toggleSelectedPick",
        "toggleSelectedReject",
        "updateCullingControls",
        "RAW decode is not enabled in this alpha",
    ]:
        require(marker in source, f"demo removal/culling marker missing: {marker}", failures)
    for forbidden in [
        "demo-",
        "/Volumes/Photography",
        "Tokyo Street Walk",
        "Cafe Reviews",
        "Portrait Session",
        "Open Sample Project",
        "Rate 5",
    ]:
        require(forbidden not in source, f"fake demo state marker must be removed: {forbidden}", failures)

    workflow_order = [
        "setLibraryState(\"open\")",
        "runImportCommand",
        "renderLibraryGrid",
        "applySelectedFlags",
        "openLoupeView",
        "previewDevelopEdit",
        "commitDevelopEdit",
        "runJpegExport",
        "runCacheClearCommand",
    ]
    for marker in workflow_order:
        require(marker in source, f"workflow marker missing: {marker}", failures)

    require(
        "exportOutputLooksUnsafe" in source and "Output path must differ" in source,
        "export workflow must block writing over the referenced original path",
        failures,
    )
    require(
        "Desktop runtime unavailable; updated local grid preview." in source,
        "static culling path must state when desktop runtime is unavailable",
        failures,
    )
    require(
        "desktop runtime is required to persist edits" in source,
        "static develop path must not claim catalog persistence",
        failures,
    )
    require(
        "desktop runtime writes the JPEG and catalog record" in source,
        "static export path must not claim to write files",
        failures,
    )
    require(
        parser.ids.get("developExposureSlider", {}).get("min") == "-5"
        and parser.ids.get("developExposureSlider", {}).get("max") == "5"
        and parser.ids.get("developExposureSlider", {}).get("step") == "0.05",
        "develop exposure slider must use edit graph bounds -5..5 step 0.05",
        failures,
    )
    require(
        parser.ids.get("developContrastSlider", {}).get("min") == "-100"
        and parser.ids.get("developContrastSlider", {}).get("max") == "100"
        and parser.ids.get("developContrastSlider", {}).get("step") == "1",
        "develop contrast slider must use edit graph bounds -100..100 step 1",
        failures,
    )
    require(
        parser.ids.get("developToneCurveMidpointSlider", {}).get("min") == "0"
        and parser.ids.get("developToneCurveMidpointSlider", {}).get("max") == "1"
        and parser.ids.get("developToneCurveMidpointSlider", {}).get("step") == "0.01",
        "tone curve midpoint slider must use normalized 0..1 point bounds",
        failures,
    )
    for control_id in [
        "developToneCurveChannelRed",
        "developToneCurveChannelGreen",
        "developToneCurveChannelBlue",
        "developToneCurveParametric",
    ]:
        require(
            "disabled" in parser.ids.get(control_id, {}),
            f"#{control_id} must stay disabled until end-to-end support exists",
            failures,
        )
    require(
        parser.ids.get("developHslHueSlider", {}).get("min") == "-100"
        and parser.ids.get("developHslHueSlider", {}).get("max") == "100"
        and parser.ids.get("developHslHueSlider", {}).get("step") == "1"
        and parser.ids.get("developHslSaturationSlider", {}).get("min") == "-100"
        and parser.ids.get("developHslSaturationSlider", {}).get("max") == "100"
        and parser.ids.get("developHslSaturationSlider", {}).get("step") == "1"
        and parser.ids.get("developHslLuminanceSlider", {}).get("min") == "-100"
        and parser.ids.get("developHslLuminanceSlider", {}).get("max") == "100"
        and parser.ids.get("developHslLuminanceSlider", {}).get("step") == "1",
        "HSL hue/saturation/luminance sliders must use edit graph bounds -100..100 step 1",
        failures,
    )
    require(
        parser.ids.get("exportFormat", {}).get("value") == "JPEG"
        and parser.ids.get("exportColorSpace", {}).get("value") == "sRGB"
        and parser.ids.get("exportQuality", {}).get("value") == "90",
        "export workflow must keep JPEG quality 90 and sRGB as default",
        failures,
    )
    require(
        parser.ids.get("exportColorSrgb", {}).get("data-export-color") == "srgb"
        and parser.ids.get("exportColorDisplayP3", {}).get("data-export-color") == "display_p3",
        "export workflow must expose explicit sRGB and Display P3 color choices",
        failures,
    )

    if failures:
        for failure in failures:
            print(f"ui-workflow-smoke failed: {failure}", file=sys.stderr)
        return 1

    print("ui workflow smoke ok: open/create -> import -> grid/cull -> loupe -> develop -> export")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
