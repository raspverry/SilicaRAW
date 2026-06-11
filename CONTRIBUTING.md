# Contributing to SilicaRAW

SilicaRAW is an early-stage local-first photo editor. Contributions are welcome when they keep the project honest about current capability and preserve original-file safety.

## Start Here

Before implementation work, read:

- [AGENTS.md](AGENTS.md)
- [README.md](README.md)
- [Public Trust](docs/wiki/topics/public-trust.md)
- [Post-Alpha Product Roadmap](docs/wiki/roadmaps/post-alpha-product-roadmap.md)
- [Git and PR Workflow](docs/wiki/contributing/git-and-pr-workflow.md)

## Branches

Use short scoped branches:

- `feature/<topic>` for new scoped behavior.
- `fix/<topic>` for bug fixes.
- `docs/<topic>` for documentation-only changes.
- `spike/<topic>` for explicitly bounded research work.

Keep each branch atomic and committable. Do not mix unrelated product, docs, release, and refactor work in one PR.

## Pull Requests

Every PR should include:

- A concrete summary.
- Explicit scope and out-of-scope notes.
- Validation commands and results.
- Documentation updates when behavior, claims, or roadmap status changes.
- Dependency inventory updates in [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md) for every new dependency.

Run the harness before claiming a PR is ready:

```bash
scripts/harness/check.sh
```

Use narrower commands only when the full harness cannot run, and explain why.

## Local-First Safety Rules

- Do not modify original photo files.
- Do not add network upload, telemetry, analytics, or cloud sync by default.
- Do not add MLX, MCP, plugin runtime, Homebrew, auto-update, or Mac App Store distribution unless the roadmap explicitly scopes the task.
- Do not claim broad RAW support, color correctness, or production readiness without fixture-backed evidence and documented tolerance checks.
- Do not expose raw SQLite access outside typed storage/core APIs.

## Dependencies

New dependencies are allowed only when they are needed for the scoped task and recorded in [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md).

The dependency entry must include version, purpose, license, source, alternatives, risk notes, binary size impact, security notes, and verification source.

## Documentation

The wiki is for both people and LLM agents. Keep it factual:

- Record decisions in ADRs instead of silently changing accepted direction.
- Update roadmap status only after implementation and validation land.
- Mark future or deferred work clearly.
- Avoid marketing language that implies unsupported features are available.

## Issue Triage

Use the issue templates. If a report involves original-file mutation, catalog corruption, export path safety, or private security details, treat it as high priority and avoid posting sensitive exploit details publicly.
