#!/usr/bin/env python3
import argparse
import base64
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import time
import urllib.request
from pathlib import Path
from tempfile import TemporaryDirectory
from urllib.parse import urlparse


ROOT = Path(__file__).resolve().parents[2]
ARTIFACTS = ROOT / ".tmp/final-visual-responsive-qa"
FIXTURES = ARTIFACTS / "fixtures"
SCREENSHOTS = ARTIFACTS / "screenshots"
RESULTS = ARTIFACTS / "visual-qa-results.json"

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
    ("M014-edit-clipboard-sync", "clipboard"),
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


def read_json(url):
    with urllib.request.urlopen(url, timeout=1) as response:
        return json.loads(response.read().decode("utf-8"))


def find_chrome_executable():
    candidates = []
    env_path = os.environ.get("SILICARAW_CHROME")
    if env_path:
        candidates.append(Path(env_path).expanduser())
    candidates.extend(
        [
            Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            Path("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        ]
    )
    candidates.extend(
        sorted(
            Path.home().glob(
                ".agent-browser/browsers/chrome-*/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
            ),
            reverse=True,
        )
    )
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate
    raise RuntimeError(
        "final visual QA requires Chrome. Set SILICARAW_CHROME to a Chromium-compatible executable."
    )


class WebSocketCdpClient:
    def __init__(self, websocket_url):
        parsed = urlparse(websocket_url)
        if parsed.scheme != "ws":
            raise RuntimeError(f"unsupported CDP websocket URL: {websocket_url}")
        self._socket = socket.create_connection((parsed.hostname, parsed.port), timeout=10)
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        path = parsed.path
        if parsed.query:
            path = f"{path}?{parsed.query}"
        request = (
            f"GET {path} HTTP/1.1\r\n"
            f"Host: {parsed.hostname}:{parsed.port}\r\n"
            "Upgrade: websocket\r\n"
            "Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            "Sec-WebSocket-Version: 13\r\n"
            "\r\n"
        )
        self._socket.sendall(request.encode("ascii"))
        response = self._read_until(b"\r\n\r\n").decode("iso-8859-1")
        if " 101 " not in response.split("\r\n", 1)[0]:
            raise RuntimeError(f"CDP websocket upgrade failed: {response}")

    def close(self):
        try:
            self._send_frame(b"", opcode=0x8)
        except OSError:
            pass
        try:
            self._socket.close()
        except OSError:
            pass

    def send_json(self, payload):
        self._send_frame(json.dumps(payload).encode("utf-8"), opcode=0x1)

    def recv_json(self):
        while True:
            message = self._recv_message()
            if message is None:
                continue
            return json.loads(message)

    def _read_until(self, marker):
        data = bytearray()
        while marker not in data:
            chunk = self._socket.recv(4096)
            if not chunk:
                raise RuntimeError("CDP websocket closed during handshake")
            data.extend(chunk)
        return bytes(data)

    def _read_exact(self, size):
        data = bytearray()
        while len(data) < size:
            chunk = self._socket.recv(size - len(data))
            if not chunk:
                raise RuntimeError("CDP websocket closed")
            data.extend(chunk)
        return bytes(data)

    def _send_frame(self, payload, opcode):
        first = 0x80 | opcode
        length = len(payload)
        if length < 126:
            header = struct.pack("!BB", first, 0x80 | length)
        elif length < 65536:
            header = struct.pack("!BBH", first, 0x80 | 126, length)
        else:
            header = struct.pack("!BBQ", first, 0x80 | 127, length)
        mask = os.urandom(4)
        masked = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        self._socket.sendall(header + mask + masked)

    def _recv_message(self):
        payloads = []
        opcode = None
        while True:
            first, second = self._read_exact(2)
            fin = bool(first & 0x80)
            frame_opcode = first & 0x0F
            masked = bool(second & 0x80)
            length = second & 0x7F
            if length == 126:
                length = struct.unpack("!H", self._read_exact(2))[0]
            elif length == 127:
                length = struct.unpack("!Q", self._read_exact(8))[0]
            mask = self._read_exact(4) if masked else None
            payload = self._read_exact(length) if length else b""
            if mask:
                payload = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))

            if frame_opcode == 0x8:
                raise RuntimeError("CDP websocket closed")
            if frame_opcode == 0x9:
                self._send_frame(payload, opcode=0xA)
                return None
            if frame_opcode == 0xA:
                return None
            if frame_opcode in (0x1, 0x2):
                opcode = frame_opcode
                payloads = [payload]
            elif frame_opcode == 0x0:
                payloads.append(payload)
            if fin:
                message = b"".join(payloads)
                if opcode == 0x1:
                    return message.decode("utf-8")
                return message


class ChromeCdpSession:
    def __init__(self):
        self.client = None
        self.process = None
        self.profile = None
        chrome = find_chrome_executable()
        self.port = find_port()
        self.profile = TemporaryDirectory(prefix="silicaraw-visual-qa-chrome-")
        try:
            self.process = subprocess.Popen(
                [
                    str(chrome),
                    "--headless=new",
                    f"--remote-debugging-port={self.port}",
                    "--no-first-run",
                    "--no-default-browser-check",
                    "--disable-background-networking",
                    "--disable-component-update",
                    "--disable-default-apps",
                    "--disable-features=Translate",
                    "--hide-scrollbars",
                    "--enable-unsafe-swiftshader",
                    f"--user-data-dir={self.profile.name}",
                    "--window-size=1280,800",
                    "about:blank",
                ],
                cwd=ROOT,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            websocket_url = self._wait_for_page()
            self.client = WebSocketCdpClient(websocket_url)
            self._next_id = 1
            self.command("Page.enable")
            self.command("Runtime.enable")
            self.command(
                "Emulation.setEmulatedMedia",
                {"features": [{"name": "prefers-color-scheme", "value": "dark"}]},
            )
        except Exception:
            self.close()
            raise

    def close(self):
        if self.client is not None:
            try:
                self.client.close()
            except Exception:
                pass
            self.client = None
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        self.process = None
        if self.profile is not None:
            self.profile.cleanup()
            self.profile = None

    def __enter__(self):
        return self

    def __exit__(self, _exc_type, _exc, _traceback):
        self.close()

    def command(self, method, params=None):
        message_id = self._next_id
        self._next_id += 1
        payload = {"id": message_id, "method": method}
        if params is not None:
            payload["params"] = params
        self.client.send_json(payload)
        while True:
            message = self.client.recv_json()
            if message.get("id") != message_id:
                continue
            if "error" in message:
                raise RuntimeError(f"CDP {method} failed: {message['error']}")
            return message.get("result", {})

    def evaluate(self, script):
        result = self.command(
            "Runtime.evaluate",
            {
                "expression": script,
                "awaitPromise": True,
                "returnByValue": True,
                "userGesture": True,
            },
        )
        if "exceptionDetails" in result:
            raise RuntimeError(f"CDP Runtime.evaluate failed: {result['exceptionDetails']}")
        return result.get("result", {}).get("value")

    def open(self, url):
        self.command("Page.navigate", {"url": url})
        self.wait_for_ready()

    def screenshot(self, path):
        result = self.command(
            "Page.captureScreenshot",
            {"format": "png", "fromSurface": True, "captureBeyondViewport": False},
        )
        path.write_bytes(base64.b64decode(result["data"]))

    def set_viewport(self, width, height):
        self.command(
            "Emulation.setDeviceMetricsOverride",
            {
                "width": width,
                "height": height,
                "deviceScaleFactor": 1,
                "mobile": False,
                "screenWidth": width,
                "screenHeight": height,
            },
        )

    def wait(self, milliseconds):
        time.sleep(milliseconds / 1000)

    def wait_for_ready(self):
        for _ in range(80):
            try:
                if self.evaluate("document.readyState") == "complete":
                    return
            except RuntimeError:
                pass
            time.sleep(0.1)
        raise RuntimeError("Chrome page did not finish loading")

    def _wait_for_page(self):
        url = f"http://127.0.0.1:{self.port}/json/list"
        for _ in range(100):
            if self.process.poll() is not None:
                raise RuntimeError("Chrome exited before CDP became available")
            try:
                for target in read_json(url):
                    if target.get("type") == "page" and target.get("webSocketDebuggerUrl"):
                        return target["webSocketDebuggerUrl"]
            except Exception:
                pass
            time.sleep(0.1)
        raise RuntimeError("Chrome CDP page target did not become ready")


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

  function setHistogramState(withPhotos) {{
    const histogram = document.querySelector("#photoHistogram");
    const bars = document.querySelector("#photoHistogramBars");
    const status = document.querySelector("#photoHistogramStatus");
    bars.replaceChildren();
    if (!withPhotos) {{
      histogram.dataset.histogramState = "empty";
      status.value = "No histogram";
      status.textContent = "No histogram";
      return;
    }}
    histogram.dataset.histogramState = "ready";
    status.value = "Luminance histogram ready.";
    status.textContent = status.value;
    [18, 34, 62, 89, 76, 54, 43, 68, 91, 73, 48, 31, 24, 39, 58, 82, 96, 84, 61, 37, 22, 28, 46, 64, 79, 71, 53, 41, 35, 29, 20, 14].forEach((height) => {{
      const bar = document.createElement("span");
      bar.className = "sr-histogram-bar";
      bar.style.height = `${{height}}%`;
      bars.append(bar);
    }});
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
    setHistogramState(withPhotos);
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
    document.querySelector("#developToneCurveMidpointSlider").value = "0.62";
    document.querySelector("#developToneCurveMidpointValue").value = "0.62";
    document.querySelector("#developToneCurveSupportStatus").value = "Point RGB";
    document.querySelector("#developToneCurveSupportStatus").textContent = "Point RGB";
    document.querySelector("#developToneCurvePanel").dataset.toneCurveState = "enabled";
    document.querySelector("#developToneCurveLine").setAttribute("points", "0,100 50,38 100,0");
    document.querySelector("#developHslPanel").dataset.hslState = "enabled";
    document.querySelector("#developHslSupportStatus").value = "JPEG/JPG";
    document.querySelector("#developHslSupportStatus").textContent = "JPEG/JPG";
    document.querySelector("#developHslChannelBlue").setAttribute("aria-pressed", "true");
    document.querySelector("#developHslHueSlider").value = "-12";
    document.querySelector("#developHslHueValue").value = "-12";
    document.querySelector("#developHslSaturationSlider").value = "24";
    document.querySelector("#developHslSaturationValue").value = "24";
    document.querySelector("#developHslLuminanceSlider").value = "-8";
    document.querySelector("#developHslLuminanceValue").value = "-8";
    document.querySelector("#developDetailPanel").dataset.detailState = "active-unsupported";
    document.querySelector("#developDetailSupportStatus").value = "Blocked";
    document.querySelector("#developDetailSupportStatus").textContent = "Blocked";
    document.querySelector("#developDetailBoundaryStatus").value = "Detail preview/export is unsupported until renderer support exists.";
    document.querySelector("#developDetailBoundaryStatus").textContent = "Detail preview/export is unsupported until renderer support exists.";
    document.querySelector("#developDetailSharpeningAmountSlider").value = "42";
    document.querySelector("#developDetailSharpeningAmountValue").value = "42";
    document.querySelector("#developLensGeometryPanel").dataset.geometryState = "enabled";
    document.querySelector("#developGeometrySupportStatus").value = "Geometry Ready";
    document.querySelector("#developGeometrySupportStatus").textContent = "Geometry Ready";
    document.querySelector("#developGeometryCropStatus").value = "Crop 50% x 80% at 10%, 5%, rotation 90, horizontal flip.";
    document.querySelector("#developGeometryCropStatus").textContent = "Crop 50% x 80% at 10%, 5%, rotation 90, horizontal flip.";
    document.querySelector("#developLensSupportStatus").value = "Lens correction unavailable.";
    document.querySelector("#developLensSupportStatus").textContent = "Lens correction unavailable.";
    document.querySelector("#developGeometryTransformStatus").value = "Transform unsupported.";
    document.querySelector("#developGeometryTransformStatus").textContent = "Transform unsupported.";
    document.querySelector("#developGeometryCropXSlider").value = "0.10";
    document.querySelector("#developGeometryCropXValue").value = "0.10";
    document.querySelector("#developGeometryCropYSlider").value = "0.05";
    document.querySelector("#developGeometryCropYValue").value = "0.05";
    document.querySelector("#developGeometryCropWidthSlider").value = "0.50";
    document.querySelector("#developGeometryCropWidthValue").value = "0.50";
    document.querySelector("#developGeometryCropHeightSlider").value = "0.80";
    document.querySelector("#developGeometryCropHeightValue").value = "0.80";
    document.querySelector("#developGeometryFlipHorizontal").setAttribute("aria-pressed", "true");
    document.querySelector("#developGeometryFlipHorizontal").classList.add("is-active");
    document.querySelector("#developGeometryFlipVertical").setAttribute("aria-pressed", "false");
    [
      "#developDetailSharpeningAmountSlider",
      "#developDetailSharpeningAmountValue",
      "#developDetailSharpeningRadiusSlider",
      "#developDetailSharpeningRadiusValue",
      "#developDetailSharpeningDetailSlider",
      "#developDetailSharpeningDetailValue",
      "#developDetailSharpeningMaskingSlider",
      "#developDetailSharpeningMaskingValue",
      "#developDetailNoiseLuminanceSlider",
      "#developDetailNoiseLuminanceValue",
      "#developDetailNoiseDetailSlider",
      "#developDetailNoiseDetailValue",
      "#developDetailNoiseContrastSlider",
      "#developDetailNoiseContrastValue",
      "#developDetailNoiseColorSlider",
      "#developDetailNoiseColorValue",
      "#developDetailNoiseColorDetailSlider",
      "#developDetailNoiseColorDetailValue",
      "#developDetailMlxDenoise",
      "#developLensProfileCorrection",
      "#developLensChromaticAberration",
      "#developLensDistortionSlider",
      "#developLensVignettingSlider",
      "#developGeometryTransformScaleSlider",
      "#developGeometryTransformVerticalSlider",
      "#developGeometryTransformHorizontalSlider",
    ].forEach((selector) => {{
      const control = document.querySelector(selector);
      if (control) control.disabled = true;
    }});
    document.querySelector("#developEditState").textContent = "Clean";
    document.querySelector("#developBeforeView").disabled = false;
    document.querySelector("#developAfterView").disabled = false;
    document.querySelector("#developBeforeView").classList.remove("is-active");
    document.querySelector("#developAfterView").classList.add("is-active");
    document.querySelector("#developBeforeView").setAttribute("aria-pressed", "false");
    document.querySelector("#developAfterView").setAttribute("aria-pressed", "true");
    document.querySelector("#developPreviewSurface").dataset.beforeAfterMode = "after";
    document.querySelectorAll("[data-basic-preset]").forEach((button) => {{
      button.disabled = false;
    }});
    [
      "#developExposureSlider",
      "#developExposureValue",
      "#developExposureReset",
      "#developContrastSlider",
      "#developContrastValue",
      "#developContrastReset",
      "#developWhiteBalanceMode",
      "#developTemperatureSlider",
      "#developTemperatureValue",
      "#developTintSlider",
      "#developTintValue",
      "#developHighlightsSlider",
      "#developHighlightsValue",
      "#developShadowsSlider",
      "#developShadowsValue",
      "#developWhitesSlider",
      "#developWhitesValue",
      "#developBlacksSlider",
      "#developBlacksValue",
      "#developVibranceSlider",
      "#developVibranceValue",
      "#developSaturationSlider",
      "#developSaturationValue",
      "#developToneCurveMidpointSlider",
      "#developToneCurveMidpointValue",
      "#developToneCurveReset",
      "#developHslHueSlider",
      "#developHslHueValue",
      "#developHslHueReset",
      "#developHslSaturationSlider",
      "#developHslSaturationValue",
      "#developHslSaturationReset",
      "#developHslLuminanceSlider",
      "#developHslLuminanceValue",
      "#developHslLuminanceReset",
      "#developGeometryCropXSlider",
      "#developGeometryCropXValue",
      "#developGeometryCropYSlider",
      "#developGeometryCropYValue",
      "#developGeometryCropWidthSlider",
      "#developGeometryCropWidthValue",
      "#developGeometryCropHeightSlider",
      "#developGeometryCropHeightValue",
      "#developGeometryCropClear",
      "#developGeometryRotateLeft",
      "#developGeometryRotateRight",
      "#developGeometryOrientationReset",
      "#developGeometryFlipHorizontal",
      "#developGeometryFlipVertical",
      "#developResetBasic",
      "#developCommitEdit",
      "#developRevertEdit",
    ].forEach((selector) => {{
      const control = document.querySelector(selector);
      if (control) control.disabled = false;
    }});
    developSurface.dataset.previewStatus = "ready";
    developSurface.dataset.hasPreviewImage = "true";
    developSurface.querySelectorAll(".sr-develop-image").forEach((image) => image.remove());
    developSurface.prepend(thumb(imagePath, "sr-develop-image"));
    document.querySelector("#developLensGeometryPanel").scrollIntoView({{ block: "center" }});
  }} else if (state === "clipboard") {{
    openLibraryBase(true);
    setMode("develop");
    document.querySelector("#developPhotoName").textContent = "synthetic-gradient.jpg";
    document.querySelector("#developPreviewStatus").textContent = "Preview Ready";
    document.querySelector("#developPreviewMessage").textContent = "Exposure 0.40, contrast 12.";
    document.querySelector("#selectionSummary").dataset.multiSelectionState = "multi";
    document.querySelector("#primarySelectedPhotoName").textContent = "synthetic-gradient.jpg";
    document.querySelector("#multiSelectionCount").value = "2 selected";
    document.querySelector("#multiSelectionCount").textContent = "2 selected";
    document.querySelector("#developClipboardPanel").dataset.clipboardState = "ready";
    document.querySelector("#developClipboardSource").value = "Copied 3 subset(s) from synthetic-gradient.jpg.";
    document.querySelector("#developClipboardSource").textContent = "Copied 3 subset(s) from synthetic-gradient.jpg.";
    document.querySelector("#developClipboardSelectionCount").value = "2 selected on this page";
    document.querySelector("#developClipboardSelectionCount").textContent = "2 selected on this page";
    document.querySelector("#clipboardSubsetBasic").checked = true;
    document.querySelector("#clipboardSubsetTone").checked = true;
    document.querySelector("#clipboardSubsetColor").checked = false;
    document.querySelector("#clipboardSubsetGeometry").checked = true;
    document.querySelector("#clipboardSubsetDetail").checked = false;
    document.querySelector("#clipboardSubsetDetail").disabled = true;
    document.querySelector("#clipboardSubsetLens").checked = false;
    document.querySelector("#clipboardSubsetLens").disabled = true;
    document.querySelector("#copyEditClipboard").disabled = false;
    document.querySelector("#pasteEditClipboard").disabled = false;
    document.querySelector("#syncEditClipboard").disabled = false;
    document.querySelector("#editClipboardStatus").value = "Ready to paste Basic, Tone, Geometry to the primary photo or sync 2 selected on this page.";
    document.querySelector("#editClipboardStatus").textContent = document.querySelector("#editClipboardStatus").value;
    const plan = document.querySelector("#editClipboardPlanList");
    plan.replaceChildren(
      ...[
        ["ready", "synthetic-gradient.jpg", "ready", "Ready for batch sync."],
        ["unchanged", "synthetic-checker.jpeg", "unchanged: no_effect", "Clipboard payload does not change this target."],
      ].map(([status, name, stateText, message]) => {{
        const row = document.createElement("div");
        row.className = "sr-edit-clipboard-plan-row";
        row.dataset.clipboardTargetStatus = status;
        row.setAttribute("role", "listitem");
        const title = document.createElement("strong");
        title.textContent = name;
        const targetStatus = document.createElement("span");
        targetStatus.textContent = stateText;
        const targetMessage = document.createElement("small");
        targetMessage.textContent = message;
        row.append(title, targetStatus, targetMessage);
        return row;
      }})
    );
    developSurface.dataset.previewStatus = "ready";
    developSurface.dataset.hasPreviewImage = "true";
    developSurface.querySelectorAll(".sr-develop-image").forEach((image) => image.remove());
    developSurface.prepend(thumb(imagePath, "sr-develop-image"));
    document.querySelector("#developClipboardPanel").scrollIntoView({{ block: "center" }});
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
  const text = (selector) => (document.querySelector(selector)?.textContent || "").trim();
  const disabled = (selector) => Boolean(document.querySelector(selector)?.disabled);
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
  const presetButtons = Array.from(document.querySelectorAll("[data-basic-preset]"));
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
    developState: {{
      photoName: text("#developPhotoName"),
      histogramStatus: text("#photoHistogramStatus"),
      beforeDisabled: disabled("#developBeforeView"),
      afterDisabled: disabled("#developAfterView"),
      activePresetButtons: presetButtons.filter((button) => button.getAttribute("aria-pressed") === "true").length,
      disabledPresetButtons: presetButtons.filter((button) => button.disabled).length,
      toneCurveVisible: Boolean(box("#developToneCurvePanel")),
      toneCurveMidpoint: document.querySelector("#developToneCurveMidpointSlider")?.value || "",
      toneCurveStatus: text("#developToneCurveSupportStatus"),
      toneCurveUnsupportedDisabled: [
        "#developToneCurveChannelRed",
        "#developToneCurveChannelGreen",
        "#developToneCurveChannelBlue",
        "#developToneCurveParametric",
      ].every((selector) => disabled(selector)),
      hslVisible: Boolean(box("#developHslPanel")),
      hslStatus: text("#developHslSupportStatus"),
      hslBlueActive: document.querySelector("#developHslChannelBlue")?.getAttribute("aria-pressed") === "true",
      hslHue: document.querySelector("#developHslHueSlider")?.value || "",
      hslSaturation: document.querySelector("#developHslSaturationSlider")?.value || "",
      hslLuminance: document.querySelector("#developHslLuminanceSlider")?.value || "",
      detailVisible: Boolean(box("#developDetailPanel")),
      detailStatus: text("#developDetailSupportStatus"),
      detailBoundary: text("#developDetailBoundaryStatus"),
      detailAmount: document.querySelector("#developDetailSharpeningAmountSlider")?.value || "",
      detailControlsDisabled: [
        "#developDetailSharpeningAmountSlider",
        "#developDetailSharpeningAmountValue",
        "#developDetailSharpeningRadiusSlider",
        "#developDetailSharpeningRadiusValue",
        "#developDetailSharpeningDetailSlider",
        "#developDetailSharpeningDetailValue",
        "#developDetailSharpeningMaskingSlider",
        "#developDetailSharpeningMaskingValue",
        "#developDetailNoiseLuminanceSlider",
        "#developDetailNoiseLuminanceValue",
        "#developDetailNoiseDetailSlider",
        "#developDetailNoiseDetailValue",
        "#developDetailNoiseContrastSlider",
        "#developDetailNoiseContrastValue",
        "#developDetailNoiseColorSlider",
        "#developDetailNoiseColorValue",
        "#developDetailNoiseColorDetailSlider",
        "#developDetailNoiseColorDetailValue",
        "#developDetailMlxDenoise",
      ].every((selector) => disabled(selector)),
      geometryVisible: Boolean(box("#developLensGeometryPanel")),
      geometryStatus: text("#developGeometrySupportStatus"),
      geometryCropStatus: text("#developGeometryCropStatus"),
      geometryCropWidth: document.querySelector("#developGeometryCropWidthSlider")?.value || "",
      geometryCropHeight: document.querySelector("#developGeometryCropHeightSlider")?.value || "",
      geometryFlipHorizontal: document.querySelector("#developGeometryFlipHorizontal")?.getAttribute("aria-pressed") === "true",
      geometryLensStatus: text("#developLensSupportStatus"),
      geometryTransformStatus: text("#developGeometryTransformStatus"),
      geometryUnsupportedDisabled: [
        "#developLensProfileCorrection",
        "#developLensChromaticAberration",
        "#developLensDistortionSlider",
        "#developLensVignettingSlider",
        "#developGeometryTransformScaleSlider",
        "#developGeometryTransformVerticalSlider",
        "#developGeometryTransformHorizontalSlider",
      ].every((selector) => disabled(selector)),
    }},
    clipboardState: {{
      visible: Boolean(box("#developClipboardPanel")),
      source: text("#developClipboardSource"),
      selectedCount: text("#developClipboardSelectionCount"),
      status: text("#editClipboardStatus"),
      planRows: Array.from(document.querySelectorAll(".sr-edit-clipboard-plan-row")).filter(visible).length,
      basicChecked: Boolean(document.querySelector("#clipboardSubsetBasic")?.checked),
      toneChecked: Boolean(document.querySelector("#clipboardSubsetTone")?.checked),
      geometryChecked: Boolean(document.querySelector("#clipboardSubsetGeometry")?.checked),
      detailDisabled: disabled("#clipboardSubsetDetail"),
      lensDisabled: disabled("#clipboardSubsetLens"),
      pasteDisabled: disabled("#pasteEditClipboard"),
      syncDisabled: disabled("#syncEditClipboard"),
    }},
  }};
}})()
"""


def capture(url):
    failures = []
    results = []
    with ChromeCdpSession() as browser:
        for viewport_name, width, height in VIEWPORTS:
            browser.set_viewport(width, height)
            browser.open(url)
            browser.wait(500)
            for surface_name, state in SURFACES:
                browser.evaluate(state_script(state))
                browser.wait(150)
                metrics = browser.evaluate(metric_script(surface_name))
                screenshot_path = SCREENSHOTS / f"{viewport_name}-{surface_name}.png"
                browser.screenshot(screenshot_path)
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
                if state == "develop":
                    develop_state = metrics["developState"]
                    if develop_state["photoName"] != "synthetic-gradient.jpg":
                        failures.append(f"{viewport_name} {surface_name}: develop photo selection not visible")
                    if develop_state["histogramStatus"] == "No photo selected.":
                        failures.append(f"{viewport_name} {surface_name}: histogram still reports no selection")
                    if develop_state["beforeDisabled"] or develop_state["afterDisabled"]:
                        failures.append(f"{viewport_name} {surface_name}: before/after controls disabled")
                    if develop_state["activePresetButtons"] != 1 or develop_state["disabledPresetButtons"] != 0:
                        failures.append(f"{viewport_name} {surface_name}: basic preset controls not active")
                    if not develop_state["toneCurveVisible"] or develop_state["toneCurveMidpoint"] != "0.62":
                        failures.append(f"{viewport_name} {surface_name}: tone curve panel state not visible")
                    if develop_state["toneCurveStatus"] != "Point RGB":
                        failures.append(f"{viewport_name} {surface_name}: tone curve support status wrong")
                    if not develop_state["toneCurveUnsupportedDisabled"]:
                        failures.append(f"{viewport_name} {surface_name}: unsupported tone curve controls enabled")
                    if not develop_state["hslVisible"] or develop_state["hslStatus"] != "JPEG/JPG":
                        failures.append(f"{viewport_name} {surface_name}: HSL panel support state not visible")
                    if not develop_state["hslBlueActive"] or develop_state["hslHue"] != "-12" or develop_state["hslSaturation"] != "24" or develop_state["hslLuminance"] != "-8":
                        failures.append(f"{viewport_name} {surface_name}: HSL seeded blue channel state not visible")
                    if not develop_state["detailVisible"] or develop_state["detailStatus"] != "Blocked":
                        failures.append(f"{viewport_name} {surface_name}: Detail blocked panel state not visible")
                    if develop_state["detailBoundary"] != "Detail preview/export is unsupported until renderer support exists.":
                        failures.append(f"{viewport_name} {surface_name}: Detail renderer boundary copy wrong")
                    if develop_state["detailAmount"] != "42" or not develop_state["detailControlsDisabled"]:
                        failures.append(f"{viewport_name} {surface_name}: Detail disabled readback state not preserved")
                    if not develop_state["geometryVisible"] or develop_state["geometryStatus"] != "Geometry Ready":
                        failures.append(f"{viewport_name} {surface_name}: geometry panel support state not visible")
                    if develop_state["geometryCropWidth"] != "0.5" or develop_state["geometryCropHeight"] != "0.8":
                        failures.append(f"{viewport_name} {surface_name}: geometry seeded crop state not visible")
                    if not develop_state["geometryFlipHorizontal"]:
                        failures.append(f"{viewport_name} {surface_name}: geometry flip state not visible")
                    if develop_state["geometryLensStatus"] != "Lens correction unavailable.":
                        failures.append(f"{viewport_name} {surface_name}: lens unsupported state copy wrong")
                    if develop_state["geometryTransformStatus"] != "Transform unsupported." or not develop_state["geometryUnsupportedDisabled"]:
                        failures.append(f"{viewport_name} {surface_name}: unsupported geometry/lens controls enabled")
                if state == "clipboard":
                    clipboard_state = metrics["clipboardState"]
                    if not clipboard_state["visible"]:
                        failures.append(f"{viewport_name} {surface_name}: edit clipboard panel not visible")
                    if clipboard_state["selectedCount"] != "2 selected on this page":
                        failures.append(f"{viewport_name} {surface_name}: edit clipboard selected count unclear")
                    if "synthetic-gradient.jpg" not in clipboard_state["source"]:
                        failures.append(f"{viewport_name} {surface_name}: edit clipboard source not visible")
                    if clipboard_state["planRows"] < 2:
                        failures.append(f"{viewport_name} {surface_name}: edit clipboard target plan rows missing")
                    if not (
                        clipboard_state["basicChecked"]
                        and clipboard_state["toneChecked"]
                        and clipboard_state["geometryChecked"]
                    ):
                        failures.append(f"{viewport_name} {surface_name}: edit clipboard subset choices not visible")
                    if not clipboard_state["detailDisabled"] or not clipboard_state["lensDisabled"]:
                        failures.append(f"{viewport_name} {surface_name}: unsupported clipboard subsets enabled")
                    if clipboard_state["pasteDisabled"] or clipboard_state["syncDisabled"]:
                        failures.append(f"{viewport_name} {surface_name}: edit clipboard paste/sync controls disabled")
    return results, failures


def parse_args():
    parser = argparse.ArgumentParser(description="Run final visual/responsive QA screenshots.")
    parser.add_argument("--keep-artifacts", action="store_true", help="Keep existing artifact directory before running.")
    return parser.parse_args()


def main():
    args = parse_args()

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
