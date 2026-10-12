// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::desktop_capture::PixelRect;

#[test]
fn constants_keep_monitor_crop_and_canvas_placement_separate() {
  let constants = PieceConstants::new(
    [1600, 900],
    [3840, 2160],
    CapturePiece {
      display_id: 7,
      source_pixels: PixelRect {
        x: 120,
        y: 80,
        width: 1920,
        height: 1080,
      },
      destination: PixelRect {
        x: 400,
        y: 0,
        width: 1200,
        height: 900,
      },
    },
  );
  assert_eq!(constants.output_size, [1600, 900]);
  assert_eq!(constants.source_size, [3840, 2160]);
  assert_eq!(constants.source_origin, [120, 80]);
  assert_eq!(constants.destination_origin, [400, 0]);
}
