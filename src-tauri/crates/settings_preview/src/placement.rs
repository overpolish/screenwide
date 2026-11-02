// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a preview shows in its page, as the page measures it: every rect is
//! in the page's points, from the top left of what the page draws.

use serde::Deserialize;
use ts_rs::TS;

/// A rectangle in the page's points.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct PreviewArea {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

impl PreviewArea {
  pub fn right(self) -> f64 {
    self.x + self.width
  }

  pub fn bottom(self) -> f64 {
    self.y + self.height
  }
}

/// A scroll area's fade at its ends, which the page draws with a CSS mask
/// the native layer cannot see.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct PreviewFade {
  /// The scroll area's viewport.
  pub area: PreviewArea,
  /// Fades along the vertical axis, rather than the horizontal.
  pub vertical: bool,
  /// Points faded at the top or left.
  pub start: f64,
  /// Points faded at the bottom or right.
  pub end: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PreviewPlacement {
  /// The page's box the picture shows in.
  pub frame: PreviewArea,
  /// Where the picture is. A tool's picture is wider than the box and
  /// centred on it, so a resize moves it rather than redrawing it; a
  /// recording's fills the box.
  pub picture: PreviewArea,
  /// The part of the page it may be seen in.
  pub clip: PreviewArea,
  pub fade: Option<PreviewFade>,
  /// How round the box's corners are, in points.
  pub radius: f64,
  pub visible: bool,
  /// Physical pixels per point, from the window rather than the page.
  #[serde(skip)]
  #[ts(skip)]
  pub scale: f64,
}

impl PreviewPlacement {
  pub fn hidden() -> Self {
    Self {
      frame: PreviewArea::default(),
      picture: PreviewArea::default(),
      clip: PreviewArea::default(),
      fade: None,
      radius: 0.0,
      visible: false,
      scale: 1.0,
    }
  }

  /// The picture's size in physical pixels.
  pub fn picture_pixels(&self) -> (u32, u32) {
    let pixels = |points: f64| (points * self.scale).round().max(1.0) as u32;
    (pixels(self.picture.width), pixels(self.picture.height))
  }

  /// The part of the clip the preview shows where the platform has no
  /// gradient mask for a scroll area's fade: the picture stops at the middle
  /// of each faded end, where the page's content is half gone.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub(crate) fn seen_halfway_faded(&self) -> PreviewArea {
    let clip = self.clip;
    let Some(fade) = self.fade else {
      return clip;
    };
    let area = fade.area;
    let (start, end) = (fade.start / 2.0, fade.end / 2.0);
    let faded = if fade.vertical {
      PreviewArea {
        y: area.y + start,
        height: area.height - start - end,
        ..area
      }
    } else {
      PreviewArea {
        x: area.x + start,
        width: area.width - start - end,
        ..area
      }
    };
    let left = clip.x.max(faded.x);
    let top = clip.y.max(faded.y);
    let right = clip.right().min(faded.right());
    let bottom = clip.bottom().min(faded.bottom());
    PreviewArea {
      x: left,
      y: top,
      width: (right - left).max(0.0),
      height: (bottom - top).max(0.0),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn area(x: f64, y: f64, width: f64, height: f64) -> PreviewArea {
    PreviewArea {
      x,
      y,
      width,
      height,
    }
  }

  #[test]
  fn a_fade_without_a_mask_is_met_halfway() {
    let placement = PreviewPlacement {
      clip: area(0.0, 0.0, 400.0, 300.0),
      fade: Some(PreviewFade {
        area: area(0.0, 20.0, 400.0, 260.0),
        vertical: true,
        start: 16.0,
        end: 24.0,
      }),
      ..PreviewPlacement::hidden()
    };
    assert_eq!(
      placement.seen_halfway_faded(),
      area(0.0, 28.0, 400.0, 260.0 - 8.0 - 12.0)
    );
  }
}
