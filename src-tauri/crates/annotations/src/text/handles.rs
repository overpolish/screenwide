// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box's grips, as the native chrome needs them.

use std::collections::BTreeMap;

use super::look;
use super::model::TextControlPoint;
use crate::handles::{normalised_point, stroke_width, NativeAnnotationHandles};
use crate::{AnnotationKind, AnnotationPoint, AnnotationStyle};
use screenwide_text::{TailShape, TextStyle};

/// The text a box's grips are read from.
pub(crate) struct TextGrips<'a> {
  pub origin: AnnotationPoint,
  pub text: &'a str,
  pub look: &'a TextStyle,
  pub controls: &'a BTreeMap<String, TextControlPoint>,
}

/// A text box reads the arrow's slots its own way: `start` is the box's
/// top-left corner, normalised over the source; `end` is its tail's tip as
/// the draw record's `p1` carries it - against the box and never normalised,
/// or normalised over the source where it is placed in the picture;
/// `middle` is the text block's width and height and `width` the type size,
/// all as shares of the drawn width, so the native side places the box and
/// its tail in isotropic display points; `start_head` is the record's
/// `head`, its alignment and its tail's bits; `radius`, `tilt_x` and
/// `tilt_y` are the tail's base and tip radii and its blend, in ems.
pub(crate) fn grips(
  text: TextGrips<'_>,
  style: &AnnotationStyle,
  index: u32,
  source: (u32, u32),
  image_width: f64,
) -> NativeAnnotationHandles {
  let TextGrips {
    origin,
    text,
    look,
    controls,
  } = text;
  let (start_x, start_y) = normalised_point(origin, source);
  let font = look::type_size(look, style.width);
  let [_, [end_x, end_y], block] = super::native::draw_points(origin, text, look, controls, font);
  let head = super::native::head(look, controls, style.align.raw());
  let (end_x, end_y) = match look::tail(look, controls).map(|tail| tail.point) {
    Some(TextControlPoint::Picture(point)) => normalised_point(point, source),
    _ => (f64::from(end_x), f64::from(end_y)),
  };
  let shape = look::tail(look, controls).map_or_else(TailShape::default, |tail| tail.shape);
  let share = stroke_width(style, image_width);
  // The share of the drawn width one output pixel of type is.
  let per_output = if style.width > 0.0 {
    share / style.width
  } else {
    0.0
  };
  NativeAnnotationHandles {
    layer_id: -1,
    index,
    kind: AnnotationKind::Text.raw(),
    flags: 0,
    start_x,
    start_y,
    middle_x: f64::from(block[0]) * per_output,
    middle_y: f64::from(block[1]) * per_output,
    end_x,
    end_y,
    start_head: f64::from(head),
    end_head: 0.0,
    width: font * per_output,
    radius: f64::from(shape.base),
    tilt_x: f64::from(shape.tip),
    tilt_y: f64::from(shape.blend),
  }
}
