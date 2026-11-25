// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a face is followed in a pane reaching under a platform's interface
//! (`ClearArea`): about the part of the pane the platform leaves clear.

use screenwide_placement::ClearArea;

use crate::geometry::Rect;

/// The middle of the part of `frame` `clear` leaves uncovered on a canvas
/// `canvas` large, or of `frame` itself where none of it is: where a face
/// followed in a box reaching under the platform's interface is held, so it
/// is not hidden behind a header or a caption.
pub fn clear_middle(clear: ClearArea, frame: Rect, canvas: (f64, f64)) -> (f64, f64) {
  let left = frame.x.max(clear.side * canvas.0);
  let right = (frame.x + frame.width).min(canvas.0 * (1.0 - clear.side));
  let top = frame.y.max(clear.top * canvas.1);
  let bottom = (frame.y + frame.height).min(canvas.1 * (1.0 - clear.bottom));
  if right <= left || bottom <= top {
    return frame.centre();
  }
  ((left + right) / 2.0, (top + bottom) / 2.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  const TIKTOK: ClearArea = ClearArea {
    top: 0.07,
    bottom: 0.25,
    side: 0.05,
  };

  #[test]
  fn a_box_under_the_interface_is_followed_about_its_clear_part() {
    let canvas = (1080.0, 1920.0);
    let whole = Rect {
      x: 0.0,
      y: 0.0,
      width: 1080.0,
      height: 1920.0,
    };
    let (x, y) = clear_middle(TIKTOK, whole, canvas);
    assert!((x - 540.0).abs() < 1e-9);
    assert!((y - (0.07 + 0.75) / 2.0 * 1920.0).abs() < 1e-9);
  }

  #[test]
  fn a_box_clear_of_the_interface_keeps_its_own_middle() {
    let frame = Rect {
      x: 300.0,
      y: 600.0,
      width: 400.0,
      height: 300.0,
    };
    assert_eq!(
      clear_middle(TIKTOK, frame, (1080.0, 1920.0)),
      frame.centre()
    );
    let covered = Rect { y: 1700.0, ..frame };
    assert_eq!(
      clear_middle(TIKTOK, covered, (1080.0, 1920.0)),
      covered.centre()
    );
  }
}
