// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a text's look makes of it: the face its type is set in and at what
//! size, the text as it shows, and the tail the look draws to one of its
//! control points, from where the text has put that point or where the look
//! starts it.

use std::borrow::Cow;
use std::collections::BTreeMap;

use super::model::TextControlPoint;
use crate::AnnotationPoint;
use screenwide_text::engine::face::TypeFace;
use screenwide_text::{BoxPoint, ControlSpace, TailShape, TextControl, TextStyle};

/// The face `look` sets its type in.
pub fn face(look: &TextStyle) -> TypeFace {
  TypeFace::new(
    look.font.family.as_deref(),
    look.font.weight,
    look.font.italic,
  )
}

/// The type size `look` sets a text `width` points large at: the text's own
/// size, scaled by the look's.
pub fn type_size(look: &TextStyle, width: f64) -> f64 {
  width.max(0.0) * look.drawn_size_percent() / 100.0
}

/// `text` as `look` shows it.
pub fn shown<'a>(look: &TextStyle, text: &'a str) -> Cow<'a, str> {
  if look.all_caps {
    Cow::Owned(text.to_uppercase())
  } else {
    Cow::Borrowed(text)
  }
}

/// How much room `text` takes as `look` sets it at `width` points, in the
/// same units.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn block(look: &TextStyle, text: &str, width: f64) -> [f64; 2] {
  screenwide_text::engine::metrics::face_text_block(
    &face(look),
    &shown(look, text),
    type_size(look, width),
  )
}

/// The tail a look draws, the control point it is drawn to, and where that
/// point is.
#[derive(Clone, Debug, PartialEq)]
pub struct Tail<'a> {
  pub shape: TailShape,
  pub control: &'a TextControl,
  pub point: TextControlPoint,
}

impl Tail<'_> {
  /// Whether the tail reaches out of its box: one tucked into it is not
  /// drawn.
  pub fn is_out(&self) -> bool {
    match &self.point {
      TextControlPoint::Box(point) => point.is_out(),
      TextControlPoint::Picture(_) => true,
    }
  }
}

/// The tail `look` draws, with its point where `controls` put it. A point
/// stored in another space than the look now holds it in, as after a change
/// of template, starts again from the look's default.
pub fn tail<'a>(
  look: &'a TextStyle,
  controls: &BTreeMap<String, TextControlPoint>,
) -> Option<Tail<'a>> {
  let (control, shape) = look.tail()?;
  let point = match (control.space, controls.get(&control.id)) {
    (ControlSpace::Box, Some(TextControlPoint::Box(point))) => {
      TextControlPoint::Box(point.within(&control.limits))
    }
    (ControlSpace::Picture, Some(TextControlPoint::Picture(point))) => {
      TextControlPoint::Picture(*point)
    }
    _ => TextControlPoint::Box(control.default),
  };
  Some(Tail {
    shape,
    control,
    point,
  })
}

/// Where a box point is in the picture, against a box `size` large with its
/// top-left corner at `corner`, for type `em` large, all in one space.
pub fn box_point_at(
  corner: AnnotationPoint,
  size: [f64; 2],
  em: f64,
  point: &BoxPoint,
) -> AnnotationPoint {
  let axis = |start: f64, extent: f64, along: f64, reach: f64| {
    let half = extent / 2.0;
    start + half + along * half + (reach * em).copysign(along)
  };
  AnnotationPoint {
    x: axis(corner.x, size[0], point.along[0], point.reach[0]),
    y: axis(corner.y, size[1], point.along[1], point.reach[1]),
  }
}
