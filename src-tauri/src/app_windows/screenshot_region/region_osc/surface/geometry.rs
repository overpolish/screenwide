// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn hwnd(&self) -> WindowId {
    self.hwnd
  }

  pub(crate) fn is_root(&self) -> bool {
    matches!(self.kind, Kind::Root { .. })
  }

  pub(crate) fn is_preview(&self) -> bool {
    matches!(self.kind, Kind::Preview { .. })
  }

  /// The peer's monitor rectangle in physical pixels, for diffing a rebuild.
  pub(crate) fn peer_geometry(&self) -> Option<(Rect, f64)> {
    match self.kind {
      Kind::Root { .. } | Kind::Preview { .. } => None,
      Kind::Peer { bounds, scale } => Some((bounds, scale)),
    }
  }

  /// Physical pixels per logical point.
  pub(super) fn scale(&self) -> f64 {
    match self.kind {
      Kind::Root { host } => super::host().scale(host),
      Kind::Peer { scale, .. } | Kind::Preview { scale, .. } => scale,
    }
  }

  /// The window whose client area the surface covers: the webview's for the
  /// anchor, its own for a peer.
  fn covered_window(&self) -> WindowId {
    match self.kind {
      Kind::Root { host } => host,
      Kind::Peer { .. } | Kind::Preview { .. } => self.hwnd,
    }
  }

  /// Client size in physical pixels.
  pub(super) fn client_size(&self) -> (u32, u32) {
    match self.kind {
      Kind::Preview { size, .. } => size,
      _ => super::host().client_size(self.covered_window()),
    }
  }

  /// Converts a client-space physical point to surface-local logical points.
  pub(crate) fn logical_point(&self, x: f64, y: f64) -> Point {
    let scale = self.scale().max(0.1);
    Point {
      x: x / scale,
      y: y / scale,
    }
  }

  /// Surface-local logical points lifted into the desktop plane, which is what
  /// the controller and the semantic events are configured in.
  pub(crate) fn desktop_point(&self, point: Point) -> Point {
    Point {
      x: point.x + self.desktop_offset.x,
      y: point.y + self.desktop_offset.y,
    }
  }

  /// The reverse: a desktop-global rect in this surface's own coordinates.
  pub(crate) fn local_rect(&self, rect: Rect) -> Rect {
    Rect {
      origin: Point {
        x: rect.origin.x - self.desktop_offset.x,
        y: rect.origin.y - self.desktop_offset.y,
      },
      size: rect.size,
    }
  }

  pub(crate) fn set_desktop_offset(&mut self, origin: Point) {
    if self.desktop_offset != origin {
      self.desktop_offset = origin;
      self.draw();
    }
  }

  /// True when the pointer is over this surface's window.
  pub(crate) fn contains_screen_point(&self, point: ScreenPoint) -> bool {
    let Some(rect) = super::host().screen_rect(self.covered_window()) else {
      return false;
    };
    let (x, y) = (f64::from(point.x), f64::from(point.y));
    x >= rect.origin.x
      && x < rect.origin.x + rect.size.width
      && y >= rect.origin.y
      && y < rect.origin.y + rect.size.height
  }

  pub(crate) fn screen_to_client(&self, x: i32, y: i32) -> (f64, f64) {
    super::host().screen_to_client(self.hwnd, ScreenPoint { x, y })
  }

  /// Surface-local logical size, the space every chrome layout works in.
  pub(crate) fn logical_size(&self) -> Size {
    let (width, height) = self.client_size();
    let scale = self.scale().max(0.1);
    Size {
      width: f64::from(width) / scale,
      height: f64::from(height) / scale,
    }
  }

  pub(crate) fn desktop_offset(&self) -> Point {
    self.desktop_offset
  }
}
