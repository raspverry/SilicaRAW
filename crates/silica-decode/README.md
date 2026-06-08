# silica-decode

RAW decode abstraction boundary for SilicaRAW.

Spike 002 selected Core Image RAW primary with LibRaw deferred until legal fixtures expose a concrete coverage gap.

Phase 5.1 adds a preview decode readiness contract. Raster candidates such as JPEG can be marked ready by reference, unsupported catalog entries return a clear unsupported state, and RAW candidates return a Core Image RAW blocked state until fixture-backed probe coverage exists.

No RAW decoding implementation, Core Image binding, LibRaw binding, fixture loader, or image processing backend is present yet.
