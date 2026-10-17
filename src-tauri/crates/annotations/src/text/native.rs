// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box in the retained draw record, and the look its type is drawn
//! in beside it.
//!
//! A text box puts its top-left corner in `p0`; its tail's tip in `p1`,
//! against the box as [`encoded`] carries it or, where `head` has
//! [`HEAD_TAIL_PLACED`], the source pixel it was put on; and its text block,
//! measured at the record's type size `width`, in `p2`. The alignment and the
//! tail's bits ride in `head`, the tail's shape in `params`, the shown text
//! in the side buffer, and the rest of its look in the looks beside the
//! records, which `p3` names.

use std::collections::BTreeMap;

use super::geometry::{HEAD_TAIL, HEAD_TAIL_PLACED};
use super::look;
use super::model::{encoded, TextClock, TextControlPoint};
use crate::native::NativeAnnotation;
use crate::{annotation_colour, AnnotationPoint};
use screenwide_text::captions::moves_text;
use screenwide_text::engine::face::TypeFace;
use screenwide_text::{rgba, TailShape, TextBackground, TextShadow, TextSpan, TextStyle};

/// How a text box's type is drawn, which its record has no room for.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeTextLook {
  pub face: TypeFace,
  /// Whether the type sits on the record's box, in the record's colour.
  /// Without one the record's colour is the type's.
  pub boxed: bool,
  pub ink: [f32; 4],
  /// The outline's colour, which the shader draws a little see-through,
  /// where the type has one.
  pub outline: Option<[f32; 4]>,
  pub shadow: Option<TextShadow>,
  /// The text as its words, where it is drawn word by word: where its look
  /// moves it at a recording's frame, or sets chosen words apart.
  pub words: Option<NativeTextWords>,
}

/// A text drawn word by word, as a caption's words are: in its look whole,
/// with the words of `emphasis` set apart, and where the look moves it, at
/// `clock` in its clip.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeTextWords {
  pub look: Box<TextStyle>,
  /// The text as typed, which every byte range here is into.
  pub text: String,
  pub emphasis: Vec<TextSpan>,
  pub clock: Option<TextClock>,
}

impl NativeTextLook {
  fn of(look: &TextStyle, (text, emphasis): (&str, &[TextSpan]), clock: Option<TextClock>) -> Self {
    let clock = clock.filter(|_| moves_text(look));
    let apart = look::sets_words_apart(look, text, emphasis);
    Self {
      face: look::face(look),
      boxed: look.background == TextBackground::Box,
      ink: rgba(look.ink()).unwrap_or([1.0; 4]),
      outline: look.outline.then(|| rgba(&look.outline_color)).flatten(),
      shadow: look.shadow.clone(),
      words: (clock.is_some() || apart).then(|| NativeTextWords {
        look: Box::new(look.clone()),
        text: text.to_owned(),
        emphasis: if apart { emphasis.to_vec() } else { Vec::new() },
        clock,
      }),
    }
  }
}

/// The record's points for `text` with the words of `emphasis` set apart,
/// its type set at `font`.
pub(crate) fn draw_points(
  origin: AnnotationPoint,
  (text, emphasis): (&str, &[TextSpan]),
  look: &TextStyle,
  controls: &BTreeMap<String, TextControlPoint>,
  font: f64,
) -> [[f32; 2]; 3] {
  let block = look::item_block(look, text, emphasis, font);
  let tip = match look::tail(look, controls).map(|tail| tail.point) {
    Some(TextControlPoint::Box(point)) => encoded(&point),
    Some(TextControlPoint::Picture(point)) => [point.x as f32, point.y as f32],
    None => [0.0, 1.0],
  };
  [
    [origin.x as f32, origin.y as f32],
    tip,
    [block[0] as f32, block[1] as f32],
  ]
}

/// The record's `head`: the alignment, and whether a tail is drawn and its
/// tip placed in the picture.
pub(crate) fn head(
  look: &TextStyle,
  controls: &BTreeMap<String, TextControlPoint>,
  align: u32,
) -> u32 {
  align
    | match look::tail(look, controls).map(|tail| tail.point) {
      Some(TextControlPoint::Box(_)) => HEAD_TAIL,
      Some(TextControlPoint::Picture(_)) => HEAD_TAIL | HEAD_TAIL_PLACED,
      None => 0,
    }
}

/// Fills what a text's record carries beyond its points, and keeps its look
/// in `looks`, named by the record's `p3`, one past its place there so a
/// record with none reads zero.
pub(crate) fn fill(
  record: &mut NativeAnnotation,
  looks: &mut Vec<NativeTextLook>,
  (look, controls): (&TextStyle, &BTreeMap<String, TextControlPoint>),
  words: (&str, &[TextSpan]),
  clock: Option<TextClock>,
) {
  let shape = look::tail(look, controls).map_or_else(TailShape::default, |tail| tail.shape);
  record.params = [shape.base, shape.tip, shape.blend];
  record.color = annotation_colour(if look.background == TextBackground::Box {
    &look.background_color
  } else {
    look.ink()
  });
  record.p3 = [(looks.len() + 1) as f32, 0.0];
  looks.push(NativeTextLook::of(look, words, clock));
}

/// The look a text's record names, if it names one.
pub fn look_of<'a>(
  record: &NativeAnnotation,
  looks: &'a [NativeTextLook],
) -> Option<&'a NativeTextLook> {
  let named = record.p3[0] as usize;
  named.checked_sub(1).and_then(|index| looks.get(index))
}
