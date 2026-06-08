---
title: Wiki Conventions
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/superpowers/specs/2026-06-08-public-llm-wiki-design.md
---

# Wiki Conventions

## Summary

These rules keep the SilicaRAW wiki readable for people and reliable for LLM agents.

## Page Frontmatter

Every wiki page should start with:

```yaml
---
title: Page Title
status: draft | active | superseded
audience: contributors | maintainers | agents | all
updated: YYYY-MM-DD
source_of_truth: path-or-none
---
```

## Status Values

- `draft`: useful but incomplete or not yet reviewed.
- `active`: current and safe to use as orientation.
- `superseded`: retained for history, but replaced by another page or decision.

## Page Shape

Most pages should include:

- `Summary`: short explanation of why the page exists.
- `Key Points`: concise facts, decisions, or constraints.
- `Links`: local docs and external references.
- `Notes for LLM Agents`: action-oriented guidance for agents.

Decision records use the ADR shape defined in [Decision Records](decisions/index.md).

## Writing Rules

- Write in clear English.
- Prefer short sections and direct links.
- Do not copy long sections from the numbered specifications.
- Clearly separate facts, accepted decisions, recommendations, and open questions.
- Use exact file paths for local references.
- Use direct URLs for external references.
- Do not add hidden instructions that conflict with [Agent Rules](../../codex/AGENT_RULES.md).

## Maintenance Rules

- Update [index.md](index.md) when adding a new wiki page.
- Append a dated entry to [log.md](log.md) for meaningful wiki changes.
- Record durable architecture or sequencing choices as ADRs under `docs/wiki/decisions/`.
- Keep research notes under `docs/wiki/sources/` or topic pages, not in random files.
- Mark stale or replaced pages as `superseded` instead of deleting them casually.

## Notes for LLM Agents

When editing the wiki, preserve human readability. Do not turn public pages into private scratchpads, chain-of-thought logs, or task transcripts.

