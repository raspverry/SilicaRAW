# Public and LLM-Readable Wiki Design

Date: 2026-06-08
Status: Approved for implementation planning
Scope: SilicaRAW documentation wiki scaffold

## Purpose

SilicaRAW needs a public-facing wiki that is useful to both people and LLM agents.

The wiki is not a private agent scratchpad and it is not a duplicate copy of the main specification documents. It is a maintained knowledge layer that explains the project, connects authoritative sources, records decisions, tracks risks, and gives future contributors and LLM agents a reliable orientation point.

## Audience

The wiki serves three audiences:

- Human contributors who need to understand what SilicaRAW is, why decisions were made, and where to start.
- Maintainers who need a clear place to record decisions, unresolved questions, and project risks.
- LLM agents that need concise, structured, source-linked context before editing code or documentation.

## Design Principles

- Write in clear English.
- Prefer short pages with explicit links over long omnibus documents.
- Keep authoritative specifications in `docs/`; use the wiki to connect, interpret, and track them.
- Put stable project knowledge in topic pages.
- Put decisions in ADR-style decision records.
- Put external references and research notes in source pages.
- Make each page useful when read alone.
- Include enough structure for LLM retrieval without making pages unpleasant for humans.
- Avoid hidden implementation instructions that contradict `codex/AGENT_RULES.md` or the core specifications.

## Proposed Location

```txt
docs/wiki/
```

This location keeps the wiki inside the documentation tree while separating it from numbered specification documents.

## Initial Structure

```txt
docs/wiki/
  index.md
  README.md
  conventions.md
  log.md

  overview/
    project.md
    architecture.md
    roadmap.md

  decisions/
    index.md
    adr-0001-monorepo-foundation.md

  topics/
    raw-decoding.md
    metal-rendering.md
    color-management.md
    data-safety.md
    edit-graph.md
    mlx.md
    plugins-and-mcp.md

  sources/
    index.md
    karpathy-llm-wiki.md
    karpathy-autoresearch.md
    huggingface-ml-intern.md

  risks/
    index.md
    architecture-risks.md

  questions/
    open-questions.md
```

## Category Roles

### `overview/`

Human-readable orientation pages. These pages explain the project, architecture, and roadmap without requiring readers to open every specification document.

### `decisions/`

ADR-style decision records. Each decision page should explain context, decision, consequences, and links to relevant specifications or tasks.

### `topics/`

Stable topic pages for domains that future work will revisit repeatedly. These pages should summarize current project stance, authoritative sources, key constraints, and known open issues.

### `sources/`

External reference notes. These pages should capture what was learned from a source, why it matters to SilicaRAW, and what should not be copied blindly.

### `risks/`

Risk register pages. These pages track project risks that affect architecture, sequencing, implementation safety, or release trust.

### `questions/`

Open questions that need maintainer decisions, spikes, tests, or external research.

### `log.md`

Append-only wiki change log. This gives humans and LLM agents a compact timeline of wiki updates and important project context changes.

## Page Format

Every wiki page should use this general shape:

```md
---
title: Page Title
status: draft | active | superseded
audience: contributors | maintainers | agents | all
updated: YYYY-MM-DD
source_of_truth: path-or-none
---

# Page Title

## Summary

Short summary in plain English.

## Key Points

- Important point.
- Important point.

## Links

- Related local document or source.

## Notes for LLM Agents

Concise guidance for agents reading this page before taking action.
```

The exact sections may vary by page type, but each page should include a summary, links, and a clear status.

## ADR Format

Decision records should use:

```md
---
title: ADR 0001: Decision Title
status: accepted | proposed | superseded
updated: YYYY-MM-DD
---

# ADR 0001: Decision Title

## Context

## Decision

## Consequences

## Alternatives Considered

## Links
```

## LLM Readability Rules

- Avoid clever prose, jokes, and ambiguous shorthand.
- Use exact file paths for local references.
- Use direct links for external sources.
- Clearly separate facts, decisions, recommendations, and open questions.
- Do not instruct agents to ignore project guardrails.
- Prefer explicit "Do" and "Do not" bullets where future agent behavior matters.

## Human Readability Rules

- Avoid turning every page into a machine checklist.
- Start with why the page exists.
- Keep sections short enough to scan.
- Link to authoritative docs instead of copying long sections.
- Explain tradeoffs, not just outcomes.

## Initial Source Notes

The first source pages should cover:

- Andrej Karpathy's LLM Wiki gist: useful as inspiration for public, LLM-readable knowledge structures.
- `karpathy/autoresearch`: useful for the idea that Markdown context can steer agent behavior, but SilicaRAW should not become an autonomous experiment repo.
- `huggingface/ml-intern`: useful for agent operations, traceability, tool routing, and public documentation style, but SilicaRAW should avoid telemetry or cloud assumptions by default.

## Out of Scope

- Implementing product features.
- Adding RAW decoding, Metal viewer, MLX, plugin, MCP, telemetry, or cloud features.
- Replacing numbered specification documents.
- Creating a private task log that only agents can understand.

## Acceptance Criteria

- `docs/wiki/index.md` exists and explains the wiki's purpose and navigation.
- `docs/wiki/conventions.md` defines page format and maintenance rules.
- Initial overview, decision, topic, source, risk, and question pages exist.
- All wiki content is in English.
- External source pages distinguish inspiration from adopted project decisions.
- The root docs index points readers to the wiki.
- No product code or dependencies are added.

