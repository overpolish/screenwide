// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Portable ready-toolbar policy. Platform adapters measure native text and
//! host the material surfaces, while placement remains shared for the later
//! Windows adapter. Reusable confirmation behavior lives in `osc::controls`.

use screenwide_osc::controls::control_spacing;
use screenwide_osc::geometry::{Rect, Size};

pub const CONTROL_COUNT: usize = 4;
/// `--spacing-control`, the gap between neighbouring controls in a row.
const GAP: f64 = control_spacing().control;
/// `--spacing-control-inset`, the inset from the selection and the viewport.
const MARGIN: f64 = control_spacing().control_inset;

pub fn layout(
  selection: Rect,
  viewport: Size,
  widths: [f64; CONTROL_COUNT],
  height: f64,
) -> [Rect; CONTROL_COUNT] {
  let total = widths.iter().sum::<f64>() + GAP * (CONTROL_COUNT - 1) as f64;
  let max_left = (viewport.width - total - MARGIN).max(MARGIN);
  let selection_center = selection.origin.x + selection.size.width * 0.5;
  let left = (selection_center - total * 0.5).clamp(MARGIN, max_left);
  let below = selection.bottom() + MARGIN;
  let top = if below + height <= viewport.height - MARGIN {
    below
  } else {
    (selection.origin.y - height - MARGIN).max(MARGIN)
  };
  let mut x = left;
  std::array::from_fn(|index| {
    let rect = Rect::from_xywh(x, top, widths[index], height);
    x += widths[index] + GAP;
    rect
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn toolbar_centres_individual_controls_below_the_selection() {
    let rects = layout(
      Rect::from_xywh(100.0, 100.0, 400.0, 200.0),
      Size {
        width: 800.0,
        height: 600.0,
      },
      [90.0, 130.0, 24.0, 24.0],
      24.0,
    );
    assert_eq!(rects[0], Rect::from_xywh(160.0, 308.0, 90.0, 24.0));
    assert_eq!(rects[1].origin.x, 254.0);
    assert_eq!(rects[3].right(), 440.0);
    assert_eq!((rects[0].origin.x + rects[3].right()) * 0.5, 300.0);
  }

  #[test]
  fn toolbar_moves_above_and_clamps_to_the_viewport() {
    let rects = layout(
      Rect::from_xywh(760.0, 560.0, 30.0, 30.0),
      Size {
        width: 800.0,
        height: 600.0,
      },
      [90.0, 130.0, 24.0, 24.0],
      24.0,
    );
    assert_eq!(rects[0].origin.y, 528.0);
    assert_eq!(rects[0].origin.x, 512.0);
    assert_eq!(rects[3].right(), 792.0);
  }
}
