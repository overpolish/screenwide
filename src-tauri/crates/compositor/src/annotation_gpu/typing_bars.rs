// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The caret and the selection of text typed into word by word, a text box's
//! or a caption's: bars set where its words are laid out, so what is typed
//! lands where it shows.

use screenwide_text::captions::{CaptionPiece, CaptionPieceKind, Measure, TextItemLayout};
use screenwide_text::engine::typing::TypingMarks;
use screenwide_text::TextFace;

/// How wide the caret is, as a share of how tall: about a twentieth of an
/// em of the type beside it.
const CARET_SHARE: f64 = 0.04;
/// How much of the type's ink a selection lays behind the words, as it does
/// behind a box's type set in one face.
const SELECTION_SHARE: f32 = 0.28;

/// `color` written `#rrggbbaa`, its alpha `share` of its own.
fn hex([red, green, blue, alpha]: [f32; 4], share: f32) -> String {
  let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
  format!(
    "#{:02x}{:02x}{:02x}{:02x}",
    byte(red),
    byte(green),
    byte(blue),
    byte(alpha * share)
  )
}

/// A bar `[left, top, right, bottom]` from the block's corner `corner`, in
/// `color`, set beside type of `size`.
fn bar(corner: [f64; 2], rect: [f64; 4], color: String, size: f64) -> CaptionPiece {
  let centre = [
    corner[0] + (rect[0] + rect[2]) / 2.0,
    corner[1] + (rect[1] + rect[3]) / 2.0,
  ];
  CaptionPiece {
    kind: CaptionPieceKind::Bar {
      color,
      block: [rect[2] - rect[0], rect[3] - rect[1]],
    },
    text: String::new(),
    face: TextFace::Main,
    centre,
    previous_centre: centre,
    size,
    scale: [1.0; 2],
    opacity: [1.0; 2],
    turn: [[0.0; 3]; 2],
    line_turn: None,
    word: None,
  }
}

/// The selection's bars, which go under the words, and the caret, which
/// goes over them, of `text` laid out as `layout` with its block's top-left
/// corner at `corner`, typed into with `marks`, in the type's `ink`. `size`
/// is the type size the bars are set beside.
pub(crate) fn layout_bars(
  (layout, corner): (&TextItemLayout, [f64; 2]),
  text: &str,
  (marks, ink): (TypingMarks, [f32; 4]),
  size: f64,
  measure: &mut Measure,
) -> (Vec<CaptionPiece>, Option<CaptionPiece>) {
  let selection = layout
    .selection(text, marks.start, marks.end, measure)
    .into_iter()
    .map(|rect| bar(corner, rect, hex(ink, SELECTION_SHARE), size))
    .collect();
  let caret = (marks.caret && marks.start == marks.end).then(|| {
    let [x, top, bottom] = layout.caret(text, marks.start, measure);
    let width = CARET_SHARE * (bottom - top);
    bar(corner, [x, top, x + width, bottom], hex(ink, 1.0), size)
  });
  (selection, caret)
}
