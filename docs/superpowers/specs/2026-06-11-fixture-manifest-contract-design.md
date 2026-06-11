# Fixture Manifest Contract Design

## Goal

Define the Task 10.1 contract for legal RAW and color fixture manifests before adding RAW decoding, Core Image probes, LibRaw, ICC parsing, golden-image comparisons, or color correctness claims.

This is a contract and guardrail task. It records provenance, licensing, integrity, expected app states, and future probe expectations. It does not prove RAW support or color correctness.

## Background

The post-alpha product roadmap starts with evidence and trust gates. Task 10.1 requires a manifest format and validation checks for legal RAW fixture classes and tagged color fixtures.

Current local-alpha fixture tooling already generates synthetic JPEG/JPG samples, unsupported files, and explicit RAW-blocked placeholders under ignored `.tmp/` paths. That local-alpha QA manifest is useful for install/runtime smoke tests, but it is not enough for post-alpha RAW/color evidence. Task 10.1 adds a canonical RAW/color fixture manifest contract that remains separate from catalog, sidecar, edit graph, and export state.

## Accepted Approach

Use a schema-first, example-backed manifest contract plus a deterministic Python stdlib harness check.

Create:

- `schemas/fixture_manifest.schema.json`
- `schemas/fixture_manifest.example.json`
- `scripts/harness/check-fixture-manifest-contract.py`

Update:

- `scripts/harness/check.sh`
- `docs/19_Schema_Reference.md`
- `docs/wiki/topics/raw-decoding.md`
- `docs/wiki/topics/color-management.md`
- `docs/wiki/roadmaps/post-alpha-product-roadmap.md`
- `docs/wiki/log.md`

Do not update the synthetic fixture generator in Task 10.1 unless required to preserve an existing harness contract. The generated local-alpha fixture manifest and the post-alpha RAW/color fixture manifest may remain separate because they serve different gates.

## Manifest Scope

The manifest describes immutable test inputs and expected gate behavior.

It may include:

- fixture source and acquisition information
- license and redistribution review information
- privacy guardrails
- relative fixture paths
- SHA-256 and byte-size expectations
- image/media metadata when known
- RAW camera metadata when available
- color profile expectations for Class F
- current local-alpha app behavior expectations
- future probe expectations marked unverified

It must not include:

- catalog rows
- `photo_flags`
- `sidecar.flags`
- edit graph payloads
- export history
- user workspace/session state
- decoded pixel results
- Core Image or LibRaw probe results
- ICC validation results
- golden-image tolerance results

## Top-Level Manifest Shape

Required top-level fields:

- `schema`: constant `silica.fixture_manifest`
- `version`: integer, initially `1`
- `manifest_kind`: `synthetic-local-alpha`, `raw-fixtures`, `color-fixtures`, or `mixed`
- `source_policy`: object describing generated, committed, local-only, or external-reference-only fixture policy
- `maintained_by`: maintainer or team string
- `updated_at`: ISO-like timestamp string
- `fixtures`: non-empty array
- `expected_source_hashes`: object mapping every fixture `relative_path` to that fixture's SHA-256

Optional top-level fields:

- `notes`
- `extensions`

## Fixture Classes

Task 10.1 defines these classes:

- RAW Class A: ordinary Core Image candidate RAW files.
- RAW Class B: high-risk or edge-case RAW files.
- RAW Class C: Fuji RAF candidates.
- RAW Class D: Apple ProRAW DNG candidates.
- RAW Class E: RAW-like files that should stay unsupported or blocked.
- Color Class F: tagged and untagged raster fixtures for color-management proof.

Class names are contract labels, not support claims. RAW Classes A-E must remain blocked until later RAW probe tasks record evidence. Color Class F must remain expectation metadata until later color fixture and tolerance tasks record evidence.

## Per-Fixture Shape

Required fields for every fixture:

- `id`: stable unique fixture ID
- `class`: `A`, `B`, `C`, `D`, `E`, or `F`
- `kind`: `raw`, `tagged_raster`, `untagged_raster`, `unsupported`, or `raw_blocked_placeholder`
- `relative_path`: relative path only, no absolute paths and no `..`
- `availability`: `generated`, `committed`, `local_ignored`, or `external_reference_only`
- `source`: object with name, URL, acquired date, and origin
- `license`: object with name, URL, rights holder, redistribution permission, commit permission, review date, and reviewer
- `privacy`: object recording whether the file is a user photo, contains identifiable people, has a model release, and whether GPS metadata is expected
- `integrity`: object with `sha256` and `size_bytes`
- `media`: object with extension, MIME type, dimensions, orientation, bit depth when known, and profile tag when known
- `expected_app_state`: current app expectation such as import support, preview status, and visible alpha path
- `expected_probe_state`: future RAW/color probe expectation, always unverified in Task 10.1

RAW fixtures also require:

- `raw`: object with format, camera make/model when available, lens when available, ISO when available, and scene tags
- `decode_gate`: object whose state remains `blocked_pending_task_12` for real RAW classes in Task 10.1

Color Class F fixtures also require:

- `color`: object with subclass `srgb_jpeg`, `display_p3_jpeg`, `display_p3_heic`, or `untagged_jpeg`
- `profile_expectation`: object with embedded ICC expectation, input profile expectation, untagged policy, expected default export profile, and expected default ICC embedding behavior

## Validation Rules

The harness must use Python standard library only.

It must validate:

- schema and example files exist
- JSON loads successfully
- top-level `schema`, `version`, `manifest_kind`, `fixtures`, and `expected_source_hashes` are valid
- fixture IDs are unique
- every fixture path is relative, normalized, and does not contain `..`
- every fixture has source, license, privacy, integrity, media, expected app state, and expected probe state fields
- `expected_source_hashes` exactly matches fixture `relative_path -> integrity.sha256`
- committed fixtures cannot have unknown licenses
- committed fixtures cannot be user photos
- RAW classes A-E cannot claim decoded support before Task 12
- RAW placeholders cannot be treated as real RAW fixtures
- Class C must use RAF format if present
- Class D must use Apple ProRAW DNG format if present
- Color Class F must include at least one sRGB, one Display P3, and one untagged raster expectation in the example
- sRGB and Display P3 fixtures require embedded ICC expectation
- untagged raster fixtures require no embedded ICC and an explicit untagged policy
- example manifests do not contain absolute user paths
- docs contain guardrail language forbidding RAW support and color correctness claims before fixture-backed proof

The harness may verify actual file hashes when a future external manifest path is explicitly supplied, but CI/default mode must not require real RAW/color fixture files.

## Documentation Rules

The RAW topic page must state:

- the repository has no committed legal RAW fixture corpus
- Task 10.1 defines only fixture provenance and expectations
- RAW support claims remain blocked until fixture-backed Core Image probe work
- RAW placeholders are blocked-state fixtures, not decodable RAW evidence

The color topic page must state:

- Class F covers sRGB, Display P3, and untagged raster expectations
- hashes and profile declarations do not prove color correctness
- color correctness claims remain blocked until fixture-backed proof and tolerance policy exist

The schema reference must list the new schema and example as authoritative contract files.

## Exclusions

Do not add:

- RAW decoding
- Core Image probing
- LibRaw or bindings
- EXIF parsing dependencies
- ICC parsing dependencies
- golden image comparisons
- pixel tolerance logic
- sidecar read/write behavior
- catalog migrations
- edit graph changes
- fixture downloaders
- CI network fetches
- real user photos
- local absolute sample paths
- new dependencies

## Acceptance Criteria

- Schema and example exist and validate through the new harness.
- Harness is wired into `scripts/harness/check.sh`.
- Documentation explains RAW/color fixture classes and trust boundaries.
- Local-alpha synthetic fixture tooling remains intact.
- `scripts/harness/check.sh` passes.
- No RAW/color correctness or support claim is introduced.

## Open Decisions Deferred

These are intentionally deferred:

- which legal RAW files become the first real external corpus
- how external local-only fixture directories are configured
- exact Core Image probe output fields
- exact color tolerance policy
- exact ICC/profile inspection mechanism
- whether generated synthetic local-alpha manifests are migrated into the new schema later

## Implementation Plan Handoff

After this design is approved, write an implementation plan for Task 10.1 with small tasks:

1. Add schema and example.
2. Add fixture manifest contract checker.
3. Wire checker into harness.
4. Update schema reference and wiki topics.
5. Run full harness and review impact.
