# Contributing to SilicaRAW

Thanks for your interest in SilicaRAW.

## Project Identity

SilicaRAW is a RAW photo editor first. MLX, plugins, and MCP are secondary.

## Before You Start

Read:

- `MANIFEST.md`
- `docs/18_Final_Master_Plan.md`
- `codex/AGENT_RULES.md`

## What We Want Early

- Feasibility spikes
- Tests
- Design system components
- Catalog/storage safety
- Metal viewer work
- Documentation improvements

## What We Do Not Want Yet

- Object removal
- Generative fill
- Cloud sync
- Mobile app
- Plugin marketplace
- Dangerous MCP tools
- Arbitrary native plugins

## UI Rules

Use design tokens. Do not hard-code colors, radius, spacing, or component styles.

## Safety Rules

Never modify original photo files. Any code touching file paths must be reviewed carefully.

## PR Checklist

- Scope is small
- Tests added or rationale provided
- No unrelated changes
- No original mutation
- No unapproved dependencies
- Docs updated if behavior changed
