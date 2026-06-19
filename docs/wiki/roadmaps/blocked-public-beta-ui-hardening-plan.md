---
title: Blocked Public Beta UI Hardening Plan
status: active
audience: maintainers
updated: 2026-06-18
source_of_truth: docs/wiki/roadmaps/public-beta-readiness-audit.md
---

# Blocked Public Beta UI Hardening Plan

## Summary

Task 27.2 remains blocked by signing, notarization, checksums, and clean-Mac downloaded-artifact QA. While that external gate is blocked, maintainers may still fix bugs found during local developer-preview QA.

This plan covers the small UI shell and keyboard-access issues found during local app inspection. It does not start public beta release-candidate work and does not change the signed/notarized DMG gate.

## Atomic Tasks

### Consultation Notes

- Visual review agreed the current bug is a mismatch between a `64px` icon rail and text-list sidebar markup.
- Keyboard review agreed the right fix is a thin global dismiss layer plus a read-only shortcuts surface, not a full shortcut manager.
- The `max-width: 1279px` text-hiding breakpoint is too aggressive for this product. SilicaRAW is closer to a Lightroom/RapidRAW-style photo workflow than a Photoshop-style icon-tool palette. The default and 1280px compact desktop states should keep a readable text sidebar.

### H1: Sidebar Navigation Hierarchy and Breakpoints

#### H1.1: Restore Text Sidebar Width

- Default sidebar width: readable text sidebar around `240px`.
- The app's default `1180px` window must show text labels.
- The `1280px` visual QA state must show text labels.
- No `Library`, `Folders`, `Collections`, or row labels may be clipped in normal desktop state.

#### H1.2: Limit Rail Behavior

- Remove the current `max-width: 1279px` text-hiding behavior.
- Use narrow rail behavior only below a true narrow threshold, around `1024px`, or when the user explicitly collapses the sidebar.
- Collapsed state may hide text, but must be intentional and reversible with the sidebar toggle.

#### H1.3: Fix AI Hierarchy

- AI Review must not look like primary navigation.
- Prefer secondary placement in the right inspector or a muted secondary section inside the Library sidebar.
- Do not add runtime MLX, MCP, plugin, or agent behavior.

#### H1.4: Verify Responsive States

- Check standard desktop, `1280px`, and large desktop screenshots.
- No horizontal overflow.
- Toolbar and inspector remain usable.

### H2: Escape Dismissal

#### H2.1: Define Dismiss Priority

Use this order:

```txt
Shortcuts dialog
Preferences dialog
Export dialog
Import issue review
Import panel
Loupe
AI Review
Library grid multi-selection
```

#### H2.2: Add Thin Global Key Handler

- `Escape` closes exactly one topmost open surface.
- When no dismissible surface is open, Library grid `Escape` keeps clearing multi-selection.
- Text inputs should ignore character shortcuts; `Escape` remains allowed for dismissal.

#### H2.3: Restore Focus

- Dialog close returns focus to its opener when possible.
- Surface close returns focus to grid or the relevant panel trigger.

### H3: Shortcuts Preferences Surface

#### H3.1: Add Discoverable Shortcuts Dialog

- Wire the existing Welcome `Keyboard Shortcuts` button to a real dialog.
- Add a Preferences entry point to the same dialog.
- Add a keyboard path such as `?` for opening the shortcuts dialog.

#### H3.2: Keep Shortcuts Read-Only

- List active local alpha shortcuts only.
- Do not implement custom rebinding, mode profiles, command palette, or shortcut persistence.

#### H3.3: Document Future Remapping Boundary

- If remapping is added later, it needs a separate app-session preference task.
- Do not store open dialog state or dismiss stack.

## Stop Rules

- Do not call this a public beta release candidate.
- Do not implement signing, notarization, release publication, auto-update, telemetry, cloud sync, plugin runtime, MCP runtime, or MLX runtime.
- Do not add dependencies.
- Do not mutate original photo files.

## Validation

Use:

```bash
python3 scripts/harness/check-static-ui.py
python3 scripts/harness/check-ui-workflow-smoke.py
scripts/harness/check.sh
```

Visual QA should confirm the standard desktop sidebar text is readable and AI remains secondary.

## Links

- [Public Beta Readiness Audit](public-beta-readiness-audit.md)
- [Current LLM Route](../llm/current-route.md)
- [UI Visual and Responsive QA](../topics/ui-visual-responsive-qa.md)
- [Screen Inventory and Wireframe Specification](../../06_Screen_Inventory_and_Wireframe_Specification.md)

## Notes for LLM Agents

Treat this as blocked-gate hardening only. If signing prerequisites become available, return to Task 27.2 before Phase 28.
