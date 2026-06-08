# Public and LLM-Readable Wiki Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the initial `docs/wiki/` scaffold as a public-facing, LLM-readable knowledge layer for SilicaRAW.

**Architecture:** The wiki lives under `docs/wiki/` and complements, but does not replace, the numbered specification documents. Pages are split by responsibility: overview, decisions, topics, sources, risks, questions, conventions, and an append-only log.

**Tech Stack:** Markdown only. No product code, runtime dependencies, RAW decoding, Metal viewer, MLX, MCP, plugin implementation, telemetry, or cloud behavior.

---

## File Structure

- Create: `docs/wiki/index.md` — main wiki entry point and navigation.
- Create: `docs/wiki/README.md` — short directory-level explanation for repository browsers.
- Create: `docs/wiki/conventions.md` — page format, status rules, and maintenance rules.
- Create: `docs/wiki/log.md` — append-only wiki change log.
- Create: `docs/wiki/overview/project.md` — project identity and scope.
- Create: `docs/wiki/overview/architecture.md` — architecture orientation and source links.
- Create: `docs/wiki/overview/roadmap.md` — roadmap orientation and current task sequence.
- Create: `docs/wiki/decisions/index.md` — decision record index.
- Create: `docs/wiki/decisions/adr-0001-monorepo-foundation.md` — initial monorepo foundation decision.
- Create: `docs/wiki/topics/raw-decoding.md` — RAW decoder topic page.
- Create: `docs/wiki/topics/metal-rendering.md` — Metal rendering topic page.
- Create: `docs/wiki/topics/color-management.md` — color management topic page.
- Create: `docs/wiki/topics/data-safety.md` — data safety topic page.
- Create: `docs/wiki/topics/edit-graph.md` — edit graph topic page.
- Create: `docs/wiki/topics/mlx.md` — MLX topic page.
- Create: `docs/wiki/topics/plugins-and-mcp.md` — plugin and MCP topic page.
- Create: `docs/wiki/sources/index.md` — external source index.
- Create: `docs/wiki/sources/karpathy-llm-wiki.md` — source note for Karpathy's LLM Wiki gist.
- Create: `docs/wiki/sources/karpathy-autoresearch.md` — source note for `karpathy/autoresearch`.
- Create: `docs/wiki/sources/huggingface-ml-intern.md` — source note for `huggingface/ml-intern`.
- Create: `docs/wiki/risks/index.md` — risk register index.
- Create: `docs/wiki/risks/architecture-risks.md` — initial architecture risks.
- Create: `docs/wiki/questions/open-questions.md` — open question register.
- Modify: `docs/00_INDEX.md` — link to the wiki.
- Modify: `README.md` — add a short wiki pointer.

## Task 1: Create Wiki Directories

**Files:**
- Create directories under `docs/wiki/`.

- [ ] **Step 1: Create directories**

Run:

```bash
mkdir -p docs/wiki/overview docs/wiki/decisions docs/wiki/topics docs/wiki/sources docs/wiki/risks docs/wiki/questions
```

Expected: command exits with status 0.

## Task 2: Add Wiki Entry and Conventions

**Files:**
- Create: `docs/wiki/index.md`
- Create: `docs/wiki/README.md`
- Create: `docs/wiki/conventions.md`
- Create: `docs/wiki/log.md`

- [ ] **Step 1: Write the main entry pages**

Add Markdown pages that define the wiki purpose, navigation, page rules, status values, and initial log entry.

- [ ] **Step 2: Verify required entry files exist**

Run:

```bash
test -f docs/wiki/index.md && test -f docs/wiki/README.md && test -f docs/wiki/conventions.md && test -f docs/wiki/log.md
```

Expected: command exits with status 0.

## Task 3: Add Overview Pages

**Files:**
- Create: `docs/wiki/overview/project.md`
- Create: `docs/wiki/overview/architecture.md`
- Create: `docs/wiki/overview/roadmap.md`

- [ ] **Step 1: Write overview pages**

Add human-readable project, architecture, and roadmap summaries that link to authoritative local documents.

- [ ] **Step 2: Verify overview files exist**

Run:

```bash
test -f docs/wiki/overview/project.md && test -f docs/wiki/overview/architecture.md && test -f docs/wiki/overview/roadmap.md
```

Expected: command exits with status 0.

## Task 4: Add Decision Pages

**Files:**
- Create: `docs/wiki/decisions/index.md`
- Create: `docs/wiki/decisions/adr-0001-monorepo-foundation.md`

- [ ] **Step 1: Write the decision index and ADR**

Record the accepted monorepo foundation decision without adding new architecture beyond the existing specifications.

- [ ] **Step 2: Verify decision files exist**

Run:

```bash
test -f docs/wiki/decisions/index.md && test -f docs/wiki/decisions/adr-0001-monorepo-foundation.md
```

Expected: command exits with status 0.

## Task 5: Add Topic Pages

**Files:**
- Create topic pages under `docs/wiki/topics/`.

- [ ] **Step 1: Write topic pages**

Create pages for RAW decoding, Metal rendering, color management, data safety, edit graph, MLX, and plugins/MCP. Each page must state what is known, what is blocked by a spike or later task, and which authoritative documents apply.

- [ ] **Step 2: Verify topic files exist**

Run:

```bash
for page in raw-decoding metal-rendering color-management data-safety edit-graph mlx plugins-and-mcp; do test -f docs/wiki/topics/$page.md; done
```

Expected: command exits with status 0.

## Task 6: Add Source Pages

**Files:**
- Create source pages under `docs/wiki/sources/`.

- [ ] **Step 1: Write source notes**

Create source notes for Karpathy's LLM Wiki gist, `karpathy/autoresearch`, and `huggingface/ml-intern`. Each source page must separate useful inspiration from adopted SilicaRAW decisions.

- [ ] **Step 2: Verify source files exist**

Run:

```bash
test -f docs/wiki/sources/index.md && test -f docs/wiki/sources/karpathy-llm-wiki.md && test -f docs/wiki/sources/karpathy-autoresearch.md && test -f docs/wiki/sources/huggingface-ml-intern.md
```

Expected: command exits with status 0.

## Task 7: Add Risk and Question Pages

**Files:**
- Create: `docs/wiki/risks/index.md`
- Create: `docs/wiki/risks/architecture-risks.md`
- Create: `docs/wiki/questions/open-questions.md`

- [ ] **Step 1: Write risk and question registers**

Add the initial risks and questions already present in the planning documents, with links back to authoritative sources.

- [ ] **Step 2: Verify register files exist**

Run:

```bash
test -f docs/wiki/risks/index.md && test -f docs/wiki/risks/architecture-risks.md && test -f docs/wiki/questions/open-questions.md
```

Expected: command exits with status 0.

## Task 8: Link Wiki from Existing Docs

**Files:**
- Modify: `docs/00_INDEX.md`
- Modify: `README.md`

- [ ] **Step 1: Add wiki links**

Add concise links to `docs/wiki/index.md` from the docs index and root README.

- [ ] **Step 2: Verify links are present**

Run:

```bash
rg -n "docs/wiki/index.md|Wiki" README.md docs/00_INDEX.md
```

Expected: output includes both `README.md` and `docs/00_INDEX.md`.

## Task 9: Final Verification

**Files:**
- All created and modified Markdown files.

- [ ] **Step 1: Verify no product code was added by this wiki task**

Run:

```bash
find docs/wiki -type f | sort
```

Expected: output lists only Markdown files under `docs/wiki/`.

- [ ] **Step 2: Verify no new dependencies were added**

Run:

```bash
cargo metadata --format-version 1 --no-deps >/tmp/silicaraw-cargo-metadata.json && jq '[.packages[] | {name, dependency_count:(.dependencies|length)}]' /tmp/silicaraw-cargo-metadata.json
```

Expected: every `dependency_count` is `0`.

- [ ] **Step 3: Verify build and tests still pass**

Run:

```bash
cargo fmt --all --check
cargo build --workspace
cargo test --workspace
```

Expected: all commands exit with status 0.

