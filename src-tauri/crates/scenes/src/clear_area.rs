// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The part of a canvas a platform's own interface leaves uncovered. Short
//! video apps draw their header over the top of the picture, the caption,
//! sound line and buttons over the bottom, and a column of buttons down the
//! end side; what matters (the screen, a face) is laid out clear of them,
//! while the background still fills the whole frame. The twin of
//! `ClearArea` in `src/features/editor/recording/formats/platform-clear-area.ts`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::geometry::Rect;

/// What a platform covers at each edge, as shares of the canvas: the top and
/// the bottom of its height, and each side of its width. The button rail
/// covers only the end side, but both keep its margin so a screen laid out
/// between them stays centred.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClearArea {
  pub top: f64,
  pub bottom: f64,
  pub side: f64,
}

impl ClearArea {
  /// The top and bottom of the clear part of a canvas `height` tall, kept at
  /// least `margin` from the canvas's own edges.
  pub fn rows(self, height: f64, margin: f64) -> (f64, f64) {
    (
      margin.max(self.top * height),
      (height - margin).min(height * (1.0 - self.bottom)),
    )
  }

  /// The middle of the part of `frame` left clear on a canvas `canvas`
  /// large, or of `frame` itself where none of it is: where a face followed
  /// in a box reaching under the platform's interface is held, so it is not
  /// hidden behind a header or a caption.
  pub fn middle_of(self, frame: Rect, canvas: (f64, f64)) -> (f64, f64) {
    let left = frame.x.max(self.side * canvas.0);
    let right = (frame.x + frame.width).min(canvas.0 * (1.0 - self.side));
    let top = frame.y.max(self.top * canvas.1);
    let bottom = (frame.y + frame.height).min(canvas.1 * (1.0 - self.bottom));
    if right <= left || bottom <= top {
      return frame.centre();
    }
    ((left + right) / 2.0, (top + bottom) / 2.0)
  }
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
    let (x, y) = TIKTOK.middle_of(whole, canvas);
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
    assert_eq!(TIKTOK.middle_of(frame, (1080.0, 1920.0)), frame.centre());
    let covered = Rect { y: 1700.0, ..frame };
    assert_eq!(
      TIKTOK.middle_of(covered, (1080.0, 1920.0)),
      covered.centre()
    );
  }
}
