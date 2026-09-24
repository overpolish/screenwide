// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making a text box, and the dress a fresh one wears.
//!
//! A text box's `style.width` is its type size in output pixels: the box, its
//! padding, its corners and its pointer all follow from that and from the
//! text, so there is nothing else to size.

use crate::editor::annotations::AnnotationPoint;
#[cfg(any(target_os = "macos", target_os = "windows", test))]
use crate::editor::annotations::{
  Annotation, AnnotationAlign, AnnotationHead, AnnotationShape, AnnotationStyle,
};
use serde::{Deserialize, Serialize};

/// A text box's pointer, held against its box rather than in the picture, so
/// it keeps its place on the box however the box is moved, retyped or resized.
///
/// `along` is where the tip sits against the box in each axis, as a share of
/// the box's half size from its centre: -1 at the left or top edge, 1 at the
/// right or bottom. `reach` is how far past that edge the tip goes in each
/// axis, in ems of the box's type, and is zero in an axis the tip is inside
/// the box in. A pointer that reaches nowhere is not drawn, but keeps its
/// place, so its grip is where it was left.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPointer {
  pub along: AnnotationPoint,
  pub reach: AnnotationPoint,
}

impl Default for TextPointer {
  /// Tucked into the middle of the bottom edge, where a pointer is most often
  /// drawn out from.
  fn default() -> Self {
    Self {
      along: AnnotationPoint { x: 0.0, y: 1.0 },
      reach: AnnotationPoint::default(),
    }
  }
}

impl TextPointer {
  pub(crate) fn is_drawn(&self) -> bool {
    self.reach.x > 0.0 || self.reach.y > 0.0
  }

  /// The pointer as the native record carries it: one number per axis, a
  /// share of the half size up to 1 and one more for every em past the edge,
  /// so it rides in a single point slot. The twin of `text_pointer_axis` in
  /// [`super::geometry`], which reads it back.
  #[cfg(any(target_os = "macos", target_os = "windows", test))]
  pub(crate) fn encoded(&self) -> [f32; 2] {
    let axis = |along: f64, reach: f64| {
      let along = along.clamp(-1.0, 1.0);
      if reach > 0.0 {
        (1.0 + reach).copysign(along) as f32
      } else {
        along as f32
      }
    };
    [
      axis(self.along.x, self.reach.x),
      axis(self.along.y, self.reach.y),
    ]
  }

  fn finite(&self) -> bool {
    [self.along.x, self.along.y, self.reach.x, self.reach.y]
      .iter()
      .all(|value| value.is_finite())
  }
}

/// The type sizes the text control offers, in output pixels: two small steps
/// for labels, then steps that grow by a ratio so the large ones are big
/// enough to title a screen. The twin of `ANNOTATION_TEXT_SIZES` in
/// `src/components/shared/annotation-style/widths.ts`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const TEXT_SIZES: [f64; 5] = [20.0, 28.0, 48.0, 80.0, 128.0];

/// The type size a fresh text box is set at: the third step, large enough to
/// read at a glance on a Retina screenshot.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) const NEW_TEXT_WIDTH: f64 = TEXT_SIZES[2];

/// The dress a fresh text box is drawn in before anything has been chosen:
/// the same first colour every annotation takes.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn default_text_style() -> AnnotationStyle {
  AnnotationStyle {
    align: AnnotationAlign::Left,
    color: crate::editor::annotations::model::NEW_ANNOTATION_COLOR.to_owned(),
    head: AnnotationHead::None,
    width: NEW_TEXT_WIDTH,
  }
}

/// An empty text box centred on `point`, in `style` or in the tool's own
/// first dress, so its caret starts under the hand; typing grows it right and
/// down from there. `source_per_output` sizes the box in source pixels, and
/// where it is unknown the box's corner lands on `point` instead. Its pointer
/// is tucked in: one is drawn out of the box afterwards, by its grip.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn new_text(
  id: String,
  point: AnnotationPoint,
  style: Option<&AnnotationStyle>,
  source_per_output: f64,
) -> Annotation {
  let style = style.cloned().unwrap_or_else(default_text_style);
  let empty = super::snap::text_box(point, "", style.width, source_per_output);
  Annotation {
    above_camera: false,
    animated: true,
    id,
    reveal: crate::editor::annotations::reveal::AnnotationReveal::default(),
    shape: AnnotationShape::Text {
      origin: AnnotationPoint {
        x: point.x - empty.width / 2.0,
        y: point.y - empty.height / 2.0,
      },
      pointer: TextPointer::default(),
      text: String::new(),
    },
    style,
  }
}

/// Whether a text box is somewhere it can be drawn. A document read from disk
/// carries whatever it was written with.
pub(crate) fn placed(origin: AnnotationPoint, pointer: &TextPointer) -> bool {
  origin.x.is_finite() && origin.y.is_finite() && pointer.finite()
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
pub(crate) fn has_content(text: &str) -> bool {
  !text.trim().is_empty()
}

/// Whether a box in `style` has its text inked dark: the compositor's rule,
/// which reads the colour's luminance so the palette's yellow takes black and
/// its near-black takes white. The caret typed with follows it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) fn dark_ink(style: &AnnotationStyle) -> bool {
  let [red, green, blue, _] = crate::editor::annotations::model::annotation_colour(&style.color);
  0.2126 * red + 0.7152 * green + 0.0722 * blue > 0.6
}
