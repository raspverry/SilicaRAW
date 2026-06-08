---
title: Karpathy LLM Wiki
status: active
audience: all
updated: 2026-06-08
source_of_truth: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
---

# Karpathy LLM Wiki

## Summary

Andrej Karpathy's LLM Wiki gist describes a pattern where an LLM helps maintain a persistent, structured Markdown wiki instead of repeatedly re-deriving answers from raw sources.

This is the main inspiration for `docs/wiki/`.

## Useful Ideas

- Treat the wiki as a compounding artifact that improves as sources and questions accumulate.
- Keep raw sources distinct from the maintained wiki.
- Maintain a schema or conventions document so the LLM behaves like a disciplined wiki maintainer.
- Use `index.md` as the content-oriented map.
- Use `log.md` as the chronological, append-only history.
- Periodically lint the wiki for stale claims, contradictions, orphan pages, and missing cross-references.

## SilicaRAW Adaptation

SilicaRAW adapts this pattern as a public project wiki:

- `docs/` remains the authoritative specification layer.
- `schemas/` remains the authoritative schema layer.
- `docs/wiki/` becomes the maintained, human-readable and LLM-readable knowledge layer.
- `docs/wiki/conventions.md` acts as the wiki schema and maintenance protocol.
- `docs/wiki/index.md` and `docs/wiki/log.md` are required navigation files.

## Not Adopted

- The wiki is not private by default.
- The LLM does not own the wiki without human review.
- No extra search, vector, Obsidian, or MCP tooling is required for the initial scaffold.
- No source ingestion automation is added in this task.

## Links

- Source: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
- [Wiki Conventions](../conventions.md)
- [Wiki Log](../log.md)

## Notes for LLM Agents

Maintain the wiki as a public artifact. Do not write private reasoning traces or unreviewed speculation as if it were project fact.

