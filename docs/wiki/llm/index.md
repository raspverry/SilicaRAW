---
title: LLM Routing Index
status: active
audience: agents
updated: 2026-06-13
source_of_truth: docs/wiki/index.md
---

# LLM Routing Index

## Summary

This page is the short routing layer for LLM agents. Use it to choose the smallest useful read set before working on SilicaRAW.

## Always Read First

- [Agent Rules](../../../codex/AGENT_RULES.md)
- [Wiki Index](../index.md)
- This page
- [Current LLM Route](current-route.md)

If the task changes schemas, dependencies, architecture, release behavior, or product scope, also read the specific source-of-truth document linked from the relevant route below.

## Current Route

Read [Current LLM Route](current-route.md) for the active work area and minimal read set.

## Completed Context

Read [Completed LLM Context](completed-context.md) when a task needs historical phase context.

## Read Avoidance Rules

- Prefer phase briefs over full roadmaps.
- Prefer task cards over phase briefs when the task is already selected.
- Prefer the master execution plan over new phase-wide planning when choosing Phase 14+ order.
- Prefer topic pages for durable facts.
- Use numbered docs and schemas only when the task touches their source-of-truth area.
- Do not use `docs/archive/` for implementation.

## Stop Gates

Stop and report before proceeding if a task would:

- Modify original photo files.
- Add RAW product pixels before fixture-backed proof exists.
- Add LibRaw or another decoder dependency without updating `docs/DEPENDENCIES.md`.
- Add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, Homebrew, or Mac App Store scope.
- Treat a wiki summary as permission to bypass schemas or agent rules.

## Notes for LLM Agents

This page is a routing index, not a replacement for source-of-truth files. Read the smallest linked set that can answer the task, then verify with the relevant harness.
