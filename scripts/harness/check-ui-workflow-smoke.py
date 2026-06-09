#!/usr/bin/env python3
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "apps/desktop/static/index.html"

ALLOWED_COMMANDS = {
    "create_library",
    "open_library",
    "import_folder",
    "list_library_photos",
    "set_photo_flags",
    "open_photo_preview",
    "preview_exposure_contrast_edit",
    "commit_exposure_contrast_edit",
    "export_photo_jpeg_srgb",
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
            "importStatus",
            "importSafetyNote",
        ],
        "commands": ["import_folder"],
        "text": [
            "Import by Reference",
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
            "selectedPhotoName",
            "selectedPhotoRating",
            "rateSelectedPhoto",
            "pickSelectedPhoto",
            "rejectSelectedPhoto",
        ],
        "commands": ["list_library_photos", "set_photo_flags"],
        "text": ["All Photos", "Pick", "Reject"],
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
        "name": "develop-exposure-contrast",
        "ids": [
            "modeDevelop",
            "developPreviewSurface",
            "developExposureSlider",
            "developContrastSlider",
            "developCommitEdit",
            "developRevertEdit",
            "developEditState",
        ],
        "commands": [
            "preview_exposure_contrast_edit",
            "commit_exposure_contrast_edit",
        ],
        "text": ["Draft not committed", "Commit Edit", "Revert Draft"],
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
            "exportQuality",
        ],
        "commands": ["export_photo_jpeg_srgb"],
        "text": [
            "JPEG",
            "sRGB",
            "Original files will not be modified",
            "Output path must differ from the original source path.",
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
        "thumbnailPath",
        "thumbnailBytes",
        "renderThumbnailArt",
        "URL.createObjectURL",
        "sr-thumb-image",
    ]:
        require(marker in source, f"real thumbnail grid marker missing: {marker}", failures)
    for marker in [
        "previewBytes",
        "loupeObjectUrls",
        "renderLoupePreviewImage",
        "sr-loupe-image",
    ]:
        require(marker in source, f"real loupe preview marker missing: {marker}", failures)

    workflow_order = [
        "setLibraryState(\"open\")",
        "runImportCommand",
        "renderLibraryGrid",
        "applySelectedFlags",
        "openLoupeView",
        "previewDevelopEdit",
        "commitDevelopEdit",
        "runJpegExport",
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
        parser.ids.get("exportFormat", {}).get("value") == "JPEG"
        and parser.ids.get("exportColorSpace", {}).get("value") == "sRGB"
        and parser.ids.get("exportQuality", {}).get("value") == "90",
        "export workflow must lock local alpha to JPEG sRGB quality 90",
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
