// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The part of a canvas a platform's own interface leaves uncovered. Short
//! video apps draw their header over the top of the picture, the caption,
//! sound line and buttons over the bottom, and a column of buttons down the
//! end side; what matters (the screen, a face, the captions, the shortcuts)
//! is laid out clear of them, while the background still fills the whole
//! frame. The twin of `ClearArea` in
//! `src/features/editor/recording/formats/platform-clear-area.ts`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
  /// A canvas no platform covers.
  pub const NONE: Self = Self {
    top: 0.0,
    bottom: 0.0,
    side: 0.0,
  };

  /// The top and bottom of the clear part of a canvas `height` tall, kept at
  /// least `margin` from the canvas's own edges.
  pub fn rows(self, height: f64, margin: f64) -> (f64, f64) {
    (
      margin.max(self.top * height),
      (height - margin).min(height * (1.0 - self.bottom)),
    )
  }

  /// How far an overlay held at one of the nine places stays from each edge
  /// of a canvas `width` by `height`: `margin` on a canvas no platform
  /// covers, and past what the platform covers where that reaches further.
  /// Left, top, right, bottom.
  pub fn margins(self, (width, height): (f64, f64), margin: f64) -> [f64; 4] {
    let side = margin.max(self.side * width);
    [
      side,
      margin.max(self.top * height),
      side,
      margin.max(self.bottom * height),
    ]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn an_overlay_keeps_its_margin_where_the_platform_covers_less() {
    let tiktok = ClearArea {
      top: 0.07,
      bottom: 0.25,
      side: 0.05,
    };
    let [left, top, right, bottom] = tiktok.margins((1080.0, 1920.0), 100.0);
    assert_eq!([left, top, right], [100.0, 134.4, 100.0]);
    assert_eq!(bottom, 480.0);
    assert_eq!(
      ClearArea::default().margins((1920.0, 1080.0), 59.4),
      [59.4; 4]
    );
  }
}
