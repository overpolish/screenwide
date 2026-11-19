// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where one composition lands: the canvas's own geometry, and where in its
//! target the canvas is drawn.

use super::*;
use screenwide_output::OutputPlacement;

/// The geometry a canvas's settings resolve to, in canvas pixels: its size,
/// where the picture sits in it, and the rounding of the clip and of the
/// canvas. The macOS workspace keeps one a layer and moves it with a native
/// gesture before the settings catch up.
#[derive(Clone, Copy, Debug)]
pub struct CanvasGeometry {
  pub size: (u32, u32),
  pub placement: OutputPlacement,
  pub radius: f32,
  pub background_radius: f32,
}

impl CanvasGeometry {
  pub fn of(source: (u32, u32), settings: &ScreenshotOutputSettings) -> Result<Self, String> {
    let placement = output_placement(source.0, source.1, settings)?;
    let crop = (
      f64::from(placement.crop_width),
      f64::from(placement.crop_height),
    );
    let shortest_output = settings.width.min(settings.height) as f32;
    Ok(Self {
      size: (settings.width, settings.height),
      placement,
      // While the crop tool is open the ghost is the whole uncropped source,
      // so its rounding moves to the cropped layer drawn over it.
      radius: if crop_preview_rect(settings).is_some() {
        0.0
      } else {
        settings.corner_radius_pixels(crop.0, crop.1) as f32
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
/// x, y, width and height in target pixels, and the target's size. What lies
/// past the rectangle's edge is drawn over the rest of the target.
#[derive(Clone, Copy)]
pub struct LayerPlacement {
  pub rect: [f32; 4],
  pub target: (u32, u32),
}

/// Part of a canvas an unplaced draw's target holds, in canvas pixels: a
/// canvas larger than the device draws is drawn a band at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanvasBand {
  pub origin: (u32, u32),
  pub size: (u32, u32),
}

/// What part of a picture larger than the device takes a source texture
/// holds, and how finely: where its first texel lies in the picture, and how
/// many of the picture's pixels a texel spans each way. The texture's `size`
/// is still the whole picture's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceWindow {
  pub origin: (u32, u32),
  pub scale: u32,
}

/// How one composition is drawn into its target.
#[derive(Clone, Copy)]
pub struct LayerDraw {
  /// Where the canvas lands; `None` draws it at its own size from the
  /// target's corner.
  pub placement: Option<LayerPlacement>,
  /// The part of the canvas the target holds, where it is drawn unplaced and
  /// holds less than all of it.
  pub band: Option<CanvasBand>,
  /// The part of the picture the source texture holds, where it holds less
  /// than all of it at its own size.
  pub source_window: Option<SourceWindow>,
  /// Whether the whole target is cleared first, rather than drawn over.
  pub clear: bool,
  /// The redacted copy the draw leaves its picture in. Layers drawn into one
  /// frame each take their own, so every one stays readable to the end.
  pub redaction_slot: usize,
  /// Whether the source's pixels never change under its texture, so a redraw
  /// with the same redactions reuses the copy it left.
  pub retained_source: bool,
}

impl LayerDraw {
  /// The size of the target the canvas is drawn into, for a canvas `canvas`
  /// pixels in size.
  pub(super) fn frame(&self, canvas: (u32, u32)) -> (u32, u32) {
    match (self.placement, self.band) {
      (Some(placement), _) => placement.target,
      (None, Some(band)) => band.size,
      (None, None) => canvas,
    }
  }

  /// The `Canvas.placement` row: where the canvas's corner lies in the
  /// target, and the canvas pixels one target pixel covers.
  pub(super) fn placement_row(&self, canvas: (u32, u32)) -> [f32; 4] {
    match (self.placement, self.band) {
      (Some(placement), _) => {
        let [x, y, width, height] = placement.rect;
        [
          x,
          y,
          canvas.0 as f32 / width.max(1.0),
          canvas.1 as f32 / height.max(1.0),
        ]
      }
      (None, Some(band)) => [-(band.origin.0 as f32), -(band.origin.1 as f32), 1.0, 1.0],
      (None, None) => [0.0, 0.0, 1.0, 1.0],
    }
  }
}

impl LayerPlacement {
  /// How opaque what lies past the canvas's edge is drawn: faded, so it reads
  /// as off the canvas and not exported.
  pub(super) const OVERFLOW_OPACITY: f32 = 0.55;
}
