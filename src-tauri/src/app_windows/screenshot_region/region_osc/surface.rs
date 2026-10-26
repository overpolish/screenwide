// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Region OSC surfaces. The anchor display draws into a window the host lays
//! over the Tauri window; every other display gets a peer window of its own.
//! All of them share one [`Gpu`] - the app's wgpu device and one pipeline -
//! and own only their window and drawable. This is the attach and the frame
//! draw, with the desktop peers and their frozen snapshots; OCR and the Ruler
//! draw on top.

mod content;
mod creation;
mod cutout;
mod geometry;
mod gpu;
mod render;
mod submit;
mod textures;
use screenwide_gpu::surface::Frame;
use screenwide_osc::gpu::vertices::{bind_group_layout, pipeline, sampler, shader_module};
pub(super) use textures::upload_rgba;
use textures::{upload_rgba_with_levels, RGBA};

use std::sync::Arc;

use super::host::{host, RegionDrawable, ScreenPoint, WindowId};
use super::input;
use super::ocr;
use super::renderer::{self, RenderConstants, Vertex};
use super::ruler;
use super::segment::Segment;
use screenwide_osc::geometry::{Point, Rect, Size};

/// The lens anchor and dragged-edge bitmask. The sampling window and box size
/// are fixed, so a frame can rebuild the constants from these two values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MagnifierAnchor {
  pub point: Point,
  pub edges: u32,
}

#[derive(Clone)]
pub(crate) struct Texture {
  view: wgpu::TextureView,
  size: (u32, u32),
}

/// Everything shared by the anchor surface and its peers, so a peer costs one
/// window plus one swap chain. macOS shared the `MTLDevice` the same way.
pub(crate) struct Gpu {
  shared: &'static screenwide_gpu::Gpu,
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  linear_sampler: wgpu::Sampler,
  point_sampler: wgpu::Sampler,
  /// Every texture slot is bound each draw; unused ones read one transparent
  /// texel.
  placeholder: wgpu::TextureView,
  /// Makes the picking backdrop's levels, which its blur reads.
  mips: screenwide_gpu::mips::Mips,
}

impl Gpu {
  pub(super) fn device(&self) -> &'static screenwide_gpu::Gpu {
    self.shared
  }
}

/// Where a surface takes its geometry and DPI from.
enum Kind {
  /// The anchor display: a child of the Tauri window, sized to its client area.
  Root { host: WindowId },
  /// One non-anchor display: a top-level popup covering its monitor. Its scale
  /// comes from the binding because a peer may sit on a different-DPI monitor.
  Peer { bounds: Rect, scale: f64 },
  /// A tool shown at work inside another window, such as a Settings
  /// picture. Its owner places the drawable and drives its frames, so the
  /// surface has no window of its own to show, raise or time.
  #[cfg_attr(
    target_os = "windows",
    expect(dead_code, reason = "tool previews have a macOS host only")
  )]
  Preview { size: (u32, u32), scale: f64 },
}

pub(crate) struct Surface {
  gpu: Arc<Gpu>,
  kind: Kind,
  hwnd: WindowId,
  chain: Box<dyn RegionDrawable>,
  vertex_buffer: Option<wgpu::Buffer>,
  vertex_capacity: usize,
  /// One constants block per draw call, read at a dynamic offset.
  constants_buffer: Option<wgpu::Buffer>,
  constants_capacity: usize,
  vertices: Vec<Vertex>,
  magnifier_source: Option<Texture>,
  snapshot: Option<Texture>,
  /// The desktop behind window picking, frozen as picking starts.
  backdrop: Option<Texture>,

  // What the surface presents.
  pub(crate) display_id: u32,
  pub(crate) region: Rect,
  pub(crate) visible: bool,
  pub(crate) show_frame: bool,
  pub(crate) show_handles: bool,
  /// The region is a window being picked: a cutout as round as the window.
  pub(crate) window_highlight: bool,
  pub(crate) cutout: cutout::CutoutFade,
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
    if self.is_preview() {
      return;
    }
    if self.animating {
      host().kill_timer(self.hwnd, input::ANIMATION_TIMER);
    }
    host().destroy_window(self.hwnd);
  }
}

/// The root's window belongs to the webview and follows the scene alone; a
/// peer is only ordered on screen while the desktop is presented.
pub(crate) fn should_show(is_root: bool, visible: bool, desktop_presented: bool) -> bool {
  visible && (is_root || desktop_presented)
}

pub(crate) fn cursor_position() -> Option<ScreenPoint> {
  host().cursor_position()
}

fn is_empty(rect: Rect) -> bool {
  !rect.valid() || rect.size.width <= 0.0 || rect.size.height <= 0.0
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod render_tests;
