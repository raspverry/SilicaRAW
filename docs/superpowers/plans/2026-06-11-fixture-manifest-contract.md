# Fixture Manifest Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Task 10.1 by adding a legal RAW/color fixture manifest schema, example, deterministic harness check, and documentation guardrails without adding decoding, color proof, real fixture files, or dependencies.

**Architecture:** The fixture manifest is a static contract under `schemas/`, validated by a Python stdlib harness script. It remains separate from catalog, sidecar, edit graph, export, and generated local-alpha QA fixture state. Documentation records the trust boundary: the manifest describes provenance and expectations, not RAW support or color correctness proof.

**Tech Stack:** JSON Schema draft 2020-12, JSON example file, Python 3 standard library, Markdown docs, existing `scripts/harness/check.sh`.

---

## File Structure

- Create: `schemas/fixture_manifest.schema.json`
  - Canonical schema for post-alpha RAW/color fixture manifests.
- Create: `schemas/fixture_manifest.example.json`
  - Example-only manifest with no real fixture files and no user photos.
- Create: `scripts/harness/check-fixture-manifest-contract.py`
  - Stdlib validation for schema/example/docs guardrails.
- Modify: `scripts/harness/check.sh`
  - Runs the new checker near existing QA fixture checks.
- Modify: `docs/19_Schema_Reference.md`
  - Adds the new schema and example to the authoritative schema list.
- Modify: `docs/wiki/topics/raw-decoding.md`
  - Documents RAW fixture classes and blocked support-state boundary.
- Modify: `docs/wiki/topics/color-management.md`
  - Documents Color Class F and color-correctness boundary.
- Modify: `docs/wiki/roadmaps/post-alpha-product-roadmap.md`
  - Marks Task 10.1 completed after implementation.
- Modify: `docs/wiki/log.md`
  - Adds an append-only Task 10.1 entry.

Do not modify `scripts/harness/generate-legal-fixtures.py` in this task. Its local-alpha synthetic fixture manifest remains a separate generated QA artifact.

## Task 1: Add Fixture Manifest Schema and Example

**Files:**
- Create: `schemas/fixture_manifest.schema.json`
- Create: `schemas/fixture_manifest.example.json`

- [ ] **Step 1: Create schema file**

Create `schemas/fixture_manifest.schema.json` with these required top-level rules:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://silicaraw.local/schemas/fixture_manifest.schema.json",
  "title": "SilicaRAW Fixture Manifest v1",
  "type": "object",
  "required": [
    "schema",
    "version",
    "manifest_kind",
    "source_policy",
    "maintained_by",
    "updated_at",
    "fixtures",
    "expected_source_hashes"
  ],
  "additionalProperties": false,
  "properties": {
    "schema": { "const": "silica.fixture_manifest" },
    "version": { "type": "integer", "const": 1 },
    "manifest_kind": {
      "type": "string",
      "enum": ["synthetic-local-alpha", "raw-fixtures", "color-fixtures", "mixed"]
    },
    "source_policy": { "$ref": "#/$defs/source_policy" },
    "maintained_by": { "type": "string", "minLength": 1 },
    "updated_at": { "type": "string", "minLength": 1 },
    "fixtures": {
      "type": "array",
      "minItems": 1,
      "items": { "$ref": "#/$defs/fixture" }
    },
    "expected_source_hashes": {
      "type": "object",
      "additionalProperties": {
        "type": "string",
        "pattern": "^[0-9a-f]{64}$"
      }
    },
    "notes": { "type": "string" },
    "extensions": {
      "type": "object",
      "additionalProperties": true
    }
  }
}
```

Then add `$defs` for:

- `source_policy`
- `source`
- `license`
- `privacy`
- `integrity`
- `media`
- `expected_app_state`
- `expected_probe_state`
- `raw`
- `decode_gate`
- `color`
- `profile_expectation`
- `fixture`

The `fixture` definition must require:

```json
[
  "id",
  "class",
  "kind",
  "relative_path",
  "availability",
  "source",
  "license",
  "privacy",
  "integrity",
  "media",
  "expected_app_state",
  "expected_probe_state"
]
```

Use these enum values:

```json
{
  "class": ["A", "B", "C", "D", "E", "F"],
  "kind": ["raw", "tagged_raster", "untagged_raster", "unsupported", "raw_blocked_placeholder"],
  "availability": ["generated", "committed", "local_ignored", "external_reference_only"],
  "preview_status": ["ready_by_reference", "raw_decode_blocked", "unsupported", "missing", "not_probed"],
  "probe_state": ["unverified", "blocked_pending_task_12", "blocked_pending_task_13"]
}
```

- [ ] **Step 2: Create example manifest**

Create `schemas/fixture_manifest.example.json` with:

- one RAW Class A external-reference-only fixture
- one RAW Class C Fuji RAF external-reference-only fixture
- one RAW Class D Apple ProRAW DNG external-reference-only fixture
- three Color Class F external-reference-only fixtures: sRGB JPEG, Display P3 JPEG, untagged JPEG
- all `expected_probe_state.state` values set to `unverified` or a blocked pending state
- all RAW `decode_gate.state` values set to `blocked_pending_task_12`
- no absolute paths
- no real local sample paths
- no user photos

Use fixed example SHA-256 strings such as:

```json
"00000000000000000000000000000000000000000000000000000000000000a1"
```

Use `availability: "external_reference_only"` for all entries so the example is clearly not a committed fixture corpus.

- [ ] **Step 3: Verify JSON loads**

Run:

```bash
python3 -m json.tool schemas/fixture_manifest.schema.json >/tmp/fixture-schema.json
python3 -m json.tool schemas/fixture_manifest.example.json >/tmp/fixture-example.json
```

Expected: both commands exit with status 0.

## Task 2: Add Fixture Manifest Contract Checker

**Files:**
- Create: `scripts/harness/check-fixture-manifest-contract.py`

- [ ] **Step 1: Create checker skeleton**

Create `scripts/harness/check-fixture-manifest-contract.py` with this structure:

```python
#!/usr/bin/env python3
import json
import sys
from pathlib import PurePosixPath, Path


ROOT = Path(__file__).resolve().parents[2]
SCHEMA = ROOT / "schemas/fixture_manifest.schema.json"
EXAMPLE = ROOT / "schemas/fixture_manifest.example.json"
RAW_DOC = ROOT / "docs/wiki/topics/raw-decoding.md"
COLOR_DOC = ROOT / "docs/wiki/topics/color-management.md"
SCHEMA_REFERENCE = ROOT / "docs/19_Schema_Reference.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def load_json(path, failures):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        failures.append(f"failed to load {path.relative_to(ROOT)}: {exc}")
        return {}


def is_safe_relative_path(value):
    if not isinstance(value, str) or not value:
        return False
    path = PurePosixPath(value)
    return not path.is_absolute() and ".." not in path.parts
```

- [ ] **Step 2: Add schema and top-level validation**

Add a `validate_manifest(manifest, failures)` function that checks:

```python
require(manifest.get("schema") == "silica.fixture_manifest", "manifest schema must be silica.fixture_manifest", failures)
require(manifest.get("version") == 1, "manifest version must be 1", failures)
require(manifest.get("manifest_kind") in {"synthetic-local-alpha", "raw-fixtures", "color-fixtures", "mixed"}, "manifest_kind must be known", failures)
require(isinstance(manifest.get("fixtures"), list) and manifest["fixtures"], "fixtures must be a non-empty list", failures)
require(isinstance(manifest.get("expected_source_hashes"), dict), "expected_source_hashes must be an object", failures)
```

- [ ] **Step 3: Add per-fixture validation**

In `validate_manifest`, iterate over fixtures and check:

```python
required = [
    "id",
    "class",
    "kind",
    "relative_path",
    "availability",
    "source",
    "license",
    "privacy",
    "integrity",
    "media",
    "expected_app_state",
    "expected_probe_state",
]
for key in required:
    require(key in fixture, f"{fixture_id} missing {key}", failures)
```

Also check:

- fixture IDs are unique
- `relative_path` passes `is_safe_relative_path`
- `expected_source_hashes[relative_path]` equals `integrity.sha256`
- `integrity.sha256` is 64 lowercase hex characters
- committed fixtures cannot have `license.name` equal to `Unknown`
- committed fixtures cannot have `privacy.is_user_photo` set to true
- `raw` fixtures have `raw` and `decode_gate`
- Color Class F fixtures have `color` and `profile_expectation`

- [ ] **Step 4: Add RAW gate checks**

Add checks:

```python
if fixture.get("class") in {"A", "B", "C", "D", "E"}:
    decode_gate = fixture.get("decode_gate", {})
    require(decode_gate.get("state") == "blocked_pending_task_12", f"{fixture_id} RAW decode gate must stay blocked_pending_task_12", failures)
    require(fixture.get("expected_probe_state", {}).get("actual_result") in (None, "not_recorded"), f"{fixture_id} must not record actual RAW probe results in Task 10.1", failures)
```

Add class-specific checks:

- Class C requires `raw.format` equal to `raf`
- Class D requires `raw.format` equal to `dng` and `raw.apple_proraw` equal to true
- `raw_blocked_placeholder` must not be used as a real RAW fixture

- [ ] **Step 5: Add Color Class F checks**

Check that the example includes subclasses:

```python
required_subclasses = {"srgb_jpeg", "display_p3_jpeg", "untagged_jpeg"}
seen_subclasses = {
    fixture.get("color", {}).get("subclass")
    for fixture in fixtures
    if fixture.get("class") == "F"
}
require(required_subclasses.issubset(seen_subclasses), "Class F example must include srgb_jpeg, display_p3_jpeg, and untagged_jpeg", failures)
```

For each Class F fixture:

- `srgb_jpeg` and `display_p3_jpeg` require `profile_expectation.embedded_icc` true
- `untagged_jpeg` requires `profile_expectation.embedded_icc` false
- `untagged_jpeg` requires `profile_expectation.untagged_policy` equal to `assume_srgb`
- no Class F fixture may claim `color_correctness_proven` true

- [ ] **Step 6: Add docs guardrail checks**

In `main()`, read docs and assert:

```python
raw_doc = RAW_DOC.read_text(encoding="utf-8")
color_doc = COLOR_DOC.read_text(encoding="utf-8")
schema_reference = SCHEMA_REFERENCE.read_text(encoding="utf-8")
require("fixture_manifest.schema.json" in schema_reference, "schema reference must list fixture manifest schema", failures)
require("no committed legal RAW fixture corpus" in raw_doc, "RAW docs must state no committed legal RAW fixture corpus", failures)
require("RAW support claims remain blocked" in raw_doc, "RAW docs must preserve support-claim boundary", failures)
require("Color Class F" in color_doc, "color docs must describe Color Class F", failures)
require("do not prove color correctness" in color_doc, "color docs must preserve color-correctness boundary", failures)
```

- [ ] **Step 7: Run checker and observe failure before docs updates**

Run:

```bash
python3 scripts/harness/check-fixture-manifest-contract.py
```

Expected before Task 3 docs updates: failure mentioning missing schema reference and missing docs guardrails.

## Task 3: Wire Checker Into Harness

**Files:**
- Modify: `scripts/harness/check.sh`

- [ ] **Step 1: Add harness call**

Insert after the release template checks and before QA fixture generation:

```bash
echo "==> Checking RAW/color fixture manifest contract"
python3 scripts/harness/check-fixture-manifest-contract.py
```

- [ ] **Step 2: Run narrow harness command**

Run:

```bash
python3 scripts/harness/check-fixture-manifest-contract.py
```

Expected before Task 4 docs updates: schema/example checks pass, docs guardrail checks fail.

## Task 4: Update Schema Reference and Wiki Topics

**Files:**
- Modify: `docs/19_Schema_Reference.md`
- Modify: `docs/wiki/topics/raw-decoding.md`
- Modify: `docs/wiki/topics/color-management.md`
- Modify: `docs/wiki/roadmaps/post-alpha-product-roadmap.md`
- Modify: `docs/wiki/log.md`

- [ ] **Step 1: Update schema reference**

Add `schemas/fixture_manifest.schema.json` and `schemas/fixture_manifest.example.json` to `docs/19_Schema_Reference.md`.

Include this wording:

```md
The fixture manifest schema records legal RAW/color fixture provenance, licensing, integrity, expected app behavior, and future probe expectations. It does not prove RAW support or color correctness.
```

- [ ] **Step 2: Update RAW topic**

In `docs/wiki/topics/raw-decoding.md`, add a `Task 10.1 Fixture Manifest Contract` section with:

```md
The repository has no committed legal RAW fixture corpus. Task 10.1 defines RAW fixture provenance and expected gate states only. RAW support claims remain blocked until fixture-backed Core Image probe work in Phase 12 records evidence. RAW-blocked placeholders are blocked-state fixtures, not decodable RAW evidence.
```

- [ ] **Step 3: Update color topic**

In `docs/wiki/topics/color-management.md`, add a `Task 10.1 Color Class F Contract` section with:

```md
Color Class F covers tagged sRGB, tagged Display P3, and untagged raster fixture expectations. Hashes, profile declarations, and manifest entries do not prove color correctness. Color correctness claims remain blocked until fixture-backed proof and a tolerance policy exist.
```

- [ ] **Step 4: Mark Task 10.1 completed in roadmap**

In `docs/wiki/roadmaps/post-alpha-product-roadmap.md`, add a status paragraph under Task 10.1:

```md
**Status:** Completed on 2026-06-11. Added the fixture manifest schema, example, and harness contract for legal RAW classes and Color Class F fixture expectations. This records provenance, license, privacy, integrity, expected app states, and future probe expectations only; it does not add RAW decoding, Core Image probing, real fixture files, ICC parsing, or color correctness proof.
```

- [ ] **Step 5: Add wiki log entry**

Add an entry near the top of `docs/wiki/log.md`:

```md
## [2026-06-11] phase-10 | Fixture manifest contract added

- Added the RAW/color fixture manifest schema and example for Task 10.1.
- Added a deterministic harness check for fixture provenance, license, path, hash, RAW gate, and Color Class F expectations.
- Recorded that manifests are provenance and expectation metadata, not RAW support or color correctness proof.
```

## Task 5: Verify Contract and Harness

**Files:**
- Validate all files changed by Tasks 1-4.

- [ ] **Step 1: Run fixture contract checker**

Run:

```bash
python3 scripts/harness/check-fixture-manifest-contract.py
```

Expected:

```txt
fixture manifest contract ok
```

- [ ] **Step 2: Run existing QA fixture check**

Run:

```bash
python3 scripts/harness/check-qa-fixtures.py
```

Expected:

```txt
qa fixtures and installed-app preflight ok
```

- [ ] **Step 3: Run Markdown link check**

Run:

```bash
python3 scripts/harness/check-md-links.py
```

Expected: local links ok.

- [ ] **Step 4: Run Python compile check**

Run:

```bash
python3 -m py_compile scripts/harness/check-fixture-manifest-contract.py
```

Expected: command exits with status 0.

Remove generated `scripts/harness/__pycache__` after this command:

```bash
rm -rf scripts/harness/__pycache__
```

- [ ] **Step 5: Run full harness**

Run:

```bash
scripts/harness/check.sh
```

Expected:

```txt
==> Harness checks passed
```

## Task 6: Review and Commit

**Files:**
- All files from Tasks 1-4.

- [ ] **Step 1: Inspect git diff**

Run:

```bash
git diff --check
git diff --stat
```

Expected: no whitespace errors; stat includes schema, example, checker, harness, and docs.

- [ ] **Step 2: Run code-review graph impact**

Run the code-review graph incremental update and review context:

```txt
build_or_update_graph_tool(repo_root="/Users/hansol/dev/personal/SilicaRAW", base="main", postprocess="minimal")
get_review_context_tool(repo_root="/Users/hansol/dev/personal/SilicaRAW", base="main", detail_level="minimal", max_depth=1, include_source=false)
```

Expected: low risk; no product runtime nodes impacted.

- [ ] **Step 3: Commit implementation**

Run:

```bash
git add schemas/fixture_manifest.schema.json \
  schemas/fixture_manifest.example.json \
  scripts/harness/check-fixture-manifest-contract.py \
  scripts/harness/check.sh \
  docs/19_Schema_Reference.md \
  docs/wiki/topics/raw-decoding.md \
  docs/wiki/topics/color-management.md \
  docs/wiki/roadmaps/post-alpha-product-roadmap.md \
  docs/wiki/log.md
git commit -m "test(fixtures): add RAW color manifest contract"
```

Expected: commit succeeds with only Task 10.1 files.

## Stop Conditions

Stop and ask for review if any implementation step requires:

- adding a new dependency
- committing real RAW/color fixture files
- using local user photo paths
- adding RAW decoding or Core Image calls
- adding LibRaw
- parsing ICC profiles
- adding golden-image tolerance comparisons
- touching catalog, sidecar, edit graph, or export runtime code

## Final Verification

Before opening the PR, run:

```bash
scripts/harness/check.sh
git status --short --branch
```

Expected:

- harness passes
- branch has only committed Task 10.1 implementation files
- no generated fixture output or `__pycache__` directory is tracked
