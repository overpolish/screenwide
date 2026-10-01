// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Region OSC surfaces. The anchor display draws into a
//! `WS_EX_NOREDIRECTIONBITMAP` child of the Tauri window; every other display
//! gets a `WS_POPUP` peer, the Win32 twin of the macOS `NSPanel` peers. All of
//! them share one [`Gpu`] - the app's wgpu device and one pipeline - and own
//! only their window, composition tree and swap chain.
//!
//! Port of `screenshot_region_osc_macos.m`'s attach and master frame draw
//! (`:39-174`), `+desktop.m` and `+snapshot.m`, minus OCR and ruler.

#[path = "surface/content.rs"]
mod content;
#[path = "surface/creation.rs"]
mod creation;
#[path = "surface/geometry.rs"]
mod geometry;
#[path = "surface/gpu.rs"]
mod gpu;
#[path = "surface/render.rs"]
mod render;
#[path = "surface/submit.rs"]
mod submit;
#[path = "surface/textures.rs"]
mod textures;
#[path = "surface/window.rs"]
mod window;
use crate::osc::gpu::windows::{bind_group_layout, pipeline, sampler, shader_module};
pub(crate) use crate::windows::overlay_surface::set_capture_affinity;
use crate::windows::overlay_surface::{self, disable_transitions, Frame, GpuSurface};
use textures::upload_icons;
pub(super) use textures::upload_rgba;
use window::light_mode;
pub(crate) use window::{create_on_owning_thread, set_pointer_passthrough};

use std::ffi::c_void;
use std::sync::{Arc, OnceLock};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{COLORREF, ERROR_SUCCESS, HINSTANCE, HWND, POINT, RECT},
    Graphics::Gdi::ScreenToClient,
    System::{
      LibraryLoader::GetModuleHandleW,
      Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
    },
    UI::{
      HiDpi::GetDpiForWindow,
      WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetClientRect, GetCursorPos, GetWindowLongPtrW,
        GetWindowRect, KillTimer, LoadCursorW, SetCursor, SetLayeredWindowAttributes, SetTimer,
        SetWindowLongPtrW, SetWindowPos, ShowWindowAsync, CS_DBLCLKS, GWL_EXSTYLE, HMENU, HWND_TOP,
        HWND_TOPMOST, IDC_CROSS, LWA_ALPHA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
        SW_HIDE, SW_SHOWNOACTIVATE, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TRANSPARENT, WS_POPUP,
      },
    },
  },
};

use super::input;
use super::ocr::{self, Segment};
use super::renderer::{self, RenderConstants, Vertex};
use super::ruler;
use crate::osc::geometry::{Point, Rect, Size};

/// Windows reports DPI relative to this baseline.
const BASE_DPI: f64 = 96.0;

static OVERLAY_CLASS: OnceLock<u16> = OnceLock::new();
static PEER_CLASS: OnceLock<u16> = OnceLock::new();

/// The lens anchor and dragged-edge bitmask. The sampling window and box size
/// are fixed, so a frame can rebuild the constants from these two values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MagnifierAnchor {
  pub point: Point,
  pub edges: u32,
}

struct Texture {
  view: wgpu::TextureView,
  size: (u32, u32),
}

/// Everything shared by the anchor surface and its peers, so a peer costs one
/// window plus one swap chain. macOS shared the `MTLDevice` the same way.
pub(crate) struct Gpu {
  shared: &'static crate::gpu::Gpu,
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  /// The frozen-desktop variant: `srcA = ONE`, preserving opaque target alpha
  /// while translucent chrome is drawn above the snapshot.
  opaque_pipeline: wgpu::RenderPipeline,
  linear_sampler: wgpu::Sampler,
  point_sampler: wgpu::Sampler,
  /// Every texture slot is bound each draw; unused ones read one transparent
  /// texel.
  placeholder: wgpu::TextureView,
  /// The shared control-icon atlas, bound for every frame.
  icons: wgpu::TextureView,
}

impl Gpu {
  pub(super) fn device(&self) -> &'static crate::gpu::Gpu {
    self.shared
  }
}

// Surfaces are reached through the context registry from the UI thread; the
// window handles they own are process-wide tokens guarded by the context
// mutex.
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

/// Where a surface takes its geometry and DPI from.
enum Kind {
  /// The anchor display: a child of the Tauri window, sized to its client area.
  Root { host: HWND },
  /// One non-anchor display: a top-level popup covering its monitor. Its scale
  /// comes from the binding because a peer may sit on a different-DPI monitor.
  Peer { bounds: Rect, scale: f64 },
}

pub(crate) struct Surface {
  gpu: Arc<Gpu>,
  kind: Kind,
  hwnd: HWND,
  chain: GpuSurface,
  vertex_buffer: Option<wgpu::Buffer>,
  vertex_capacity: usize,
  /// One constants block per draw call, read at a dynamic offset.
  constants_buffer: Option<wgpu::Buffer>,
  constants_capacity: usize,
  vertices: Vec<Vertex>,
  magnifier_source: Option<Texture>,
  snapshot: Option<Texture>,

  // Presentation mirror - the fields `+state.m` kept on the ObjC object.
  pub(crate) display_id: u32,
  pub(crate) region: Rect,
  pub(crate) visible: bool,
  pub(crate) show_frame: bool,
  pub(crate) show_handles: bool,
  pub(crate) input_enabled: bool,
  pub(crate) exclusion_rect: Rect,
  pub(crate) magnifier: Option<MagnifierAnchor>,
  pub(crate) snapshot_presented: bool,
  pub(crate) snapshot_composited: bool,
  /// Peers are only ordered on screen while the desktop is presented; the root
  /// ignores this, its window is the webview's own.
  pub(crate) desktop_presented: bool,
  /// This surface's origin in the desktop plane. `region` is desktop-global, so
  /// drawing subtracts the offset and input adds it - the macOS
  /// `desktopOffset`.
  desktop_offset: Point,
  pub(crate) gesture_active: bool,
  pub(crate) cursor: input::CursorShape,
  /// The OCR overlay this surface hosts: highlights plus the chrome that used
  /// to live in its own material surfaces.
  pub(crate) ocr: ocr::Chrome,
  /// The Ruler overlay this surface hosts: world artifacts, the pooled labels
  /// and the cursor readout.
  pub(crate) ruler: ruler::Ruler,
  animating: bool,
  shown: bool,
  window_size: (u32, u32),
  drawing: bool,
  pending: bool,
  drawables_released: bool,
}

impl Drop for Surface {
  fn drop(&mut self) {
    if self.animating {
      let _ = unsafe { KillTimer(Some(self.hwnd), input::ANIMATION_TIMER) };
    }
    let _ = unsafe { KillTimer(Some(self.hwnd), input::CONFIRM_TIMER) };
    let _ = unsafe { DestroyWindow(self.hwnd) };
  }
}

/// The root's window belongs to the webview and follows the scene alone; a
/// peer is only ordered on screen while the desktop is presented.
pub(crate) fn should_show(is_root: bool, visible: bool, desktop_presented: bool) -> bool {
  visible && (is_root || desktop_presented)
}

fn opaque_snapshot_target(snapshot_presented: bool, has_snapshot: bool) -> bool {
  snapshot_presented && has_snapshot
}

pub(crate) fn cursor_position() -> Option<POINT> {
  let mut point = POINT::default();
  unsafe { GetCursorPos(&mut point) }.ok().map(|()| point)
}

fn is_empty(rect: Rect) -> bool {
  !rect.valid() || rect.size.width <= 0.0 || rect.size.height <= 0.0
}

/// The R8 icon atlas the shader reads for kinds 22-26. The pixels come from
/// the portable `screenwide_osc_icon_atlas`, which decodes the shared Lucide
/// sheet once; macOS cached one texture per `MTLDevice` and this caches one
/// per [`Gpu`], which is the same thing.
///
/// The atlas lives in a private module of the frozen `osc::controls` tree, so
/// it is reached through the `#[no_mangle]` export the macOS renderer already
/// used rather than by widening that module's visibility.
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeIconAtlas {
  pixels: *const u8,
  length: usize,
  width: u32,
  height: u32,
  columns: u32,
}

extern "C" {
  fn screenwide_osc_icon_atlas() -> NativeIconAtlas;
}

const fn peer_pointer_style(style: isize, passthrough: bool) -> isize {
  if passthrough {
    style | WS_EX_TRANSPARENT.0 as isize
  } else {
    style & !(WS_EX_TRANSPARENT.0 as isize)
  }
}

#[cfg(test)]
#[path = "surface/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "surface/render_tests.rs"]
mod render_tests;
