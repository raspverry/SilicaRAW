---
title: UI Mockups
status: active
audience: all
updated: 2026-06-17
source_of_truth: MockupUI/MANIFEST.md
---

# UI Mockups

## Summary

`MockupUI/` contains the high-fidelity visual target screens for SilicaRAW. Treat these PNGs as implementation references for product UI tasks, not as test fixtures or color-management evidence.

## Current Stance

- M004 Library Loupe and M005 Develop are the primary preview-related target screens.
- M003 Library Grid is the primary catalog browsing target screen.
- M007 Export Dialog is the primary export workflow target screen.
- M008 Preferences Appearance is the Phase 21 preferences information-architecture target.
- M010 AI Review is the Phase 24.4 read-only AI review target screen.
- Compact and large variants define responsive expectations for later UI work.

## Phase 5.1 Relationship

Phase 5.1 adds the command/status path that future M004 and M005 implementations can consume:

```txt
catalog photo -> preview candidate -> decode readiness -> render readiness -> desktop command status
```

It does not implement the M004/M005 screens, native viewer layout, or final interaction behavior.

## Phase 5.3 Relationship

Phase 5.3 adds the command/API path for the future M005 Develop exposure/contrast controls:

```txt
catalog photo -> active/default edit graph -> draft exposure/contrast render request -> commit active edit graph
```

It does not implement the M005 screen, product sliders, actual pixel rendering, or final viewer interaction behavior.

## Phase 5.5 Relationship

Phase 5.5 is the first UI MVP vertical slice. It uses `MockupUI/` as the visual and information-structure target for the connected local alpha workflow:

```txt
M001 welcome -> M003 library grid -> M004 preview/loupe -> M005 develop -> M007 export
```

Task 5.5.1 establishes the token and source hierarchy baseline before screen implementation begins. Later Task 5.5 subtasks should inspect the relevant mockup before editing UI code.

## Phase 5.6 Relationship

Phase 5.6 keeps the Phase 5.5 screen hierarchy but replaces static placeholders with product runtime behavior. M001, M002, M003, M004, M005, M007, M008-minimal, and M009 remain the relevant visual references for the local alpha. M006 mask active and M010 AI Review were out of scope during Phase 5.6 until mask and AI/MLX work became explicitly scheduled.

## Phase 24 Relationship

Task 24.4 implements the first M010-inspired AI Review surface as read-only blur review. The visible structure follows the mockup's AI Tools navigation, review list, summary, and action-preview areas.

Task 24.5 adds explicit approval/rejection controls to the same surface. Controls stay disabled for review-only rows and become active only for stored, unapproved, approvable suggestion payloads.

## Phase 21 Relationship

Phase 21 replaces the minimal maintenance surface with a complete Preferences surface. Task 21.1 uses `M008_Preferences_Appearance.png` for the compact section-list and pane structure only. Task 21.2 enables supported Appearance controls through app-session state and tokenized CSS. Task 21.3 moves supported Library default path storage and disposable Cache status/clear controls into Preferences. Task 21.4 wires supported Color and Export defaults to the existing catalog export settings path. Task 21.5 keeps Advanced agent, MCP, and plugin access visible but disabled and explains the Phase 23 permission gate.

## Links

- [Mockup Manifest](../../../MockupUI/MANIFEST.md)
- [Screen Inventory](../../06_Screen_Inventory_and_Wireframe_Specification.md)
- [UI MVP Baseline](ui-mvp-baseline.md)
- [Product Alpha Runtime Completion](product-alpha-runtime-completion.md)
- [Metal Rendering](metal-rendering.md)
- [Color Management](color-management.md)

## Notes for LLM Agents

When implementing UI screens, inspect the relevant `MockupUI/` image first and preserve the screen's information structure. Do not use mockup PNGs as photographic fixtures, decoder fixtures, or color correctness evidence.
