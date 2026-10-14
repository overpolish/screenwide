// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pieces one caption draws at a moment: on its way on, read, or on its
//! way off once its time is over.

use super::place::{self, Placed};
use super::rolling::rolling_words;
use super::shown::{Moving, Shown};
use super::{full_hex, ink_on, CaptionMoment, CaptionPiece, CaptionPieceKind};
use crate::motion::Mirror;
use crate::style::{CaptionBackground, CaptionFace, CaptionStyle};
use crate::{Caption, CaptionFrame, CaptionWord, Measure};

/// A word as it is drawn: its place among the caption's words, where it
/// sits, where it sat as the frame's exposure began, and when it starts
/// going off.
pub(super) struct DrawnWord<'a> {
  pub(super) word: &'a CaptionWord,
  pub(super) index: usize,
  pub(super) placed: Placed,
  pub(super) previous: [f64; 2],
  pub(super) leaves_ms: f64,
}

/// The words a caption draws, each placed, with the box its lines take
/// where every word is set at once, and its own type size.
struct Words<'a> {
  drawn: Vec<DrawnWord<'a>>,
  whole: Option<([f64; 2], [f64; 2])>,
  size: f64,
}

/// `caption`'s words as `style` sets them at `moment`: every one in its
/// lines, going off with the caption from `exits_ms`, or a few at a time
/// rolling through.
fn words_of<'a>(
  caption: &'a Caption,
  style: &CaptionStyle,
  (frame, moment, exits_ms): (CaptionFrame, CaptionMoment, f64),
  measure: &mut Measure,
) -> Words<'a> {
  if let Some(visible) = style.rolling_words() {
    let (drawn, size) = rolling_words(caption, style, visible, (frame, moment, exits_ms), measure);
    return Words {
      drawn,
      whole: None,
      size,
    };
  }
  let placement = place::place(caption, style, frame, measure);
  Words {
    drawn: caption
      .lines
      .iter()
      .flatten()
      .zip(placement.words)
      .enumerate()
      .map(|(index, (word, placed))| DrawnWord {
        word,
        index,
        previous: placed.centre,
        placed,
        leaves_ms: exits_ms,
      })
      .collect(),
    whole: Some((placement.centre, placement.block)),
    size: placement.size,
  }
}

/// What `caption` draws at `moment` into the edited video, set as `style`
/// sets it, normalized, and its offsets from its edge turned round as
/// `mirror` says, in draw order.
pub(super) fn pieces_of(
  caption: &Caption,
  style: &CaptionStyle,
  frame: CaptionFrame,
  moment: CaptionMoment,
  measure: &mut Measure,
  mirror: Mirror,
) -> Vec<CaptionPiece> {
  let (at_ms, exposure_ms) = (moment.at_ms, moment.exposure_ms);
  let motion = &style.motion;
  // The caption goes off over the end of its time, so it is gone as its
  // time ends, where the next one may come on; one too short for its exit
  // goes off from its start.
  let exits_ms = (caption.end_ms - motion.exit_ms()).max(caption.start_ms);
  let words = words_of(caption, style, (frame, moment, exits_ms), measure);
  let caption_exit = (&motion.caption_out[..], at_ms - exits_ms);
  let unmoved = (&[][..], -1.0);
  // The caption comes on whole, unless its words come on one by one, and
  // goes off whole as well as word by word.
  let whole_entrance = Some((&motion.caption[..], at_ms - caption.start_ms));
  let revealing = style.reveal_words || words.whole.is_none();
  // Moved by its motion, in ems of its own type, now and as the exposure
  // began, from where it sat then.
  let piece = |kind,
               text: &str,
               [now, previous]: [[f64; 2]; 2],
               (size, face): (f64, CaptionFace),
               shown: Shown| {
    let moved =
      |at: [f64; 2], offset: [f64; 2]| [at[0] + offset[0] * size, at[1] + offset[1] * size];
    CaptionPiece {
      kind,
      text: text.to_owned(),
      face,
      centre: moved(now, shown.offset[0]),
      previous_centre: moved(previous, shown.offset[1]),
      size,
      scale: shown.scale,
      opacity: shown.opacity,
    }
  };
  // The word being said: the last that has started, held through the pause
  // after it until the next one starts, and on through the caption's exit.
  let said = words
    .drawn
    .iter()
    .filter(|drawn| drawn.word.start_ms <= at_ms)
    .map(|drawn| drawn.index)
    .max();

  let mut pieces = Vec::new();
  // Behind every line, or, as words are revealed, behind those said so far
  // and not yet going off on their own, growing with each.
  let backdrop = match (style.background, words.whole.filter(|_| !revealing)) {
    (CaptionBackground::Box, Some(whole)) => Some((whole, whole_entrance)),
    (CaptionBackground::Box, None) => place::block_of(
      words
        .drawn
        .iter()
        .filter(|drawn| {
          drawn.word.start_ms <= at_ms && (drawn.leaves_ms >= exits_ms || at_ms < drawn.leaves_ms)
        })
        .map(|drawn| &drawn.placed),
    )
    .map(|block| (block, None)),
    _ => None,
  };
  if let Some(((centre, block), entrance)) = backdrop {
    let moving = Moving {
      entrance,
      exits: [caption_exit, unmoved],
    };
    pieces.push(piece(
      CaptionPieceKind::Backdrop {
        color: full_hex(&style.background_color),
        block,
      },
      &caption.text,
      [centre; 2],
      (words.size, CaptionFace::Main),
      moving.shown(exposure_ms, mirror),
    ));
  }
  let highlight = full_hex(&style.highlight_color);
  let text_ink = full_hex(&style.text_color);
  let accent_ink = full_hex(&style.accent_color);
  let outline = style.outline.then(|| full_hex(&style.outline_color));
  let mut pill = None;
  let mut words_drawn = Vec::new();
  for drawn in &words.drawn {
    let (word, placed) = (drawn.word, &drawn.placed);
    let entrance = if !revealing {
      whole_entrance
    } else if word.start_ms > at_ms {
      continue;
    } else {
      Some((&motion.word[..], at_ms - word.start_ms))
    };
    let shown = Moving {
      entrance,
      exits: [caption_exit, (&motion.word_out, at_ms - drawn.leaves_ms)],
    }
    .shown(exposure_ms, mirror);
    let is_marked = said == Some(drawn.index) && style.highlight_words;
    let set = (placed.size, placed.face);
    let kind = if is_marked && style.background == CaptionBackground::Word {
      // The pill goes under every word, so a neighbour it reaches under
      // stays on top, and the word on it takes type that reads on it.
      pill = Some(piece(
        CaptionPieceKind::Pill {
          color: highlight.clone(),
        },
        &word.text,
        [placed.centre, drawn.previous],
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
    words_drawn.push(piece(
      kind,
      &word.text,
      [placed.centre, drawn.previous],
      set,
      shown,
    ));
  }
  pieces.extend(pill);
  pieces.extend(words_drawn);
  pieces
}
