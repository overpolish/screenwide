// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn hwnd(&self) -> HWND {
    self.hwnd
  }

  pub(crate) fn is_root(&self) -> bool {
    matches!(self.kind, Kind::Root { .. })
  }

  /// The peer's monitor rectangle in physical pixels, for diffing a rebuild.
  pub(crate) fn peer_geometry(&self) -> Option<(Rect, f64)> {
    match self.kind {
      Kind::Root { .. } => None,
      Kind::Peer { bounds, scale } => Some((bounds, scale)),
    }
  }

  /// Physical pixels per logical point.
  pub(super) fn scale(&self) -> f64 {
    match self.kind {
      Kind::Root { host } => {
        let dpi = unsafe { GetDpiForWindow(host) };
        if dpi == 0 {
          1.0
        } else {
          f64::from(dpi) / BASE_DPI
        }
      }
      Kind::Peer { scale, .. } => scale,
    }
  }

  /// Client size in physical pixels.
  pub(super) fn client_size(&self) -> (u32, u32) {
    let window = match self.kind {
      Kind::Root { host } => host,
      Kind::Peer { .. } => self.hwnd,
    };
    let mut rect = RECT::default();
    if unsafe { GetClientRect(window, &mut rect) }.is_err() {
      return (0, 0);
    }
    (
      (rect.right - rect.left).max(0) as u32,
      (rect.bottom - rect.top).max(0) as u32,
    )
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
  pub(crate) fn contains_screen_point(&self, point: POINT) -> bool {
    let window = match self.kind {
      Kind::Root { host } => host,
      Kind::Peer { .. } => self.hwnd,
    };
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(window, &mut rect) }.is_err() {
      return false;
    }
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
  }

  pub(crate) fn screen_to_client(&self, x: i32, y: i32) -> (f64, f64) {
    let mut point = POINT { x, y };
    let _ = unsafe { ScreenToClient(self.hwnd, &mut point) };
    (f64::from(point.x), f64::from(point.y))
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
