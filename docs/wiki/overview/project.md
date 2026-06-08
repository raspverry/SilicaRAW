---
title: Project Overview
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/01_Vision_and_Positioning.md
---

# Project Overview

## Summary

SilicaRAW is an early-stage, open-source RAW photo editor for Apple Silicon. Its primary identity is a local-first, non-destructive RAW editor with Metal-first performance.

AI, MLX, plugins, and MCP are secondary extensions. They should support the editor, not redefine it.

## Current State

- The repository has planning documents, schemas, mockups, and a Rust workspace scaffold.
- The project is not production-ready.
- The app shell, initial catalog migration foundation, local library create/open path, folder import scanner, catalog-backed flag persistence, and minimal preview readiness path exist, but visual culling and editing workflows are not implemented yet.
- RAW decoding, the product Metal viewer bridge, color fixture proof, export, and local alpha UI workflows still require explicit implementation tasks.
- `MockupUI/` contains the high-fidelity target screens. M004 Library Loupe and M005 Develop are the relevant preview consumers, but they are not implemented as product screens yet.
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
