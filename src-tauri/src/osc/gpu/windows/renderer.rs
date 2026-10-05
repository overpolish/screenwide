// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! CPU vertex builder for every Windows GPU OSC surface. It mirrors the
//! macOS OSC vertex builders. Geometry is pure math on top-left pixel
//! coordinates, so tool surfaces only provide their semantic scene.

mod geometry;
pub(crate) use geometry::add_line;
pub(crate) use geometry::add_quad;
pub(crate) use geometry::add_texture_quad;
use geometry::is_empty;
use geometry::ndc;
use geometry::push_quad;

use super::{RenderConstants, Vertex};
use crate::osc::geometry::{Point, Rect, Size};

/// Lens edge length in points; the box is this times the display scale.
pub(crate) const MAGNIFIER_BOX_POINTS: f64 = 96.0;

impl RenderConstants {
  /// Port of `screenwide_region_magnifier_make`. The box is sized and centred
  /// in physical pixels because the shader reads the fragment position
  /// directly.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn set_magnifier(
    &mut self,
    point: Point,
    scale: f64,
    edges: u32,
    source: (u32, u32),
    sample: (f32, f32),
    source_min: (f32, f32),
    source_max: (f32, f32),
  ) {
    let size = ((MAGNIFIER_BOX_POINTS * scale).round() as i64).max(1);
    let x = (point.x * scale).round() as i64 - size / 2;
    let y = (point.y * scale).round() as i64 - size / 2;
    self.magnifier_box = [x as f32, y as f32, size as f32, size as f32];
    self.magnifier_source = [source.0 as f32, source.1 as f32, scale as f32, 0.0];
    self.magnifier_sample = [unit(sample.0), unit(sample.1), 0.0, 0.0];
    self.magnifier_source_range = [
      unit(source_min.0),
      unit(source_min.1),
      unit(source_max.0),
      unit(source_max.1),
    ];
    self.magnifier_flags = [edges, 1, 0, 0];
  }

  pub(crate) fn clear_magnifier(&mut self) {
    self.magnifier_box = [0.0; 4];
    self.magnifier_flags = [0; 4];
  }
}

fn unit(value: f32) -> f32 {
  value.clamp(0.0, 1.0)
}

/// Port of `screenwide_region_magnifier_anchor`: snaps the lens centre to the
/// dragged edge, clamped inside the frame.
pub(crate) fn magnifier_anchor(point: Point, frame: Rect, edges: u32) -> Point {
  let x = if edges & 1 != 0 {
    frame.origin.x
  } else if edges & 2 != 0 {
    frame.right()
  } else {
    point.x
  };
  let y = if edges & 4 != 0 {
    frame.origin.y
  } else if edges & 8 != 0 {
    frame.bottom()
  } else {
    point.y
  };
  Point {
    x: x.clamp(frame.origin.x, frame.right()),
    y: y.clamp(frame.origin.y, frame.bottom()),
  }
}

/// Hairline snapping: land the core on a pixel centre.
pub(crate) fn snap(value: f64, scale: f64) -> f64 {
  ((value * scale).floor() + 0.5) / scale
}

/// Handle centres snap to whole pixels instead, so their circles stay round.
pub(crate) fn snap_handle_center(value: f64, scale: f64) -> f64 {
  (value * scale).round() / scale
}

fn snap_handle_point(point: Point, scale: f64) -> Point {
  Point {
    x: snap_handle_center(point.x, scale),
    y: snap_handle_center(point.y, scale),
  }
}

const UNIT_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

mod chrome;
mod pixel;
use pixel::rect_corners;
mod ruler;
mod selection;

pub(crate) use chrome::{add_icon, add_label, add_plate};
pub(crate) use pixel::{
  add_pixel_aligned_quad, add_pixel_aligned_texture_quad, pixel_aligned_rect,
};
pub(crate) use ruler::{add_ruler_arc, add_ruler_box};
pub(crate) use selection::{
  add_annotation_handles, add_crop, add_crop_with_handles, add_group_frame, add_selection,
  add_turned_selection,
};

/// The lens is emitted last as a quad over `magnifier_box`. It has its own
/// kind, 49, because crop corners use 45.
pub(crate) fn add_magnifier(out: &mut Vec<Vertex>, view: Size, constants: &RenderConstants) {
  let [x, y, width, height] = constants.magnifier_box;
  if constants.magnifier_flags[1] == 0 || width <= 0.0 || height <= 0.0 {
    return;
  }
  add_quad(
    out,
    view,
    Rect::from_xywh(
      f64::from(x),
      f64::from(y),
      f64::from(width),
      f64::from(height),
    ),
    49,
  );
}

#[cfg(test)]
mod tests;
