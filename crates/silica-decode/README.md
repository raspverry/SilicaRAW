# silica-decode

RAW decode abstraction boundary for SilicaRAW.

Spike 002 selected Core Image RAW primary with LibRaw deferred until legal fixtures expose a concrete coverage gap.

Phase 5.1 added a preview decode readiness contract. Under the current local alpha source contract, JPEG/JPG, PNG, TIF, and TIFF rows can be marked ready by reference through the raster route. RAW, HEIC, WebP, database, sidecar-like, and other unsupported rows return a clear unsupported state in the product preview route.

Phase 12 adds the non-default `core-image-raw-probe` macOS feature for fixture-backed Core Image probe evidence. Phase 15.2 adds a narrow fixture-backed Core Image path that can write bounded JPEG sRGB RAW preview artifacts for supported local fixture classes.

This crate still does not provide broad RAW camera support, LibRaw support, final color correctness, full-resolution RAW export, or original-file mutation behavior.
