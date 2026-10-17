// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box's type drawn word by word, as a caption's words are: where its
//! look moves it at a recording's frame, or sets chosen words apart. A box
//! being typed into stands still, and its caret and selection are bars set
//! where the same words put them, so what is typed lands where it shows.

use std::borrow::Cow;

use super::super::captions::{caption_arrows, caption_measure, CaptionFaces};
use super::super::typing_bars::layout_bars;
use super::super::PreparedArrows;
use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::text::native::NativeTextWords;
use screenwide_text::captions::{
  text_item_pieces, CaptionMoment, CaptionPiece, Measure, TextItemLayout, TextItemSetting,
};
use screenwide_text::engine::typing::TypingMarks;
use screenwide_text::{TextMotion, TextStyle};

/// Where a text's words are set: the middle of its block, its type size,
/// both in canvas pixels, and the record's alignment.
pub(super) struct WordsSetting {
  pub(super) centre: [f64; 2],
  pub(super) size: f64,
  pub(super) align: u32,
}

/// The selection's bars, which go under the words, and the caret, which
/// goes over them, of `words` typed into with `marks`, in the type's `ink`.
fn typing_bars(
  words: &NativeTextWords,
  look: &TextStyle,
  (setting, align): (&WordsSetting, f64),
  typing: (TypingMarks, [f32; 4]),
  measure: &mut Measure,
) -> (Vec<CaptionPiece>, Option<CaptionPiece>) {
  let text = &words.text;
  let layout = TextItemLayout::new(text, look, &words.emphasis, setting.size, align, measure);
  let corner = [0, 1].map(|axis| setting.centre[axis] - layout.block[axis] / 2.0);
  layout_bars((&layout, corner), text, typing, setting.size, measure)
}

/// `words` drawn at their clip's moment, or standing still where their look
/// does not move them or they are typed into with `typing`, each piece as
/// solid as the text's own reveal leaves it, so a text with no exit of its
/// own still goes with its box. `ink` is the type's colour, which the caret
/// and the selection are drawn in.
pub(super) fn word_pieces(
  words: &NativeTextWords,
  setting: &WordsSetting,
  reveal: AnnotationReveal,
  (typing, ink): (Option<TypingMarks>, [f32; 4]),
) -> PreparedArrows {
  let align = f64::from(setting.align.min(2)) / 2.0;
  let clock = words.clock.filter(|_| typing.is_none());
  let look = match clock {
    Some(_) => Cow::Borrowed(&*words.look),
    None => Cow::Owned(TextStyle {
      motion: TextMotion::default(),
      reveal_words: false,
      ..(*words.look).clone()
    }),
  };
  let (duration_ms, moment) = clock.map_or((0.0, CaptionMoment::still(0.0)), |clock| {
    (
      f64::from(clock.duration_ms),
      CaptionMoment {
        at_ms: f64::from(clock.elapsed_ms),
        exposure_ms: f64::from(clock.frame_ms),
      },
    )
  });
  let faces = CaptionFaces::of(&look);
  let mut measure = caption_measure(&faces);
  let item = TextItemSetting {
    centre: setting.centre,
    size: setting.size,
    align,
    duration_ms,
  };
  let mut pieces = text_item_pieces(
    &words.text,
    &look,
    &words.emphasis,
    item,
    moment,
    &mut measure,
  );
  if let Some(marks) = typing {
    let (selection, caret) =
      typing_bars(words, &look, (setting, align), (marks, ink), &mut measure);
    pieces.splice(0..0, selection);
    pieces.extend(caret);
  }
  let present = [reveal.opacity, reveal.previous[3]].map(|share| f64::from(share.clamp(0.0, 1.0)));
  for piece in &mut pieces {
    piece.opacity = [piece.opacity[0] * present[0], piece.opacity[1] * present[1]];
  }
  caption_arrows(&pieces, &faces)
}
