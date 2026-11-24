// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::output::{output_dimensions, ScreenshotOutputSettings};

#[derive(Clone, Copy, Debug)]
pub struct OutputPlacement {
  pub crop_height: u32,
  pub crop_width: u32,
  pub crop_x: i32,
  pub crop_y: i32,
  /// The image's size as placed, not rounded: its height follows its width
  /// by the source's aspect, so the picture keeps one scale on both axes,
  /// as a scaled video export relies on (see `FrameFit` in the export).
  pub image_height: f64,
  pub image_width: f64,
  pub image_x: f64,
  pub image_y: f64,
  pub source_crop_height: u32,
  pub source_crop_width: u32,
  pub source_crop_x: i32,
  pub source_crop_y: i32,
}

/// How many canvases a placement may reach past the canvas it is on, the
/// same eight the percentages it was once stored as allowed.
const PLACEMENT_REACH: f64 = 8.0;

/// Whether a layer's placement is finite and sits within reach of its
/// `canvas` (width, height): its crop and image no wider or taller than
/// eight canvases, nor further off than that. Each axis is held to its own
/// side, so a canvas narrowed too far fails on width alone.
fn placement_in_reach(settings: &ScreenshotOutputSettings, canvas: (u32, u32)) -> bool {
  let width_limit = f64::from(canvas.0) * PLACEMENT_REACH;
  let height_limit = f64::from(canvas.1) * PLACEMENT_REACH;
  [
    settings.crop_height,
    settings.crop_width,
    settings.crop_x,
    settings.crop_y,
    settings.image_width,
    settings.image_x,
    settings.image_y,
  ]
  .iter()
  .all(|value| value.is_finite())
    && (1.0..=width_limit).contains(&settings.crop_width)
    && (1.0..=height_limit).contains(&settings.crop_height)
    && settings.crop_x.abs() <= width_limit
    && settings.crop_y.abs() <= height_limit
    && (1.0..=width_limit).contains(&settings.image_width)
    && settings.image_x.abs() <= width_limit
    && settings.image_y.abs() <= height_limit
}

/// How a canvas resize moves its top left corner, per axis: the share of the
/// change in size the corner takes. 0 where the far edge moves (right,
/// bottom), 1 where the near one does (left, top), ½ where both move about
/// the middle. A placement is measured from that corner, so it shifts by as
/// much.
#[derive(Clone, Copy, Debug)]
pub struct CanvasAnchor {
  pub x: f64,
  pub y: f64,
}

/// The narrowest and shortest the canvas `settings` is on can be resized to,
/// moving its corner as `anchor` says, while its placement stays within
/// reach (`placement_in_reach`); a resize never moves or scales what the
/// canvas holds, so past this it leaves the picture many canvases off or
/// across. In the canvas's own pixels, (width, height).
pub fn narrowest_in_reach(settings: &ScreenshotOutputSettings, anchor: CanvasAnchor) -> (f64, f64) {
  // Placed at `place` on a canvas `size` across, a resize to `to` puts it at
  // `place - share * (size - to)`, which stays within the reach of `to` on
  // either side from the size this gives on.
  let from = |place: f64, size: f64, share: f64| {
    let fixed = place - share * size;
    (fixed / (PLACEMENT_REACH - share)).max(-fixed / (PLACEMENT_REACH + share))
  };
  let width = f64::from(settings.width);
  let height = f64::from(settings.height);
  let across = [
    from(settings.crop_x, width, anchor.x),
    from(settings.image_x, width, anchor.x),
    settings.crop_width / PLACEMENT_REACH,
    settings.image_width / PLACEMENT_REACH,
  ];
  let down = [
    from(settings.crop_y, height, anchor.y),
    from(settings.image_y, height, anchor.y),
    settings.crop_height / PLACEMENT_REACH,
  ];
  (
    across.into_iter().fold(0.0, f64::max),
    down.into_iter().fold(0.0, f64::max),
  )
}

/// Resolve a layer's stored pixel placement into whole output pixels.
///
/// Placement arrives in the output's own pixels, so the work here is limited
/// to validating it, deriving the image's height from the source's aspect and
/// rounding the crops to the integer rectangles the compositor clips by.
pub fn output_placement(
  source_width: u32,
  source_height: u32,
  settings: &ScreenshotOutputSettings,
) -> Result<OutputPlacement, String> {
  let (output_width, output_height) = output_dimensions(settings)?;
  settings.source_crop.validate()?;
  let source_crop = settings.source_crop;
  if source_width == 0
    || source_height == 0
    || !placement_in_reach(settings, (output_width, output_height))
  {
    return Err("The screenshot placement is not valid".to_owned());
  }
  let image_width = settings.image_width.max(1.0);
  let image_height = (image_width * f64::from(source_height) / f64::from(source_width)).max(1.0);
  let mut crop_height = settings.crop_height.round().max(1.0) as u32;
  let mut crop_width = settings.crop_width.round().max(1.0) as u32;
  let mut crop_x = settings.crop_x.round() as i32;
  let mut crop_y = settings.crop_y.round() as i32;
  let image_x = settings.image_x;
  let image_y = settings.image_y;
  let source = [
    image_x + image_width * source_crop.x,
    image_y + image_height * source_crop.y,
    image_width * source_crop.width,
    image_height * source_crop.height,
  ];
  // A scene that pads its box shows the picture only in the window it
  // padded; the rest of the crop is the inset colour.
  let [source_crop_x, source_crop_y, source_width, source_height] = settings
    .scene_window
    .map_or(source, |window| windowed(source, window));
  let source_crop_width = source_width.round().max(1.0) as u32;
  let source_crop_height = source_height.round().max(1.0) as u32;
  // Preview scaling can round otherwise coincident frame/source edges apart.
  let image_left = image_x.round() as i32;
  let image_top = image_y.round() as i32;
  let image_right = image_left.saturating_add(image_width.round() as i32);
  let image_bottom = image_top.saturating_add(image_height.round() as i32);
  let crop_right = crop_x.saturating_add(crop_width as i32);
  let crop_bottom = crop_y.saturating_add(crop_height as i32);
  if (crop_x - image_left).abs() <= 1
    && (crop_right - image_right).abs() <= 1
    && (crop_y - image_top).abs() <= 1
    && (crop_bottom - image_bottom).abs() <= 1
  {
    crop_x = image_left;
    crop_y = image_top;
    crop_width = image_width.round() as u32;
    crop_height = image_height.round() as u32;
  }
  Ok(OutputPlacement {
    crop_height,
    crop_width,
    crop_x,
    crop_y,
    image_height,
    image_width,
    image_x,
    image_y,
    source_crop_height,
    source_crop_width,
    source_crop_x: source_crop_x.round() as i32,
    source_crop_y: source_crop_y.round() as i32,
  })
}

/// `source`, a left, top, width and height, cut to what of it lies inside
/// `window`; a source wholly outside it keeps no width or height, at the
/// window's nearest edge.
fn windowed(source: [f64; 4], window: [f64; 4]) -> [f64; 4] {
  let left = source[0].max(window[0]);
  let top = source[1].max(window[1]);
  let right = (source[0] + source[2]).min(window[0] + window[2]);
  let bottom = (source[1] + source[3]).min(window[1] + window[3]);
  [left, top, (right - left).max(0.0), (bottom - top).max(0.0)]
}
