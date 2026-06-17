# silica-export

Export coordination boundary for SilicaRAW.

This crate owns the local alpha JPEG export path, disposable JPEG thumbnails, and disposable Develop JPEG previews.

Task 13.6 adds ICC embedding proof for JPEG export. sRGB remains the default export target. Display P3 is exposed only as an explicit JPEG export option with ICC evidence; this is a profile/ICC capability claim, not a visual color-correctness claim.

Task 17.2.1 applies the same deterministic local white-balance adjustment to supported JPEG/JPG Develop previews and JPEG exports. This keeps preview/export parity for the committed edit graph; it is not a fixture-backed color-correctness claim.

Task 17.2.2 applies the same deterministic local tone recovery adjustment to supported JPEG/JPG Develop previews and JPEG exports. This keeps highlights, shadows, whites, and blacks aligned with the committed edit graph without broad RAW or color-correctness claims.

Task 17.2.3 applies the same deterministic local color presence adjustment to supported JPEG/JPG Develop previews and JPEG exports. This keeps vibrance and saturation aligned with the committed edit graph without broad RAW or color-correctness claims.

Task 17.3 computes Develop histogram data for supported JPEG/JPG sources through the same local adjustment order used by preview/export, then delegates binning to `silica-render`.
