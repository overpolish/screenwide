// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn root(gpu: Arc<Gpu>, host: HWND, overlay: HWND) -> Result<Self, String> {
    Self::new(gpu, Kind::Root { host }, overlay, 0)
  }

  pub(crate) fn peer(
    gpu: Arc<Gpu>,
    hwnd: HWND,
    display_id: u32,
    bounds: Rect,
    scale: f64,
  ) -> Result<Self, String> {
    Self::new(gpu, Kind::Peer { bounds, scale }, hwnd, display_id)
  }

  pub(super) fn new(
    gpu: Arc<Gpu>,
    kind: Kind,
    hwnd: HWND,
    display_id: u32,
  ) -> Result<Self, String> {
    let chain = gpu.shared.create_swap_chain(hwnd)?;
    Ok(Self {
      gpu,
      kind,
      hwnd,
      chain,
      vertex_buffer: None,
      vertex_capacity: 0,
      vertices: Vec::new(),
      magnifier_source: None,
      snapshot: None,
      display_id,
      region: Rect::default(),
      visible: false,
      show_frame: true,
      show_handles: true,
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
    })
  }
}
