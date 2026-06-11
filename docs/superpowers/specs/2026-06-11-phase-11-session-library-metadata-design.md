# Phase 11 Session, Library, and Metadata Design

## Goal

Define the full Phase 11 implementation plan before adding session persistence, scalable grid behavior, metadata surfaces, or recursive import.

Phase 11 must turn the local alpha from a mostly single-session workflow into a real local desktop app that can launch, relaunch, browse larger catalogs, preserve workspace state, show truthful metadata, and handle import errors without fake rows or silent behavior.

## Background

Phase 10 completed the evidence, recovery, and public trust gate. Phase 11 is the next product foundation layer.

The current codebase has these important seams:

- `silica-storage` owns library-local catalog data in `<library_root>/catalog.db`.
- `silica-core` exposes thin workflow APIs over storage, export, render planning, and edit validation.
- `apps/desktop/src-tauri` exposes Tauri commands.
- `apps/desktop/static/index.html` currently keeps active library, selected photo, mode, and layout-like state in frontend memory.
- The current Library grid uses a full-list path through `list_library_photos`.
- Core currently generates thumbnails for all eligible JPEG/JPG rows when listing photos.
- Welcome recents are empty and disabled.
- Metadata UI is a small placeholder surface.
- Import error review is not a real review surface.

These seams mean Phase 11 must begin with app session truth, then query scalability, then UI scale behavior, then metadata, then recursive import.

## Accepted Phase Order

Keep this order:

1. Session state boundary and typed app session store.
2. Real recents and Welcome recents UI.
3. Relaunch restore for last library, mode, and selected photo.
4. Layout preferences and reset.
5. Paged, sorted, and filtered query API.
6. Page-driven and virtualized grid.
7. Keyboard and multi-select grid behavior.
8. Metadata extraction and storage.
9. Metadata inspector, search, and filters.
10. Reviewable import errors, then opt-in recursive import.

Do not implement virtualized grid before paged queries exist. Virtualization is not useful if the backend still loads every catalog row and every thumbnail.

Do not implement metadata UI before the stored metadata contract exists. Unknown metadata must stay unknown.

Do not implement recursive import before import errors are modeled and reviewable. Recursive scans increase recoverable failure cases.

## Persistence Domains

Use two separate persistence domains.

### App-Level Session State

App-level state is per-user desktop state and lives outside every library.

It owns:

- recent library list
- last library root path
- last active mode
- last selected photo id per library
- sidebar, inspector, and filmstrip layout preferences
- thumbnail size
- default sort and filter preferences
- schema and version for app session migration

It does not own:

- photo rows
- culling flags
- edit states
- exports
- sidecar status
- cache records
- backup or restore artifacts
- original file hashes

Recommended local path:

```txt
~/Library/Application Support/dev.silicaraw.desktop/app-session.json
```

The desktop shell should resolve this path through the Tauri v2 path resolver. Tauri v2 exposes path resolution through `app.path()` from the `Manager` trait and `BaseDirectory::Config`; implementation tasks must verify the exact API against current Tauri docs before coding.

Do not use webview `localStorage` as the source of truth. It can mirror UI-only ephemeral state if needed, but the durable app session store belongs behind Rust commands.

### Library Catalog State

Library state stays in:

```txt
<library_root>/catalog.db
```

It owns:

- libraries
- folders
- photos
- `photo_metadata`
- `photo_flags`
- edit states and history
- sidecar status
- cache records
- export records
- action history when implemented

The catalog remains the authority for live photo flags, queryable metadata, edit state, and export records.

### Sidecars

Sidecars remain portable recovery state under:

```txt
<library_root>/sidecars/
```

Do not add app session state, layout preferences, recent library paths, export history, or UI selection state to sidecars.

## App Session JSON v1

Use a typed, versioned JSON file.

Recommended shape:

```json
{
  "schema": "silica.desktop_session",
  "version": 1,
  "last_library_root_path": null,
  "last_mode": "library",
  "recents": [],
  "layout": {
    "sidebar_collapsed": false,
    "inspector_collapsed": false,
    "filmstrip_visible": true,
    "thumbnail_size": 180,
    "sort": "imported_at_desc",
    "filters": {
      "min_rating": null,
      "picked": null,
      "rejected": null,
      "file_type": null,
      "search": ""
    }
  },
  "per_library": {}
}
```

Recommended per-library entry:

```json
{
  "selected_photo_id": null,
  "last_mode": "library",
  "last_opened_at": "2026-06-11T00:00:00Z"
}
```

Rules:

- Unknown schema or newer version returns safe defaults and a warning state.
- Corrupt JSON returns safe defaults and preserves the corrupt file for manual inspection when feasible.
- Atomic writes use temp file plus rename.
- Failed writes must not destroy the previous valid session file.
- Recents are only written after successful create or open.
- Missing recent paths are displayed as unavailable. Do not silently remove them during read.
- Last library restore validates the path and catalog before opening.
- Selected photo restore validates the photo still exists in the catalog before selection.
- If a restored mode requires a selected photo and that photo is missing, restore to Library mode instead of a broken Develop, Loupe, or Export state.

## Top-Level Task Mapping

The existing roadmap keeps Task 11.1 through Task 11.9 as top-level milestones. Implement Phase 11 through the atomic subtasks below.

## Sprint 1: Session Truth and Recents

Goal: Make app session state real, typed, and separate from catalog state.

Demo/Validation:

- First launch shows no fake recents.
- Create/open a real library records a real recent.
- Relaunch can inspect session state without touching original files.
- Missing recent paths are visible as unavailable.

### Task 11.1.1: Phase 11 Design Gate

- **Location:** `docs/superpowers/specs/`, `docs/wiki/roadmaps/post-alpha-product-roadmap.md`, `docs/wiki/log.md`
- **Description:** Add this design gate and link it from the roadmap before implementation.
- **Dependencies:** Phase 10
- **Acceptance Criteria:**
  - Persistence domains are documented.
  - Atomic task order is documented.
  - Stop gates are documented.
- **Validation:**
  - `python3 scripts/harness/check-md-links.py`
  - `scripts/harness/check.sh`

### Task 11.1.2: App Session Schema and Core Types

- **Location:** `crates/silica-core`
- **Description:** Add typed app session structs, defaults, validation, clamping, JSON read/write helpers, and atomic write behavior with caller-injected path.
- **Dependencies:** Task 11.1.1
- **Acceptance Criteria:**
  - App session state is not stored in `catalog.db`, sidecars, or frontend-only storage.
  - Missing or corrupt session files return safe defaults.
  - Invalid mode, sort, filter, and thumbnail values are clamped or rejected deterministically.
  - Atomic write leaves old state intact on write failure.
- **Validation:**
  - `cargo test -p silica-core app_session`
- **Status:** Completed on 2026-06-11. Added app session v1 core types, defaults, validation, clamping, safe missing/corrupt/newer handling, JSON round-trip helpers, and temp-file plus rename writes with caller-injected paths. This does not add desktop Tauri commands, recents recording, restore behavior, frontend persistence, catalog storage, or sidecar storage.

### Task 11.1.3: Desktop Session Path and Commands

- **Location:** `apps/desktop/src-tauri`
- **Description:** Resolve the app session path through Tauri and expose commands to read, write, reset, and inspect session state.
- **Dependencies:** Task 11.1.2
- **Acceptance Criteria:**
  - Desktop chooses the app-session path; core receives it as an injected path.
  - No frontend-only durable persistence is introduced.
  - Desktop tests can use a temp app-session path.
- **Validation:**
  - `cargo test -p silica-desktop app_session`
- **Status:** Completed on 2026-06-11. Added desktop app-session DTOs, app-config path resolution through Tauri `BaseDirectory::AppConfig`, and read/write/reset/inspect command handlers with temp-path test helpers. This does not record real recents, restore last state on launch, or persist frontend layout interactions.

### Task 11.2.1: Record Real Recent Libraries

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`
- **Description:** Add recents after successful `create_library` or `open_library` only.
- **Dependencies:** Task 11.1.3
- **Acceptance Criteria:**
  - Failed create/open does not add a recent.
  - Recents dedupe by canonical library root path or catalog path.
  - Recents are capped to a documented limit.
  - Existing missing recents remain visible as unavailable on read.
- **Validation:**
  - `cargo test -p silica-core recent`
  - `cargo test -p silica-desktop recent`
- **Status:** Completed on 2026-06-11. Added app-session recent recording after successful core library sessions, with dedupe, a documented cap, last-library update, and desktop create/open integration. Failed create/open paths do not update the app-session file. Welcome rendering remains Task 11.2.2.

### Task 11.2.2: Welcome Recent Libraries UI

- **Location:** `apps/desktop/static/`, `scripts/harness/`
- **Description:** Replace the empty-only recents block with real recent rows, empty state, and unavailable path state.
- **Dependencies:** Task 11.2.1
- **Acceptance Criteria:**
  - No fictional recent rows.
  - Empty state stays honest on first launch.
  - Missing recent paths are disabled or clearly unavailable.
  - Selecting a valid recent opens that library path.
- **Validation:**
  - `python3 scripts/harness/check-static-ui.py`
  - `python3 scripts/harness/check-ui-workflow-smoke.py`
- **Status:** Completed on 2026-06-11. Welcome now reads real app-session recents, renders an honest first-launch empty state, disables unavailable library paths, and opens available recent libraries through the existing desktop command path. The desktop DTO marks recent availability from the real catalog file; relaunch restore remains Task 11.3.

### Task 11.3.1: Relaunch Restore State Machine

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Restore last valid library and last valid mode on app launch.
- **Dependencies:** Task 11.2.2
- **Acceptance Criteria:**
  - Missing library falls back to Welcome.
  - Missing catalog falls back to Welcome.
  - Restore does not create, migrate, import, rescan, sidecar-sync, or backup/restore automatically.
  - Static/demo rows remain absent.
- **Validation:**
  - `cargo test -p silica-core app_session_restore`
  - `python3 scripts/harness/check-connected-runtime-smoke.py`
- **Status:** Completed on 2026-06-11. Added a read-only launch restore resolver that validates the last library directory and `catalog.db` without creating support directories, migrating, importing, rescanning, sidecar-syncing, or backup/restore. Desktop exposes the decision through `resolve_launch_restore`, and the UI applies Welcome vs Library state on boot without calling `open_library`. Requested Develop/Export mode is preserved in the response but resolves to Library until Task 11.3.2 verifies selected-photo restore.

### Task 11.3.2: Selected Photo Restore

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Restore the primary selected photo only when it still exists in the opened catalog.
- **Dependencies:** Task 11.3.1
- **Acceptance Criteria:**
  - Existing selected photo restores.
  - Missing selected photo clears selection without crash.
  - Mode restore remains valid when selection is cleared.
- **Validation:**
  - `cargo test -p silica-core selected_photo_restore`
  - connected runtime smoke update
- **Status:** Completed on 2026-06-11. Added read-only selected-photo validation for launch restore, app-session selection/mode recording through `record_app_session_selection`, and UI restore wiring that applies the saved selected photo before mode restore. Existing selected photos restore, missing selected photos clear selection and resolve mode back to Library, and launch restore still avoids normal open/migrate/write-side workflows.

## Sprint 2: Layout Preferences

Goal: Make workspace layout a real persisted app preference, not a transient DOM side effect.

Demo/Validation:

- Sidebar, inspector, filmstrip, thumbnail size, sort, and filters restore after relaunch.
- Reset returns documented defaults.
- Compact, desktop, and large widths remain stable.

### Task 11.4.1: Layout Preference Model

- **Location:** `crates/silica-core`, `docs/wiki/topics/ui-mvp-baseline.md`
- **Description:** Document and expose reset behavior for layout preferences already defined by the app session v1 schema.
- **Dependencies:** Task 11.1.2
- **Acceptance Criteria:**
  - Defaults are documented.
  - Invalid values are clamped or reset.
  - Sort and filter defaults align with the future paged query contract.
- **Validation:**
  - `cargo test -p silica-core layout_preferences`
  - `python3 scripts/harness/check-md-links.py`
- **Status:** Completed on 2026-06-11. Added core default/reset helpers for `AppSession.layout`, verified reset preserves non-layout app-session state, and documented the layout defaults plus invalid-value behavior in the UI MVP wiki. Desktop command and UI wiring remain Task 11.4.2.

### Task 11.4.2: Persist Layout Interactions

- **Location:** `apps/desktop/static/`, `apps/desktop/src-tauri`
- **Description:** Wire sidebar, inspector, filmstrip, thumbnail size, sort, filter, and reset controls to app-session commands.
- **Dependencies:** Task 11.4.1
- **Acceptance Criteria:**
  - Preferences restore after relaunch.
  - Reset layout returns to documented defaults.
  - Controls do not overlap or resize unpredictably.
- **Validation:**
  - `python3 scripts/harness/check-static-ui.py`
  - targeted visual responsive QA
- **Status:** Completed on 2026-06-11. Added desktop layout record/reset commands, wired sidebar, inspector, filmstrip, thumbnail size, sort, and filter controls to app-session layout state, and restored saved layout preferences on launch/app-session reads. The static UI and workflow smoke checks cover the interaction contract; dedicated layout screenshots remain Task 11.4.3.

### Task 11.4.3: Layout Visual QA States

- **Location:** `scripts/harness/`, visual QA artifacts
- **Description:** Add Phase 11 layout states to visual QA.
- **Dependencies:** Task 11.4.2
- **Acceptance Criteria:**
  - Viewports cover `1280x800`, `1440x900`, and `1728x965`.
  - Sidebar collapsed, inspector collapsed, and reset states are captured.
  - No horizontal overflow or clipped controls.
- **Validation:**
  - visual responsive QA runner
- **Status:** Completed on 2026-06-11. Extended `scripts/harness/run-final-visual-qa.py` with sidebar-collapsed, inspector-collapsed, and layout-reset states across `1280x800`, `1440x900`, and `1728x965`. The runner now asserts the expected layout state metrics and passed with 33 screenshots, zero horizontal overflow, zero toolbar overlap, and zero clipped controls.

## Sprint 3: Paged Queries and Scalable Grid

Goal: Stop relying on full-list catalog reads before adding virtualized grid behavior.

Demo/Validation:

- A large catalog can be queried in bounded pages.
- Sort and filter inputs are typed and whitelisted.
- The grid consumes pages without loading every row or every thumbnail at once.
- Thumbnail hydration is page- or viewport-scoped before virtualization depends on it.

### Task 11.5.1: Paged Query Contract

- **Location:** `crates/silica-catalog`
- **Description:** Define typed request and response contracts for paged library queries.
- **Dependencies:** Task 11.4.1
- **Acceptance Criteria:**
  - Request uses bounded offset pagination, sort enum, and filter struct.
  - Page size is bounded.
  - Sort and filter options are whitelisted enums and structs.
  - Ordering is deterministic with explicit tie breakers.
  - Cursor pagination is deferred until benchmark evidence requires it.
  - No arbitrary SQL, column names, or raw predicates can cross from UI to storage.
- **Validation:**
  - `cargo test -p silica-catalog library_query`
- **Status:** Completed on 2026-06-11. Added typed `silica-catalog` request/response contracts for bounded offset library queries, whitelisted sort and filter enums, deterministic order fields with explicit tie breakers, and a cursor-pagination deferral marker. Storage indexes and storage/core query execution were later completed by Task 11.5.2 and Task 11.5.3.

### Task 11.5.2: Query Index and Migration Plan

- **Location:** `crates/silica-catalog`, `crates/silica-storage`
- **Description:** Add only the indexes required by accepted sort and filter fields.
- **Dependencies:** Task 11.5.1
- **Acceptance Criteria:**
  - Indexes are represented in the catalog schema contract.
  - Migration is idempotent.
  - Query order is deterministic with tie breakers.
- **Validation:**
  - `cargo test -p silica-storage query_index`
  - `cargo test -p silica-catalog`
- **Status:** Completed on 2026-06-11. Raised the catalog schema contract to version 3, added a normalized `photos.file_type` migration/backfill for `jpeg`, `raw`, and `unsupported`, and added the accepted paged-query indexes for imported-time sorting, filename/path sorting, file-type filtering, and rating sorting. Storage/core query execution was later completed by Task 11.5.3.

### Task 11.5.3: Storage and Core Paged Query API

- **Location:** `crates/silica-storage`, `crates/silica-core`
- **Description:** Implement page-scoped catalog reads and core wrappers.
- **Dependencies:** Task 11.5.2
- **Acceptance Criteria:**
  - Query returns bounded rows and page metadata.
  - Empty pages are deterministic.
  - Filters use normalized columns.
  - Existing full-list path remains only as compatibility until UI migration completes, or is explicitly replaced.
  - Query does not mutate originals, catalog state, sidecars, caches, or thumbnails.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core library_query`
- **Status:** Completed on 2026-06-11. Added read-only `silica-storage` paged library queries with bounded rows, total-count metadata, deterministic empty pages, normalized `photos.file_type` filtering, fixed enum-backed sort clauses, and a `silica-core` wrapper that does not hydrate thumbnails or mutate cache/catalog state. Desktop command wiring was later completed by Task 11.5.4.

### Task 11.5.4: Desktop Paged Grid Command

- **Location:** `apps/desktop/src-tauri`
- **Description:** Add a desktop command for page-based grid queries.
- **Dependencies:** Task 11.5.3
- **Acceptance Criteria:**
  - Command accepts only typed page/sort/filter fields.
  - Command response does not include unnecessary thumbnail bytes for rows outside the page.
  - Error context remains structured.
- **Validation:**
  - `cargo test -p silica-desktop paged_grid`
- **Status:** Completed on 2026-06-11. Added the `query_library_photos` desktop command with a typed page/sort/filter request DTO, whitelist parsing for sort and file-type filters, a `photoGridPage` response carrying page metadata and deterministic order fields, and structured error context for invalid request values. Page-scoped thumbnail hydration was later completed by Task 11.5.5.

### Task 11.5.5: Page-Scoped Thumbnail Hydration

- **Location:** `crates/silica-storage`, `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Stop paged grid behavior from generating or returning thumbnails for the whole catalog.
- **Dependencies:** Task 11.5.4
- **Acceptance Criteria:**
  - Only requested page or viewport photo IDs hydrate thumbnails.
  - RAW, unsupported, and missing rows remain honest unavailable states.
  - No original files are mutated.
  - The old eager all-catalog thumbnail path is not used by the product grid.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core thumbnail`
  - UI smoke marker for page-scoped thumbnail requests
- **Status:** Completed on 2026-06-11. Added a core page-scoped thumbnail hydration query that hydrates JPEG thumbnails only for rows in the requested page, leaves RAW/unsupported rows without thumbnails, preserves original files, and updates the desktop paged grid command/static product grid to use `query_library_photos` instead of the old full-list product grid path. Full page UI states were completed by Task 11.6.1.

### Task 11.6.1: Page-Driven Grid UI

- **Location:** `apps/desktop/static/`, `scripts/harness/`
- **Description:** Move the grid UI from full-list rendering to page-driven rendering.
- **Dependencies:** Task 11.5.5
- **Acceptance Criteria:**
  - Loading, empty, page, and error states are visible.
  - Selected photo remains coherent across page changes.
  - Grid does not claim unavailable rows exist.
- **Validation:**
  - `python3 scripts/harness/check-ui-workflow-smoke.py`
- **Status:** Completed on 2026-06-11. The grid shell now renders explicit loading, empty, page, and error states, exposes previous/next page controls backed by `query_library_photos` metadata, keeps selection coherent when page contents change, and reports total vs visible page rows without implying unavailable rows are loaded.

### Task 11.6.2: Virtualized Grid Window

- **Location:** `apps/desktop/static/`, visual QA scripts
- **Description:** Render only visible rows plus a small overscan window.
- **Dependencies:** Task 11.6.1
- **Acceptance Criteria:**
  - Stable card dimensions.
  - Object URLs are cleaned up when thumbnails leave the window.
  - No horizontal overflow or clipped grid controls.
  - Virtualization does not request every catalog row.
- **Validation:**
  - targeted visual responsive QA
  - UI smoke markers for virtualization
- **Status:** Completed on 2026-06-11. The product grid now renders a page-local virtual window with overscan spacer rows, preserves stable card aspect ratios through the existing grid layout, revokes grid-owned thumbnail object URLs when rows leave the rendered window, and keeps backend access bounded to the existing paged `query_library_photos` request.

### Task 11.6.3: Keyboard Grid Navigation

- **Location:** `apps/desktop/static/`, browser or UI harness
- **Description:** Add roving focus and keyboard navigation for the grid.
- **Dependencies:** Task 11.6.2
- **Acceptance Criteria:**
  - Arrow keys, Home, End, PageUp, PageDown, and Enter-to-Loupe work.
  - Focus styling is visible.
  - Keyboard navigation does not lose selection when pages load.
- **Validation:**
  - browser automation or focused UI harness
- **Status:** Completed on 2026-06-11. The grid now exposes a roving-focus keyboard contract for the current page: Arrow keys move by item or row, Home/End jump to page bounds, PageUp/PageDown jump by viewport rows, and Enter opens the loupe for the selected photo. Rendered cards keep visible focus styling and page reloads retain a coherent selected photo.

### Task 11.6.4: Multi-Select Semantics

- **Location:** `apps/desktop/static/`
- **Description:** Add primary selection, range selection, toggle selection, selection count, and clear selection.
- **Dependencies:** Task 11.6.3
- **Acceptance Criteria:**
  - Primary selected photo remains explicit.
  - Range selection uses a stable anchor.
  - Toggle selection works without fake aggregate metadata.
  - Inspector clearly distinguishes primary and multi-selection states.
  - Batch edits remain out of scope unless separately planned.
- **Validation:**
  - focused UI harness
  - visual QA for selected and multi-selected states
- **Status:** Completed on 2026-06-11. The grid now tracks an explicit primary photo plus current-page multi-selection state, supports Shift range selection with a stable anchor, Cmd/Ctrl or Space toggle selection, Escape/clear-button collapse back to the primary selection, and shows primary vs multi-selection count in the inspector while keeping batch edits out of scope.

## Sprint 4: Metadata

Goal: Store and display real metadata without pretending unavailable metadata exists.

Demo/Validation:

- Width, height, orientation, capture time, camera, lens, and file metadata are stored when available.
- Missing metadata is shown as unavailable.
- Search and filters only enable fields backed by stored data.

### Task 11.7.1: Metadata Schema and Dependency Gate

- **Location:** `crates/silica-catalog`, `docs/DEPENDENCIES.md`, `docs/wiki/topics/catalog.md`
- **Description:** Decide the metadata storage shape and whether a new metadata parser dependency is needed.
- **Dependencies:** Task 11.5.3
- **Acceptance Criteria:**
  - `photo_metadata` normalized fields are documented.
  - Width, height, orientation, capture time, camera, lens, file size, and modified time behavior is explicit.
  - If an EXIF parser is added, `docs/DEPENDENCIES.md` is updated in the same PR.
  - If no parser is added, unavailable camera/lens metadata remains explicitly unavailable.
- **Validation:**
  - `python3 scripts/harness/check-cargo-deps.py`
  - `python3 scripts/harness/check-md-links.py`
- **Status:** Completed on 2026-06-11. `silica-catalog` now records the Task 11.7 metadata field contract and explicit no-parser dependency gate. The wiki, schema reference, and dependency policy document that camera make/model, lens, orientation, and EXIF capture time remain unavailable until a parser is selected and documented.

### Task 11.7.2: Metadata Backfill and JPEG-Only Extraction Policy

- **Location:** `docs/wiki/topics/catalog.md`, `crates/silica-storage`, `crates/silica-core`
- **Description:** Define how existing imports without metadata behave and what metadata can be extracted without adding RAW decode.
- **Dependencies:** Task 11.7.1
- **Acceptance Criteria:**
  - No automatic metadata backfill runs on launch or session restore.
  - Existing imports without metadata stay unknown until import or an explicit scoped backfill task populates them.
  - JPEG/JPG dimensions may use the existing raster path.
  - RAW metadata does not imply RAW decode support.
  - Camera and lens remain unavailable unless a documented parser dependency is added.
- **Validation:**
  - `python3 scripts/harness/check-md-links.py`
- **Status:** Completed on 2026-06-11. `silica-storage` and `silica-core` now expose a metadata extraction policy without running backfill: library open/session restore do not populate `photo_metadata`, existing unknown metadata remains unknown until explicit import/backfill work, JPEG/JPG dimensions may use the existing raster path, and RAW/camera/lens metadata does not imply RAW decode or parser support.

### Task 11.7.3: Metadata Migration and Extraction

- **Location:** `crates/silica-storage`, `crates/silica-core`
- **Description:** Extract and persist basic metadata for imported image files.
- **Dependencies:** Task 11.7.2
- **Acceptance Criteria:**
  - Originals remain unchanged.
  - Missing metadata is stored as null or explicit unavailable state.
  - Unsupported files do not produce fake metadata.
  - Backfill behavior for existing imports is documented.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core metadata`
  - original hash safety checks
- **Status:** Completed on 2026-06-11. Migration 4 now adds nullable `photo_metadata.width`, `photo_metadata.height`, and `photo_metadata.orientation`; storage can upsert normalized metadata by original path for imported, supported photos only. Core import persists JPEG/JPG width and height via the existing JPEG-only image path, keeps missing RAW/camera/lens/orientation/capture metadata unavailable, skips unsupported files, and verifies original hashes remain unchanged.

### Task 11.7.4: Metadata Query API

- **Location:** `crates/silica-storage`, `crates/silica-core`, `apps/desktop/src-tauri`
- **Description:** Expose metadata through typed core and desktop APIs.
- **Dependencies:** Task 11.7.3
- **Acceptance Criteria:**
  - Metadata responses distinguish known, unknown, and unavailable.
  - Query APIs do not read original files during inspector display unless explicitly scoped.
- **Validation:**
  - `cargo test -p silica-core metadata`
  - `cargo test -p silica-desktop metadata`
- **Status:** Completed on 2026-06-11. Storage, core, and desktop now expose a typed `get_photo_metadata` path that reads catalog data only and returns explicit `known`, `unknown`, or `unavailable` field states. Tests remove originals before metadata query to verify inspector-style reads do not reopen source files.

### Task 11.8.1: Metadata Inspector UI

- **Location:** `apps/desktop/static/`, `scripts/harness/`
- **Description:** Replace placeholder metadata rows with real Library and Loupe metadata sections.
- **Dependencies:** Task 11.7.4
- **Acceptance Criteria:**
  - Inspector displays real metadata only.
  - Missing metadata uses `Unavailable` or equivalent honest state.
  - Multi-selection does not invent aggregate metadata.
- **Validation:**
  - `python3 scripts/harness/check-static-ui.py`
  - `python3 scripts/harness/check-ui-workflow-smoke.py`
- **Status:** Completed on 2026-06-11. The shared Library/Loupe inspector now calls `get_photo_metadata`, displays stored file facts and metadata with honest `Unavailable` states, and reports multi-selection as primary-photo-only instead of aggregating fields. Static UI and workflow smoke harnesses now cover the metadata rows and command wiring.

### Task 11.8.2: Metadata Search and Filters

- **Location:** `apps/desktop/static/`, `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`
- **Description:** Enable search and filter behavior only for fields backed by stored metadata and query APIs.
- **Dependencies:** Task 11.8.1
- **Acceptance Criteria:**
  - Search/filter controls are disabled until their backing indexed query exists.
  - Enabled filters produce real query results.
  - Empty and missing states are clear.
- **Validation:**
  - `cargo test -p silica-core metadata_filter`
  - `python3 scripts/harness/check-ui-workflow-smoke.py`
- **Status:** Completed on 2026-06-11. Added an indexed `has_dimensions` metadata query filter backed only by stored `photo_metadata.width` and `photo_metadata.height`, wired through core, desktop DTOs, persisted layout state, and the static Library toolbar. Camera/lens filters stay disabled until parser and index support exists.

## Sprint 5: Reviewable Import Errors and Recursive Import

Goal: Make import broader without making it silent, unsafe, or hard to recover from.

Demo/Validation:

- Import errors and unsupported files are reviewable.
- Recursive import is opt-in after error review exists.
- Browsing continues after recoverable import errors.

### Task 11.9.1: Import Error and Recursive Import Policy

- **Location:** `docs/wiki/topics/catalog.md`, `docs/wiki/roadmaps/post-alpha-product-roadmap.md`
- **Description:** Document structured import error categories and recursive import behavior before implementation.
- **Dependencies:** Task 11.8.2
- **Acceptance Criteria:**
  - Recoverable failures and unsupported files have reviewable categories.
  - Recursive import defaults off.
  - Symlink directory behavior is explicit.
  - Hidden files, packages, max depth, permissions errors, and unsupported files are explicit.
  - Originals remain referenced by path only.
- **Validation:**
  - `python3 scripts/harness/check-md-links.py`

### Task 11.9.2: Structured Import Error Model

- **Location:** `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`
- **Description:** Implement structured recoverable import errors for the existing non-recursive import path.
- **Dependencies:** Task 11.9.1
- **Acceptance Criteria:**
  - Recoverable errors are returned in a reviewable list.
  - Unsupported files are visible.
  - Browsing can continue after recoverable errors.
  - Non-recursive behavior remains the default.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core import_error`

### Task 11.9.3: Import Error Review UI

- **Location:** `apps/desktop/static/`, `apps/desktop/src-tauri`, `scripts/harness/`
- **Description:** Replace placeholder `View Errors` behavior with a real review surface.
- **Dependencies:** Task 11.9.2
- **Acceptance Criteria:**
  - Error review shows failed and unsupported entries.
  - The library remains browseable while errors are visible.
  - The review surface does not require recursive scanning to exist.
- **Validation:**
  - `python3 scripts/harness/check-ui-workflow-smoke.py`
  - visual QA for import-error state

### Task 11.9.4: Opt-In Recursive Scanner

- **Location:** `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Implement explicit recursive scanning after structured errors and review UI exist.
- **Dependencies:** Task 11.9.3
- **Acceptance Criteria:**
  - Recursive import is user-selected and defaults off.
  - Recoverable recursive errors are returned in the same reviewable model.
  - Unsupported files are visible.
  - Browsing can continue after recoverable errors.
  - Symlink directory handling follows policy.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core recursive_import`
  - `python3 scripts/harness/check-ui-workflow-smoke.py`

### Task 11.9.5: Connected Runtime Smoke for Phase 11

- **Location:** `scripts/harness/`, `apps/desktop/src-tauri`
- **Description:** Extend connected runtime smoke for the completed Phase 11 user path.
- **Dependencies:** Task 11.9.4
- **Acceptance Criteria:**
  - Create/open records recents.
  - Relaunch restores valid last state.
  - Missing library/photo fallbacks are safe.
  - Paged grid path works.
  - Metadata display uses real stored values.
  - Recursive import errors are reviewable.
  - Original hashes remain unchanged.
- **Validation:**
  - `python3 scripts/harness/check-connected-runtime-smoke.py`
  - `scripts/harness/check.sh`

## Validation Strategy

Use smallest useful verification while developing each atomic task.

- Docs-only tasks: `python3 scripts/harness/check-md-links.py`
- Dependency changes: `python3 scripts/harness/check-cargo-deps.py`
- Session/core logic: targeted `cargo test -p silica-core <filter>`
- Storage/query/import logic: targeted `cargo test -p silica-storage -p silica-core <filter>`
- Desktop command changes: targeted `cargo test -p silica-desktop <filter>`
- UI contract changes: `python3 scripts/harness/check-static-ui.py` and `python3 scripts/harness/check-ui-workflow-smoke.py`
- Integrated runtime changes: `python3 scripts/harness/check-connected-runtime-smoke.py`
- PR completion: always run `scripts/harness/check.sh`

Do not add broad fallback systems or large test matrices before evidence requires them. Phase 11 fallback behavior should stay limited to safe defaults, warnings, disabled/unavailable UI states, and documented stop gates.

## Stop Gates

Stop and redesign if any Phase 11 task would:

- store app session state in a library catalog
- store app session state in sidecars
- rely on webview `localStorage` as durable source of truth
- mutate, move, copy, overwrite, or delete original photo files
- run import, rescan, sidecar sync, backup restore, or migration automatically during session restore
- accept arbitrary SQL, column names, or predicates from UI
- add dependencies without updating `docs/DEPENDENCIES.md`
- claim broad RAW support or color correctness
- silently enable recursive import
- follow symlink directories without explicit policy
- keep loading every catalog row after the paged query API is introduced
- show fake metadata, fake recents, fake errors, or fake aggregate multi-select state
- add broad fallback systems or large test matrices without evidence

## Rollback Plan

Each Phase 11 PR must be revertible.

- Session schema changes must preserve old valid session files or safely reset to defaults.
- Catalog migrations must be forward-only and tested on empty and existing catalog shapes.
- UI changes must keep existing create/open/import/grid/cull/loupe/develop/export/cache clear path working.
- New commands must leave existing command paths available until replacement is validated.
- If a task fails visual or runtime QA, revert that atomic PR rather than mixing recovery work into the next task.

## Agent Consultation Summary

Architecture agent conclusion:

- Phase 11 order should be session truth, scalable query path, grid behavior, metadata, then recursive import.
- Last-session restore is not backup restore UI.
- Current full-list grid and eager thumbnail path must be fixed before virtualization.

Storage guardian conclusion:

- Use app-level JSON outside every library for recents/session/layout.
- Keep `catalog.db` for library state and sidecars for portable recovery only.
- Inject the app-session path into core; let desktop resolve the real Tauri app path.
- Query filters and sorts must be typed and whitelisted.

Frontend agent conclusion:

- Start from current seams, not mockups alone.
- Welcome recents, layout state, virtualized grid, metadata inspector, and import errors all need new harness coverage.
- Visual QA must include recents, multiselect, metadata, and import-error states.

Test agent conclusion:

- Keep `scripts/harness/check.sh` as the merge gate.
- Use narrow task-specific checks during development.
- Avoid broad fallback systems and unnecessary test bloat.

Re-audit tightening:

- Use bounded offset pagination for Phase 11; defer cursor pagination until measurements justify it.
- Add page-scoped thumbnail hydration before virtualized grid work depends on it.
- Treat metadata backfill as an explicit policy, not an automatic launch or restore side effect.
- Model and review import errors before enabling recursive scanning.
