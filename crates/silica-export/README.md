# silica-export

Export coordination boundary for SilicaRAW.

This crate owns the local alpha JPEG export path, disposable JPEG thumbnails, and disposable Develop JPEG previews.

Task 13.6 adds ICC embedding proof for JPEG export. sRGB remains the default export target. Display P3 is available only through an explicit export request and is not yet wired to a user-facing option.

Task 17.2.1 applies the same deterministic local white-balance adjustment to supported JPEG/JPG Develop previews and JPEG exports. This keeps preview/export parity for the committed edit graph; it is not a fixture-backed color-correctness claim.

Task 17.2.2 applies the same deterministic local tone recovery adjustment to supported JPEG/JPG Develop previews and JPEG exports. This keeps highlights, shadows, whites, and blacks aligned with the committed edit graph without broad RAW or color-correctness claims.
