# 20 — SilicaRAW v1.1 Architecture Patch

Status: REQUIRED BEFORE CODEX IMPLEMENTATION  
Purpose: Address external review findings and close execution-blocking gaps.

---

## 1. Tauri + Metal Fallback Strategy

The Tauri + native Metal viewer bridge is the highest-risk spike.

### Spike 001 must verify

```txt
Tauri window can host or coordinate a native Metal-rendered view.
Metal output appears in the app window.
Resize works.
Retina scaling works.
Mouse/trackpad events map correctly.
UI remains responsive.
Render timing is available.
Metal render loop can be controlled from Rust/Core.
```

### Path A — Success

```txt
Continue with Tauri + Rust Core + native Metal viewer.
```

### Path B — Partial Success

```txt
Keep Tauri shell and controls, but isolate the viewer behind a stronger native AppKit/Metal bridge.
```

### Path C — Failure

```txt
Move to SwiftUI/AppKit native shell + Rust Core.
```

Rule:

```txt
Metal-first editor identity takes priority over Tauri.
Do not force Tauri if it blocks the core editor.
```

Electron is not preferred and should be considered only after SwiftUI/AppKit + Rust Core is proven impractical.

---

## 2. RAW Decoder Decision Gate

RAW decoder choice affects:

```txt
input profile
camera profile
color pipeline
RAW format support
Apple ProRAW support
lens metadata
LibRaw FFI/distribution
```

### Decoder-dependent tags

```txt
RAW Decode: decoder-blocking
Camera Profile: decoder-dependent
Lens Correction: decoder-dependent
Color Baseline: decoder-dependent
Fuji RAF support: high-risk decoder-dependent
Apple ProRAW: CoreImage-preferred
Broad camera support: LibRaw-preferred
```

### Spike 002 outcomes

```txt
Path A: Core Image RAW primary
Path B: LibRaw primary
Path C: Hybrid
```

Recommendation:

```txt
Initial MVP:
Core Image RAW primary + LibRaw spike.

After Spike 002:
Decide whether v0.2 uses Core Image only or hybrid.
```

---

## 3. License Gate

Gate A now requires:

```txt
Provisional license strategy selected.
Dependency license policy documented.
```

Public beta gate requires:

```txt
Final project license selected.
Third-party dependency license inventory.
Model license manifest.
Sample asset license manifest.
```

No public beta without final license decision.

---

## 4. Dependency Policy

New dependencies must be documented in:

```txt
docs/DEPENDENCIES.md
```

Every dependency entry must include:

```txt
Name
Version
Purpose
License
Repository/homepage
Why needed
Alternatives considered
Risk notes
Binary size impact
Security notes
```

Codex must not add dependencies without updating that file.

---

## 5. Doc 06 Single Source Rule

The authoritative UI screen document is:

```txt
docs/06_Screen_Inventory_and_Wireframe_Specification.md
```

Archived files under:

```txt
docs/archive/
```

are historical and must not be used for implementation.

---

## Final Verdict

This patch upgrades the docs from:

```txt
Codex-ready but risky
```

to:

```txt
Codex-ready with explicit gates and schemas
```
