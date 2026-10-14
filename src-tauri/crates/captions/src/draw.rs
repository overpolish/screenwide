// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the captions draw at one moment of the edited video, laid out on the
//! canvas: the caption showing then, set line by line where its position
//! has it, a piece for each word so the one being said can stand out, and
//! the box or pill behind them as the style has it. The compositor draws
//! the pieces as text boxes of its own; everything about where they sit and
//! how they move is decided here, so preview and export agree.

use crate::motion::{pose, CaptionMotionTrack, CaptionProperty};
use crate::style::{rgba, CaptionBackground, CaptionFace, CaptionStyle};
use crate::{Caption, CaptionFrame, Measure, RecordingCaptions};

mod place;
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

/// How a piece is drawn at one moment of its motion: how large about its
/// centre and how solid, now and a frame ago, which the compositor blurs
/// between, and how far it is moved, in ems of its type.
#[derive(Clone, Copy)]
struct Shown {
  scale: [f64; 2],
  opacity: [f64; 2],
  offset: [f64; 2],
}

/// A piece `elapsed_ms` into `tracks`. A growing piece stays where it
/// settles, and the size a frame ago is the size now, so growth is drawn
/// sharp rather than blurred as an annotation's is; how solid it is blurs
/// across the frame.
fn shown(tracks: &[CaptionMotionTrack], elapsed_ms: f64) -> Shown {
  let now = pose(tracks, elapsed_ms);
  let before = pose(tracks, elapsed_ms - FRAME_MS);
  Shown {
    scale: [now.get(CaptionProperty::Scale); 2],
    opacity: [
      now.get(CaptionProperty::Opacity),
      before.get(CaptionProperty::Opacity),
    ],
    offset: [
      now.get(CaptionProperty::OffsetX),
      now.get(CaptionProperty::OffsetY),
    ],
  }
}

/// A piece at rest.
const STILL: Shown = Shown {
  scale: [1.0; 2],
  opacity: [1.0; 2],
  offset: [0.0; 2],
};

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
/// type as large as a video sets it.
pub fn caption_fit(
  caption: &Caption,
  style: &CaptionStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> f64 {
  let block = place::place(caption, &style.normalized(), frame, measure).block;
  let room = place::room(frame);
  (0..2)
    .filter(|&axis| block[axis] > 0.0)
    .map(|axis| room[axis] / block[axis])
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
  let Some(caption) = showing(captions, at_ms).filter(|_| recording.enabled) else {
    return Vec::new();
  };
  let style = recording.style.normalized();
  let placement = place::place(caption, &style, frame, measure);
  // The caption comes on whole, unless its words come on one by one.
  let whole = shown(&style.motion.caption, at_ms - caption.start_ms);
  let piece = |kind, text: &str, at: [f64; 2], (size, face): (f64, CaptionFace), shown: Shown| {
    CaptionPiece {
      kind,
      text: text.to_owned(),
      face,
      // Moved by its motion, in ems of its own type.
      centre: [
        at[0] + shown.offset[0] * size,
        at[1] + shown.offset[1] * size,
      ],
      size,
      scale: shown.scale,
      opacity: shown.opacity,
    }
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
  // Behind every line, or, as words are revealed, behind those said so far,
  // growing with each.
  let backdrop = match (style.background, style.reveal_words) {
    (CaptionBackground::Box, false) => Some((placement.centre, placement.block, whole)),
    (CaptionBackground::Box, true) => place::block_of(
      caption
        .lines
        .iter()
        .flatten()
        .zip(&placement.words)
        .filter(|(word, _)| word.start_ms <= at_ms)
        .map(|(_, placed)| placed),
    )
    .map(|(centre, block)| (centre, block, STILL)),
    _ => None,
  };
  if let Some((centre, block, shown)) = backdrop {
    pieces.push(piece(
      CaptionPieceKind::Backdrop {
        color: full_hex(&style.background_color),
        block,
      },
      &caption.text,
      centre,
      (placement.size, CaptionFace::Main),
      shown,
    ));
  }
  let highlight = full_hex(&style.highlight_color);
  let text_ink = full_hex(&style.text_color);
  let accent_ink = full_hex(&style.accent_color);
  let outline = style.outline.then(|| full_hex(&style.outline_color));
  let mut pill = None;
  let mut words_drawn = Vec::new();
  for (index, (word, placed)) in caption
    .lines
    .iter()
    .flatten()
    .zip(&placement.words)
    .enumerate()
  {
    let shown = if !style.reveal_words {
      whole
    } else if word.start_ms > at_ms {
      continue;
    } else {
      shown(&style.motion.word, at_ms - word.start_ms)
    };
    let is_marked = said == Some(index) && style.highlight_words;
    let set = (placed.size, placed.face);
    let kind = if is_marked && style.background == CaptionBackground::Word {
      // The pill goes under every word, so a neighbour it reaches under
      // stays on top, and the word on it takes type that reads on it.
      pill = Some(piece(
        CaptionPieceKind::Pill {
          color: highlight.clone(),
        },
        &word.text,
        placed.centre,
        set,
        shown,
      ));
      // The pill sets the word apart, so it needs no outline to read.
      CaptionPieceKind::Word {
        ink: ink_on(&highlight),
        outline: None,
      }
    } else {
      CaptionPieceKind::Word {
        ink: if is_marked {
          highlight.clone()
        } else if word.emphasis {
          accent_ink.clone()
        } else {
          text_ink.clone()
        },
        outline: outline.clone(),
      }
    };
    words_drawn.push(piece(kind, &word.text, placed.centre, set, shown));
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
