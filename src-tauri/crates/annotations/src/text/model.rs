// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a text box, the dress a fresh one wears, and where its look's
//! control points are.
//!
//! A text box's `style.width` is its type size in points before its look's
//! own size: the box, its padding, its corners and its tail all follow from
//! that and from the text, so there is nothing else to size.

use crate::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::{Annotation, AnnotationAlign, AnnotationHead, AnnotationShape, AnnotationStyle};
use screenwide_text::{BoxPoint, TextBackground, TextStyle, TextTemplateId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The template a text takes its look from until another is chosen.
pub const LABEL_TEMPLATE: &str = "label";

/// The look a text box takes where none is stored: the Label template as it
/// ships, which draws the solid box text boxes have always been.
pub fn label_look() -> Box<TextStyle> {
  Box::new(TextTemplateId::from(LABEL_TEMPLATE).shipped_style())
}

/// Where one of a look's control points has been moved to: against the
/// text's box, or, for a point held in the picture, the source pixel it was
/// put on.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(tag = "space", rename_all = "kebab-case")]
#[ts(export)]
pub enum TextControlPoint {
  Box(BoxPoint),
  Picture(AnnotationPoint),
}

impl TextControlPoint {
  fn finite(&self) -> bool {
    match self {
      Self::Box(point) => point
        .along
        .iter()
        .chain(&point.reach)
        .all(|value| value.is_finite()),
      Self::Picture(point) => point.x.is_finite() && point.y.is_finite(),
    }
  }
}

/// How far into its clip a text is at the frame being drawn, and how long
/// the clip is and the frame's exposure, all in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextClock {
  pub elapsed_ms: f32,
  pub duration_ms: f32,
  pub frame_ms: f32,
}

/// A box point as the native record carries it: one number per axis, a
/// share of the half size up to 1 and one more for every em past the edge,
/// so it rides in a single point slot. The twin of `text_pointer_axis` in
/// [`super::geometry`], which reads it back.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn encoded(point: &BoxPoint) -> [f32; 2] {
  let axis = |along: f64, reach: f64| {
    let along = along.clamp(-1.0, 1.0);
    if reach > 0.0 {
      (1.0 + reach).copysign(along) as f32
    } else {
      along as f32
    }
  };
  [
    axis(point.along[0], point.reach[0]),
    axis(point.along[1], point.reach[1]),
  ]
}

/// `look` wearing `color` where a text's one colour goes: its box's, where
/// it has one, otherwise its type's.
pub fn dressed_in(look: &mut TextStyle, color: &str) {
  if look.background == TextBackground::Box {
    look.background_color = color.to_owned();
  } else {
    look.text_color = color.to_owned();
  }
}

/// The type sizes the text control offers, in points: two small steps for
/// labels, then steps that grow by a ratio so the large ones are big enough
/// to title a screen. The twin of `ANNOTATION_TEXT_SIZES` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const TEXT_SIZES: [f64; 5] = [10.0, 14.0, 24.0, 40.0, 64.0];

/// The type size a fresh text box is set at: the third step, large enough to
/// read at a glance.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_TEXT_WIDTH: f64 = TEXT_SIZES[2];

/// The dress a fresh text box is drawn in before anything has been chosen:
/// the same first colour every annotation takes.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_text_style() -> AnnotationStyle {
  AnnotationStyle {
    align: AnnotationAlign::Left,
    blur: false,
    color: crate::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::None,
    hand_drawn: false,
    manual: false,
    tint: false,
    radius: 0.0,
    redaction: Default::default(),
    shadow: false,
    softness: 0.0,
    strength: 0.0,
    width: NEW_TEXT_WIDTH,
  }
}

/// An empty text box centred on `point`, in `style` or in the tool's own
/// first dress, and in `look` or else the Label look worn in that dress's
/// colour, so its caret starts under the hand; typing grows it right and down
/// from there.
/// `source_per_size` sizes the box in source pixels, and where it is unknown
/// the box's corner lands on `point` instead. Its control points start where
/// the look puts them: a tail is drawn out afterwards, by its grip.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn new_text(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
  look: Option<&TextStyle>,
  source_per_size: f64,
) -> Annotation {
  let style = style.cloned().unwrap_or_else(default_text_style);
  let look = match look {
    Some(look) => Box::new(look.clone()),
    None => {
      let mut look = label_look();
      dressed_in(&mut look, &style.color);
      look
    }
  };
  let empty = super::snap::text_box(point, "", &look, &[], style.width, source_per_size);
  Annotation {
    above_camera: false,
    animated: true,
    id,
    held: None,
    pen: false,
    reveal: crate::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Text {
      origin: AnnotationPoint {
        x: point.x - empty.width / 2.0,
        y: point.y - empty.height / 2.0,
      },
      text: String::new(),
      look,
      controls: Default::default(),
      emphasis: Vec::new(),
      clock: None,
    },
    style,
  }
}

/// Whether a text box is somewhere it can be drawn. A document read from disk
/// carries whatever it was written with.
pub(crate) fn placed<'a>(
  origin: AnnotationPoint,
  mut controls: impl Iterator<Item = &'a TextControlPoint>,
) -> bool {
  origin.x.is_finite() && origin.y.is_finite() && controls.all(TextControlPoint::finite)
}

/// Text as a text box holds it: every line break a single `\n`, so a
/// paste from a Windows document measures and draws the same lines as one
/// typed here.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn normalised_text(text: &str) -> String {
  text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Whether the text holds anything to show. A box committed with nothing but
/// spaces and line breaks in it is a box nobody can read, so it is removed
/// the way an empty one is.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub fn has_content(text: &str) -> bool {
  !text.trim().is_empty()
}
