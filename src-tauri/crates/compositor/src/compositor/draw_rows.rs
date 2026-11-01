// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rows of a layer's [`Constants`] that describe its shadows, cursor and
//! camera, worked out from the canvas settings before the layer is drawn.

use screenwide_cursor_effects::GpuCursor;

use super::*;

/// The camera a layer composites, with its geometry, whether it casts a
/// shadow, and whether it sits on top of the screen.
pub(super) type LayerCamera<'a> = Option<(&'a SourceTexture, BakeGeometry, bool, bool)>;

/// How far the screen's shadow spreads, for the visible frame
/// `[left, top, right, bottom]` on a `size` canvas. A frame's shadow falls
/// outside it, so it narrows to fit the canvas around the frame; a person cut
/// out casts theirs inside the frame, with room around them however little
/// canvas is left.
pub(super) fn screen_shadow_sigma(visible: [f32; 4], size: (f32, f32), cut_out: bool) -> f32 {
  let [left, top, right, bottom] = visible;
  let sigma = ((right - left).max(0.0).min((bottom - top).max(0.0)) * 0.055).clamp(10.0, 110.0);
  if cut_out {
    sigma
  } else {
    sigma.min(edge_margin(visible, size) * 0.45)
  }
}

/// How far the cropped layer's shadow spreads while the crop tool previews
/// `rect` (`[x, y, width, height]`) on a `size` canvas.
pub(super) fn crop_preview_shadow_sigma(rect: [f32; 4], size: (f32, f32)) -> f32 {
  let [x, y, width, height] = rect;
  (width.min(height) * 0.055)
    .clamp(10.0, 110.0)
    .min(edge_margin([x, y, x + width, y + height], size) * 0.45)
}

/// The least canvas between the box `[left, top, right, bottom]` and the
/// edges of a `size` canvas.
fn edge_margin([left, top, right, bottom]: [f32; 4], (width, height): (f32, f32)) -> f32 {
  left
    .min(top)
    .min(width - right)
    .min(height - bottom)
    .max(0.0)
}

/// The cursor's geometry, effects and blur rows. The blur row carries the
/// cursor's motion delta, then the spotlight's deviation and strength.
pub(super) fn cursor_rows(cursor: Option<GpuCursor>, spotlight_blur: [f32; 2]) -> [[f32; 4]; 3] {
  let geometry = cursor.map_or([0.0; 4], |cursor| {
    [cursor.x, cursor.y, cursor.width, cursor.height]
  });
  let effects = cursor.map_or([0.0; 4], |cursor| {
    [cursor.opacity, 0.0, cursor.rotation_radians, cursor.scale]
  });
  let [deviation, strength] = spotlight_blur;
  let [delta_x, delta_y] = cursor.map_or([0.0; 2], |cursor| {
    [cursor.blur_delta_x, cursor.blur_delta_y]
  });
  [geometry, effects, [delta_x, delta_y, deviation, strength]]
}

/// The camera's frame, crop and effects rows.
pub(super) fn camera_rows(
  camera: LayerCamera<'_>,
  settings: &ScreenshotOutputSettings,
) -> [[f32; 4]; 3] {
  let Some((_, geometry, drop_shadow, camera_on_top)) = camera else {
    return [[0.0; 4]; 3];
  };
  let frame = [
    geometry.frame_x as f32,
    geometry.frame_y as f32,
    geometry.frame_width as f32,
    geometry.frame_height as f32,
  ];
  let crop = [
    geometry.crop_x as f32,
    geometry.crop_y as f32,
    geometry.crop_width as f32,
    geometry.crop_height as f32,
  ];
  let shortest = geometry.frame_width.min(geometry.frame_height) as f32;
  let sigma = if drop_shadow {
    (shortest * 0.055).clamp(3.0, 110.0)
  } else {
    0.0
  };
  // A scene's order, crossing over while it changes, overrides the
  // composition's own.
  let in_front = settings
    .scene_camera_front
    .unwrap_or(if camera_on_top { 1.0 } else { 0.0 });
  let effects = [geometry.radius as f32, 1.0, sigma, in_front.clamp(0.0, 1.0)];
  [frame, crop, effects]
}

/// Whether the screen and the camera are enhanced, then whether each is a
/// person cut out of their background.
pub(super) fn source_flags(source: &SourceTexture, camera: LayerCamera<'_>) -> [f32; 4] {
  let flag = |on: bool| f32::from(u8::from(on));
  let camera = |test: fn(&SourceTexture) -> bool| camera.is_some_and(|(camera, ..)| test(camera));
  [
    flag(source.enhanced),
    flag(camera(|camera| camera.enhanced)),
    flag(source.cut_out),
    flag(camera(|camera| camera.cut_out)),
  ]
}
