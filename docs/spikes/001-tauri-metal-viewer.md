# Spike 001: Tauri + Native Metal Viewer

Status: completed  
Date: 2026-06-08  
Result: Path B - partial success

## Question

Can the selected Tauri shell host or coordinate a native Metal-rendered view without blocking the Metal-first editor direction?

## Result

Path B:

```txt
Keep Tauri shell and controls, but isolate the viewer behind a stronger native AppKit/Metal bridge.
```

Tauri can host a native `MTKView` inside the macOS app window. The spike proved native Metal output, Retina backing scale, Rust-controlled render loop, and resize behavior. It also exposed that a naive overlay can cover web content, so the product viewer must be isolated behind a dedicated native bridge and reserved layout contract.

This is not Path A because the layout and lifecycle bridge is not mature enough to treat the product viewer as solved. It is not Path C because Tauri is not blocked by Metal hosting.

## Implementation

- Added a non-default Cargo feature: `metal-host-spike`.
- Added a macOS-only `MTKView` subclass under `apps/desktop/src-tauri`.
- Attached the native view to the Tauri main window's AppKit content view.
- Added a minimal `MTKViewDelegate` that creates a command buffer, clears the drawable, presents it, and logs render timing.
- Added mouse, scroll, and magnify event logging methods on the native view subclass.
- Added `SILICA_SPIKE_AUTO_RESIZE=1` as a manual validation aid for resize evidence.

The default desktop app path remains the minimal Tauri shell. The spike code is not compiled unless `--features metal-host-spike` is passed.

## Evidence

Commands:

```sh
cargo check -p silica-desktop
cargo check -p silica-desktop --features metal-host-spike
SILICA_SPIKE_AUTO_RESIZE=1 cargo run -p silica-desktop --features metal-host-spike
```

Representative runtime log:

```txt
[SilicaRAW Spike 001] MTKView drawable resized to 850x790px
[SilicaRAW Spike 001] MTKView backing properties changed
[SilicaRAW Spike 001] Installed MTKView frame 425x395pt, drawable 850x790px, backing scale 2.0
[SilicaRAW Spike 001] Render loop drew 1 frame(s) over 55.460958ms
[SilicaRAW Spike 001] Requested automatic window resize probe
[SilicaRAW Spike 001] MTKView drawable resized to 703x710px
[SilicaRAW Spike 001] MTKView frame resized to 352x355pt
[SilicaRAW Spike 001] Render loop drew 120 frame(s) over 2.0379375s
```

Representative manual input log:

```txt
[SilicaRAW Spike 001] mouseDown at 812.1,224.5 button 0 clicks 1 delta 0.00,0.00 pressure 1.00
[SilicaRAW Spike 001] mouseDragged at 794.6,211.6 button 0 clicks 1 delta 1.00,0.00 pressure 1.00
[SilicaRAW Spike 001] scrollWheel at 838.2,166.1 delta 0.00,-5.21 phase NSEventPhase(0) momentum NSEventPhase(0)
[SilicaRAW Spike 001] magnifyWithEvent at 764.3,203.9 magnification 0.0594 phase NSEventPhase(4)
```

Visual evidence:

- A native Metal surface was visible inside the Tauri window during manual execution.
- The first proof surface used a high-contrast debug clear color and overlapped shell text.
- The spike was revised to use a smaller neutral proof surface and a shell layout that does not cover text.

## Checklist

| Criterion | Result | Evidence |
| --- | --- | --- |
| Metal output in app window | Passed | Native `MTKView` visible inside the Tauri window. |
| Resize works | Passed | Automatic resize probe changed frame and drawable size. |
| Retina scaling works | Passed | Runtime log recorded backing scale `2.0` and doubled drawable pixels. |
| Mouse/trackpad event mapping | Passed for spike | User manual input on the native surface produced mouse down, drag, scroll wheel, and magnify logs. |
| UI remains responsive | Passed | Web shell remained visible while the Metal render loop logged repeated frames. |
| Render timing available | Passed | Render loop logs frame counts and elapsed time. |
| Render loop can be controlled from Rust/Core | Passed for spike | Rust delegate creates command buffers and presents drawables. |

## Follow-Up

Before product viewer work:

- Define a dedicated AppKit/Metal viewer bridge boundary.
- Reserve native viewer layout explicitly instead of overlaying arbitrary web content.
- Decide which events are owned by web UI and which are owned by the native viewer.
- Add a focused manual harness for mouse, trackpad scroll, magnify, resize, and Retina movement across displays.
- Replace the clear-color proof with the real viewer only after the render pipeline task begins.

## Guardrails

Do not use this spike as a product viewer. It does not implement RAW decoding, image display, shader passes, color management, export, MLX, MCP, plugins, or product UI screens.
