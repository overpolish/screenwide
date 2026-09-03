<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# macOS region OSC native compositor — architecture map (D3D11 port reference)

Research snapshot 2026-09-01. Paths relative to `src-tauri\`.

## 0. File inventory / sizes

| File | Lines | Role |
| --- | --- | --- |
| `src/exports/screenshot_region_osc_macos.m` | 324 | Object lifecycle, layer attach, master frame draw |
| `…_private.h` | 386 | ObjC class `ScreenwideRegionOSC` + all FFI structs / prototypes |
| `…+appearance.m` | 39 | Dark/light change observer |
| `…+desktop.m` | 271 | Multi-display peer windows |
| `…+input.m` | 702 | Global NSEvent monitor, hit-test, cursor |
| `…+ocr.m` | 185 | OCR highlight rects + status pill |
| `…+ocr_cancel.m` | 212 | "Cancel" button surface |
| `…+ocr_toolbar.m` | 299 | 4-button OCR toolbar |
| `…+ocr_toolbar_input.m` | 73 | Toolbar hit routing + confirm state machine |
| `…+ruler.m` | 2015 | Rulers: probes, guides, gaps, radii, centerlines, labels, loupe |
| `…+snapshot.m` | 91 | Frozen desktop screenshot install |
| `…+state.m` | 106 | Magnifier source, input-enabled, exclusion rect, show frame/handles |
| `region_osc_renderer_macos.m/.h` | 490 / 152 | Vertex builders + encode + pipelines |
| `region_osc_renderer_macos_shader.h` | 356 | Whole Metal source as a raw string literal |
| `region_osc_pipeline_macos.m` | 54 | Pipeline descriptors / blend state |
| helpers: `osc_text_texture_macos.m` (204), `osc_material_surface_macos.m` (41), `osc_icon_renderer_macos.m` (43), `region_magnifier_macos.m` (74), `region_cursor_macos.m` (94) |  | text rasterization, blur surfaces, icon atlas, magnifier, cursors |

## 1. Overall structure: views, layers, windows

### The object

`ScreenwideRegionOSC` (`…_private.h:175-295`) is a plain `NSObject`, **not** a view. Attached to a host `NSView` via `objc_setAssociatedObject` (`screenshot_region_osc_macos.m:240`), looked up by `screenwide_region_osc_for_view` (`:247`). The host view is the Tauri/WKWebView's `ns_view()`. `attach()` boxes the Rust `OscRuntime` with a `release` fn pointer, so the ObjC object owns the Rust context (`dealloc` at `:8-27`).

### Layer stack (attach, `screenshot_region_osc_macos.m:222-237`)

```
view.wantsLayer = YES            // the Tauri webview host view's backing layer
  ├─ snapshotLayer  (plain CALayer, contents = CGImage, kCAGravityResize, hidden by default)
  └─ layer          (CAMetalLayer, BGRA8Unorm, framebufferOnly=NO, opaque=NO)
  + subviews added with addSubview:positioned:NSWindowAbove:
      ocrStatusSurface, ocrCancelSurface, 4× ocrToolbarSurfaces,
      rulerSurface, N× ruler{Measurement,Probe,GuideGap,Radius}LabelSurfaces,
      appearanceObserver (1×1 invisible NSView)
```

- **One main `CAMetalLayer` per display surface** for all world-space drawing (dim, region frame, handles, OCR rects, ruler lines/arcs, magnifier).
- **Plus N small `CAMetalLayer`s**, one per floating chrome element: each is the `contentLayer` of `ScreenwideOscMaterialSurfaceView`, an `NSVisualEffectView` subclass (`osc_material_surface_macos.m:12-41`, material `UnderWindowBackground`, blending `WithinWindow`). Chrome = system blur + Metal layer on top, `cornerRadius` masking. **Biggest porting hazard — no NSVisualEffectView on Windows.** Port decision: fold all chrome into the single swapchain and (later) blur the frozen snapshot texture ourselves.
- `framebufferOnly = NO` because the compute magnifier writes directly to the drawable (port decision: quad instead).
- Everything is _inside_ the webview's view/layer hierarchy — no separate overlay window for the anchor display.

### Multi-display (`+desktop.m`)

"Desktop" = union of all `NSScreen.frame`s (`desktop_frame`, `:9-14`). The anchor display uses the real Tauri window; every other screen gets a borderless `NSPanel` (`make_panel`, `:78-111`): `NSWindowStyleMaskNonactivatingPanel`, `CanJoinAllSpaces | FullScreenAuxiliary | Stationary`, `sharingType = NSWindowSharingNone` (excluded from capture), `level` copied from parent. Each panel's contentView is recursively attached as a **peer** (`rebuild_surfaces`, `:113-186`) sharing the _same_ `rustContext` (with `release = NULL` so only the root frees it).

Every peer carries `desktopOffset` = its screen origin in desktop-local coordinates (`local_origin`, `:16-19`, **y flipped**: the whole native side works top-left while AppKit is bottom-left). `screenwide_region_osc_surfaces()` returns `[root] + peers`; virtually every mutator loops over it.

`layout_matches` (`:50-62`) + `NSApplicationDidChangeScreenParametersNotification` (`:205-217`) drive rebuilds; the notification calls Rust `layoutChanged`, which restarts Ruler/OCR sessions (`state.rs:362-382`).

### Draw loop (`screenshot_region_osc_macos.m:39-174`)

No display link. Drawing is **event driven**: any state change calls `screenwide_region_osc_draw(s)`. Throttling via `drawInFlight`/`drawPending` + `[drawable addPresentedHandler:]` (`:158-171`) — at most one drawable in flight, so pointer sampling never blocks on the display server (load-bearing for latency). Animations: `dispatch_after(16ms)` self-rescheduling chains guarded by a monotonically increasing `revision` counter (`+ruler.m:577-587`, `+ocr_cancel.m:92-102`, `+ocr_toolbar.m:167-179`).

Frame body:

1. resize layer + `drawableSize = bounds * backingScaleFactor`
2. allocate CPU vertex array: `262 + ocr_vertex_capacity + ruler_vertex_capacity` (`:67-68`)
3. build geometry: snapshot quad (kind 33) → region shade+frame+handles → OCR → ruler
4. **pass 1**: clear-only render pass (clearColor 0)
5. **compute pass**: magnifier writes a rounded-rect lens into the drawable (only if `showFrame && magnifier.active && magnifierSource`)
6. **pass 2**: `LoadActionLoad`, one `newBufferWithBytes` VB, single `drawPrimitives:Triangle` for everything, `vertexCount = count`
7. `presentDrawable` + `commit`

## 2. The renderer (`region_osc_renderer_macos.m`, `region_osc_pipeline_macos.m`)

### Vertex format (`region_osc_renderer_macos.h:12-17`, static-asserted stride 24)

```c
struct ScreenwideRegionOscVertex {
  float2 position;  // ALREADY in NDC — see ndc() at renderer.m:32
  float2 uv;
  uint32 kind;      // the "shader program selector"
  uint32 padding;
};
```

No instance/uniform buffer, no MVP. `ndc()` (`:32-37`) converts top-left pixel coords to clip space on the CPU: `x → 2x/w - 1`, `y → 1 - 2y/h`. Everything expands to non-indexed triangle lists (6 verts/quad) on the CPU each frame. Ports to a `D3D11_USAGE_DYNAMIC` VB + `Map(WRITE_DISCARD)` + one `Draw()`.

### Geometry builders

| Function | renderer.m | Emits |
| --- | --- | --- |
| `add_quad` | :39 | 6 verts, uv 0..1 |
| `add_texture_quad` | :55 | 6 verts with explicit uv rect (glyph atlas cells, snapshot viewport) |
| `add_line` | :75 | quad expanded along a direction, **extended by half-width at both ends** (round caps from SDF) |
| `add_pattern_quad` (static) | :110 | quad whose uv encodes distance in units of 12pt/scale → marching ants |
| `add_circle` (static) | :143 | quad of `radius+margin`, circle in FS |
| `add_pill` (static) | :162 | 12×6 (or 6×12) quad, kind 16 |
| `add_selection_frame` (static) | :187 | two passes (halo kind 2 wide, line kind 0 thin) × 4 edges |
| `add_ruler_box` | :223 | hover halo quad (34/35) + 4 hairlines (28) |
| `add_ruler_arc_quad` / `add_ruler_arc` | :265/:303 | quarter-circle: uv is `(p-center)*sign/radius` so FS sees a unit-radius quadrant |
| `add_selection` | :321 | frame + 8 handles (circles corners, pills edges) + optional corner-radius handle |
| `add_crop_with_handles` | :355 | 4 dim rects (kind 6) + 4 marching-ant edges (kinds 8/10) + 8 handles |

Pixel snapping: `screenwide_region_osc_snap(v, scale) = (floor(v*scale)+0.5)/scale` (`:174`) for hairlines; `snap_handle_center` = `round(v*scale)/scale` for handle centers.

### Uniforms (`encode()`, `renderer.m:422-469`) — Metal push constants (`setFragmentBytes`)

| slot | contents | size |
| --- | --- | --- |
| b0 | `light_mode` (uint) | 4 |
| b1 | `magnifier_box` float4 (x,y,w,h px) | 16 |
| b2 | `action_fills` float[8] = `{primary_fill, secondary/foreground}` | 32 |
| b3 | `control_fill` + `control_outline` packed | 32 |
| b4 | `ocr_colors` float[32] (8 × float4, first 8 fields of `ScreenwideOscOcrPalette`) | 128 |
| b5 | `overlay_shade` float4 | 16 |
| b6 | `ruler_colors` float[8] = `{primary, info}` | 32 |
| b7 | `ruler_sample` float4 — picked pixel color (RGBA from `rulerColor` u32, `screenshot_region_osc_macos.m:143-146`) | 16 |
| b8 | `ruler_animation` float4 = `{copiedProgress, hoverAlpha, hoverWidthPx, toleranceProgress}` | 16 |

Port: ONE cbuffer, same field order, float4 rows.

Textures: t0 `label` (glyph atlas or single-string texture), t1 `secondary_label` (tolerance label), t2 `icons` (R8 6-cell atlas, device-cached, `osc_icon_renderer_macos.m:5-32`), t3 `snapshot`. Placeholder = 1×1 transparent RGBA (`renderer.m:16-30`) keeps the interface fully bound.

`ScreenwideRegionOscRenderState` (`renderer.h:38-49`) is the CPU aggregate; `screenwide_region_osc_render_state(light_mode)` (`pipeline.m:6-22`) builds it from **Rust-exported palette functions** (`src/osc/style.rs:41,60,103` etc.). All colors live in Rust — the port keeps this for free.

### Pipelines / blending (`region_osc_pipeline_macos.m:24-54`)

Two render pipelines, same shader pair, differing only in blend:

- normal: `srcRGB=SrcAlpha, dstRGB=InvSrcAlpha, srcA=SrcAlpha, dstA=InvSrcAlpha`
- **snapshot ("opaque composition")**: `srcA = One` — used when `snapshotComposited` so the composited frozen desktop yields opaque alpha. Selected per-frame at `screenshot_region_osc_macos.m:138-139`. One compute pipeline `region_magnifier`. Metal library compiled at runtime from source string (`:197-200`) — on D3D11: precompile via build.rs.

## 3. The shader — kind-by-kind (`region_osc_renderer_macos_shader.h`)

`region_osc_vertex_main` + `region_osc_fragment` (uber-shader: one big `if` ladder on `in.kind`) + `region_magnifier` (compute). Central trick: `fwidth(in.uv)` recovers pixel dimensions of the quad — `dimensions = 1/fwidth(uv)`. Works identically in HLSL.

### `region_magnifier` (kernel, :27-73)

Reads a `device uchar4*` **buffer** of desktop pixels, writes RGBA into the drawable at `[box_x,box_y]` with a 4px rounded-rect SDF mask; magnification = fixed **40px source window mapped across the box** (`:48-49`), nearest-neighbour so pixels stay crisp; out-of-source → dark gray `(0.15,0.15,0.16)`; `edges` bitmask (L/R/T/B) shades the corresponding half 10% toward black/white to indicate the dragged edge (`:62-70`); outer 1px ring (`distance > -1`) forced to the dark border color. Box is 96pt × scale (`region_magnifier_macos.m:34`), centered on an anchor snapped to the dragged edge (`screenwide_region_magnifier_anchor`, `:13-23`). **Port as a pixel-shader kind drawn last; drop the cutout.**

### Fragment kinds (evaluation order)

| kind | Element | Technique |
| --- | --- | --- |
| **33** | Frozen desktop snapshot | `snapshot.sample(uv)`; uv rect = zoomed viewport window (`screenshot_region_osc_macos.m:78-85`) |
| **34 / 35** | Ruler measurement-box halo (34 = hovered/animated width `ruler_animation.z`, 35 = static 3px) | rounded-rect _ring_ SDF from `fwidth`-derived dims; alpha × `ruler_animation.y` or fixed 0.32 |
| **39 / 40 / 41** | Corner-radius arcs: 39 solid, 40 hover halo, 41 low-confidence dashed | radial SDF `abs(len(local)-radius)-halfWidth` ∩ quadrant mask; endpoint caps via two point-distance mins; 41 adds `fmod(arclength, 7) < 4` dashes with `fwidth` AA |
| — | Magnifier cutout (:186-193) | inside lens rounded rect → `discard_fragment()` (drop in port — draw lens last) |
| **37** | Ruler tolerance label ("Balanced"/"Clear edges") | samples `secondary_label`, un-premultiplies `rgb/a`, alpha × `ruler_animation.w` |
| **11 / 15** | Text glyph quads (11 = `label` t0, 15 = `secondary_label`) | un-premultiply; kind 11 alpha × `(1 - ruler_animation.w)` (hex text cross-fades vs tolerance text) |
| **28** | Ruler primary hairline (crosshair, probe lines, ticks, measurement box edges) | flat `ruler.primary` |
| **42 / 43 / 44** | Centerlines: 42 normal (α 0.45), 43 aligned (α 0.85), 44 inner-object outline (α 0.30) | flat `ruler.primary` scaled |
| **36** | Guide / guide-gap line | flat `ruler.info` (blue) |
| **38** | Guide hover halo | `ruler.info` × `ruler_animation.y` |
| **31** | ruler primary at α 0.32 |  |
| **32** | Probe hover halo | `ruler.primary` × `ruler_animation.y` |
| **29** | Ruler color swatch (picked pixel) | 4px rounded rect SDF filled with `ruler_sample`, α × `(1-copied) × (1-tolerance)` |
| **30** | "Copied" checkmark | `actions.secondary` × `ruler_animation.x` × `(1-tolerance)`; check = two `add_line` quads scaled by animation (`+ruler.m:763-776`) |
| **22–26** | Icons from R8 atlas | `atlas_uv = ((kind-21)+uv.x)/6`; `.r` = coverage; tinted `actions.secondary`. `add_icon` emits `21 + icon` (`osc_icon_renderer_macos.m:34-43`); atlas from Rust `screenwide_osc_icon_atlas()` |
| **12 / 13 / 14** | Material-control fills (12 primary fill, 13 foreground/secondary, 14 primary) | deliberately rectangular, no radius — radius owner is the surface `cornerRadius` (comment :260-263) |
| **6** | Dim / mask outside region | flat `overlay_shade` (black α 0.48) |
| **17–20** | OCR rects: 17 text line, 18 QR, 19 QR error, 20 selection | 2px rounded rect SDF; fill/outline from `ocr` palette; `outline_mix = clamp(0.5 + (dist+w)/aa)` inner stroke; outline width 2 for 17/18 else 1 |
| **7–10** | Region frame marching ants (7/8 horizontal, 9/10 vertical) | capsule SDF repeated with `fract(longitudinal)*period`, `half_segment = 3`; outer capsule owns both fill and outline so AA sits at the exterior (comment :306-308) |
| **3 / 16** | Round handle (3) / pill handle (16) | circle SDF; 16 stretches to capsule along longer axis |
| **4 / 5** | Guides (hardcoded): 5 = blue `(0.055,0.647,0.914)` dark / `(0.008,0.518,0.780)` light, 4 = amber `(0.918,0.702,0.031)` | flat |
| **odd (default)** | bit0 set and not a guide → circular coverage | `1 - smoothstep(-aa, aa, len(p)-r)` |
| **≥2 (default)** | halo → `controls.outline`, else `controls.fill` | kinds 0 (thin line), 2 (halo) |

Porting notes: everything is procedural SDF + `fwidth`; only 4 textures. `discard_fragment()` → `discard`. Un-premultiply `rgb/a` before returning is required (sources premultiplied, blend straight `SrcAlpha`).

## 4. Rulers (`+ruler.m`, 2015 lines)

**No edge ruler bars with tick marks** — "Ruler" is a measurement/inspection tool. Two channels:

**(A) Main layer**, via `screenwide_region_osc_ruler_add_vertices` (`:1797-2015`), in order:

1. Crosshair — full-width/height 1px lines through `rulerPoint`, kind 28 (`:1802-1815`)
2. Probes — `NativeRulerProbe{axis, start, end, position}`: optional hover halo (32, width = animated `hoverWidth`), 1px span line (28), two end ticks perpendicular at `±spacing.control` (28) (`:1816-1863`)
3. Guides — full-span lines at `guide.position`, hover halo 38 + line 36 (`:1864-1902`)
4. Guide gaps — line + two ticks, kinds 38/36 (`:1903-1941`)
5. Corner radii — quarter arcs via `add_ruler_arc`, kinds 39/40/41 (`:1942-1964`)
6. Centerlines — v+h midlines of a detected box, 42 or 43 if aligned (`:1965-1981`)
7. Inner objects — 1px rect outline (44) + 2.5px center tick (43) per aligned axis (`:1982-2006`)
8. Measurements — `add_ruler_box` per rect: halo (34/35) + 4 hairlines (28) (`:2007-2014`)

Vertex budget: `screenwide_region_osc_ruler_vertex_capacity` (`:1196-1204`): `12 crosshair + 48/measurement + 24/probe + 12/guide + 24/gap + 12/radius + 12/centerline + 36/innerObject`.

**(B) Floating blurred chrome** — one material surface per label:

- `rulerSurface` — cursor readout loupe (`render`, `:654-828`): rounded background quad (12), color swatch (29) with sampled pixel, `#RRGGBB` hex text, optional second row `"W × H px"` from the two live probes, animated ✓ (30), tolerance text (37). Follows pointer with edge clamping (`layout`, `:624-652`). Hidden while `rulerInteractionActive` or hovering an artifact (`:656-661`).
- `rulerMeasurementLabelSurfaces`, `rulerProbeLabelSurfaces`, `rulerGuideGapLabelSurfaces`, `rulerRadiusLabelSurfaces` — pooled arrays grown/shrunk to match item counts (`render_measurement_labels` :1428, `render_probe_labels` :1634, `render_guide_gap_labels` :1686, `render_radius_labels` :1756), inserted below `rulerSurface`.

### Text

Pre-rasterized monospace atlas `screenwide_osc_mono_hex_atlas` (`osc_text_texture_macos.m:132-204`). Glyph set `"#0123456789ABCDEF× px≈"` — 22 fixed-width cells, 1px transparent gutter both sides so linear filtering can't bleed; `atlasGlyphUOffset = (gutter+0.5)/W`, `atlasGlyphUWidth = (glyphPx-1)/W`. Drawn with CoreText into a CGBitmapContext (sRGB, premultiplied-last), uploaded RGBA8. Fonts: **Roboto Mono** (mono) / **Inter** (proportional), registered from bundle resources, semibold, fallback system.

Emission: `glyph_index()` (`:149-159`), `glyph_texture_rect()` (`:161-166`), one `add_texture_quad(kind 11)` per char (`+ruler.m:738-746`, `:1367-1375`, `:1573-1581`). Non-atlas strings ("Balanced", "Cancel", "Copy all") use `screenwide_osc_text_texture` — whole-string texture, re-rasterized only when `scale` or `lightMode` changes.

Labels measured with `decimal_digit_count(desktopSize)` and `%*ld` width-padded formatting (`:461-476`, `:1262-1285`) so they don't resize as numbers change.

### Data flow Rust → native

Native **pulls** on every input result. `screenwide_region_osc_apply_ruler_result` (`:919-1071`): if `ruler_flags & 1`, calls the 8 pull FFI functions (count-then-fill convention: `fn(ctx, NULL, 0)` returns count). Results stored as byte blobs per surface; **change detection = byte equality**, with "labelled" projections that mask flags affecting only shape, so labels don't re-render when only geometry moved (`:993-1017`).

Hover state is smuggled in **`padding[0]` of each struct as 0-255 opacity** (`hovered_artifact_key`, `:851-896`); hovered artifact key = `(id << 3) | kindTag`.

`NativeOscResult.ruler_flags`: bit0 = is-ruler-result, bit1 = crosshair, bit2 = copied, bit3 = tolerance visible, bits4-5 = tolerance mode, bit6 = interaction active, bit7 = animation active (`:921-966`).

Viewport: per-display `{zoom, origin}` (`NativeRulerViewport`), applied by `project_measurement`/`project_probe`/`project_world_rect` (`:502-575`) as `(world - desktopOffset - origin) * zoom`; same transform feeds the snapshot quad's uv rect.

### Ruler input

Label hit-testing against the **AppKit frames of the material surfaces** (`screenwide_region_osc_ruler_label_hit`, `:44-109`) — walks the four label arrays, `NSPointInRect`; result `{id, kind(1=measurement,2=probe,3=gap,4=radius), center}`. Label interaction via `native_osc_ruler_label_input(ctx, operation, kind, id, pointerX, pointerY, centerX, centerY, out)`, operations `1=begin drag, 2=drag, 3=end, 4=cancel, 5=right-click label, 6=right-click empty, 7=hover` (`+input.m:193-238`, `:321-344`, `:667-672`). Viewport interaction: `native_osc_ruler_viewport_input(ctx, displayId, operation, anchorX, anchorY, deltaX, deltaY, out)`, `1=zoom, 2=pan, 3=reset (double-click)` (`+input.m:381-398`).

## 5. OCR panels

### Main layer (`+ocr.m:100-135`)

- Phase 1/2: 4-quad 1px border around the selection region, kind 18.
- One quad per `ScreenwideRegionOcrRect`, kind map: `1 Line→17`, `2 Qr→18`, `3 QrError→19`, `4 Selection→20` (`:127-129`).

`screenwide_region_osc_set_ocr(view, phase, rects, count, message)` (`:137-185`): loops all surfaces, translates each rect to surface-local by subtracting `desktopOffset`, **keeps only rects intersecting that surface**, appends into `surface.ocrRects` (packed 40-byte records). Picks `target` surface = largest overlap with `region`, for the status pill / toolbar. Full re-push of the rect list on every selection change; no incremental buffer. Colors from Rust `ocr_palette`.

### Status pill (`+ocr.m:40-66`)

Material surface + a real `NSTextField` (the only live AppKit text). Centered on region, 28pt tall, colors from `loading_fill`/`status_error_fill` (phase 3 = error). Port: fold into the GPU text path.

### Cancel button (`+ocr_cancel.m`)

One material surface with its own tiny render: background quad (12), icon 1, "Cancel" text texture (11) (`render`, `:31-90`). Centered horizontally, 48pt from top. Hit state/animation in **Rust** (`screenwide_osc_control_group_*`) — native only forwards `hover/down/up/clear` and re-renders on `update.changed`, self-scheduling 16ms frames while `update.animating`.

### Toolbar (`+ocr_toolbar.m`)

Four controls: `Copy all`, `Copy as paragraph`, `Recognize another area` (icon-only), `Close text recognition` (icon-only). Layout from Rust `screenwide_ocr_toolbar_layout(selection, viewport, widths, height, out, cap)` (`…_private.h:382-386`). Icons 2/3/4.

Close is a **two-stage confirm**: `screenwide_osc_confirm_*` (Rust state machine, `{idle_icon 1, armed_icon 5, colors 0/2, timeout 2000ms}`). While armed, `screenwide_osc_confirm_layers` returns up to 2 crossfading icon layers `{icon, foreground, opacity, scale}` drawn as extra draw calls with re-pushed `action_fills` (`:126-156`). 2s dispatch timeout expires armed state (`+ocr_toolbar_input.m:20-34`).

Activation → Rust as `input(ctx, 8 + update.activated, …)`; confirmed close = phase 12 (`+ocr_toolbar_input.m:57-64`); Cancel = phase 8 (`+ocr_cancel.m:161`).

## 6. Input handling (`+input.m`)

### Model: global local event monitor, not a responder chain

`screenwide_region_osc_input_install` (`:400-661`): `[NSEvent addLocalMonitorForEventsMatchingMask:]` for MouseMoved, L down/drag/up, RightMouseDown, OtherMouse down/drag/up, ScrollWheel, Magnify, KeyDown, KeyUp, FlagsChanged. Returning `nil` consumes; returning `event` passes to webview. All material surfaces override `hitTest:` → `nil` — visually present, transparent to AppKit hit-testing; hit-testing is manual in Rust control groups.

Every monitored surface sees every event; `processInput` bails if `event.window != s.host.window` (`:179-183`) — that's how the correct peer claims an event.

### Consumption rules

| Event | Consumed? |
| --- | --- |
| Mouse move/down/drag/up | passed through (`return event` :660) — native processes them AND the webview sees them. Except… |
| left double-click hitting ruler viewport reset | consumed (`:645-649`) |
| RightMouseDown handled by ruler labels | consumed (`:582-587`) |
| ScrollWheel (ctrl = zoom, else pan) | consumed if `processRulerViewportInput` handled (`:588-604`) |
| Magnify gesture | consumed (`:605-613`) |
| Middle-button pan | consumed (`:614-644`) |
| KeyDown mapping to a ruler phase | consumed (`:544-559`) |
| KeyUp of held range/guide/radius key | consumed (`:440-471`) |
| Cmd+A / Cmd+C during OCR phase 2 | processed but passed through (`:573-577`) |
| FlagsChanged (Option) | passed through, forwarded as phase 30 |

Keycodes (macOS virtual): 7=`X`→13, 48=Tab→14, 51/117=Delete→16, Cmd+8=17, Cmd+6=18/19(shift), Cmd+16=19, 17=`T`→29, 46=`M`→33, 18/19=`1`/`2`→20/21 (held: range), 9/4=`V`/`H`→26/27 (held: guide), 15=`R`→31 (held: radius). Held keys latch (`rulerRangeKeyCode` etc.) and fire release phase (22/28/32) on key-up (`:440-469`) and teardown (`:673-690`).

### `processInput` ordering (`:175-295`)

1. bail if disabled / wrong window; else `cursor_claim`
2. convert to view coords, flip y if `!host.isFlipped`
3. `screenwide_region_osc_ocr_control_input` → toolbar then cancel; if consumed, return
4. active ruler-label drag → `native_osc_ruler_label_input(2/3)`, return
5. hover/press on a ruler label → `(7)` or `(1)`, return
6. drag/up without an active gesture → release cursor, return (stale drags after a rejected press)
7. press dismisses the OCR cancel button; press inside `exclusionRect` returns (that rect is the webview's own toolbar, `+state.m:78-88`)
8. modifiers: `1=shift, 2=cmd|ctrl, 4=doubleClick, 8=option`
9. `s.input(ctx, phase, desktopX, desktopY, modifiers, &result)`; **`result.status == 255` = rejected** → release cursor
10. apply ruler result, track `gestureActive`, apply cursor, update magnifier, hide/show system cursor, apply region to all surfaces

Phases: `1=move, 2=down, 3=drag, 4=up, 5=cancel`.

### Cursor ownership (`:95-174`)

`cursor_claim` calls `[window disableCursorRects]` so WebKit can't race the native cursor; tracks a single `cursorOwner` on the root; re-enables/resets on the previous owner. `screenwide_set_region_expected_cursor()` records the expected cursor for a guard elsewhere. `claimPointerSurfaceNow` (`:141-157`) finds the surface whose window frame contains the mouse; guarded by a `cursorClaimGeneration` counter so deferred blocks go inert.

Cursor codes: 1 crosshair, 2 openHand, 3 closedHand, 4/5/6 resize (from `edgesForHandle`), 7 arrow, 8 IBeam, 9 pointingHand (`:27-59`). Handle→edge bitmask: N=4, S=8, E=2, W=1 (`:14-26`).

## 7. Desktop presentation & snapshot

### `set_desktop_presented` (`+desktop.m:247-271`)

"Presented" = the peer panels are ordered on screen. On `presented`: each peer copies parent `level`/`alphaValue`, `orderFrontRegardless`. On `!presented`: `orderOut:` + cursor claim cancel + cursor release for every surface. Window ordering only.

### Snapshot (`+snapshot.m`)

Frozen desktop screenshot behind the overlay. Rust pushes per-display RGBA8; surface selected by `displayID` (`:13-20`). Two mutually exclusive paths:

- **`snapshotComposited == YES`** (Ruler): upload to a texture and draw as kind 33 inside the GPU pass, with the viewport zoom/pan uv rect. Enables ruler zoom/pan of a frozen desktop; why the snapshot pipeline uses `srcAlpha = One`.
- **`snapshotComposited == NO`**: CGImage → `snapshotLayer.contents` — pure CoreAnimation. (Port: a DComp visual with a bitmap, or draw as an opaque quad.)

`set_snapshot_presented` (`:62-76`) toggles layer hidden (or redraw in composited mode). `set_snapshot_composited` (`:78-91`) switches modes, clears CALayer contents.

Magnifier source is separate: `+state.m:5-23` uploads RGBA into a shared buffer reused by every surface, consumed by the compute kernel (port: texture).

## 8. Platform-specific items with no direct Windows analogue

1. **NSVisualEffectView blur** → DWM backdrop (Win11-only), undocumented acrylic, or **self-blur of the frozen snapshot (recommended — desktop is frozen anyway); fold all chrome into the single swapchain**, eliminating ~10 extra swapchains.
2. **CAMetalLayer sublayer of webview NSView** → separate `WS_EX_NOREDIRECTIONBITMAP` child HWND + topmost DComp target above WebView2 (repo pattern exists).
3. **objc_setAssociatedObject** → `SetProp` on HWND or a side map keyed by HWND.
4. **NSEvent local monitor with return-nil-to-swallow** → overlay HWND takes input; explicit pass-through via `HTTRANSPARENT` / forwarding.
5. **disableCursorRects / NSCursor set/hide** → `WM_SETCURSOR` interception + `SetCursor` + `ShowCursor(FALSE)`.
6. **frameResizeCursorFromPosition / hand-drawn diagonal cursors** → `IDC_SIZENWSE`/`NESW`/`WE`/`NS` exist natively; `region_cursor_macos.m` mostly disappears.
7. **effectiveAppearance** → registry `AppsUseLightTheme` + `WM_SETTINGCHANGE`.
8. **NSPanel per-display overlays** (`CanJoinAllSpaces`, `Stationary`, `NonactivatingPanel`, `sharingType=None`) → `EnumDisplayMonitors` + `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` (+topmost as needed); `NSWindowSharingNone` ≈ `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`; screen-parameters notification → `WM_DISPLAYCHANGE`/`WM_DPICHANGED`.
9. **Coordinates**: AppKit bottom-left; ~15 y-flips in the code. Windows is top-left — delete every flip, but audit direction of each (some flip "into AppKit", some "out of").
10. **backingScaleFactor** uniform per window vs per-monitor DPI on Windows — keep scale per-surface; watch `snap` and drawable sizes.
11. **Colorspace**: bitmaps sRGB premultiplied-last; textures `RGBA8Unorm` (NOT `_sRGB`); drawable `BGRA8Unorm` — no hardware sRGB conversion anywhere. On DXGI use `B8G8R8A8_UNORM` (not `_SRGB`) to reproduce the same math.
12. **addPresentedHandler one-frame-in-flight throttle** → waitable swapchain object or non-blocking `Present(0,0)`; never block the input thread on the compositor (load-bearing for pointer latency).
13. **dispatch_after(16ms) self-rescheduling animation + CACurrentMediaTime** → `SetTimer` on the UI thread + `QueryPerformanceCounter`. No display link used anywhere.
14. **CATransaction setDisableActions** — suppresses implicit CA animations; Windows has none; delete.
15. **CoreText atlas rasterization** → GDI path (repo precedent in `label.rs`) or DirectWrite; the atlas layout (fixed cells, 1px gutter, texel-center uv) ports verbatim.
16. **Runtime Metal compilation** → build.rs `D3DCompile` (repo precedent).
17. **Compute writes into drawable (UAV)** → render the magnifier as a regular textured quad drawn last; drop the `discard` cutout.
18. **Accessibility attributes** on material surfaces → UI Automation providers on child HWNDs, or accept the regression.
19. **NSTextField status pill** — fold into the GPU text path for consistency.

## 9. Suggested port shape

- One D3D11 device + one flip-model swapchain per display surface; keep the peer-window model (`+desktop.m` maps to per-monitor HWNDs).
- Keep the CPU-side vertex builder verbatim — pure math on NDC floats. `region_osc_renderer_macos.m` is ~90% portable.
- Port the uber-shader kind-ladder 1:1 to HLSL (`fwidth`→`fwidth`, `discard_fragment()`→`discard`). Push constants → one cbuffer, same field order.
- All palettes/metrics/hit-testing/animation state machines already live in Rust and are platform-neutral — reuse unchanged.
- Biggest new work: (a) blur backdrop substitute, (b) compositor above WebView2, (c) event interception with pass-through semantics, (d) text atlas.
