#!/usr/bin/env python3
import argparse
import json
import shutil
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ARTIFACTS = ROOT / ".tmp/final-visual-responsive-qa"
FIXTURES = ARTIFACTS / "fixtures"
SCREENSHOTS = ARTIFACTS / "screenshots"
RESULTS = ARTIFACTS / "visual-qa-results.json"
SESSION = "silicaraw-final-visual-qa"

VIEWPORTS = [
    ("compact-1280", 1280, 800),
    ("desktop-1440", 1440, 900),
    ("large-1728", 1728, 965),
]

SURFACES = [
    ("M001-welcome", "welcome"),
    ("M002-library-empty", "empty"),
    ("M003-library-populated", "grid"),
    ("M004-loupe", "loupe"),
    ("M005-develop", "develop"),
    ("M007-export", "export"),
    ("M008-minimal-maintenance", "maintenance"),
    ("M009-import-progress", "import"),
    ("M013-import-issue-review", "import-review"),
    ("M010-layout-sidebar-collapsed", "layout-sidebar-collapsed"),
    ("M011-layout-inspector-collapsed", "layout-inspector-collapsed"),
    ("M012-layout-reset", "layout-reset"),
]


def run(command, **kwargs):
    result = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
        **kwargs,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(str(part) for part in command)}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result.stdout.strip()


def find_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def wait_for_server(url):
    for _ in range(60):
        try:
            with urllib.request.urlopen(url, timeout=1):
                return
        except Exception:
            time.sleep(0.1)
    raise RuntimeError(f"server did not become ready: {url}")


def agent(*args):
    return run(["agent-browser", "--session", SESSION, *args])


def eval_json(script):
    output = agent("eval", script)
    return json.loads(output)


def generate_fixtures():
    run(
        [
            "python3",
            "scripts/harness/generate-legal-fixtures.py",
            "--output",
            str(FIXTURES.relative_to(ROOT)),
            "--include-raw-placeholders",
        ]
    )


def state_script(state):
    image_path = "/.tmp/final-visual-responsive-qa/fixtures/supported/synthetic-gradient.jpg"
    secondary_path = "/.tmp/final-visual-responsive-qa/fixtures/supported/synthetic-checker.jpeg"
    return f"""
(() => {{
  const state = {json.dumps(state)};
  const imagePath = {json.dumps(image_path)};
  const secondaryPath = {json.dumps(secondary_path)};
  const app = document.querySelector("#appFrame");
  const gridShell = document.querySelector(".sr-library-grid-shell");
  const grid = document.querySelector("#libraryGrid");
  const empty = document.querySelector("#gridEmptyState");
  const loading = document.querySelector("#gridLoadingState");
  const importPanel = document.querySelector("#importPanel");
  const loupeSurface = document.querySelector("#loupeSurface");
  const loupeViewer = document.querySelector("#loupeViewer");
  const developSurface = document.querySelector("#developPreviewSurface");
  const exportDialog = document.querySelector("#exportDialog");

  function setLayout(sidebarCollapsed, inspectorCollapsed, filmstripVisible, thumbnailSize = 168) {{
    app.dataset.sidebarCollapsed = String(sidebarCollapsed);
    app.dataset.inspectorCollapsed = String(inspectorCollapsed);
    app.dataset.filmstripVisible = String(filmstripVisible);
    document.querySelector("#toggleSidebar")?.setAttribute("aria-pressed", String(sidebarCollapsed));
    document.querySelector("#toggleInspector")?.setAttribute("aria-pressed", String(inspectorCollapsed));
    document.querySelector("#toggleFilmstrip")?.setAttribute("aria-pressed", String(filmstripVisible));
    const thumbnail = document.querySelector("#thumbnailSize");
    if (thumbnail) {{
      thumbnail.value = String(thumbnailSize);
    }}
    grid.style.setProperty("--sr-thumb-min", `${{thumbnailSize}}px`);
    grid.style.gridTemplateColumns = `repeat(auto-fill, minmax(${{thumbnailSize}}px, 1fr))`;
  }}

  function setMode(mode) {{
    app.dataset.activeMode = mode;
    document.querySelectorAll("[data-mode]").forEach((button) => {{
      button.setAttribute("aria-pressed", String(button.dataset.mode === mode));
    }});
    document.querySelectorAll("[data-mode-panel]").forEach((panel) => {{
      panel.hidden = panel.dataset.modePanel !== mode;
    }});
  }}

  function setLibraryState(libraryState) {{
    app.dataset.libraryState = libraryState;
    document.querySelectorAll("[data-library-state-panel]").forEach((panel) => {{
      panel.hidden = panel.dataset.libraryStatePanel !== libraryState;
    }});
  }}

  function thumb(src, className = "sr-thumb-image") {{
    const image = document.createElement("img");
    image.className = className;
    image.src = src;
    image.alt = "";
    image.draggable = false;
    return image;
  }}

  function photoCard(photo, index) {{
    const card = document.createElement("button");
    card.className = "sr-photo-card";
    card.type = "button";
    card.classList.toggle("is-selected", index === 0);
    card.classList.toggle("is-rejected", Boolean(photo.rejected));
    if (photo.src) {{
      card.append(thumb(photo.src));
    }} else {{
      const art = document.createElement("span");
      art.className = `sr-thumb-art ${{photo.artClass || "sr-thumb-art-a"}}`;
      card.append(art);
    }}
    const badge = document.createElement("span");
    badge.className = "sr-file-badge";
    badge.textContent = photo.fileType;
    card.append(badge);
    if (photo.state) {{
      const stateBadge = document.createElement("span");
      stateBadge.className = `sr-card-state ${{photo.stateClass || ""}}`.trim();
      stateBadge.textContent = photo.state;
      card.append(stateBadge);
    }}
    const footer = document.createElement("span");
    footer.className = "sr-card-footer";
    const name = document.createElement("span");
    name.textContent = photo.fileName;
    const rating = document.createElement("span");
    rating.textContent = photo.rating || "-----";
    footer.append(name, rating);
    card.append(footer);
    return card;
  }}

  const photos = [
    {{ fileName: "synthetic-gradient.jpg", fileType: "JPG", rating: "****-", state: "Pick", stateClass: "sr-card-state-pick", src: imagePath }},
    {{ fileName: "synthetic-checker.jpeg", fileType: "JPEG", rating: "***--", src: secondaryPath }},
    {{ fileName: "blocked-raw.DNG", fileType: "DNG", rating: "-----", state: "Blocked", artClass: "sr-thumb-art-c" }},
    {{ fileName: "notes.txt", fileType: "TXT", rating: "-----", state: "Unsupported", artClass: "sr-thumb-art-d" }},
    {{ fileName: "contact-sheet-qa.jpg", fileType: "JPG", rating: "**---", src: imagePath }},
    {{ fileName: "export-candidate.jpg", fileType: "JPG", rating: "*****", src: secondaryPath }},
  ];

  function populateGrid(withPhotos) {{
    grid.replaceChildren(...(withPhotos ? photos.map(photoCard) : []));
    grid.hidden = !withPhotos;
    empty.hidden = withPhotos;
    loading.hidden = true;
    document.querySelector("#libraryPhotoCount").textContent = withPhotos ? `${{photos.length}} photos` : "0 photos";
    document.querySelector("#allPhotosCount").textContent = withPhotos ? String(photos.length) : "0";
    document.querySelector("#recentImportCount").textContent = withPhotos ? String(photos.length) : "0";
    document.querySelector("#gridStateNote").textContent = withPhotos
      ? "Imported photos are displayed by catalog reference."
      : "Import a folder by reference to populate the grid.";
    document.querySelector("#gridPhotoSubtitle").textContent = withPhotos ? `${{photos.length}} catalog rows` : "No catalog rows";
    document.querySelector("#selectedPhotoName").textContent = withPhotos ? "synthetic-gradient.jpg" : "None";
    document.querySelector("#selectedPhotoRating").textContent = withPhotos ? "****-" : "-----";
    document.querySelector("#cullingStatus").value = withPhotos ? "Rating 4. Picked." : "No photo selected.";
    document.querySelectorAll("[data-rating-value]").forEach((button) => {{
      button.disabled = !withPhotos;
      button.setAttribute("aria-pressed", String(withPhotos && button.dataset.ratingValue === "4"));
    }});
    ["#pickSelectedPhoto", "#rejectSelectedPhoto", "#clearCullingFlags"].forEach((selector) => {{
      document.querySelector(selector).disabled = !withPhotos;
    }});
    document.querySelector("#pickSelectedPhoto").setAttribute("aria-pressed", String(withPhotos));
    document.querySelector("#rejectSelectedPhoto").setAttribute("aria-pressed", "false");
  }}

  function populateFilmstrip(selector) {{
    const filmstrip = document.querySelector(selector);
    filmstrip.replaceChildren(...photos.slice(0, 5).map((photo, index) => {{
      const card = document.createElement("button");
      card.className = "sr-filmstrip-card";
      card.type = "button";
      card.classList.toggle("is-selected", index === 0);
      card.append(photo.src ? thumb(photo.src) : document.createElement("span"));
      card.lastElementChild.className ||= `sr-thumb-art ${{photo.artClass || "sr-thumb-art-a"}}`;
      const name = document.createElement("span");
      name.textContent = photo.fileName;
      card.append(name);
      return card;
    }}));
  }}

  function renderExportPreview() {{
    const card = document.querySelector(".sr-export-preview-card");
    card.querySelectorAll(".sr-export-preview-image").forEach((image) => image.remove());
    document.querySelector("#exportPreviewArt").hidden = true;
    card.prepend(thumb(imagePath, "sr-export-preview-image"));
  }}

  function openLibraryBase(withPhotos = true) {{
    setMode("library");
    setLibraryState("open");
    importPanel.hidden = true;
    loupeSurface.hidden = true;
    gridShell.hidden = false;
    exportDialog.hidden = true;
    populateGrid(withPhotos);
    populateFilmstrip("#loupeFilmstrip");
    populateFilmstrip("#developFilmstrip");
    document.querySelector("#appStatus").value = withPhotos ? "Library grid loaded." : "Open library with no imported photos.";
    document.querySelector("#cacheClearStatus").value = "Ready to clear disposable caches.";
    document.querySelector("#clearLibraryCache").disabled = false;
  }}

  setMode("library");
  setLibraryState("welcome");
  setLayout(false, false, true);
  exportDialog.hidden = true;

  if (state === "welcome") {{
    document.querySelector("#welcomeStatus").value = "Enter a local library folder path to begin.";
  }} else if (state === "empty") {{
    openLibraryBase(false);
  }} else if (state === "grid" || state === "maintenance") {{
    openLibraryBase(true);
    if (state === "maintenance") {{
      document.querySelector("#cacheClearStatus").value = "Cache clear removes only disposable previews and thumbnails.";
    }}
  }} else if (state === "import") {{
    openLibraryBase(true);
    importPanel.hidden = false;
    document.querySelector("#importProgress").value = 100;
    document.querySelector("#importProgressLabel").textContent = "Completed";
    document.querySelector("#importStatus").value = "Imported 5 supported file(s) by reference; originals unchanged.";
    document.querySelector("#importedCount").textContent = "5";
    document.querySelector("#unsupportedCount").textContent = "1";
    document.querySelector("#importErrorCount").textContent = "0";
    document.querySelectorAll("[data-import-step-progress]").forEach((progress) => {{
      progress.value = 100;
      progress.textContent = "100%";
    }});
    document.querySelectorAll("[data-import-step-output]").forEach((output) => {{
      output.textContent = "Completed";
    }});
  }} else if (state === "import-review") {{
    openLibraryBase(true);
    importPanel.hidden = false;
    document.querySelector("#importProgress").value = 100;
    document.querySelector("#importProgressLabel").textContent = "Needs Review";
    document.querySelector("#importStatus").value = "Imported 5 supported file(s) by reference; originals unchanged.";
    document.querySelector("#importedCount").textContent = "5";
    document.querySelector("#unsupportedCount").textContent = "1";
    document.querySelector("#importErrorCount").textContent = "1";
    document.querySelector("#viewImportErrors").disabled = false;
    document.querySelector("#viewImportErrors").setAttribute("aria-expanded", "true");
    const review = document.querySelector("#importIssueReview");
    const summary = document.querySelector("#importIssueReviewSummary");
    const list = document.querySelector("#importIssueList");
    review.hidden = false;
    summary.value = "3 issue(s): 1 unsupported, 1 skipped, 1 failed.";
    list.replaceChildren(
      ...[
        ["unsupported", "Unsupported file", "notes.txt", "/Import/notes.txt", "file extension is unsupported by the local alpha"],
        ["skipped", "Skipped entry", "Archive.photoslibrary", "/Import/Archive.photoslibrary", "package directories are skipped by import policy"],
        ["error", "Failed entry", "locked-folder", "/Import/locked-folder", "failed to read directory entry: permission denied"],
      ].map(([tone, label, name, path, message]) => {{
        const row = document.createElement("article");
        row.className = "sr-import-issue-row";
        row.dataset.issueTone = tone;
        row.setAttribute("role", "listitem");
        const badge = document.createElement("span");
        badge.className = "sr-import-issue-badge";
        badge.textContent = label;
        const body = document.createElement("div");
        body.className = "sr-import-issue-body";
        const title = document.createElement("strong");
        title.textContent = name;
        const pathElement = document.createElement("span");
        pathElement.textContent = path;
        const detail = document.createElement("small");
        detail.textContent = message;
        body.append(title, pathElement, detail);
        row.append(badge, body);
        return row;
      }})
    );
    document.querySelectorAll("[data-import-step-progress]").forEach((progress) => {{
      progress.value = 100;
      progress.textContent = "100%";
    }});
    document.querySelectorAll("[data-import-step-output]").forEach((output) => {{
      output.textContent = "Completed";
    }});
  }} else if (state === "loupe") {{
    openLibraryBase(true);
    gridShell.hidden = true;
    loupeSurface.hidden = false;
    document.querySelector("#loupePhotoName").textContent = "synthetic-gradient.jpg";
    document.querySelector("#loupePhotoRating").textContent = "****-";
    document.querySelector("#loupePreviewStatus").textContent = "Preview Ready";
    document.querySelector("#loupePreviewMessage").textContent = "Preview source is ready for a display-profile-aware surface.";
    loupeViewer.dataset.previewStatus = "ready";
    loupeViewer.querySelectorAll(".sr-loupe-image").forEach((image) => image.remove());
    loupeViewer.prepend(thumb(imagePath, "sr-loupe-image"));
  }} else if (state === "develop") {{
    openLibraryBase(true);
    setMode("develop");
    document.querySelector("#developPhotoName").textContent = "synthetic-gradient.jpg";
    document.querySelector("#developPreviewStatus").textContent = "Preview Ready";
    document.querySelector("#developPreviewMessage").textContent = "Exposure 0.40, contrast 12.";
    document.querySelector("#developExposureSlider").value = "0.40";
    document.querySelector("#developExposureValue").value = "0.40";
    document.querySelector("#developContrastSlider").value = "12";
    document.querySelector("#developContrastValue").value = "12";
    document.querySelector("#developEditState").textContent = "Clean";
    developSurface.dataset.previewStatus = "ready";
    developSurface.dataset.hasPreviewImage = "true";
    developSurface.querySelectorAll(".sr-develop-image").forEach((image) => image.remove());
    developSurface.prepend(thumb(imagePath, "sr-develop-image"));
  }} else if (state === "export") {{
    openLibraryBase(true);
    setMode("export");
    exportDialog.hidden = false;
    document.querySelector("#exportSelectedPhotoName").textContent = "synthetic-gradient.jpg";
    document.querySelector("#exportSelectedInline").textContent = "synthetic-gradient.jpg";
    document.querySelector("#exportOutputPath").value = "/Users/you/Pictures/Exports/synthetic-gradient_SilicaRAW.jpg";
    document.querySelector("#exportStatus").value = "Enter an output path, then export the selected photo.";
    document.querySelector("#exportSummaryFile").textContent = "synthetic-gradient_SilicaRAW.jpg";
    renderExportPreview();
  }} else if (state === "layout-sidebar-collapsed") {{
    openLibraryBase(true);
    setLayout(true, false, true, 168);
  }} else if (state === "layout-inspector-collapsed") {{
    openLibraryBase(true);
    setLayout(false, true, true, 168);
  }} else if (state === "layout-reset") {{
    openLibraryBase(true);
    setLayout(false, false, true, 168);
  }}
}})()
"""


def metric_script(surface):
    return f"""
(() => {{
  const visible = (element) => {{
    if (!element || element.hidden) return false;
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
  }};
  const box = (selector) => {{
    const element = document.querySelector(selector);
    if (!visible(element)) return null;
    const rect = element.getBoundingClientRect();
    return {{ left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom, width: rect.width, height: rect.height }};
  }};
  const mode = box("#modeNavigation");
  const actions = box(".sr-toolbar-actions");
  const toolbarOverlap = Boolean(mode && actions && mode.right > actions.left && mode.left < actions.right);
  const controlClipping = Array.from(document.querySelectorAll("button, output, label"))
    .filter(visible)
    .filter((element) => element.scrollWidth > element.clientWidth + 2)
    .map((element) => {{
      const id = element.id ? `#${{element.id}}` : element.className || element.tagName.toLowerCase();
      return `${{element.tagName.toLowerCase()}}${{id}}`;
  }})
    .slice(0, 12);
  const dialog = box("#exportDialog");
  const app = document.querySelector("#appFrame");
  return {{
    surface: {json.dumps(surface)},
    viewport: {{ width: innerWidth, height: innerHeight }},
    horizontalOverflow: Math.max(document.documentElement.scrollWidth, document.body.scrollWidth) > innerWidth + 1,
    toolbarOverlap,
    controlClipping,
    visiblePhotoCards: Array.from(document.querySelectorAll(".sr-photo-card")).filter(visible).length,
    visibleFilmstripCards: Array.from(document.querySelectorAll(".sr-filmstrip-card")).filter(visible).length,
    visibleImages: Array.from(document.querySelectorAll(".sr-thumb-image, .sr-loupe-image, .sr-develop-image")).filter(visible).length,
    importIssueReviewVisible: Boolean(box("#importIssueReview")),
    visibleImportIssueRows: Array.from(document.querySelectorAll(".sr-import-issue-row")).filter(visible).length,
    exportDialogWithinViewport: !dialog || (dialog.left >= 0 && dialog.right <= innerWidth && dialog.top >= 0 && dialog.bottom <= innerHeight),
    activeMode: app.dataset.activeMode,
    libraryState: app.dataset.libraryState,
    sidebarCollapsed: app.dataset.sidebarCollapsed,
    inspectorCollapsed: app.dataset.inspectorCollapsed,
    filmstripVisible: app.dataset.filmstripVisible,
    sidebarVisible: Boolean(box("#leftSidebar")),
    inspectorVisible: Boolean(box("#rightInspector")),
  }};
}})()
"""


def capture(url):
    failures = []
    results = []
    agent("close", "--all")
    for viewport_name, width, height in VIEWPORTS:
        agent("set", "viewport", str(width), str(height))
        agent("open", url)
        agent("wait", "500")
        for surface_name, state in SURFACES:
            agent("eval", state_script(state))
            agent("wait", "150")
            metrics = eval_json(metric_script(surface_name))
            screenshot_path = SCREENSHOTS / f"{viewport_name}-{surface_name}.png"
            agent("screenshot", str(screenshot_path))
            metrics["screenshot"] = str(screenshot_path.relative_to(ROOT))
            results.append(metrics)

            if metrics["horizontalOverflow"]:
                failures.append(f"{viewport_name} {surface_name}: horizontal overflow")
            if metrics["toolbarOverlap"]:
                failures.append(f"{viewport_name} {surface_name}: toolbar mode/actions overlap")
            if metrics["controlClipping"]:
                failures.append(
                    f"{viewport_name} {surface_name}: clipped controls {', '.join(metrics['controlClipping'])}"
                )
            if not metrics["exportDialogWithinViewport"]:
                failures.append(f"{viewport_name} {surface_name}: export dialog leaves viewport")
            if state == "import-review" and (
                not metrics["importIssueReviewVisible"] or metrics["visibleImportIssueRows"] < 3
            ):
                failures.append(f"{viewport_name} {surface_name}: import issue review not visible")
            if state == "layout-sidebar-collapsed" and (
                metrics["sidebarCollapsed"] != "true" or metrics["sidebarVisible"]
            ):
                failures.append(f"{viewport_name} {surface_name}: sidebar collapse state not applied")
            if state == "layout-inspector-collapsed" and (
                metrics["inspectorCollapsed"] != "true" or metrics["inspectorVisible"]
            ):
                failures.append(f"{viewport_name} {surface_name}: inspector collapse state not applied")
            if state == "layout-reset" and (
                metrics["sidebarCollapsed"] != "false"
                or metrics["inspectorCollapsed"] != "false"
                or metrics["filmstripVisible"] != "true"
                or not metrics["sidebarVisible"]
                or not metrics["inspectorVisible"]
            ):
                failures.append(f"{viewport_name} {surface_name}: layout reset state not applied")
    return results, failures


def parse_args():
    parser = argparse.ArgumentParser(description="Run final visual/responsive QA screenshots.")
    parser.add_argument("--keep-artifacts", action="store_true", help="Keep existing artifact directory before running.")
    return parser.parse_args()


def main():
    args = parse_args()
    if shutil.which("agent-browser") is None:
        print("final visual QA requires agent-browser on PATH", file=sys.stderr)
        return 1

    if ARTIFACTS.exists() and not args.keep_artifacts:
        shutil.rmtree(ARTIFACTS)
    SCREENSHOTS.mkdir(parents=True, exist_ok=True)
    generate_fixtures()

    port = find_port()
    url = f"http://127.0.0.1:{port}/apps/desktop/static/index.html"
    server = subprocess.Popen(
        ["python3", "-m", "http.server", str(port), "--bind", "127.0.0.1"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    try:
        wait_for_server(url)
        results, failures = capture(url)
    finally:
        try:
            agent("close", "--all")
        except Exception:
            pass
        server.terminate()
        try:
            server.wait(timeout=5)
        except subprocess.TimeoutExpired:
            server.kill()

    RESULTS.write_text(json.dumps({"results": results, "failures": failures}, indent=2) + "\n", encoding="utf-8")
    if failures:
        for failure in failures:
            print(f"final visual QA failed: {failure}", file=sys.stderr)
        print(f"results written to {RESULTS.relative_to(ROOT)}", file=sys.stderr)
        return 1

    print(f"final visual QA screenshots: {SCREENSHOTS.relative_to(ROOT)}")
    print(f"final visual QA results: {RESULTS.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
