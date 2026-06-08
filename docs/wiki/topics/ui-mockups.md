---
title: UI Mockups
status: active
audience: all
updated: 2026-06-08
source_of_truth: MockupUI/MANIFEST.md
---

# UI Mockups

## Summary

`MockupUI/` contains the high-fidelity visual target screens for SilicaRAW. Treat these PNGs as implementation references for product UI tasks, not as test fixtures or color-management evidence.

## Current Stance

- M004 Library Loupe and M005 Develop are the primary preview-related target screens.
- M003 Library Grid is the primary catalog browsing target screen.
- M007 Export Dialog is the primary export workflow target screen.
- Compact and large variants define responsive expectations for later UI work.

## Phase 5.1 Relationship

Phase 5.1 adds the command/status path that future M004 and M005 implementations can consume:

```txt
catalog photo -> preview candidate -> decode readiness -> render readiness -> desktop command status
```

It does not implement the M004/M005 screens, native viewer layout, or final interaction behavior.

## Links

- [Mockup Manifest](../../../MockupUI/MANIFEST.md)
- [Screen Inventory](../../06_Screen_Inventory_and_Wireframe_Specification.md)
- [Metal Rendering](metal-rendering.md)
- [Color Management](color-management.md)

## Notes for LLM Agents

When implementing UI screens, inspect the relevant `MockupUI/` image first and preserve the screen's information structure. Do not use mockup PNGs as photographic fixtures, decoder fixtures, or color correctness evidence.
