---
title: karpathy/autoresearch
status: active
audience: all
updated: 2026-06-08
source_of_truth: https://github.com/karpathy/autoresearch
---

# karpathy/autoresearch

## Summary

`karpathy/autoresearch` demonstrates a small repository where Markdown instructions guide autonomous agents through repeated machine learning experiments.

SilicaRAW is not adopting autonomous experimentation, but the project is useful as a reminder that clear Markdown context can shape agent behavior.

## Useful Ideas

- Keep agent-facing instructions small, explicit, and close to the work.
- Make the editable surface area clear.
- Make success criteria measurable.
- Preserve a clean boundary between human-authored direction and agent-modified work.

## SilicaRAW Adaptation

SilicaRAW uses these ideas through:

- [Codex Handoff](../../../codex/CODEX_HANDOFF.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)
- Task-scoped implementation prompts.
- Wiki pages with `Notes for LLM Agents`.

## Not Adopted

- No autonomous overnight experiment loop.
- No self-modifying research workflow.
- No GPU training process.
- No agent authority to change project direction without human review.

## Links

- Repository: https://github.com/karpathy/autoresearch
- [Roadmap Overview](../overview/roadmap.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)

## Notes for LLM Agents

Use this source as inspiration for clear operating context, not as permission to run open-ended experiments in SilicaRAW.

