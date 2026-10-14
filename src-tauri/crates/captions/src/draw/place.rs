// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each word of a caption sits on the canvas: its lines set one
//! beneath another, each as tall as its largest word, the words of a line
//! standing on one baseline, and the block held where the caption's place
//! has it, every line lined up within it as the arrangement has it.

use super::LINE_HEIGHT;
use crate::style::{CaptionArrangement, CaptionFace, CaptionStyle};
use crate::{type_size_px, Caption, CaptionFrame, CaptionWord, Measure};

/// How far in from the picture's edge a caption keeps, as a share of its
/// height: the bottom of a caption's last line sits this far up from the
/// bottom, and on every other side the caption keeps as many pixels clear.
const EDGE_SHARE: f64 = 0.1;
/// How far above its baseline a word's middle sits, in ems of its type:
/// the middle of a line of the type the compositor sets a word in, which is
/// what a piece's centre places.
const MIDDLE_ABOVE_BASELINE: f64 = 0.36;
/// How much further along each line of a staircase starts than the one
/// above it, in ems of the caption's type.
const STAIR_EMS: f64 = 1.5;

/// One word as placed.
#[derive(Clone, Copy, Debug)]
pub(super) struct Placed {
  pub centre: [f64; 2],
  pub width: f64,
  /// The top and bottom of its line.
  pub line: [f64; 2],
  pub size: f64,
  pub face: CaptionFace,
}

/// A caption's words as placed, in order, and the block they take.
pub(super) struct Placement {
  pub words: Vec<Placed>,
  /// The block's centre and size: what a box behind the lines covers.
  pub centre: [f64; 2],
  pub block: [f64; 2],
  /// The caption's own type size, which a word set in the accent is scaled
  /// from.
  pub size: f64,
}

/// One line as measured, before it is placed.
struct Line {
  /// Each word's width, size and face.
  words: Vec<(f64, f64, CaptionFace)>,
  /// The gap after each word but the last.
  gaps: Vec<f64>,
  width: f64,
  /// Its largest word's size.
  size: f64,
}

/// The room a caption has in `frame`: the picture within its margins.
pub(super) fn room(frame: CaptionFrame) -> [f64; 2] {
  let edge = f64::from(frame.height) * EDGE_SHARE;
  [
    f64::from(frame.width) - 2.0 * edge,
    f64::from(frame.height) - 2.0 * edge,
  ]
}

/// Where `caption`'s words sit, set in its lines.
pub(super) fn place(
  caption: &Caption,
  style: &CaptionStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> Placement {
  place_lines(caption.lines.iter(), caption.spaced, style, frame, measure)
}

/// Where the words of `lines` sit, a space between each where `spaced`.
pub(super) fn place_lines<'w, L: IntoIterator<Item = &'w CaptionWord>>(
  lines: impl IntoIterator<Item = L>,
  spaced: bool,
  style: &CaptionStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> Placement {
  let size = type_size_px(style, frame);
  let size_in = |face| size * style.scale_of(face);
  // The space after a word is its own face's, and the wider of two
  // neighbours' where they differ.
  let mut space = |face| {
    if spaced {
      measure(" ", face, size_in(face))
    } else {
      0.0
    }
  };
  let spaces = [space(CaptionFace::Main), space(CaptionFace::Accent)];
  let space_of = |face| spaces[usize::from(face == CaptionFace::Accent)];
  let lines: Vec<Line> = lines
    .into_iter()
    .map(|line| {
      let words: Vec<(f64, f64, CaptionFace)> = line
        .into_iter()
        .map(|word| {
          let face = if word.emphasis {
            CaptionFace::Accent
          } else {
            CaptionFace::Main
          };
          (
            measure(&word.text, face, size_in(face)),
            size_in(face),
            face,
          )
        })
        .collect();
      let gaps: Vec<f64> = words
        .windows(2)
        .map(|pair| space_of(pair[0].2).max(space_of(pair[1].2)))
        .collect();
      Line {
        width: words.iter().map(|word| word.0).sum::<f64>() + gaps.iter().sum::<f64>(),
        size: words.iter().map(|word| word.1).fold(0.0, f64::max),
        words,
        gaps,
      }
    })
    .collect();
  let staircase = style.arrangement == CaptionArrangement::Staircase;
  let indent = |line: usize| {
    if staircase {
      line as f64 * STAIR_EMS * size
    } else {
      0.0
    }
  };
  let block = [
    lines
      .iter()
      .enumerate()
      .map(|(index, line)| line.width + indent(index))
      .fold(0.0, f64::max),
    lines
      .iter()
      .map(|line| line.size * LINE_HEIGHT)
      .sum::<f64>(),
  ];
  let [across, down] = style.position.shares();
  let edge = f64::from(frame.height) * EDGE_SHARE;
  let room = room(frame);
  let left = edge + (room[0] - block[0]) * across;
  let top = edge + (room[1] - block[1]) * down;
  let mut words = Vec::new();
  let mut line_top = top;
  for (index, line) in lines.iter().enumerate() {
    // Each line lines up within the caption as the caption does within the
    // picture: from its left, about its middle, or from its right; a
    // staircase steps away from the side it is held to.
    let mut x = if !staircase {
      left + (block[0] - line.width) * across
    } else if across > 0.5 {
      left + block[0] - line.width - indent(index)
    } else {
      left + indent(index)
    };
    let pitch = line.size * LINE_HEIGHT;
    let middle = line_top + pitch / 2.0;
    for (at, &(width, word_size, face)) in line.words.iter().enumerate() {
      // A smaller word sits lower, on the largest one's baseline.
      let drop = MIDDLE_ABOVE_BASELINE * (line.size - word_size);
      words.push(Placed {
        centre: [x + width / 2.0, middle + drop],
        width,
        line: [line_top, line_top + pitch],
        size: word_size,
        face,
      });
      x += width + line.gaps.get(at).copied().unwrap_or(0.0);
    }
    line_top += pitch;
  }
  Placement {
    words,
    centre: [left + block[0] / 2.0, top + block[1] / 2.0],
    block,
    size,
  }
}

/// The centre and size of the block `words` take: what a box behind the
/// words shown so far covers, as they are revealed. None for no words.
pub(super) fn block_of<'a>(
  words: impl Iterator<Item = &'a Placed>,
) -> Option<([f64; 2], [f64; 2])> {
  let [left, right, top, bottom] = words.fold(
    [
      f64::INFINITY,
      f64::NEG_INFINITY,
      f64::INFINITY,
      f64::NEG_INFINITY,
    ],
    |[left, right, top, bottom], word| {
      [
        left.min(word.centre[0] - word.width / 2.0),
        right.max(word.centre[0] + word.width / 2.0),
        top.min(word.line[0]),
        bottom.max(word.line[1]),
      ]
    },
  );
  (left <= right).then(|| {
    (
      [(left + right) / 2.0, (top + bottom) / 2.0],
      [right - left, bottom - top],
    )
  })
}
