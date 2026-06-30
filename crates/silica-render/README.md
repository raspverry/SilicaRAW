# silica-render

Render request and renderer boundary for SilicaRAW.

Spike 003 selected Core Image/ColorSync-compatible color management first, with a linear Display P3 working space recommendation, display-profile-aware preview, and sRGB-default export with Display P3 support.

Phase 5.1 adds a preview render readiness contract that wraps decode readiness with the selected display-profile-aware preview behavior.

Phase 5.3 adds a render request contract for draft exposure/contrast preview updates. This records the requested adjustment values and preview readiness, but still does not render pixels.

Phase 13.3 adds a non-default `color-probe` feature that reads JPEG marker/profile metadata for fixture proof and records source hashes. This is probe evidence only; it does not render pixels or apply transforms.

Task 17.2.1 carries white-balance-family values through preview/export request planning. It still does not create a Metal viewer, shader, Core Image context, ColorSync transform, ICC embedding, or render pixels.

Task 17.2.2 carries tone recovery values through preview/export request planning under the same boundary. Pixel adjustment remains owned by `silica-export` for supported raster paths.

Task 17.2.3 carries color presence values through preview/export request planning under the same boundary.

Task 17.3 computes 256-bin RGB and luminance histograms from real RGB pixel buffers. It does not decode files or own cache persistence.
