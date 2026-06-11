# Phase 10 Evidence and Recovery Design

## Goal

Define the full Phase 10 design before implementing Task 10.3 or later recovery work.

Phase 10 is the evidence, recovery, and public-trust gate after the local DMG alpha. It must make the product safer to extend without pretending that RAW decoding, color correctness, broad export proof, or public beta readiness are already solved.

## Background

Task 10.1 added the legal RAW/color fixture manifest contract. Task 10.2 added the golden image and tolerance policy baseline.

The remaining Phase 10 tasks are tightly related:

- Task 10.3 writes and reads portable sidecars.
- Task 10.4 dry-runs catalog rebuild from those sidecars.
- Task 10.5 defines backup, WAL, checkpoint, and restore behavior.
- Task 10.6 publishes the open-source trust package.

These tasks must be designed together because a poor sidecar shape can make rebuilds ambiguous, a poor rebuild model can make restore unsafe, and unclear trust docs can invite contributors to make claims the app cannot yet prove.

## Accepted Task Order

Keep the existing order:

1. Task 10.1: Legal RAW and Color Fixture Manifest Contract. Completed.
2. Task 10.2: Golden Image and Tolerance Policy. Completed.
3. Task 10.3: Sidecar v1 Read/Write Foundation.
4. Task 10.4: Catalog Rebuild Dry-Run from Sidecars.
5. Task 10.5: Backup, WAL, Checkpoint, and Restore Policy.
6. Task 10.6: Public OSS Trust Package.

Do not merge Task 10.4 before Task 10.3 exists, because rebuild behavior needs validated sidecar input. Do not merge Task 10.5 before Task 10.4 exists, because backup and restore policy must know what rebuild can report. Do not merge Task 10.6 before Task 10.5 exists, because the public trust package must describe the actual recovery boundaries.

## Phase 10 Responsibilities

### Task 10.3: Sidecar v1 Read/Write Foundation

Task 10.3 creates an explicit sidecar API for the current catalog photo state.

It owns:

- sidecar path selection
- sidecar JSON construction
- sidecar schema validation
- nested edit graph schema validation
- atomic sidecar file write
- validated sidecar read
- minimal `sidecar_status` update after successful write
- tests proving original source files are unchanged

It does not own:

- automatic sidecar synchronization on every edit or flag change
- rebuild from sidecars
- user-facing conflict resolution
- backup or restore workflows
- RAW decoding evidence
- export correctness evidence
- color correctness evidence

### Task 10.4: Catalog Rebuild Dry-Run from Sidecars

Task 10.4 consumes Task 10.3 sidecars and reports what a rebuild would do without mutating the live catalog.

It owns:

- deterministic dry-run output
- precedence rules
- conflict reporting
- missing sidecar reporting
- malformed sidecar reporting
- tests proving no live catalog mutation

It does not own:

- writing sidecars
- restoring or overwriting catalog state
- conflict UI
- broad import rescanning
- backup archive creation

### Task 10.5: Backup, WAL, Checkpoint, and Restore Policy

Task 10.5 turns the recovery model into concrete backup and restore behavior.

It owns:

- catalog backup boundaries
- WAL checkpoint policy
- restore behavior for catalog state
- restore behavior for sidecars
- restore behavior for edit states and export records
- exclusion of disposable caches
- migration failure recovery notes

It does not own:

- public beta readiness
- auto-update rollback
- cloud sync
- Homebrew distribution
- signed release artifact automation

### Task 10.6: Public OSS Trust Package

Task 10.6 completes the public contribution and trust docs after the recovery behavior is real enough to describe.

It owns:

- final project license decision
- dependency/license inventory status
- security policy
- contribution guide
- issue and PR templates
- README limitations and local-first safety claims
- public statements about what is not implemented

It does not own:

- notarized release publishing
- Homebrew Cask
- auto-update
- Mac App Store distribution
- plugin, MCP, or MLX enablement

## Task 10.3 Sidecar Decisions

### Sidecar Location

Task 10.3 sidecars must live inside the library root:

```txt
<library_root>/sidecars/<photo_id>.silicaraw.sidecar.json
```

Do not write sidecars next to original photo files in Task 10.3. The local alpha imports by reference, and the app must not assume it can write into source photo folders.

### Path Identity

Use `photo_id` as the sidecar file identity. Validate `photo_id` before using it in a path.

Allowed path characters for Task 10.3:

```txt
A-Z
a-z
0-9
-
_
.
```

Reject empty IDs, path separators, dot-dot segments, absolute paths, backslashes, and control characters.

### Storage API

Expose a small storage-level API:

```rust
write_photo_sidecar(library_root, photo_id, app_version) -> SidecarWriteResult
read_photo_sidecar(library_root, photo_id) -> Option<ValidatedSidecar>
sidecar_path_for_photo(library_root, photo_id) -> PathBuf
```

The storage crate owns filesystem paths, SQLite reads needed to construct the sidecar, JSON validation, and atomic writes.

The core crate may expose thin workflow wrappers, but it must not duplicate sidecar path logic or schema logic.

### Sidecar Payload Source

Build the sidecar from the current catalog state:

- `photos` supplies photo identity, original path, file name, and fingerprint.
- `edit_states` supplies the active edit graph when present.
- `photo_flags` supplies rating, picked, rejected, and color label.

If no active edit graph exists, build a schema-valid default edit graph in memory only. Do not insert a new `edit_states` row just because a sidecar was written for an unedited photo.

### Flag Scope

`sidecar.flags` contains exactly:

```txt
rating
picked
rejected
color_label
```

Do not add:

```txt
edited
exported
exports
export_history
```

`edited` is derived from edit graph state during rebuild. `exported` and export history are catalog/workflow state and are not part of sidecar v1 flags.

### Metadata Mirror

At sidecar write time, mirror the same four portable flag values into `edit_graph.metadata`:

```txt
rating
picked
rejected
color_label
```

During Task 10.3 reads, preserve `sidecar.flags` and `edit_graph.metadata` as separate values. Do not silently repair differences. Task 10.4 owns rebuild precedence and conflict reporting.

### Validation

Before a sidecar write succeeds:

- the sidecar JSON must validate against `schemas/sidecar.schema.json`
- the nested edit graph must validate against `schemas/edit_graph.schema.json`
- color labels must be rejected unless they are `red`, `orange`, `yellow`, `green`, `blue`, `purple`, or `null`
- `edited`, `exported`, and export-history fields must not appear in `sidecar.flags`

Treat sidecar JSON read from disk as untrusted input.

### Atomic Write

Write sidecars through a temp file in the same `sidecars/` directory, validate the temp payload, then rename into place.

Task 10.3 should prevent partial final JSON from replacing a valid sidecar. It does not need to claim full crash-consistency across every filesystem failure mode.

### Hash Policy

Keep `sync.sidecar_hash` as `null` in Task 10.3 unless a deterministic non-self-referential hash contract is designed first.

Do not put a hash of a sidecar inside the same JSON object without specifying exactly which fields are excluded from that hash. That would make the value ambiguous.

### Status Update

Only after a successful write, update `sidecar_status` minimally:

- `photo_id`
- library-relative sidecar path
- `last_written_at`
- `conflict_state = clean`

Reads do not mutate catalog state in Task 10.3.

## Task 10.4 Rebuild Decisions

Task 10.4 must be a dry-run first.

Precedence:

1. valid `sidecar.flags`
2. valid `edit_graph.metadata`
3. defaults:
   - `rating = 0`
   - `picked = false`
   - `rejected = false`
   - `color_label = null`

Dry-run output must report:

- sidecars that would create or update photo flag state
- sidecars with malformed JSON
- sidecars that fail schema validation
- sidecars whose `photo.photo_id` does not match the expected path identity
- sidecars whose flags and edit graph metadata disagree
- sidecars whose original path or fingerprint cannot be reconciled with catalog state

The dry-run must not:

- mutate the live catalog
- infer that a photo is edited merely because a sidecar exists
- restore `exported` from sidecar flags
- write conflict resolutions

## Task 10.5 Recovery Decisions

Task 10.5 must treat these as durable recovery data:

- `catalog.db`
- SQLite WAL and SHM state as required by the chosen checkpoint policy
- `sidecars/`
- edit states
- export records
- migration metadata

Task 10.5 must treat these as disposable:

- thumbnails
- previews
- render caches
- AI caches
- transient logs unless a later support-bundle task explicitly includes them

The backup format and restore command can be implemented after Task 10.4, but the policy must preserve these invariants:

- backups never include original referenced photo files
- restore does not write into original photo folders
- caches are not required to make a restored library meaningful
- sidecar paths stored in catalog state remain library-relative
- migration failure behavior is documented and tested

## Task 10.6 Trust Package Decisions

Task 10.6 must describe the actual product honestly.

The public package should include:

- `LICENSE`
- `README.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- issue templates
- PR template
- dependency/license inventory links
- known limitations
- local-first and non-destructive scope
- current unsigned vs signed/notarized release status

Do not describe SilicaRAW as:

- a production RAW editor
- fixture-proven for broad RAW support
- color-correct for all workflows
- publicly beta-ready
- plugin/MCP/MLX-enabled
- auto-updating

## RAW, Color, and Export Guardrails

Phase 10 sidecars are not evidence containers for RAW, color, or export correctness.

Do not add these to sidecars:

- RAW probe results
- Core Image or LibRaw success proof
- ICC inspection results
- golden image hashes
- manual visual review status
- export output hashes
- export history
- preview/render proof booleans

Existing edit graph fields such as `profile.decoder_backend` may be persisted only as actual current edit graph data. Do not fabricate `decoder_backend = "core_image_raw"` from a file extension or future spike expectation.

## Harness and Test Strategy

Use Rust tests for runtime behavior and Python harness checks for static trust contracts.

### Task 10.3 Checks

Rust tests should cover:

- valid sidecar write and read
- sidecar schema validation
- nested edit graph schema validation
- flag scope limited to rating, picked, rejected, and color label
- metadata mirroring of the same four values
- original source file hash unchanged after sidecar write
- sidecar path located under the library `sidecars/` directory
- failed atomic write does not replace an existing valid sidecar
- `sidecar_status` updates only after successful write
- read rejects malformed JSON
- read rejects wrong schema/version
- read rejects photo ID mismatch
- read does not overwrite catalog flags

Add a small Python stdlib guard:

```txt
scripts/harness/check-sidecar-contract.py
```

It should check the static schema/docs contract:

- sidecar schema marker and version remain v1
- `sidecar.flags` contains exactly the four allowed fields
- `edited` and `exported` are not sidecar flag fields
- docs preserve the catalog-vs-sidecar authority language

### Task 10.4 Checks

Rust tests should cover:

- dry-run reports deterministic output
- dry-run does not mutate the live catalog
- sidecar flags win over edit graph metadata
- edit graph metadata falls back only when sidecar flags are absent or invalid by the dry-run rule
- defaults are used when no valid portable flags exist
- conflicts are reported instead of silently resolved

### Task 10.5 Checks

Rust tests should cover:

- backup excludes disposable caches
- backup includes catalog and sidecars
- restore preserves edit states, flags, sidecar status, and export records
- restore does not touch original referenced files
- WAL/checkpoint behavior follows the documented policy

If documentation grows a formal recovery checklist, add a small Python check for policy markers.

### Task 10.6 Checks

Markdown/harness checks should cover:

- links are valid
- dependency/license inventory is current
- security policy exists
- contribution guide exists
- issue/PR templates exist
- README limitations do not overclaim unsupported features

## Stop Gates

Stop and redesign if any Task 10.3 implementation tries to:

- write sidecars next to original photo files
- modify original photo files
- trigger broad automatic sidecar sync
- add conflict UI before dry-run semantics exist
- add RAW decoding, Core Image probes, LibRaw, ICC parsing, or golden image comparison
- add dependencies without updating `docs/DEPENDENCIES.md`
- store `edited`, `exported`, or export history in `sidecar.flags`
- infer edited state from sidecar existence alone
- fabricate decoder/color/export proof fields

Stop and redesign if Task 10.5 cannot prove restore without original-file mutation.

## Atomic Implementation Handoff

After this design is accepted, implement Phase 10 in small PRs:

1. Task 10.3 design-to-plan PR if needed: write the detailed implementation plan for sidecar v1.
2. Task 10.3.1: add static sidecar contract harness and docs guard.
3. Task 10.3.2: add sidecar path validation tests and path helper.
4. Task 10.3.3: add sidecar payload construction tests and structs.
5. Task 10.3.4: add schema validation for sidecar and nested edit graph.
6. Task 10.3.5: add atomic write and read behavior.
7. Task 10.3.6: update `sidecar_status` after successful writes only.
8. Task 10.3.7: add core workflow wrappers and original-hash safety test.
9. Task 10.4.1: design and test dry-run report types.
10. Task 10.4.2: implement dry-run scanning and precedence.
11. Task 10.4.3: add conflict and malformed-sidecar reporting.
12. Task 10.5.1: document backup/WAL/checkpoint policy.
13. Task 10.5.2: implement backup boundaries.
14. Task 10.5.3: implement restore boundaries and safety tests.
15. Task 10.6.1: finalize license and public trust docs.
16. Task 10.6.2: add public contribution/security templates and static checks.

Each PR must remain atomic and must run the smallest useful verification plus `scripts/harness/check.sh` before completion.

## Deferred Decisions

These are intentionally deferred until later phases:

- next-to-original sidecars
- automatic sidecar synchronization policy
- user-facing sidecar conflict resolution UI
- full sidecar hash contract
- broad RAW decode proof
- fixture-backed Core Image probe output format
- ICC parser choice
- golden image comparison implementation
- release signing and notarization automation
- Homebrew Cask
- auto-update
- plugin, MCP, or MLX enablement
