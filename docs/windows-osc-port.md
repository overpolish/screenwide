<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Windows region OSC port — D3D11 GPU compositor

Status: all four stages landed (started 2026-09-01). Mac is the parity reference. Region selection, desktop peers and snapshots, OCR, and Ruler have been user-verified on Windows, including a region spanning multiple monitors.

Implementation deviations so far:

- The host window is placed over the anchor monitor by `native_osc_windows/desktop.rs::place_over_anchor` (macOS parity: `rebuild_surfaces` sets the parent frame).
- Overlay windows disable DWM show transitions (`platform.rs::disable_show_transitions`) to match AppKit panel ordering.
- Non-composited snapshots draw as a full-viewport kind-33 quad in the main pass instead of a second visual (macOS used a CALayer).
- Mixed-DPI desktop plane: dividing each monitor by its own scale can make neighbours overlap in the plane; uniform DPI is exact. This remains the one functional parity caveat.
- No cursor hiding under the magnifier lens (`ShowCursor` is a global refcount; the lens is anchored to the dragged edge, so the pointer is not under it).

Research reports (read these before implementing):

- `docs/windows-osc-port/macos-architecture.md` — full map of the macOS Obj-C/Metal implementation
- `docs/windows-osc-port/rust-native-contract.md` — the exact Rust↔native contract to satisfy
- `docs/windows-osc-port/windows-gpu-infra.md` — existing reusable Windows D3D11/DComp infrastructure

## Goal

Reimplement the macOS native GPU compositor ("region OSC": region selection frame, rulers, OCR overlays, magnifier, frozen-desktop snapshots, multi-monitor desktop peers) on Windows with D3D11 + DirectComposition, so `adapter/unavailable.rs` stops reporting the compositor absent and Windows reaches feature parity with macOS.

## Architecture decisions (settled — do not relitigate in implementation)

1. **No C ABI.** macOS needs `extern "C"` because the native side is Obj-C. On Windows the native side is Rust: implement `native_osc_windows` as a pure-Rust twin of `native_osc_macos`, exposing the same ~30 `native::*` functions (rust-native-contract.md §1a–1e) and calling `OscRuntime::input` / `native_osc_layout_changed`-equivalents directly. The `#[no_mangle]` palette and control-group exports in `src/osc/` are already portable Rust — call them as plain functions.
2. **All policy stays in `src/osc/`** (owners, reconciliation, desktop projection, gestures, semantic events, palettes, control/confirm state machines). The Windows module owns only HWNDs, DComp visuals, swapchains, textures, GPU submission, pointer/cursor claim and window ordering. `apply_region_scene`'s diff-and-order discipline must be preserved exactly (geometry before desktop peers; clear before hide on teardown) — port `native_osc_macos/state.rs` semantics line-for-line.
3. **Rendering model = the macOS one, not the preview's.** CPU vertex builder producing NDC quads (`#[repr(C)] Vertex { position: [f32;2], uv: [f32;2], kind: u32, padding: u32 }`, stride 24, static-asserted) into a `D3D11_USAGE_DYNAMIC` vertex buffer (`Map`/`WRITE_DISCARD`), one `Draw(count, 0)` triangle list per frame, one uber pixel shader switching on `kind` (interpolated as `nointerpolation uint`). This is the repo's first vertex buffer — justified because rulers emit hundreds of primitives, beyond the preview's fullscreen-triangle + constant-buffer style. `region_osc_renderer_macos.m` is ~90 % portable math; port it 1:1 to Rust.
4. **Uber-shader ports 1:1 to HLSL** (`fwidth` exists in HLSL; `discard_fragment()` → `discard`). Metal push constants b0–b8 become ONE cbuffer with the same fields in declaration order, padded to float4 rows (follow the repo's float4-row convention + packing test, windows-gpu-infra.md §1.5). Precompiled in `build.rs` via the existing `compile_shader()` (vs_5_0/ps_5_0 or 4_0 to match existing), `include_bytes!` at use site.
5. **Magnifier = textured quad drawn LAST, not a compute pass.** Bind the magnifier source as a Texture2D; port the `region_magnifier` kernel math (40 px source window across a 96 pt box, nearest-neighbour, rounded-rect mask, edge shading, 1 px border) into a pixel-shader kind. Drop the `discard` lens cutout entirely — it only existed because Metal wrote the lens before the raster pass.
6. **Layering above WebView2**: anchor display uses a child HWND of the Tauri window with `WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP`, `WS_CHILD | WS_CLIPSIBLINGS`, kept at `HWND_TOP`, with a **topmost** `IDCompositionTarget` — exactly the `EditorWindow`/`SelectionOverlay` pattern (windows-gpu-infra.md §1.1, §5). Composition swapchain: B8G8R8A8, FLIP_DISCARD, `DXGI_ALPHA_MODE_PREMULTIPLIED`, cleared to transparent every frame.
7. **Desktop peers** (non-anchor monitors): one `WS_POPUP` window per monitor, `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_NOREDIRECTIONBITMAP`, not activated, `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` (≈ `NSWindowSharingNone`), own DComp target + swapchain, shared D3D11 device. `desktopOffset` model ports as-is but **without y-flips** — Windows is already top-left; delete every AppKit flip (audit direction of each one). `WM_DISPLAYCHANGE`/`WM_DPICHANGED` replace the screen-parameters notification and must trigger rebuild + the Rust layout-changed callback.
8. **Own D3D11 device** for the OSC (don't entangle with the preview surface): `D3D11CreateDevice` with BGRA support, `SetMultithreadProtected(true)`, feature levels 11_1/11_0 — copy `Gpu::new` (windows-gpu-infra.md §1.1).
9. **Text**: GDI rasterization (the `label.rs` pattern — bundled Inter via `AddFontMemResourceEx`, coverage in one channel, 2× supersample + box-downsample), reproducing the macOS fixed-cell mono atlas layout (`"#0123456789ABCDEF× px≈"`, 1 px gutters, texel-center UVs). No DirectWrite. Mono font: match what macOS bundles (Roboto Mono) if present in assets, else Consolas.
10. **Material blur chrome**: stage 1 uses flat translucent rounded rects drawn in-shader (kinds 12/13/14 + rounded-rect background). The macOS `NSVisualEffectView` blur is replaced later by self-blurring the frozen snapshot texture (the desktop is frozen during these sessions, so a static blur is faithful). Fold ALL chrome into the per-surface swapchain — no extra swapchains per label.
11. **Input**: the overlay child HWND takes mouse input only when the scene has `input_enabled && allow_drawing` (else `WM_NCHITTEST` → `HTTRANSPARENT`); inside `exclusion_rect` always `HTTRANSPARENT` (that's the webview's own toolbar). `WM_MOUSEACTIVATE` → `MA_NOACTIVATE`; `SetCapture` during drags; `WM_SETCURSOR` maps `CursorIcon` (1 crosshair … 9 pointing hand) to system cursors — `IDC_SIZENWSE`/`NESW`/`WE`/`NS` replace the hand-drawn macOS cursors. Pointer events convert to desktop-local top-left coords and call `OscRuntime::input` with phases 1–5 and modifier bits (1 shift, 2 cmd/ctrl, 4 double-click, 8 alt). `result.status == 255` (Invalid) = rejected. Keyboard (ruler keys, OCR Ctrl+A/C) comes with stages 3–4.
12. **Draw scheduling**: event-driven (no render loop). Redraw on state change; 16 ms `SetTimer` drives animations while `animating`; never block the input thread on present (`Present(0, 0)`, at most one frame in flight — waitable swapchain object if needed). `IDCompositionDevice::Commit` only on visual geometry changes.
13. **Theme**: registry `AppsUseLightTheme` + `WM_SETTINGCHANGE` replaces `effectiveAppearance`.
14. **DPI**: per-surface scale from `GetDpiForWindow`/monitor DPI (peers may differ from the anchor — macOS assumed uniform `backingScaleFactor`; keep scale per-surface). Pixel snapping helpers: `snap(v) = (floor(v*scale)+0.5)/scale` for hairlines, `round(v*scale)/scale` for handle centers.
15. **Formats**: `B8G8R8A8_UNORM` everywhere (NOT `_SRGB`) to reproduce the exact macOS blend math. CPU uploads of RGBA data may use `R8G8B8A8_UNORM` textures (precedent: `screenshot_source`). Blend states port Metal's exactly: normal = SrcAlpha/InvSrcAlpha (color and alpha); snapshot-composited variant sets srcA = ONE.

## Module layout

```
src-tauri/src/windows/screenshot_region/
  native_osc_windows.rs           facade, same re-export shape as native_osc_macos.rs
  native_osc_windows/
    state.rs      context registry (keyed by Tauri-window HWND), OscRuntime ownership,
                  RegionSceneState, the ~30 native::* functions (port of macOS state.rs)
    surface.rs    per-display surface: HWND + DComp target/visual + swapchain + draw loop
    renderer.rs   vertex builder (port of region_osc_renderer_macos.m) + render state
    input.rs      window proc, pointer dispatch, cursor claim
    desktop.rs    monitor enumeration, peer windows, configure_desktop_window
    text.rs       GDI atlas + string textures (stage 3)
    shaders/region_osc.hlsl
  adapter/windows.rs              port of adapter/macos.rs (hwnd() instead of ns_view())
```

`adapter.rs` cfg selects: macos → `adapter/macos.rs`, windows → `adapter/windows.rs`, else `adapter/unavailable.rs`. Same for `text_recognition/adapter.rs` and `ruler/adapter.rs` in later stages. Keep the `Ok(false)` fallback path when surface creation fails so the webview implementation remains the safety net.

## Stages

1. **Region scene** (current): renderer.rs + region subset of the shader (kinds 0/2/3/16 frame+handles, 6 dim, 7–10 marching ants, 4/5 guides, 33 snapshot quad, magnifier), state.rs, anchor-display surface, adapter/windows.rs, pointer input, semantic events, cursors. Testable: quick-screenshot region select runs natively on Windows.
2. **Desktop peers + snapshot**: `configure_desktop_window`, per-monitor peers, `set_desktop_presented`, snapshot presented/composited paths, layout-change rebuild.
3. **OCR overlay** (done): text atlas + string textures, icon atlas (R8, from Rust `screenwide_osc_icon_atlas`), OCR rect kinds 17–20, status pill, cancel button, 4-button toolbar with confirm state machine, OCR command phases, `text_recognition/adapter/windows.rs`.
4. **Ruler** (done): the `+ruler.m` port — crosshair, probes, guides, gaps, radii, centerlines, inner objects, measurement labels, loupe readout, viewport zoom/pan, label drag, keyboard phases, `ruler/adapter/windows.rs`.

The blur chrome, topology-change delivery, and passive keyboard route are implemented. Mixed-DPI desktop projection remains approximate; accessibility differences are recorded below.

## Deviations from macOS

Recorded as they land, with the reason. Shader kinds are **append-only**: an existing kind is never renumbered or repurposed, and the packing/golden tests are updated in the same change.

### Stage 3 (OCR overlay)

1. **Kind 46 — rounded chrome plate (new).** macOS masked each floating control with its material surface's `cornerRadius`, so kinds 12/13/14 could stay deliberately rectangular. With the chrome folded into the one swap chain there is no surface to own the radius, so the plate owns it: a rounded-rect SDF filled with `action_fills[0]` and optionally stroked with `chrome_outline`. Kinds 12/13/14 are untouched and still available.
2. **Kind 47 — tinted chrome text (new).** The status pill was the only live AppKit text (`NSTextField`), and its colour came from the OCR palette's `loading_foreground` / `status_error_foreground`. A colour-baked glyph texture cannot express both, so chrome glyphs are rasterised as white coverage and tinted from `action_fills[1]` at draw time. The un-premultiplying kinds 11/15/37 keep their contract and the monospace atlas still bakes its ink, so the stage-4 ruler port is unaffected.
3. **Two appended cbuffer rows**, `chrome` (`.x` = plate radius) and `chrome_outline`, after `magnifier_flags`. Every existing offset is unchanged; the packing test asserts both the old offsets and the new ones.
4. **One draw call per control instead of one swap chain per control.** macOS gave every button its own `CAMetalLayer`; here the frame is split into segments and the constant buffer is re-pushed between them. That is the same mechanism `+ocr_toolbar.m:126-156` already used for the crossfading confirm icons, now applied to every control.
5. **Snapshot-backed blur.** Chrome samples and blurs the frozen desktop texture in the shared shader rather than using separate platform material surfaces.
6. **Icon atlas reached through the `#[no_mangle]` export.** `osc::controls::icons` is a private module of the frozen portable tree, so the Windows `Gpu` calls `screenwide_osc_icon_atlas` through an `extern "C"` declaration rather than widening that module's visibility.
7. **Keyboard shortcuts use a passive monitor.** macOS catches Cmd+A / Cmd+C with a local `NSEvent` monitor. Windows keeps the overlay nonactivating and uses a short-lived low-level keyboard hook that forwards only recognised commands to the compositor's owning thread.
8. **Accessibility regression.** The macOS material surfaces carried `NSAccessibilityButtonRole` labels. The folded-in chrome has no child HWNDs, so there is nothing for UI Automation to describe (macos-architecture.md §8.18 anticipated this).

### Stage 4 (Ruler)

1. **No new shader kinds and no new cbuffer rows.** Kinds 28-44 were already compiled in stage 1; 46 (rounded plate) replaces the material surfaces' `cornerRadius` for the loupe and the four label pools, exactly as it does for the OCR chrome. The macOS loupe drew its background with kind 12; on Windows nothing owns a radius but the plate, so it is kind 46 instead.
2. **`Segment` grew a `secondary` texture.** The tolerance notice is the only consumer of `t1` (kinds 15/37), so the per-segment push now carries both slots. Segments that do not set it fall back to the transparent placeholder, which keeps the "never null an SRV" rule.
3. **Label rectangles are the hit test.** macOS hit-tested the AppKit frames of the label material surfaces. With the chrome folded into one swap chain there are no frames, so `ruler.rs` records each label's rectangle while it lays it out and `label_hit` walks that list in the macOS pool order (measurement, probe, guide gap, radius).
4. **Label ownership is decided once, above the surfaces.** `measurement_label_surface` asked each surface for its visible world rect; the Windows twin computes every world rect in `apply_ruler_result` and calls `ruler::assign_labels` once, because only the caller holding the whole set can answer the question.
5. **Every ruler result redraws every surface.** macOS could skip the world pass when only the pointer moved, because the readout was its own layer. Folded in, the loupe _is_ the frame, so the pass is unconditional.
6. **Change detection compares bytes, not `PartialEq`.** An absent label anchor is `NaN`, so a derived `PartialEq` would report every unanchored artifact as changed on every sample. `same()` compares the packets' bytes, which is what `isEqualToData:` did. The packets are `#[repr(C)]` with explicit padding fields, so there are no uninitialised holes.
7. **Session timers replace the `dispatch_after` chains.** Three one-shot `SetTimer` ids on the root overlay stand in for the settle frame (16 ms, phase 15), the copied checkmark's 900 ms expiry and the tolerance notice's. Each is killed on arrival because `SetTimer` repeats. macOS's revision counters are unnecessary: re-arming the same timer id restarts it, which is the same guard. The hover/copied/tolerance _transitions_ ride the existing 16 ms `ANIMATION_TIMER` retimer via `Ruler::is_animating`.
8. **Wheel and gesture mapping.** macOS zoomed on `exp(scrollingDeltaY * 0.01)` and panned by raw scroll points; Windows has notches, so one notch zooms by `exp(0.1)` and pans by 40 pt. `WM_MOUSEHWHEEL` supplies the horizontal pan AppKit got from `scrollingDeltaX`. There is no `NSEventTypeMagnify` equivalent wired — a precision touchpad reports pinch as Ctrl+wheel, which already lands on the zoom path. **The scroll sign is untested** and may need flipping after the first run on hardware.
9. **Keyboard uses the same passive route as OCR.** `ruler::key_command` implements the full macOS table, including latching range/guide/radius keys and their release phases. Direct window messages are used while the child has focus, with the short-lived low-level monitor covering focus changes. The macOS keycodes in `+input.m` decode as X, Tab, Delete, **C**, **Z**, **Y**, T, M, 1/2, V/H, R (macos-architecture.md §6 lists the raw numbers 8/6/16, which are those letters, not digits).
10. **The tolerance notice is a monospace ink-baked label.** `TextCache` gained `ink_label`, because kind 37 un-premultiplies its sample rather than tinting it, so the colour has to be in the texture the way `screenwide_osc_mono_text_texture` baked it.
11. **`control_spacing` is reached through its `#[no_mangle]` export**, like the icon atlas in stage 3: `osc::controls::style` is a private module of the frozen portable tree.
12. **No per-label `ControlGroup`.** macOS read the label fills from a one-control group that nothing ever hovered, so the Windows port calls `control_visual(..., Interaction::Normal, ...)` directly for the same values.
13. **Accessibility regression continues.** The macOS labels and readout carried `NSAccessibilityStaticTextRole` values; the folded-in chrome has no child HWNDs to describe.

## Known hazards

- Flip-discard back buffers are undefined after present → always clear.
- DComp defaults to nearest sampling → `SetBitmapInterpolationMode(LINEAR)`.
- Never null an SRV slot — bind 1×1 transparent placeholder textures.
- Glyph/label textures are premultiplied sources drawn with straight-alpha blending: shader un-premultiplies (`rgb/a`) before returning — keep that.
- Re-assert `HWND_TOP` for the overlay child on every move (WebView2 sibling).
- Ruler hover opacity is smuggled in `padding[0]` of the pulled structs; change detection is byte-equality on the pulled blobs (macos-architecture.md §4).
