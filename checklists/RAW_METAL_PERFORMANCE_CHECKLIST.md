# RAW Metal Performance Checklist

Status: active
Updated: 2026-06-17
Source of truth: docs/wiki/tasks/22.4-raw-metal-performance-profiling.md

## Fixture-backed RAW

- [x] Decode timing is separated from render, UI, and export rows in the profile report.
- [x] Fixture-backed decode/export paths are marked as gated when `SILICARAW_RAW_FIXTURE_MANIFEST` is not provided.
- [x] Report language avoids broad RAW support claims.
- [ ] Run ignored fixture-backed decode/export timing locally when legal RAW fixture assets are available.

## Native viewer

- [x] Native viewer timing is limited to the feature-gated request smoke boundary.
- [x] The default app path remains feature-off for native Metal viewer behavior.
- [x] The report records that GPU pixel throughput is not measured by the current profile.
- [ ] Run installed-app interactive drag profiling on a packaged local alpha build.

## Memory

- [x] Each measured command records child-process `max_rss_kb`.
- [x] Machine memory, model, chip, OS, Rust, and app version are included in the report.

## Known limitations

- Current Task 22.4 evidence is local-machine profiling, not a performance guarantee.
- Full RAW decode/export timing requires a legal local RAW fixture manifest.
- Full UI latency requires installed-app instrumentation and should not be inferred from cargo test timing.
- Full Metal rendering throughput remains outside this task because implementing product Metal pixel rendering here would exceed Phase 22 scope.
