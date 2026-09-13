// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// How far a frame's top left corner moved, in output pixels.
pub(super) fn origin_shift(old: WorldRect, new: WorldRect) -> (f64, f64) {
  (new.x - old.x, new.y - old.y)
}

/// Renumbers a layer's placement so it keeps its place on screen after the
/// canvas's corner moved by `shift`.
pub(super) fn shift_output(output: &mut ScreenshotOutputSettings, shift: (f64, f64)) {
  output.crop_x -= shift.0;
  output.crop_y -= shift.1;
  output.image_x -= shift.0;
  output.image_y -= shift.1;
}
