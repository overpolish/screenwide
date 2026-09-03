<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Rust ↔ native OSC contract (what a Windows implementation must satisfy)

Research snapshot 2026-09-01. Paths relative to `src-tauri\`.

Note for the Windows port: the §1g C export list exists only because macOS's native side is Obj-C. On Windows the native side is Rust — implement the §1a–1e `native::*` functions directly and skip the C ABI. The `OscResult`/`OcrRectPacket` layouts still matter because portable Rust constructs them.

---

# 1. The `native::` API surface (`native_osc_macos`) a Windows module must reimplement

**Module root:** `src\windows\screenshot_region\native_osc_macos.rs` (48 lines; re-export facade).

Re-exports (`native_osc_macos.rs:23-45`):

- `DesktopBinding`, `NativeOscResult` (= `osc::protocol::OscResult`), `Purpose`, `ResultStatus` from `crate::osc`
- `configure_desktop_window` ← `desktop::configure_window`
- `NATIVE_OSC_EVENT` = `osc::semantic::REGION_EVENT` = `"screenshot-region-osc"` (`:29`)
- `NATIVE_OSC_LAYOUT_EVENT` = `osc::semantic::DESKTOP_LAYOUT_EVENT` = `"screenshot-region-desktop-layout"` (`:30`)
- OCR: `reset_text_recognition_input`, `set_ocr_cancel_visible`, `set_ocr` (`:32-35`)
- everything else from `state.rs` (`:38-45`)

## 1a. Attachment / lifecycle (`state.rs`)

All take `view: *mut c_void` — on macOS the `NSView*` from `WebviewWindow::ns_view()`. On Windows the analogue is the HWND; the pointer is opaque to Rust, but it must be the key by which the native side stores and returns the Rust context pointer.

| fn | file:line | signature | semantics |
| --- | --- | --- | --- |
| `ensure_attached` | `state.rs:395` | `(view, window: WebviewWindow, width: f64, height: f64) -> bool` | Idempotent: returns true if a context already exists (`with_context(view,\|_\|())`), else creates `Purpose::Region` context. `width/height` are **logical points** of the monitor. |
| `ensure_text_recognition_attached` | `state.rs:400` | same args | Same but `Purpose::TextRecognition`. |
| `ensure_ruler_attached` | `state.rs:410` | same args | Same but `Purpose::Ruler`. |
| `attach` (private) | `state.rs:384` |  | `Box::into_raw(OscRuntime::new(window,w,h,purpose))` then `ffi::attach`. **The native side owns the Box** and must call the supplied `release` fn (`ffi::release_context`, `ffi.rs:108`) on teardown. |
| `with_context` | `state.rs:419` | `pub(super) fn <T>(view, impl FnOnce(&Context)->T) -> Option<T>` | Fetches the stored `*mut OscRuntime`; `None` when unattached. Every other function uses this as its "is the compositor attached" probe. |

**Threading:** every one of these is called from `app.run_on_main_thread(...)` closures in `adapter/macos.rs`, `text_recognition/native_overlay_macos.rs`, `ruler/native_overlay_macos.rs`. No internal main-thread assertion in Rust — the contract is enforced by callers, who synchronise back via `std::sync::mpsc::sync_channel(1)`.

## 1b. Geometry & presentation

| fn | file:line | signature | semantics |
| --- | --- | --- | --- |
| `set_committed` | `state.rs:424` | `(view, Option<Rect>) -> bool` | Sets the controller's committed rect (no compositor call). Fails while a gesture is active (`controller.rs:69`). |
| `clear_region` | `state.rs:438` | `(view) -> bool` | Clears controller committed → `None`, sets `scene.region = Rect::default()`, `scene.visible=false`, then submits zero geometry. Used before Quick Screenshot borrows the window. |
| `present_region` | `state.rs:455` | `(view, Option<Rect>) -> bool` | Writes `scene.region`/`visible=true` and submits geometry. `None` → `Rect::default()`. |
| `apply_region_scene` | `state.rs:532` | `(view, next: RegionScene) -> bool` | **The core submit.** See ordering below. |
| `restore_normal_region_scene` | `state.rs:515` | `(view) -> bool` | Reads `RegionSceneState::normal_presentation()` and re-applies it. |
| `region_scene` | `state.rs:479` | `(view) -> Option<RegionScene>` | Currently _presented_ scene. |
| `region_scene_request_base` | `state.rs:486` | `(view, RegionSceneOwner) -> Option<RegionScene>` | `Screenshot` → presented; other owners → retained `requested_normal`. |
| `reconcile_region_scene_request` | `state.rs:500` | `(view, RegionScene, RegionSceneOwner) -> Option<RegionScene>` | Delegates to `RegionSceneState::reconcile_request`; `None` means "stale owner, drop the update". |

`apply_region_scene` ordering contract (`state.rs:532-596`) — preserve exactly:

1. Reject if `next.overlay != overlay_palette()` (`:533`).
2. Under lock: read `previous = scene.presented()`, `scene.set_presented(next)`, store `allow_drawing` atomic, `controller.set_aspect(next.interaction.aspect)`.
3. Diffed native calls, only when the field changed: `set_show_frame`, `set_show_handles`, `set_input_enabled`, `set_exclusion_rect` (unwrap_or_default when `None`), `set_snapshot_presented`, `set_snapshot_composited`.
4. **Geometry is submitted before desktop peers** (`:578-587`): submit `(x,y,w,h,visible)`; early-return `false` if it fails.
5. Only then `set_desktop_presented` if it changed.

## 1c. Individual mutators (each mirrors the scene field _and_ calls the native side)

| fn | file:line | signature |
| --- | --- | --- |
| `configure_desktop` | `state.rs:598` | `(view, DesktopBinding, local: Option<Rect>) -> bool` — rebuilds `RegionController::new(binding.virtual_monitor(), global_committed(binding, local), None)` and stores the binding. Returns false if `binding.anchor()` is `None`. No native call. |
| `set_monitor` | `state.rs:618` | `(view, width, height) -> bool` — controller-only (non-desktop path). |
| `set_allow_drawing` | `state.rs:633` | `(view, bool) -> bool` — atomic + scene, no native call. |
| `set_aspect` | `state.rs:655` | `(view, Option<f64>) -> bool` — scene + controller, no native call. |
| `set_input_enabled` | `state.rs:672` | `(view, bool) -> bool` |
| `set_show_handles` | `state.rs:686` | `(view, bool) -> bool` |
| `set_show_frame` | `state.rs:700` | `(view, bool) -> bool` |
| `set_desktop_presented` | `state.rs:714` | `(view, bool) -> bool` |
| `set_snapshot` | `state.rs:752` | `(view, display_id: u32, rgba: &[u8], width: u32, height: u32) -> bool` — uploads a frozen per-display texture. No scene mirror. |
| `set_snapshot_presented` | `state.rs:774` | `(view, bool) -> bool` |
| `set_snapshot_composited` | `state.rs:788` | `(view, bool) -> bool` |
| `set_magnifier_source` | `state.rs:645` | `(view, rgba: &[u8], width: u32, height: u32) -> bool` — **buffer borrowed for the duration of the call only; copy it.** |
| `claim_pointer_surface` | `state.rs:728` | `(view) -> bool` — makes the native surface the pointer/cursor owner. |
| `refresh_ruler_pointer` | `state.rs:736` | `(view) -> bool` (Ruler only) |
| `set_ruler_transient_chrome` | `state.rs:744` | `(view, visible: bool) -> bool` (Ruler only) |

## 1d. OCR entry points (`native_osc_macos\ocr.rs`)

| fn | line | signature |
| --- | --- | --- |
| `set_ocr` | `ocr.rs:8` | `(view, phase: u32, rects: &[OcrRectPacket], message: &CStr) -> bool` — `phase` is `VisualPhase as u32`. |
| `set_cancel_visible` | `ocr.rs:23` | `(view, bool) -> bool` |
| `reset_input` | `ocr.rs:33` | `(view) -> bool` — **`Purpose::TextRecognition` only**; clears `completed` atomic and controller committed rect. |

## 1e. Desktop discovery (`native_osc_macos\desktop.rs`)

`configure_window(view: *mut c_void, anchor_id: u32) -> Result<DesktopBinding, String>` (`desktop.rs:17`).

- Fills up to `MAX_DISPLAYS = 16` `NativeDesktopDisplay` records via the native side, which also returns desktop `width`/`height`, a **resolved** anchor id (native may substitute if the requested display vanished) and a `layout_changed` flag.
- Errors: `"AppKit could not resolve a Region monitor after losing: {anchor_id}"` when the resolved anchor is not in the returned list; `"AppKit returned no valid desktop displays"` when empty/invalid size. (Adjust wording for Windows but keep both failure modes.)
- `global_committed(&DesktopBinding, Option<Rect>) -> Option<Rect>` (`desktop.rs:70`) = `binding.project_local(local)`.

## 1f. Callbacks _into_ Rust (the native side calls these)

Declared in `state.rs`, wired at attach time (`ffi.rs:114-124`). All wrap `catch_unwind` and tolerate null `context`/`out`.

- `native_osc_input(context, phase: u32, x: f64, y: f64, modifiers: u8, out: *mut OscResult)` — `state.rs:25`. The whole pointer/command pipeline.
- `native_osc_layout_changed(context)` — `state.rs:362`. Purpose-dependent: TextRecognition → `text_recognition::restart_after_topology_change`; Ruler → `ruler::restart_after_topology_change`; Region → emits `NATIVE_OSC_LAYOUT_EVENT` with `()` payload to its own webview.
- Ruler draw-data pulls (all `(context, output: *mut T, capacity: usize) -> usize`, returning the full count so the caller can size buffers; return 0 unless `purpose == Purpose::Ruler`): `native_osc_ruler_measurements` `:48`, `_viewports` `:78`, `_probes` `:108`, `_guides` `:138`, `_guide_gaps` `:168`, `_radii` `:198`, `_centerlines` `:228`, `_inner_objects` `:258`.
- `native_osc_ruler_viewport_input(context, display_id, operation: u32, anchor_x/y, delta_x/y, out) -> i32` `:288` (returns "handled").
- `native_osc_ruler_label_input(context, operation: u32, kind: u8, id: u64, pointer_x/y, label_center_x/y, out)` `:326`.

Also global, palette-only exports the shader layer reads: `screenwide_osc_overlay_palette` (`osc\style.rs:41`), `screenwide_osc_ruler_palette` (`:60`), `screenwide_osc_control_palette` (`:103`), `screenwide_osc_ocr_palette` (`:140`); plus the control/confirm state machines in `osc\controls\ffi.rs:119-262` and `osc\controls\confirm_ffi.rs:70-148` (`screenwide_osc_control_group_*`, `screenwide_osc_confirm_*`) which native chrome uses for buttons.

## 1g. The C export list macOS uses (Windows: skip the ABI, mirror the semantics)

```
screenwide_region_osc_attach(view, context, release, input, layout_changed) -> *mut c_void
screenwide_region_osc_context(view) -> *mut c_void
screenwide_region_osc_set(view, x, y, w, h, visible: i32) -> i32
screenwide_region_osc_set_magnifier_source(view, rgba: *const u8, len, w, h) -> i32
screenwide_region_osc_set_input_enabled(view, i32)
screenwide_region_osc_set_show_frame(view, i32)
screenwide_region_osc_set_show_handles(view, i32)
screenwide_region_osc_set_exclusion_rect(view, x, y, w, h)
screenwide_region_osc_configure_desktop(view, anchor_id, displays*, capacity,
    desktop_width*, desktop_height*, resolved_anchor_id*, layout_changed*) -> usize
screenwide_region_osc_set_desktop_presented(view, i32)
screenwide_region_osc_claim_pointer_surface(view)
screenwide_region_osc_ruler_refresh_pointer(view)
screenwide_region_osc_ruler_set_transient_chrome(view, i32)
screenwide_region_osc_set_snapshot(view, display_id, rgba*, len, w, h) -> i32
screenwide_region_osc_set_snapshot_presented(view, i32)
screenwide_region_osc_set_snapshot_composited(view, i32)
screenwide_region_osc_set_ocr(view, phase: u32, rects: *const OcrRectPacket, count, message: *const c_char) -> i32
screenwide_region_osc_ocr_set_cancel_visible(view, i32)
```

ABI asserts that must hold: `size_of::<OscResult>() == 48`, `offset_of!(OscResult, x) == 8` (`ffi.rs:11-12`, also `protocol.rs:166-167`); `size_of::<OcrRectPacket>() == 40`, `offset_of!(kind) == 32` (`ffi.rs:37-38`); `size_of::<OcrPalette>() == 192` (`style.rs:81`).

---

# 2. Portable data types crossing the boundary

## `RegionScene` — `src\osc\scene.rs:41-49`

```
region: Rect          // meaningful even while hidden (lifecycle restore)
visible: bool
chrome: RegionChrome
interaction: RegionInteraction
snapshot: SnapshotPresentation
desktop_presented: bool
overlay: OverlayPalette
```

`Default` (`scene.rs:145-165`): region default, `visible=false`, `chrome{frame_visible:true, handles_visible:true}`, `interaction{input_enabled:false, allow_drawing:true, aspect:None, exclusion_rect:None}`, `snapshot` default (both false), `desktop_presented=false`, `overlay=overlay_palette()`.

Sub-structs:

- `RegionChrome` `scene.rs:16` — `frame_visible: bool`, `handles_visible: bool`
- `RegionInteraction` `scene.rs:22` — `input_enabled: bool`, `allow_drawing: bool`, `aspect: Option<f64>`, `exclusion_rect: Option<Rect>`
- `SnapshotPresentation` `scene.rs:30` — `presented: bool`, `composited: bool`
- `OverlayPalette` `src\osc\style.rs:19` — `#[repr(C)] shade: [f32;4]`; `overlay_palette()` = `[0,0,0,0.48]` (`style.rs:32-38`)

## `RegionSceneOwner` state machine — `scene.rs:52-58`, `:136-143`, `:171-185`

Variants: `Normal` (default), `DormantNormal`, `Screenshot`, `RestoringNormal`.

- `accepts_drawing(allow_drawing)` (`:137`): `Screenshot` accepts **only** `allow_drawing == true`; all three Normal variants accept **only** `allow_drawing == false`. Quick Screenshot always draws, the recording editor never does.
- `RegionScene::reconcile_owner(owner) -> Option<Self>` (`:171`): `None` if the owner rejects the scene (stale update, silently dropped). `DormantNormal` forces `visible=false, input_enabled=false, desktop_presented=false`; every other owner sets `desktop_presented = visible`.

Owner selection lives in `src\windows\region.rs:23-33` (`screenshot_region_scene_owner`), driven by three atomics: `SCREENSHOT_REGION_SESSION` → `Screenshot`; else `SCREENSHOT_REGION_RESTORING` → `RestoringNormal`; else `!RECORDING_CONTROLS_VISIBLE` → `DormantNormal`; else `Normal`. `finish_screenshot_region_restore()` (`region.rs:35`) clears the restoring flag and is called from `adapter\macos.rs:66` once the restored scene actually presented.

## `RegionSceneState` — `scene.rs:67-134`

Two-scene container: `presented` + `requested_normal`. `Deref`/`DerefMut` target `presented` (`:82-94`).

- `presented()` `:97`
- `request_base(owner)` `:101` — `Screenshot` → presented, otherwise `requested_normal`
- `reconcile_request(requested, owner)` `:110` — rejects wrong-owner scenes; retains `requested_normal` for non-Screenshot owners; returns `requested.reconcile_owner(owner)`
- `normal_presentation()` `:124` — `requested_normal.reconcile_owner(Normal).expect(...)`
- `set_presented(scene)` `:131`

## `RegionSceneRequest` — `adapter.rs:31-43`

Built in `osc_command.rs:66-78`:

```
rect: Rect, visible: bool, aspect: Option<f64>, input_enabled: bool,
exclusion_rect: Option<Rect>, show_frame: bool, show_handles: bool,
allow_drawing: bool, monitor_width: f64, monitor_height: f64,
desktop_anchor: Option<u32>
```

`RegionSceneResolution` (`adapter.rs:45-49`): `scene: RegionScene`, `controller_committed: Option<Rect>`, `layout_event: Option<SemanticEvent>`. `resolve_region_scene(request, base_scene, Option<&DesktopBinding>)` (`adapter.rs:51-91`) is fully portable and reused by any platform.

## `DesktopBinding` — `src\osc\desktop.rs:21-26`

```
displays: Vec<DesktopDisplay>, anchor_id: u32, size: Size, layout_changed: bool
```

`DesktopDisplay` (`desktop.rs:11-16`): `id: u32, origin: Point, size: Size, scale: f64`; `valid()` `:79`, `logical_monitor()` `:88`. Methods: `anchor()` `:29`, `virtual_monitor()` `:37`, `project_local()` `:41`, `reconcile_local()` `:45` → `DesktopRegion`, `display_at(Point) -> Option<u32>` `:56`. `DesktopRegion` (`desktop.rs:71-76`): `anchor_local: Rect, owner_local: Rect, global: Rect, owner_id: u32`. Free functions all portable: `global_region` `:98`, `local_projection` `:114` (deliberately _unclamped_ so one continuous frame can span monitors), `overlap_area` `:127`, `owner_for_region` `:137` (ties retain the current owner), `nearest_display` `:159`, `reconcile_region` `:202`.

Native display record: `NativeDesktopDisplay` `#[repr(C)] { id: u32, x, y, width, height, scale: f64 }` (`ffi.rs:26-33`).

## Semantic events — `src\osc\semantic.rs`

- `SemanticStatus` `:19` — `Changed | Finished | Cancelled | Layout`, serde lowercase
- `SemanticGesture` `:28` — `Drawing | Moving | Resizing { handle: SemanticHandle }`
- `SemanticHandle` `:37` — `#[repr(u8)]` `Body=1, North=2, South=3, East=4, West=5, NorthEast=6, NorthWest=7, SouthEast=8, SouthWest=9`, serde lowercase (`"northeast"` etc.)
- `SemanticRegion` `:50` — `x, y, width, height: f64`
- `SemanticEvent` `:59` — `#[serde(rename_all="camelCase")] { status, gesture: Option<SemanticGesture>, region: Option<SemanticRegion>, monitor_id: Option<u32> }`
- `semantic_handle(Handle)` `:75`, `event_payload(&ControllerEvent, Option<u32>)` `:99`

## Protocol / geometry

- `Purpose` `protocol.rs:11` — `Region | Ruler | TextRecognition`
- `InputPhase` `protocol.rs:19-53` — `#[repr(u32)]` 1..=33; 1-5 pointer, 6-12 OCR commands, 13-33 Ruler commands. `from_raw` `:56`, `pointer()` `:95`.
- `InputModifiers::from_bits(u8)` `:109` — bit0 free_aspect(shift), bit1 additive(cmd/ctrl), bit2 double_click, bit3 option/alt
- `CursorIcon` `:121` — `#[repr(u8)]` `Unchanged=0, Crosshair=1, OpenHand=2, ClosedHand=3, HorizontalResize=4, VerticalResize=5, DiagonalResize=6, Arrow=7, IBeam=8, PointingHand=9`
- `ResultStatus` `:137` — `None=0, Changed=1, Finished=2, Cancelled=3, Invalid=255`
- `RESULT_GESTURE_DRAWING=1 / MOVING=2 / RESIZING=3` `:145-147`
- `OscResult` `#[repr(C)]` `:151-164` — `status,gesture,handle,cursor,has_region: u8` then `x,y,width,height: f64`, `ruler_color: u32`, `ruler_flags: u8`, `ruler_padding: [u8;3]`
- `Point/Size/Rect/Monitor/Handle` — `src\osc\geometry.rs:8-107`; `Rect::committed()` = valid && w>1 && h>1 (`:60`); `clamp` `:69`, `snap` `:83`, `drawn_region` `:109`.

## OCR visual types — `src\text_recognition\visual.rs`

- `VisualPhase` `:13` — `#[repr(u32)]` `Idle=0, Loading=1, Ready=2, Error=3`
- `VisualKind` `:23` — `#[repr(u8)]` `Line=1, Qr=2, QrError=3, Selection=4`
- `VisualRect` `:31` — `{ rect: Rect, kind: VisualKind }`
- `VisualSnapshot` `:37` — `{ selection: Rect, rects: Vec<VisualRect> }`
- `OcrRectPacket` `#[repr(C)]` `:44-51` — `x,y,width,height: f64, kind: u8, padding: [u8;7]` (40 bytes)
- `SurfacePresentation` `:67-72` — `frame: Option<bool>, input: Option<bool>, reset: bool, claim_crosshair: bool`
- `RenderPacket` `:75-80` — `phase, rects: Vec<OcrRectPacket>, message: String, presentation`; constructors `loading()` `:83` (frame=false,input=false), `ready(&VisualSnapshot)` `:96` (frame=false,input=true), `error()` `:109` (frame=true,input=true,reset=true,claim_crosshair=true).
- `snapshot(selection, &TextRecognitionResult, &[TextRect]) -> VisualSnapshot` `:124` — projects normalized OCR rects into desktop-space, dropping non-`committed()` rects.

---

# 3. OCR → compositor call chain

```
shortcut / cmd  → text_recognition::start                       (text_recognition.rs:127)
  → start_session                                                (:139)
      capture per-monitor snapshots → adapter::install(window, anchor_id, &[(display_id, CapturedImage)])   (:193)
        → text_recognition\adapter\macos.rs:8 → native_overlay_macos::install  (native_overlay_macos.rs:12)
            run_on_main_thread:
              native::ensure_text_recognition_attached(view, window, w/scale, h/scale)   (:30)
              native::configure_desktop_window(view, anchor_id)                           (:38)
              native::configure_desktop / set_allow_drawing(true) / set_aspect(None)
                / set_show_frame(true) / set_show_handles(false) / set_input_enabled(true)
                / set_ocr_cancel_visible(true)                                            (:39-46)
              for each display: native::set_snapshot(view, display_id, rgba, w, h)         (:50)
              native::set_snapshot_presented(true); native::present_region(view, None)     (:60-61)
      → adapter::show_interactive → makeKeyAndOrderFront                                   (:194)
      → adapter::present → set_desktop_presented(true) + claim_pointer_surface             (:195)
```

Interaction loop, driven entirely by `native_osc_input`:

```
native_osc_input  (state.rs:25)
  → OscRuntime::input (runtime.rs:64)
      purpose==TextRecognition && InputPhase::Down → qr_details::hide_without_resume (:87)
      dispatch_control(window, phase)  (input.rs:36)  — OCR command phases 8..12
      if completed:  native_text_input → input::dispatch_text_input (input.rs:135)
           state.text_input(action, point) → update.snapshot
           → adapter::render_window(window, RenderPacket::ready(&snapshot))   (input.rs:153)
      else: dispatch_region(...) → ControllerEvent::Finished{committed:Some}
           → completed=true (runtime.rs:141)
           → dispatch_event → text_recognition::native_selection_finished     (runtime.rs:267)
```

`selection_finished` (`input.rs:94`):

1. `adapter::render_window(&window, RenderPacket::loading("Finding text and QR codes…"))` — `:100`
2. async: `select_desktop_region` (`snapshot.rs:107`, composes the frozen monitor crops across displays), on error `adapter::show_error` → `RenderPacket::error` (`adapter.rs:31`)
3. `recognize_current` (`text_recognition.rs:246`) → `platform_macos/platform_windows::recognize` + `qr::recognize`; on success `install_result` then `adapter::show_ready(app, generation)` (`adapter.rs:21`) which pulls `visual_snapshot(generation)` and renders `RenderPacket::ready`.

Final leg into the compositor — `native_overlay_macos.rs:149 apply_packet`:

1. `CString` of `packet.message` with NULs replaced by spaces
2. `native::set_ocr(view, packet.phase as u32, &packet.rects, &message)` — bails on false
3. `presentation.frame` → `set_show_frame`
4. `presentation.reset` → `reset_text_recognition_input`
5. `presentation.input` → `set_input_enabled`
6. `presentation.claim_crosshair` → `claim_pointer_surface`

`render` (`:170`) hops to the main thread and applies to the first window that accepts; `render_window` (`:143`) applies directly with no thread hop (already on the UI thread from the input callback).

Teardown `close_windows` (`:103`): `set_input_enabled(false)`, `set_ocr(Idle, &[], "")`, `set_snapshot_presented(false)`, `clear_region`, `set_desktop_presented(false)`, then `window.close()` — native surfaces must be concealed **before** the webview closes (comment at `text_recognition.rs:100-102`).

`text_recognition\adapter\unavailable.rs` (Windows today) makes `install` return `Ok(false)` and `render`/`render_window` no-ops, so `text_recognition.rs:194` falls through to `crate::windows::show(&window, true)` — the webview-only path.

---

# 4. Magnifier and desktop anchor / desktop windows

## Magnifier

Purely a texture handoff — Rust computes nothing about the loupe.

- Frontend `src\features\recording-sources\api.ts:140` → `prepare_screenshot_region_magnifier(app, window, monitor_id)` (`magnifier.rs:7`, registered `lib.rs:191`).
- `monitor_capture::capture_monitor_screenshot(app, monitor_id).await` → `screenshots::CapturedImage { rgba: Vec<u8>, width: u32, height: u32 }` — full monitor RGBA8.
- `adapter::set_magnifier_source(&target, image)` → main thread → `native::set_magnifier_source(view, &image.rgba, image.width, image.height)`.
- Buffer borrowed for the duration of the call only — copy into a texture before returning.

## Desktop anchor / desktop windows

- Frontend passes `desktop: bool` + `monitor_id` (`osc_command.rs:40-48`); `desktop_anchor = Some(monitor_id)` when `desktop` is set; error to request desktop mode without a monitor id.
- `adapter\macos.rs:32-48`, in order on the main thread:
  1. `native::ensure_attached(view, target, monitor_width, monitor_height)`
  2. if `desktop_anchor`: `native::configure_desktop_window(view, anchor)` → `DesktopBinding`
  3. `owner = region::screenshot_region_scene_owner()`; `base = native::region_scene_request_base(view, owner)`
  4. `resolve_region_scene(request, base, binding.as_ref())` — portable
  5. desktop path → `native::configure_desktop(view, binding, resolved.controller_committed)`; non-desktop path → `native::set_monitor(view, w, h)`
  6. non-desktop + visible → `native::set_committed(view, Some(rect))`
  7. `native::reconcile_region_scene_request(view, resolved.scene, owner)` — `None` ⇒ stale, return `Ok(true)` without touching the compositor
  8. `native::apply_region_scene(view, scene)`
  9. if presented && visible && allow_drawing && input_enabled → `native::claim_pointer_surface`
  10. if presented && owner == `RestoringNormal` → `region::finish_screenshot_region_restore()`
  11. if presented && `resolved.layout_event` → `emit_to(webview_window(label), NATIVE_OSC_EVENT, payload)`
- "Desktop windows" are compositor peer surfaces, one per display, not Tauri windows — Rust only toggles them as a unit via `set_desktop_presented`. Gating logic in `adapter\macos.rs:85-117`: restore the normal scene first when `presented && owner == Normal`, refuse to present unless the scene is `visible`, owner-consistent (`owner.accepts_drawing`) and (during a screenshot session) has no handles, then present and optionally claim the pointer.
- Region cross-display projection: `DesktopBinding::reconcile_local` gives `anchor_local` (controller/persistence), `owner_local` (frontend `SemanticEvent`) and `global` (compositor `RegionScene.region`).
- Quick Screenshot transitions: `prepare_for_screenshot` (`adapter\macos.rs:139`) clears the region _then_ hides desktop peers (order is load-bearing); `prepare_for_region_restore` (`:119`) only sets `input_enabled=false` and re-applies, retaining the last visual frame until the normal scene atomically replaces it.

---

# 5. Platform gating — what compiles on Windows today

## macOS-only modules (not compiled on Windows)

- `src\windows\screenshot_region.rs:6-7` — `native_osc_macos` (and submodules)
- `src\windows\screenshot_region\adapter.rs:17-22` — `adapter/macos.rs` vs `adapter/unavailable.rs`
- `src\text_recognition.rs:14-18` — `native_overlay_macos`, `platform_macos`; `platform_windows` is windows-gated
- `src\text_recognition\adapter.rs:12-17` — macos vs unavailable
- `src\ruler.rs:12-13` — `native_overlay_macos`; `src\ruler\adapter.rs:10-15` — macos vs unavailable
- `src\osc\cursor.rs:18-19` — `cursor::macos`
- Call sites in `src\windows\region.rs` gated at `:5, 69, 73, 87, 93, 96, 106, 217, 223, 230, 251, 253, 294, 313`

## Fully portable (compiles on Windows now)

The **entire `src\osc\` tree** except `cursor::macos`: `controller.rs`, `controls/` (incl. the `#[no_mangle]` C exports), `desktop.rs`, `geometry.rs`, `gesture.rs`, `protocol.rs`, `resize.rs`, `runtime.rs` + `runtime/desktop.rs`, `scene.rs`, `semantic.rs`, `session.rs`, `style.rs`. Also portable: `screenshot_region\adapter.rs` (`resolve_region_scene`, `RegionSceneRequest`), `osc_command.rs`, `presentation.rs`, `magnifier.rs`, `text_recognition\visual.rs`, `input.rs`, `interaction.rs`, `snapshot.rs`, `text_selection.rs`, `toolbar.rs`, `qr*.rs`, the whole `ruler\` document/analysis/render layer.

The ~300 dead-code warnings on Windows are the portable OSC engine with its only consumer compiled out — a fairly accurate inventory of the contract to satisfy. A Windows `native_osc_windows` module wired into `screenshot_region.rs` and `adapter.rs` reactivates essentially all of it.

---

# 6. Events emitted back to the webview

Both emitted with `emit_to(EventTarget::webview_window(label), ...)`, targeted at the emitting window only.

## `NATIVE_OSC_EVENT` = `"screenshot-region-osc"` (`semantic.rs:14`)

Payload = `SemanticEvent`, camelCase JSON:

```jsonc
{ "status": "changed" | "finished" | "cancelled" | "layout",
  "gesture": null | "drawing" | "moving" | { "resizing": { "handle": "body"|"north"|…|"southwest" } },
  "region": null | { "x": f64, "y": f64, "width": f64, "height": f64 },
  "monitorId": null | u32 }
```

Emission sites:

- `runtime.rs:236-242 emit_region_event` — from `dispatch_event` for `Purpose::Region` (`:251`), and from the `Down`-with-no-event path (`:145-152`) that re-broadcasts the current committed rect.
- `adapter\macos.rs:71-75` — the desktop **layout** event synthesised by `resolve_region_scene` (`adapter.rs:73-85`), with `status: "layout"`, `region` = the region in the _new owner's_ local space, `monitorId` = the new owner display.

Region coordinates are always **display-local for the reported `monitorId`** (`runtime\desktop.rs:10-47` `project_desktop_event`, which also mutates `binding.anchor_id` to follow the region across a seam).

Frontend consumer: `src\features\region-selector\use-native-screenshot-region.ts:95-135` — maps `status`/`gesture` onto `onRegionChange`, `onGesture`, `onMonitorChange`, `onFinished`, `onReconciled` (the last only for `"layout"`).

## `NATIVE_OSC_LAYOUT_EVENT` = `"screenshot-region-desktop-layout"` (`semantic.rs:15`)

Payload: `()` (unit). Emitted from `native_osc_layout_changed` (`state.rs:376-380`) only for `Purpose::Region`; TextRecognition and Ruler instead restart their sessions in Rust (`state.rs:368-375`). Frontend consumer at `use-native-screenshot-region.ts:140-146` bumps a layout revision when in desktop mode.

## Non-OSC events triggered by the same paths (context)

- `capture_overlays::emit_lifecycle(app, bool)` from `text_recognition::dismiss` / `start_session` (`text_recognition.rs:123, 200`) and the ruler equivalent.
- No event is emitted for OCR render results — the OCR overlay is entirely native; the webview only handles QR details via commands (`get_qr_details`, `close_qr_details`).

---

## Summary of the minimum Windows contract

1. Mirror `native_osc_macos`'s ~30 Rust-side functions so that `screenshot_region\adapter\windows.rs`, `text_recognition\native_overlay_windows.rs` and `ruler\native_overlay_windows.rs` can be written as line-for-line ports of the macOS files.
2. Call `native_osc_input` (or `OscRuntime::input` directly) for every pointer event and command phase, and the layout-changed path on display topology change.
3. Preserve `apply_region_scene`'s diff-and-order discipline (geometry before desktop peers; clear before hide on teardown).
4. Keep all policy in the portable `osc` code; the native module owns only surfaces, textures, GPU submission, pointer/cursor claim and window ordering.
