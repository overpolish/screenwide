// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the captions draw at one moment of the edited video, laid out on the
//! canvas: the caption showing then, set line by line where its position
//! has it, a piece for each word so the one being said can stand out, and
//! the box or pill behind them as the style has it. The compositor draws
//! the pieces as text boxes of its own; everything about where they sit and
//! how they move is decided here, so preview and export agree.

use crate::style::{rgba, CaptionBackground, CaptionEntrance};
use crate::{type_size_px, Caption, CaptionFrame, RecordingCaptions};

/// A line's height, in ems: the twin of `LINE_HEIGHT` in the annotations'
/// text metrics, which the compositor sets the type at.
const LINE_HEIGHT: f64 = 1.25;
/// How far in from the picture's edge a caption keeps, as a share of its
/// height: the bottom of a caption's last line sits this far up from the
/// bottom, and on every other side the caption keeps as many pixels clear.
const EDGE_SHARE: f64 = 0.1;
/// A frame at 60 frames a second, which a moving piece blurs across.
const FRAME_MS: f64 = 1_000.0 / 60.0;
/// How long a popping caption takes to grow in, from three quarters of its
/// size past its own and back.
const POP_MS: f64 = 220.0;
const POP_FROM: f64 = 0.75;
/// How far past its size a popping caption grows: the back ease's
/// overshoot, about 5% at the top.
const POP_OVERSHOOT: f64 = 2.5;
/// How long a popping caption takes to become solid, at the start of its
/// growth.
const POP_FADE_MS: f64 = 50.0;
/// How long a word coming on as it is said takes to fade in.
const WORD_FADE_MS: f64 = 90.0;

/// What a piece of a caption is. Colours are written `#rrggbbaa`.
#[derive(Clone, Debug, PartialEq)]
pub enum CaptionPieceKind {
  /// The box behind the lines, the size the caption's text sets it at, with
  /// no type of its own.
  Backdrop { color: String },
  /// One word's type in `ink`, with a dark outline where `outlined`.
  Word { ink: String, outlined: bool },
  /// A pill behind the word being said, a little wider than the word, with
  /// no type of its own.
  Pill { color: String },
}

/// Type that reads on `color`: black on a light one, white on a dark one.
fn ink_on(color: &str) -> String {
  let [red, green, blue, _] = rgba(color).unwrap_or([0.0; 4]);
  let light = 0.2126 * red + 0.7152 * green + 0.0722 * blue > 0.5;
  if light { "#000000ff" } else { "#ffffffff" }.to_owned()
}

/// One piece of the caption showing, in canvas pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionPiece {
  pub kind: CaptionPieceKind,
  pub text: String,
  /// The centre of its text block.
  pub centre: [f64; 2],
  /// The type size it is set at, before `scale`.
  pub size: f64,
  /// How large it is drawn about its centre now and a frame ago, which the
  /// compositor blurs between.
  pub scale: [f64; 2],
  /// How solid it is now and a frame ago.
  pub opacity: [f64; 2],
}

/// A recording's captions as an export or a preview draws them: made once
/// from the transcript and the edit, then laid out frame by frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaptionTrack {
  pub captions: Vec<Caption>,
  pub recording: RecordingCaptions,
}

impl CaptionTrack {
  /// What the track draws `at_ms` into the edited video, on a canvas
  /// `frame` in size; see [`caption_pieces`].
  pub fn pieces_at(
    &self,
    frame: CaptionFrame,
    at_ms: f64,
    measure: &mut dyn FnMut(&str, f64) -> f64,
  ) -> Vec<CaptionPiece> {
    caption_pieces(&self.captions, &self.recording, frame, at_ms, measure)
  }
}

/// The back ease: past 1 and back to it, `overshoot` setting how far.
fn ease_out_back(progress: f64, overshoot: f64) -> f64 {
  let from_end = progress - 1.0;
  1.0 + (overshoot + 1.0) * from_end.powi(3) + overshoot * from_end.powi(2)
}

/// A Pop caption's size and solidity `elapsed_ms` after it starts.
fn pop(elapsed_ms: f64) -> (f64, f64) {
  let progress = (elapsed_ms / POP_MS).clamp(0.0, 1.0);
  let scale = POP_FROM + (1.0 - POP_FROM) * ease_out_back(progress, POP_OVERSHOOT);
  (scale, (elapsed_ms / POP_FADE_MS).clamp(0.0, 1.0))
}

/// The caption showing at `at_ms`, if any. `captions` are in order and
/// never overlap.
fn showing(captions: &[Caption], at_ms: f64) -> Option<&Caption> {
  let after = captions.partition_point(|caption| caption.start_ms <= at_ms);
  let caption = captions.get(after.checked_sub(1)?)?;
  (at_ms < caption.end_ms).then_some(caption)
}

/// What the captions draw `at_ms` into the edited video, on a canvas
/// `frame` in size. `measure` gives a text's width set at a type size, both
/// in pixels, by the type the compositor sets the pieces in.
pub fn caption_pieces(
  captions: &[Caption],
  recording: &RecordingCaptions,
  frame: CaptionFrame,
  at_ms: f64,
  measure: &mut dyn FnMut(&str, f64) -> f64,
) -> Vec<CaptionPiece> {
  let Some(caption) = showing(captions, at_ms).filter(|_| recording.enabled) else {
    return Vec::new();
  };
  let style = recording.style.normalized();
  let size = type_size_px(&style, frame);
  let pitch = size * LINE_HEIGHT;
  let gap = if caption.spaced {
    measure(" ", size)
  } else {
    0.0
  };
  let widths: Vec<Vec<f64>> = caption
    .lines
    .iter()
    .map(|words| words.iter().map(|word| measure(&word.text, size)).collect())
    .collect();
  let line_widths: Vec<f64> = widths
    .iter()
    .map(|line| line.iter().sum::<f64>() + gap * line.len().saturating_sub(1) as f64)
    .collect();
  let block = [
    line_widths.iter().copied().fold(0.0, f64::max),
    pitch * caption.lines.len() as f64,
  ];
  let [across, down] = style.position.shares();
  let edge = f64::from(frame.height) * EDGE_SHARE;
  let room = [
    f64::from(frame.width) - 2.0 * edge,
    f64::from(frame.height) - 2.0 * edge,
  ];
  let left = edge + (room[0] - block[0]) * across;
  let top = edge + (room[1] - block[1]) * down;
  let centre = [left + block[0] / 2.0, top + block[1] / 2.0];
  let elapsed = at_ms - caption.start_ms;
  let (scale, opacity) = if style.entrance == CaptionEntrance::Pop {
    let (now, then) = (pop(elapsed), pop((elapsed - FRAME_MS).max(0.0)));
    ([now.0, then.0], [now.1, then.1])
  } else {
    ([1.0; 2], [1.0; 2])
  };
  // Pieces move away from the caption's centre as it grows, so it grows as
  // one block rather than each word about its own middle.
  let placed = |at: [f64; 2]| {
    [
      centre[0] + (at[0] - centre[0]) * scale[0],
      centre[1] + (at[1] - centre[1]) * scale[0],
    ]
  };
  let piece = |kind, text: &str, at: [f64; 2], opacity: [f64; 2]| CaptionPiece {
    kind,
    text: text.to_owned(),
    centre: placed(at),
    size,
    scale,
    opacity,
  };
  // The word being said: the last that has started, held through the pause
  // after it until the next one starts.
  let said = caption
    .lines
    .iter()
    .flatten()
    .enumerate()
    .filter(|(_, word)| word.start_ms <= at_ms)
    .map(|(index, _)| index)
    .last();

  let mut pieces = Vec::new();
  if style.background == CaptionBackground::Box {
    pieces.push(piece(
      CaptionPieceKind::Backdrop {
        color: full_hex(&style.background_color),
      },
      &caption.text,
      centre,
      opacity,
    ));
  }
  let highlight = full_hex(&style.highlight_color);
  let text_ink = full_hex(&style.text_color);
  let mut pill = None;
  let mut words_drawn = Vec::new();
  let mut index = 0;
  for (line, words) in caption.lines.iter().enumerate() {
    // Each line lines up within the caption as the caption does within the
    // picture: from its left, about its middle, or from its right.
    let mut left = left + (block[0] - line_widths[line]) * across;
    let y = top + pitch * (line as f64 + 0.5);
    for (word, width) in words.iter().zip(&widths[line]) {
      let at = [left + width / 2.0, y];
      left += width + gap;
      let is_said = said == Some(index);
      index += 1;
      let opacity = match style.entrance {
        CaptionEntrance::Words if word.start_ms > at_ms => continue,
        CaptionEntrance::Words => {
          let fade = |ms: f64| ((ms - word.start_ms) / WORD_FADE_MS).clamp(0.0, 1.0);
          [fade(at_ms), fade(at_ms - FRAME_MS)]
        }
        _ => opacity,
      };
      let kind = if is_said && style.background == CaptionBackground::Word {
        // The pill goes under every word, so a neighbour it reaches under
        // stays on top, and the word on it takes type that reads on it.
        pill = Some(piece(
          CaptionPieceKind::Pill {
            color: highlight.clone(),
          },
          &word.text,
          at,
          opacity,
        ));
        // The pill sets the word apart, so it needs no outline to read.
        CaptionPieceKind::Word {
          ink: ink_on(&highlight),
          outlined: false,
        }
      } else {
        CaptionPieceKind::Word {
          ink: if is_said && style.highlight_words {
            highlight.clone()
          } else {
            text_ink.clone()
          },
          outlined: style.outline,
        }
      };
      words_drawn.push(piece(kind, &word.text, at, opacity));
    }
  }
  pieces.extend(pill);
  pieces.extend(words_drawn);
  pieces
}

/// `color` written out as `#rrggbbaa`, which an annotation's style carries.
fn full_hex(color: &str) -> String {
  let [red, green, blue, alpha] = rgba(color)
    .unwrap_or([1.0; 4])
    .map(|value| (value * 255.0).round() as u8);
  format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}
