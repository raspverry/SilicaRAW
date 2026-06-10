# Connected Developer Runtime Smoke

Use this after the fixture/preflight gate and before clean-Mac DMG QA.

This smoke runs from the repository against the desktop command runtime, not from an installed DMG. It proves the local alpha workflow is connected through real generated fixtures and desktop command responses. Clean-Mac install behavior remains covered by `LOCAL_DMG_INSTALL_CHECKLIST.md`.

## Command

```bash
python3 scripts/harness/check-connected-runtime-smoke.py
```

The script:

- generates legal synthetic fixtures under `.tmp/harness/connected-runtime-smoke/fixtures`
- runs the exact `silica-desktop` runtime smoke test with fixture/output paths in environment variables
- keeps all generated artifacts under ignored `.tmp/` paths

## Covered Workflow

- create and open a local library
- import a flat generated fixture folder by reference
- load the library grid with JPEG thumbnails
- rate, pick, reject, and restore final culling flags
- open a JPEG loupe preview
- verify RAW-placeholder preview remains blocked without RAW decoding
- preview and commit exposure/contrast
- export a separate JPEG sRGB file
- clear only disposable cache directories
- reopen the library and restore flags/edit state
- compare original fixture bytes after each major stage

## Not Covered

- clean-Mac DMG install, mount, drag-to-Applications, or Gatekeeper behavior
- RAW decoding
- Metal viewer rendering
- MLX runtime, MCP tools, plugins, cloud sync, telemetry, or auto-update
- visual/responsive QA screenshots
