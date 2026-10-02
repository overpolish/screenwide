// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where one composition lands: the canvas's own geometry, and where in its
//! target the canvas is drawn.

use super::*;
use crate::screenshots::OutputPlacement;

/// The geometry a canvas's settings resolve to, in canvas pixels: its size,
/// where the picture sits in it, and the rounding of the clip and of the
/// canvas. The macOS workspace keeps one a layer and moves it with a native
/// gesture before the settings catch up.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CanvasGeometry {
  pub(crate) size: (u32, u32),
  pub(crate) placement: OutputPlacement,
  pub(crate) radius: f32,
  pub(crate) background_radius: f32,
}

impl CanvasGeometry {
  pub(crate) fn of(
    source: (u32, u32),
    settings: &ScreenshotOutputSettings,
  ) -> Result<Self, String> {
    let placement = output_placement(source.0, source.1, settings)?;
    let shortest_crop = placement.crop_width.min(placement.crop_height) as f32;
    let shortest_output = settings.width.min(settings.height) as f32;
    Ok(Self {
      size: (settings.width, settings.height),
      placement,
      // While the crop tool is open the ghost is the whole uncropped source,
      // so its rounding moves to the cropped layer drawn over it.
      radius: if crop_preview_rect(settings).is_some() {
        0.0
      } else {
        shortest_crop * (settings.radius_percent as f32 / 100.0)
      },
      background_radius: shortest_output * (settings.background_radius_percent as f32 / 100.0),
    })
  }
}

/// The crop tool's result layer, x, y, width and height in output pixels,
/// when the settings carry one that can be drawn.
pub(super) fn crop_preview_rect(settings: &ScreenshotOutputSettings) -> Option<[f64; 4]> {
  settings
    .crop_preview
    .as_ref()
    .map(|rect| [rect.x, rect.y, rect.width, rect.height])
    .filter(|rect| rect.iter().all(|value| value.is_finite()) && rect[2] > 0.0 && rect[3] > 0.0)
}

/// A canvas drawn into part of a larger target, scaled: its rectangle there,
/// x, y, width and height in target pixels, and the target's size.
#[derive(Clone, Copy)]
pub(crate) struct LayerPlacement {
  pub(crate) rect: [f32; 4],
  pub(crate) target: (u32, u32),
}

/// How one composition is drawn into its target.
#[derive(Clone, Copy)]
pub(crate) struct LayerDraw {
  /// Where the canvas lands; `None` draws it at its own size from the
  /// target's corner.
  pub(crate) placement: Option<LayerPlacement>,
  /// Whether the whole target is cleared first, rather than drawn over.
  pub(crate) clear: bool,
  /// The redacted copy the draw leaves its picture in. Layers drawn into one
  /// frame each take their own, so every one stays readable to the end.
  pub(crate) redaction_slot: usize,
  /// Whether the source's pixels never change under its texture, so a redraw
  /// with the same redactions reuses the copy it left.
  pub(crate) retained_source: bool,
}

impl LayerPlacement {
  /// The `Canvas.placement` row: the rectangle's corner, and the canvas
  /// pixels one target pixel covers.
  pub(super) fn row(placement: Option<Self>, canvas: (u32, u32)) -> [f32; 4] {
    placement.map_or([0.0, 0.0, 1.0, 1.0], |placement| {
      let [x, y, width, height] = placement.rect;
      [
        x,
        y,
        canvas.0 as f32 / width.max(1.0),
        canvas.1 as f32 / height.max(1.0),
      ]
    })
  }

  /// The target pixels the rectangle covers, as a scissor: x, y, width and
  /// height, or `None` where it lies wholly outside the target.
  pub(super) fn scissor(self) -> Option<[u32; 4]> {
    let [x, y, width, height] = self.rect;
    let left = x.floor().max(0.0);
    let top = y.floor().max(0.0);
    let right = (x + width).ceil().min(self.target.0 as f32);
    let bottom = (y + height).ceil().min(self.target.1 as f32);
    (right > left && bottom > top).then_some([
      left as u32,
      top as u32,
      (right - left) as u32,
      (bottom - top) as u32,
    ])
  }
}
