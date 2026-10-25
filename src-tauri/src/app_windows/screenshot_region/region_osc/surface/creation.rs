// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn root(gpu: Arc<Gpu>, host: WindowId, overlay: WindowId) -> Result<Self, String> {
    Self::new(gpu, Kind::Root { host }, overlay, 0)
  }

  pub(crate) fn peer(
    gpu: Arc<Gpu>,
    hwnd: WindowId,
    display_id: u32,
    bounds: Rect,
    scale: f64,
  ) -> Result<Self, String> {
    Self::new(gpu, Kind::Peer { bounds, scale }, hwnd, display_id)
  }

  /// A surface drawing into `chain`, which its owner places and sizes.
  #[cfg_attr(
    target_os = "windows",
    expect(dead_code, reason = "tool previews have a macOS host only")
  )]
  pub(crate) fn preview(
    gpu: Arc<Gpu>,
    chain: Box<dyn RegionDrawable>,
    size: (u32, u32),
    scale: f64,
  ) -> Self {
    Self::with_chain(gpu, Kind::Preview { size, scale }, WindowId(0), 0, chain)
  }

  pub(super) fn new(
    gpu: Arc<Gpu>,
    kind: Kind,
    hwnd: WindowId,
    display_id: u32,
  ) -> Result<Self, String> {
    let chain = host().create_drawable(gpu.device(), hwnd)?;
    Ok(Self::with_chain(gpu, kind, hwnd, display_id, chain))
  }

  fn with_chain(
    gpu: Arc<Gpu>,
    kind: Kind,
    hwnd: WindowId,
    display_id: u32,
    chain: Box<dyn RegionDrawable>,
  ) -> Self {
    Self {
      gpu,
      kind,
      hwnd,
      chain,
      vertex_buffer: None,
      vertex_capacity: 0,
      constants_buffer: None,
      constants_capacity: 0,
      vertices: Vec::new(),
      magnifier_source: None,
      snapshot: None,
      backdrop: None,
      display_id,
      region: Rect::default(),
      visible: false,
      show_frame: true,
      show_handles: true,
      window_highlight: false,
      cutout: Default::default(),
      input_enabled: false,
      exclusion_rect: Rect::default(),
      magnifier: None,
      snapshot_presented: false,
      snapshot_composited: false,
      desktop_presented: false,
      desktop_offset: Point::default(),
      gesture_active: false,
      cursor: input::CursorShape::None,
      ocr: ocr::Chrome::default(),
      ruler: ruler::Ruler::default(),
      animating: false,
      shown: false,
      window_size: (0, 0),
      drawing: false,
      pending: false,
      drawables_released: false,
    }
  }
}
