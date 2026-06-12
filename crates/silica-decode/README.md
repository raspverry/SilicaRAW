# silica-decode

RAW decode abstraction boundary for SilicaRAW.

Spike 002 selected Core Image RAW primary with LibRaw deferred until legal fixtures expose a concrete coverage gap.

Phase 5.1 adds a preview decode readiness contract. Raster candidates such as JPEG can be marked ready by reference, unsupported catalog entries return a clear unsupported state, and RAW candidates return a Core Image RAW blocked state until fixture-backed probe coverage exists.

Phase 12 adds the non-default `core-image-raw-probe` macOS feature for fixture-backed Core Image probe evidence. Phase 15.2 adds a narrow fixture-backed Core Image path that can write bounded JPEG sRGB RAW preview artifacts for supported local fixture classes.

This crate still does not provide broad RAW camera support, LibRaw support, final color correctness, full-resolution RAW export, or original-file mutation behavior.
