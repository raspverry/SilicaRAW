---
title: Git and PR Workflow
status: active
audience: contributors
updated: 2026-06-22
source_of_truth: none
---

# Git and PR Workflow

## Summary

SilicaRAW uses a lightweight GitHub Flow model: keep `main` stable, do work on short-lived branches, and merge through pull requests.

Do not add a long-lived `dev` branch unless the project reaches a scale where `main` cannot remain the integration branch. For now, a `dev` branch would create ambiguity for contributors and agents.

## Branch Model

- `main`: stable integration branch. It should always pass CI.
- `feature/<slug>`: product, architecture, or phase work.
- `fix/<slug>`: bug fixes.
- `docs/<slug>`: documentation, wiki, or contributor guide changes.
- `ci/<slug>`: CI, harness, release automation, or repository guardrails.
- `chore/<slug>`: repository maintenance that does not change product behavior.
- `spike/<slug>`: exploratory work that may be discarded.
- `release/vX.Y.Z-alpha.N`: release preparation branches, used only when packaging and distribution work begins.

Use issue or phase identifiers when helpful:

```text
feature/phase-2-desktop-shell
ci/phase-1-scope-guardrails
docs/git-pr-workflow
fix/catalog-path-normalization
spike/raw-decoder-license-check
release/v0.1.0-alpha.1
```

## Contributor Flow

1. Start from current `main`.
2. Create a focused branch for one task or one tightly related set of changes.
3. Keep commits small and reviewable.
4. Open a pull request into `main`.
5. Run the project harness locally before marking the PR ready.
6. Wait for GitHub Actions CI to pass.
7. Merge with squash merge after review.
8. Delete the branch after merge.

Suggested commands:

```bash
git switch main
git pull --ff-only origin main
git switch -c feature/phase-2-desktop-shell
scripts/harness/check.sh
git push -u origin feature/phase-2-desktop-shell
```

## Pull Request Rules

- Target `main`.
- Keep one task, phase slice, or issue per PR.
- Use the repository PR template.
- Fill in out-of-scope items explicitly.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not combine risky implementation work with unrelated formatting or large documentation rewrites.
- Do not merge if CI fails.

## Visual QA Gate

Keep the default `scripts/harness/check.sh` path reasonably small. UI-affecting PRs must also run:

```bash
python3 scripts/harness/run-final-visual-qa.py
```

A PR is UI-affecting when it changes any of these areas:

- `apps/desktop/static/**`
- `MockupUI/**`
- `scripts/harness/run-final-visual-qa.py`
- visual QA docs or plans that define screenshot surfaces, viewport coverage, mockup parity, or UI gate policy

GitHub Actions runs the separate **Final Visual QA** workflow for those paths and for gate-policy files such as `.github/PULL_REQUEST_TEMPLATE.md`, this page, and the workflow itself. Non-UI PRs keep the normal CI harness without screenshot generation.

## Commit Message Style

Use short, conventional-style messages without requiring a strict release automation contract yet:

```text
ci(harness): add phase 1 guardrails
docs(wiki): document git and PR workflow
feature(desktop): add minimal app shell
fix(storage): normalize catalog paths
chore(repo): update ignore rules
```

## Maintainer Settings

Recommended GitHub repository settings:

- Default branch: `main`.
- Require pull requests before merging into `main`.
- Require GitHub Actions CI before merging.
- Prefer squash merge for ordinary PRs.
- Delete head branches after merge.
- Keep merge commits available only if a release branch or multi-commit history needs it.

## Release Branches

Use `release/*` only when preparing a local DMG release. Release branches may collect packaging, signing, notarization, and final documentation fixes for a specific version.

Do not use release branches for ordinary feature integration.

## Notes for LLM Agents

- Assume `main` is the integration target unless the user explicitly says otherwise.
- Do not create or use a long-lived `dev` branch without an accepted decision record.
- Prefer a new short-lived branch per phase or issue.
- Keep PRs atomic and easy for human contributors to review.
- Preserve the explicit out-of-scope boundaries in the PR description.

## Links

- [Pull Request Template](../../../.github/PULL_REQUEST_TEMPLATE.md)
- [Project Harness](../../../harness/README.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)
