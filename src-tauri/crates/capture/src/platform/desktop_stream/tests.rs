// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn source_rect_uses_display_local_points() {
  let piece = CapturePiece {
    display_id: 2,
    source_pixels: crate::desktop_capture::PixelRect {
      x: 200,
      y: 100,
      width: 800,
      height: 400,
    },
    destination: crate::desktop_capture::PixelRect {
      x: 0,
      y: 0,
      width: 400,
      height: 200,
    },
  };
  let rect = cg::Rect::new(
    f64::from(piece.source_pixels.x) / 2.0,
    f64::from(piece.source_pixels.y) / 2.0,
    f64::from(piece.source_pixels.width) / 2.0,
    f64::from(piece.source_pixels.height) / 2.0,
  );
  assert_eq!(rect, cg::Rect::new(100.0, 50.0, 400.0, 200.0));
}
