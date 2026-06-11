---
title: Project Overview
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/01_Vision_and_Positioning.md
---

# Project Overview

## Summary

SilicaRAW is an early-stage, open-source RAW photo editor for Apple Silicon. Its primary identity is a local-first, non-destructive RAW editor with Metal-first performance.

AI, MLX, plugins, and MCP are secondary extensions. They should support the editor, not redefine it.

## Current State

- The repository has planning documents, schemas, mockups, a Rust workspace, a Tauri desktop shell, storage/core/export boundaries, and the local-alpha UI/runtime path.
- The project is not production-ready.
- The app shell, initial catalog migration foundation, local library create/open path, folder import scanner, catalog-backed flag persistence, preview readiness path, typed edit graph validation, exposure/contrast edit flow, JPEG sRGB export, UI MVP vertical slice, product alpha runtime loop, backup artifacts, staged restore boundaries, and public trust docs exist.
- RAW decoding, the product Metal viewer bridge, color fixture proof, and signed/notarized user-ready DMG release still require explicit implementation or external credentials.
- `MockupUI/` contains the high-fidelity target screens used by the UI MVP baseline and visual QA work.
- MLX is deferred from local alpha by ADR 0005.

## Project Identity

- macOS-focused RAW photo editor.
- Open-source and local-first.
- Non-destructive editing.
- Apple Silicon and Metal-first performance goals.
- MLX-assisted tools later, only when they serve photo editing workflows.

## Non-Goals

- Not a cloud photo service.
- Not an AI image generator.
- Not a Photoshop replacement.
- Not a Lightroom-compatible clone.
- Not ready for production photo work.

## Links

- [Vision and Positioning](../../01_Vision_and_Positioning.md)
- [Product Requirements](../../02_Product_Requirements_Document.md)
- [Final Master Plan](../../18_Final_Master_Plan.md)
- [Root README](../../../README.md)

## Notes for LLM Agents

Do not overemphasize AI, MLX, plugins, or MCP when working on early tasks. The product is a RAW editor first, and every implementation choice should preserve original-file safety and local-first behavior.
