#!/usr/bin/env python3
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "apps/desktop/static/index.html"
BASE_CSS = ROOT / "apps/desktop/static/styles/base.css"
APP_FRAME_CSS = ROOT / "apps/desktop/static/styles/app-frame.css"


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
        "libraryGrid",
        "gridEmptyState",
        "gridLoadingState",
        "gridStateNote",
        "libraryPhotoCount",
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
        "developCommitEdit",
        "developRevertEdit",
        "developHistoryPanel",
        "developHistoryStatus",
        "developHistoryList",
        "developUndoHistory",
        "developRedoHistory",
        "developFilmstrip",
        "developEditState",
        "openExportDialog",
        "exportDialog",
        "closeExportDialog",
        "exportSelectedPhotoName",
        "exportOutputPath",
        "runJpegExport",
        "cancelExport",
        "exportStatus",
        "exportSafetyNote",
        "exportFormat",
        "exportColorSpace",
        "exportQuality",
        "exportSummaryFormat",
        "exportSummaryColor",
        "exportSummaryQuality",
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
    ]:
        require(command in source, f"index.html must wire {command}", failures)

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
            ".sr-export-dialog",
            ".sr-export-dialog-panel",
        ]:
            require(selector in css, f"app-frame.css missing {selector}", failures)
        require("@media" in css, "app-frame.css must define responsive behavior", failures)
        require(
            re.search(r"#[0-9a-fA-F]{3,8}|rgba?\(", css) is None,
            "app-frame.css must consume color tokens instead of raw color literals",
            failures,
        )

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
        parser.ids.get("exportFormat", {}).get("value") == "JPEG",
        "#exportFormat must lock the MVP to JPEG",
        failures,
    )
    require(
        parser.ids.get("exportColorSpace", {}).get("value") == "sRGB",
        "#exportColorSpace must lock the MVP to sRGB",
        failures,
    )
    require(
        parser.ids.get("exportQuality", {}).get("value") == "90",
        "#exportQuality must expose local-alpha JPEG quality",
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
