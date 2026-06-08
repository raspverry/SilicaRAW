# silica-catalog

Catalog domain boundary for SilicaRAW.

This crate will model libraries, folders, photo identity, collections, culling flags, missing-file state, and query-facing catalog behavior.

Phase 4.1 adds the local alpha catalog schema contract:

- current schema version
- required catalog tables
- required catalog indexes
- migration bookkeeping table name

Phase 4.3 adds the local alpha import candidate contract:

- supported photo file extensions
- import candidate path/fingerprint fields
- unsupported candidate state

Phase 4.4 adds the local alpha photo flags contract:

- rating range validation
- picked and rejected state
- optional color label normalization

`silica-storage` owns SQLite execution and migration application. `silica-catalog` owns the domain-facing contract that later library create/open, import, culling, and query code should reference.
