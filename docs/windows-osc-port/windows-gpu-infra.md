<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Existing Windows GPU/windowing infrastructure (reuse map for the OSC port)

Research snapshot 2026-09-01. Paths relative to `src-tauri\`.

## 0. Executive summary

The repo already contains a complete, production D3D11 + DirectComposition compositor for the export preview (`surface_windows`), including a transparent SDF overlay pass (`SelectionOverlay`) that draws a selection frame with handles, snap guides, crop shade, and text labels — roughly 70% of what a region OSC compositor needs. The region OSC's Windows seam is a stub returning `Ok(false)` at `src/windows/screenshot_region/adapter/unavailable.rs`.

## 1. Preview compositor architecture

### 1.1 Device / DXGI / DirectComposition creation

`src/exports/preview_platform/surface_windows.rs:479-544` (`Gpu::new`) — canonical bootstrap, copy wholesale:

| Step | Line | Notes |
| --- | --- | --- |
| `D3D11CreateDevice` | `:483` | `D3D_DRIVER_TYPE_HARDWARE`, flags `BGRA_SUPPORT \| VIDEO_SUPPORT`, feature levels `[11_1, 11_0]` |
| `ID3D10Multithread::SetMultithreadProtected(true)` | `:498-499` | Required — device shared with MF + WGC callbacks on other threads |
| `device.cast::<IDXGIDevice>()` → `GetAdapter()` → `GetParent::<IDXGIFactory2>()` | `:500-503` | Factory via adapter parent |
| `DCompositionCreateDevice(&dxgi)` → `IDCompositionDevice` | `:504` |  |
| `composition.CreateTargetForHwnd(host, false)` | `:509` | non-topmost = visual tree renders BELOW the WebView2 child HWND |
| `CreateVisual()` + `target.SetRoot(&root)` | `:511-514` |  |
| `composition.CreateTargetForHwnd(editor, true)` | `:521` | **topmost** target on a separate child HWND, for overlay ABOVE WebView2 |
| `composition.Commit()` | `:530` |  |

Two-target trick (`:506-522`): one non-topmost target below the webview, one topmost target on a separate child HWND above it — the repo's answer to layering native content relative to the webview.

### 1.2 Swapchain model

Three near-identical creators, all `CreateSwapChainForComposition` (never ForHwnd):

- `Gpu::pane()` — `surface_windows.rs:546-626`
- `Backdrop::new()` — `surface_windows.rs:629-678`
- `SelectionOverlay::new()` — `surface_windows/selection.rs:113-199`

Common `DXGI_SWAP_CHAIN_DESC1` (`selection.rs:119-133`):

```
Width/Height: 2, 2            // created tiny, resized on first draw
Format:       DXGI_FORMAT_B8G8R8A8_UNORM
BufferCount:  2
Scaling:      DXGI_SCALING_STRETCH
SwapEffect:   DXGI_SWAP_EFFECT_FLIP_DISCARD
AlphaMode:    DXGI_ALPHA_MODE_PREMULTIPLIED   // overlay transparency
```

Cast to `IDXGISwapChain3` for `GetCurrentBackBufferIndex()`. Attach: `visual.SetContent(&swap_chain)` then `root.AddVisual(&visual, true/false, sibling)` (third arg = sibling reference for z-order, `:588-593`).

### 1.3 Present model

Per-frame, in `SelectionOverlay::draw` (`selection.rs:244-435`) — the exact loop a region OSC runs:

1. `ResizeBuffers` only when viewport size changed (`:260-272`).
2. `UpdateSubresource` the constant buffer (`:392-401`).
3. `GetCurrentBackBufferIndex()` → `GetBuffer::<ID3D11Texture2D>` → `CreateRenderTargetView` (`:402-409`).
4. `ClearRenderTargetView(&target, &[0.0;4])` — transparent clear, mandatory for flip-discard (`:411`).
5. Set RTV / viewport / TRIANGLELIST / VS / PS / CBs / SRVs / samplers, `Draw(3, 0)` (`:412-425`).
6. Unbind SRVs, `Present(0, DXGI_PRESENT(0))` (`:426-432`).

No vsync render thread, no persistent loop — fully event-driven (pointer messages, `WM_TIMER` 16ms for button animations via `SetTimer` at `editor.rs:181`). `IDCompositionDevice::Commit()` only when visual geometry changes (`surface_windows.rs:594, 2493, 2940, 3582, 3702`).

Batching: `begin_layout`/`finish_layout` with `batch_depth: AtomicU32` (`:2478-2484, 2766, 2924`) parks presents so sibling panes change in one pass. Preview swapchains resize with 256-px granularity headroom to avoid per-pointer-move `ResizeBuffers` stalls (`:2409-2425`).

### 1.4 Shader compilation — precompiled at build time

`build.rs:159-231`: `compile_windows_preview_shaders()` (`:160-170`) + `compile_shader()` (`:172-231`) using `D3DCompile` from `windows::Win32::Graphics::Direct3D::Fxc` as a build-dependency; entry points `vs_main`/`ps_main`, targets `vs_4_0`/`ps_4_0`, writes `{prefix}_vs.cso`/`{prefix}_ps.cso` into `OUT_DIR`; `cargo:rerun-if-changed` per shader (`:180`). Consumed via `include_bytes!(concat!(env!("OUT_DIR"), "/recording_selection_vs.cso"))` (`selection.rs:39-41`). Test asserts the `DXBC` magic (`compositor.rs:898-902`). **To add a shader: one `compile_shader(...)` call in build.rs.**

### 1.5 Constant buffer pattern

`#[repr(C)] struct Constants` of `[f32;4]` rows only (never scalars), `D3D11_USAGE_DEFAULT` + `BIND_CONSTANT_BUFFER`, updated by `UpdateSubresource` (not Map).

- Selection `Constants`: `selection.rs:43-57` — 11 float4s mirroring the cbuffer at `selection.hlsl:4-16`.
- Preview `Constants`: `compositor.rs:51-77`.
- `KeyboardConstants` at b1 uses parallel `uint4[8]`/`float4[8]` arrays (HLSL pads struct array elements to 16 bytes — `preview.hlsl:30-32`), with a packing test at `keyboard_artwork.rs:647`.

**No vertex buffer, no input layout, no instancing anywhere** — every existing pass is a fullscreen triangle from `SV_VertexID`:

```hlsl
float2 p = float2((id << 1) & 2, id & 2);
output.position = float4(p * float2(2,-2) + float2(-1,1), 0, 1);
```

(`selection.hlsl:25-30`). Geometry lives in the constant buffer, resolved analytically in the pixel shader. (The OSC port adds the repo's first dynamic VB — justified by ruler primitive counts.)

### 1.6 Text / glyph rendering

GDI rasterization → CPU bitmap → immutable D3D11 texture → single-channel coverage sampled in PS. No DirectWrite, no SDF font.

- `rasterize_label()` — `surface_windows/selection/label.rs:62-231`: `CreateCompatibleDC` → `CreateFontW` → `GetTextExtentPoint32W` → `CreateDIBSection` (top-down, 32bpp) → black fill → `SetBkMode(TRANSPARENT)` + `SetTextColor(0x00FFFFFF)` + `TextOutW`. **Red channel = glyph coverage.**
- Action labels rasterized at 2× and box-downsampled (`label.rs:69, 197-219`) — GDI grid fitting is coarse at 12px semibold.
- Bundled Inter variable font registered via `AddFontMemResourceEx` from `assets/Inter-VariableFont_opsz,wght.ttf` (`label.rs:41-55`); dimension readouts use Consolas (`label.rs:83`).
- Upload: `upload_label_texture()` — `selection/label_texture.rs:35-78`, `D3D11_USAGE_IMMUTABLE`, BGRA.
- Caching by `(text, scale_key, action)` (`selection.rs:204-241`, `label_scale_key` at `label.rs:37-39`).
- 1×1 transparent placeholder texture always bound so the SRV slot is never null (`selection.rs:100-103, 182`).
- Halo/outline computed in-shader: 8 ring taps at stroke radius, meaned, saturated (`selection.hlsl:183-190`).
- `keyboard_artwork.rs` (708 lines): second, larger GDI atlas — `measure` `:328`, `rasterize_keyboard` `:340`, `upload` `:513-534`.

## 2. What the two HLSL shaders draw

### `shaders/selection.hlsl` (208 lines) — direct template for the OSC shader

All analytic SDF/coverage in one `ps_main`; composited manually into straight color+alpha, premultiplied only at the final `return float4(color * alpha, alpha)` (`:207`).

- `circle_coverage` (`:32-38`) — AA ramp outside the radius so fills keep color to the rim.
- `line_coverage` (`:40-44`) — 1px core, one-pixel falloff.
- `rounded_distance` (`:46-50`) — rounded-box SDF.
- `composite_layer` (`:55-62`) — straight-alpha source-over.
- Selection frame: 1pt core inside 3pt halo, edges snapped to pixel centers (`:81-92`).
- 8 resize handles as circles with 1px outline ring + optional radius handle (`:103-119`), `[unroll]`.
- Dashed border in crop mode: `frac()` wave + `fwidth()` AA (`:93-102`).
- Crop shade: 0.4 black outside the frame (`:127-135`).
- Snap guides: 1px lines, canvas amber / object blue (`:142-150`).
- Magnifier cutout: early `return 0` inside a rounded box (`:75-77`).
- Button plates + text: rounded-rect coverage, glyph fill + ring-mean halo (`:155-206`).
- Theme via `viewport.z` (0 dark / 1 light) `lerp` (`:120-122`).
- Sizes in points × `label_params.y` = physical px/pt (`:84-86`).

### `shaders/preview.hlsl` (449 lines)

Offscreen composition (canvas + media): value-noise mesh gradients (`:44-60`), rounded coverage (`:61-70`), Gaussian shadow (`:71`), cursor `Texture2DArray` sampling with rotation/scale/motion blur (`:80-122`), camera layer (`:160`), keyboard-shortcut overlay with spring animation (`:183-365`). Bindings: t0 source, t1 cursor array, t2 camera, t3 keyboard; b0 canvas, b1 keyboard; s0 linear, s1 point.

## 3. Reusable building blocks

| Block | Location |
| --- | --- |
| Full D3D11 + DXGI + DComp bootstrap | `surface_windows.rs:479-544` |
| Multithread-protect device | `surface_windows.rs:498-499` |
| Composition swapchain + visual + attach | `selection.rs:113-145`; `surface_windows.rs:546-598` |
| Transparent overlay draw+present loop | `selection.rs:244-435` |
| Opaque backstop layer (clear-only present) | `surface_windows.rs:629-701` |
| Visual geometry: offset + scale + rect clip | `Pane::update_geometry` `:734-761`; `Backdrop::set_geometry` `:703-726` |
| Hide a visual (`SetOffsetX2(-100_000.0)`) | `:728-730, 763-765` |
| Linear bitmap interpolation on visuals (DComp defaults to nearest!) | `:585` |
| Pixel-alignment helpers (`scaled_edges`, `pixel_center`) | `surface_windows/window.rs:8-20` |
| Blend state (premultiplied over: ONE / INV_SRC_ALPHA) | `compositor.rs:417-437` |
| Linear + point samplers | `compositor.rs:406-416, 438-448`; `selection.rs:171-181` |
| Immutable texture from CPU RGBA/BGRA | `label_texture.rs:35-78`; `keyboard_artwork.rs:513-534` |
| Default-usage texture from CPU pixels | `compositor.screenshot_source` `compositor.rs:570-612` (uses `R8G8B8A8_UNORM`) |
| Empty GPU texture + SRV | `compositor.source` `compositor.rs:536-568` |
| GPU→GPU copy | `Compositor::copy_source` `compositor.rs:876-891` |
| GPU→CPU readback (staging + MAP_READ) | `readback_bgra` `surface_windows.rs:3316` |
| Texture2DArray atlas from N CPU buffers | `compositor.rs:472-509` |
| GDI text → coverage bitmap | `label.rs:62-231` |
| Bundled font registration | `label.rs:41-55` |
| GDI cursor/icon → premultiplied RGBA (black/white double-render alpha recovery) | `compositor.rs:163-320` |
| Constant-buffer create + UpdateSubresource | `selection.rs:156-170, 391-401` |
| Shared platform-neutral OSC control model (layout, hit-test, hover/press animation, light/dark) | `src/osc/controls/mod.rs:28-215` — already consumed by the Windows selection overlay at `selection.rs:333-357` |
| Snapping math | `surface_windows/snapping.rs:28-101` |

## 4. Desktop capture on Windows

Windows Graphics Capture (WGC), not DXGI duplication. `src/recording/platform_windows/capture.rs`:

- `CaptureTarget::{Monitor(u32), Window(u32)}` → `GraphicsCaptureItem` via `IGraphicsCaptureItemInterop` (`:36-48`).
- Separate `D3D11CreateDevice` (BGRA only) (`:60-77`), bridged to WinRT via `CreateDirect3D11DeviceFromDXGIDevice` (`:100-107`).
- `Direct3D11CaptureFramePool::CreateFreeThreaded(device, B8G8R8A8UIntNormalized, 3, size)` (`:108-114`) — frames arrive off the UI thread.
- `FrameArrived` unwraps `frame.Surface()` → `ID3D11Texture2D`, sends the texture zero-copy over `SyncSender`, drops on full (`:116-139`).
- `SetIsCursorCaptureEnabled(show_cursor)`, `SetIsBorderRequired(false)` (`:144-146`). Sizes forced even (`& !1`).
- Format produced: `DXGI_FORMAT_B8G8R8A8_UNORM` GPU textures.

Still screenshots: `xcap` with `wgc` feature → CPU `CapturedImage { rgba, width, height }` (`src/screenshots/`). `src/desktop_capture/` is platform-neutral planning only (region→per-display pieces, output sizing, frame timing).

## 5. Main-thread dispatch / message-loop patterns

**(a) Tauri `run_on_main_thread` + mpsc round-trip** with a same-thread fast path: `create_editor_on_owning_thread` — `surface_windows.rs:2352-2377`. Checks `GetWindowThreadProcessId(host) == GetCurrentThreadId()` and creates inline if already there; else wraps HWND in `struct HostHandle(HWND); unsafe impl Send` and channels back. Rationale at `:2347-2351`: Win32 queues a window's messages on its creating thread — an editor created on a worker thread is input-dead.

**(b) Dedicated thread owning its own message loop** — glide raw-input window: `src/glide/windows/input_window.rs:29-81`. `RegisterClassW` guarded by `OnceLock<u16>`, `CreateWindowExW` with `HWND_MESSAGE` parent, `RegisterRawInputDevices` `RIDEV_INPUTSINK`, `SetTimer(16ms)`, classic `GetMessageW` loop.

**(c) Async-posting Win32 calls** to avoid blocking the UI thread: `ShowWindowAsync` (`editor.rs:134-141`), `SWP_ASYNCWINDOWPOS` on every `SetWindowPos` (`editor.rs:143-164, 200-212`).

**Editor window creation (the template for an OSC overlay window)** — `editor.rs:92-128`:

- Class `CS_DBLCLKS`, `OnceLock` atom, `w!("ScreenwidePreviewEditor")`.
- `CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP, ..., WS_CHILD | WS_CLIPSIBLINGS, parent)`.
- **`WS_EX_NOREDIRECTIONBITMAP` is the key flag**: DComp owns 100% of the window content, enabling the topmost DComp target (`surface_windows.rs:517-522`).
- Z-order re-asserted to `HWND_TOP` on every move to stay above WebView2 (`editor.rs:124-126, 145-146`).

**Input handling** — `editor/input.rs`: `window_proc` `:36`, `WM_NCHITTEST`→`HTCLIENT` (`:48`), `WM_MOUSEACTIVATE`→`MA_NOACTIVATE` (`:49`), `SetCapture`/`ReleaseCapture` (`:60, 95, 99, 107`), `WM_CANCELMODE|WM_CAPTURECHANGED` (`:110`), `WM_MOUSEWHEEL` (`:114`), `WM_SETCURSOR` (`:132`), Alt via `GetKeyState(VK_MENU)` (`:26`). HWND→state routing: global registry `surface_index().by_editor: HashMap<isize, Arc<SurfaceInner>>` (`surface_windows.rs:446-471`, `handle_editor_input` `:1473`).

**Registries**: `PREVIEW_SURFACES: OnceLock<Mutex<HashMap<isize, Arc<OnceLock<Result<...>>>>>>` keyed by host HWND (`surface_windows.rs:442-459`), inner-OnceLock rationale at `:431-441` (creation round-trips to the event-loop thread and must not hold a lock that thread wants). `unsafe impl Send/Sync` for HWND-holding types at `:473-476`, `editor.rs:86-87`.

Other primitives: `WS_EX_LAYERED` + `SetLayeredWindowAttributes(LWA_ALPHA)` (`src/windows/platform.rs:437-459`), `raise_without_activation` with `HWND_TOPMOST` (`platform.rs:461+`), capture affinity `initialize_capture_affinity` (`platform.rs:427-429`).

## 6. windows-rs versions and features

`Cargo.toml`: `windows = "0.62.2"` (runtime deps and build-deps; build-deps enable only `Win32_Graphics_Direct3D` + `Win32_Graphics_Direct3D_Fxc`).

- Graphics enabled: `Win32_Graphics_Direct3D`, `_Fxc`, `Direct3D10`, `Direct3D11`, **`DirectComposition`**, `Dwm`, `Dxgi`, `Dxgi_Common`, `Gdi`.
- WinRT: `Graphics_Capture`, `Graphics_DirectX`, `Graphics_DirectX_Direct3D11`, `Graphics_Imaging`, `Media_Ocr`, `Storage_Streams`, `Win32_System_WinRT`, `_Direct3D11`, `_Graphics_Capture`.
- UI: `Win32_UI_HiDpi`, `Win32_UI_Input`, `_KeyboardAndMouse`, `_WindowsAndMessaging`, `_Accessibility`, `_Shell`.
- NOT enabled (would need adding): `Win32_Graphics_DirectWrite`, `Direct2D`, `Direct3D12`, WIC. Repo deliberately avoids DirectWrite/D2D in favor of GDI.
- Also: `wgpu 30.0.0` + `pollster` + `bytemuck` (offscreen mesh only, `src/screenshots/mesh_gpu.rs`), `nokhwa`, `wasapi`, `rdev`, `xcap`(+wgc).

## 7. Platform facade pattern

Consistent idiom: `adapter.rs` with `#[cfg]`-selected platform submodule + compile-time contract test coercing each function to an explicit `fn` pointer type.

- Region OSC: `src/windows/screenshot_region/adapter.rs:17-28`, contract at `:182-190`. Stub `unavailable.rs:19-54`.
- Rulers: `src/ruler/adapter.rs:10-19` — `install`/`present`/`set_screenshot_mode`/`show_interactive`/`close`/`available`, contract `:26-36`.
- OCR overlays: `src/text_recognition/adapter.rs:12-19` — `install`/`present`/`render`/`render_window`/`show_interactive`/`close`, contract `:35-49`. **The OCR engine itself is already implemented for Windows** (`platform_windows.rs`, `Media::Ocr::OcrEngine`) — only overlay rendering is missing.
- Preview surface (the implemented twin): `src/exports/preview_platform.rs:71-82`; module doc `:19-66` is an explicit porting guide.
- Prewarm precedent: `preview_platform::prewarm()` `:127-137` builds the D3D/DComp pipeline on a blocking thread while the window is hidden — worth mirroring.

## 8. Gaps / cautions

1. No DirectWrite/D2D — text via GDI path (2× supersample + downsample); rulers' many small labels may push this harder than the current two-button use case.
2. No instancing/VBs anywhere — the OSC port introduces the first dynamic VB (justified; rulers exceed constant-buffer + `[unroll]` capacity).
3. Constant buffers are float4-row only with hand-verified packing + test — follow and add an equivalent test.
4. `IDCompositionDevice` v1 only — `CreateVisual`, `CreateScaleTransform`, `CreateRectangleClip` in use; no `IDCompositionDevice2/3` APIs.
5. DComp defaults to nearest sampling — `SetBitmapInterpolationMode(LINEAR)` explicitly.
6. Flip-discard back buffers undefined after present — always clear.
7. `surface_windows` targets a single child HWND in one app window; the OSC needs one `IDCompositionTarget` per overlay HWND sharing one device — supported shape, not yet implemented. Per-monitor DPI helpers exist only in glide (`GetDpiForWindow`, `MonitorFromPoint`, `GetMonitorInfoW` at `glide/windows/target.rs:8-57`).
8. Device shared with MF + free-threaded WGC — reuse `RecordingPreviewSurface::device()` (`surface_windows.rs:2586-2588`) or create own with `SetMultithreadProtected(true)`. (Port decision: own device.)
