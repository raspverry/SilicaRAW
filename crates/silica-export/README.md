# silica-export

Export coordination boundary for SilicaRAW.

This crate owns the local alpha JPEG export path, disposable JPEG thumbnails, and disposable Develop JPEG previews.

Task 13.6 adds ICC embedding proof for JPEG export. sRGB remains the default export target. Display P3 is available only through an explicit export request and is not yet wired to a user-facing option.
