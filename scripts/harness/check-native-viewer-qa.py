#!/usr/bin/env python3
"""Check Phase 14 native viewer QA routing and checklist coverage."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CHECKLIST = ROOT / "checklists" / "NATIVE_VIEWER_QA.md"
METAL_RENDERING = ROOT / "docs" / "wiki" / "topics" / "metal-rendering.md"
STATIC_UI = ROOT / "apps" / "desktop" / "static" / "index.html"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_text(text: str, needle: str, label: str) -> None:
    require(needle in text, f"{label} missing required text: {needle}")


def main() -> None:
    require(CHECKLIST.is_file(), "native viewer QA checklist is missing")
    checklist = CHECKLIST.read_text(encoding="utf-8")
    metal = METAL_RENDERING.read_text(encoding="utf-8")
    static_ui = STATIC_UI.read_text(encoding="utf-8")

    for required in [
        "manual proof",
        "automated proof",
        "feature-gated",
        "default CI",
        "cargo test -p silica-desktop --features native-metal-viewer",
        "lifecycle_smoke_evidence_is_neutral_and_reviewable",
        "input_smoke_evidence_is_manual_review_ready",
        "render_request_smoke_evidence_is_reviewable",
        "texture_lifecycle_smoke_evidence_is_reviewable",
        "mouse down",
        "mouse drag",
        "scroll",
        "magnify",
        "resize",
        "Retina",
        "external display",
        "UI responsiveness",
        "web controls",
        "1280x800",
        "1440x900",
        "1728x965",
    ]:
        require_text(checklist, required, "native viewer QA checklist")

    require_text(metal, "Native Viewer QA Checklist", "metal rendering topic")
    require_text(static_ui, 'data-native-viewer-host="reserved"', "static UI")
    require_text(static_ui, "SilicaRAWViewerHost", "static UI")

    print("native viewer QA checks ok")


if __name__ == "__main__":
    main()
