# silica-render

Render request and renderer boundary for SilicaRAW.

Spike 003 selected Core Image/ColorSync-compatible color management first, with a linear Display P3 working space recommendation, display-profile-aware preview, and sRGB-default export with Display P3 support.

This crate currently records color gate metadata only. No Metal viewer, shader, Core Image context, ColorSync transform, ICC embedding, or image processing implementation is present yet.
