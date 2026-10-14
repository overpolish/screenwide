// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the captions draw at one moment of the edited video, laid out on the
//! canvas: the caption showing then, set line by line where its position
//! has it, a piece for each word so the one being said can stand out, and
//! the box or pill behind them as the style has it. The compositor draws
//! the pieces as text boxes of its own; everything about where they sit and
//! how they move is decided here, so preview and export agree.

use crate::motion::Mirror;
use crate::style::{rgba, CaptionFace, CaptionStyle};
use crate::{Caption, CaptionFrame, Measure, RecordingCaptions};

mod caption;
mod place;
mod rolling;
mod shown;
mod upcoming;

/// A line's height, in ems: the twin of `LINE_HEIGHT` in the annotations'
/// text metrics, which the compositor sets the type at.
const LINE_HEIGHT: f64 = 1.25;
/// A frame at 60 frames a second, which a moving piece blurs across.
const FRAME_MS: f64 = 1_000.0 / 60.0;

/// What a piece of a caption is. Colours are written `#rrggbbaa`.
#[derive(Clone, Debug, PartialEq)]
pub enum CaptionPieceKind {
  /// The box behind the lines, around a block of text `block` in size, with
  /// no type of its own.
  Backdrop { color: String, block: [f64; 2] },
  /// One word's type in `ink`, with an outline in `outline` where it has
  /// one.
  Word {
    ink: String,
    outline: Option<String>,
  },
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
  /// The face its type is set in, and a word's pill measured in.
  pub face: CaptionFace,
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
    measure: &mut Measure,
  ) -> Vec<CaptionPiece> {
    caption_pieces(&self.captions, &self.recording, frame, at_ms, measure)
  }
}

/// The caption showing at `at_ms`, if any. `captions` are in order and
/// never overlap.
fn showing(captions: &[Caption], at_ms: f64) -> Option<&Caption> {
  let after = captions.partition_point(|caption| caption.start_ms <= at_ms);
  let caption = captions.get(after.checked_sub(1)?)?;
  (at_ms < caption.end_ms).then_some(caption)
}

/// The share of its size `caption` takes to fit in `frame` within the
/// picture's margins, set as `style` sets it: 1 where it fits already. A
/// style's preview sets its sample at this, where the tile is too small for
/// type as large as a video sets it. A caption rolling through its words
/// need only fit as many as show at once.
pub fn caption_fit(
  caption: &Caption,
  style: &CaptionStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> f64 {
  let style = style.normalized();
  let room = place::room(frame);
  let fit = |block: [f64; 2]| {
    (0..2)
      .filter(|&axis| block[axis] > 0.0)
      .map(|axis| room[axis] / block[axis])
      .fold(1.0, f64::min)
  };
  let Some(visible) = style.rolling_words() else {
    return fit(place::place(caption, &style, frame, measure).block);
  };
  let words: Vec<_> = caption.lines.iter().flatten().collect();
  words
    .windows(visible.min(words.len()).max(1))
    .map(|window| {
      let line = [window.iter().copied()];
      fit(place::place_lines(line, caption.spaced, &style, frame, measure).block)
    })
    .fold(1.0, f64::min)
}

/// What the captions draw `at_ms` into the edited video, on a canvas
/// `frame` in size. `measure` gives a text's width set in a face at a type
/// size, both in pixels, by the type the compositor sets the pieces in.
pub fn caption_pieces(
  captions: &[Caption],
  recording: &RecordingCaptions,
  frame: CaptionFrame,
  at_ms: f64,
  measure: &mut Measure,
) -> Vec<CaptionPiece> {
  if !recording.enabled {
    return Vec::new();
  }
  let style = recording.style.normalized();
  // An offset measured from the caption's edge turns round for one along
  // the top or at the right; through the middle there is no edge to come
  // from, so it moves as along the bottom.
  let [across, down] = style.position.shares();
  let mirror = Mirror {
    across: across >= 1.0,
    down: down <= 0.0,
  };
  let Some(caption) = showing(captions, at_ms) else {
    return Vec::new();
  };
  caption::pieces_of(caption, &style, frame, at_ms, measure, mirror)
}

/// `color` written out as `#rrggbbaa`, which an annotation's style carries.
fn full_hex(color: &str) -> String {
  let [red, green, blue, alpha] = rgba(color)
    .unwrap_or([1.0; 4])
    .map(|value| (value * 255.0).round() as u8);
  format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}
