// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which words show together. A caption never runs across a place the edit
//! joins two parts of the recording that were not said one after the other,
//! the end of a sentence, a long pause, or a break placed by hand. Between
//! those, the words are shared out over as few captions as fit, of about the
//! same size, each ending at the gap near that size that reads most like an
//! ending.

use std::ops::Range;

use crate::settings::{CaptionBreak, CaptionBreakKind, CaptionEmphasisKind};
use crate::style::{CaptionArrangement, CaptionStyle};
use crate::text::break_score;
use crate::widths::{Emphasis, Widths};

/// A word a caption shows, placed on the edited video.
#[derive(Clone, Debug)]
pub(crate) struct Shown {
  /// As the caption shows it.
  pub text: String,
  /// As it was said, which decides where a caption reads best ending.
  pub said: String,
  pub source_start_ms: f64,
  pub source_end_ms: f64,
  pub output_start_ms: f64,
  pub output_end_ms: f64,
  /// Whether the edited video plays it straight after the word shown
  /// before it, as it was said.
  pub follows: bool,
  /// Whether it was set apart in the accent by hand, or kept plain.
  pub emphasis: Option<CaptionEmphasisKind>,
}

/// How many words a stacked line holds at most.
const STACK_WORDS: usize = 2;

/// A pause at least this long, on the edited video, always ends a caption.
const LONG_PAUSE_MS: f64 = 1_000.0;
/// How strongly the end of a sentence asks for a break: always heeded.
const SENTENCE_END: u8 = 3;

/// What the gap before each word asks for: `Some(true)` a caption must start
/// there, `Some(false)` it must not, `None` either may happen. A caption
/// never runs past the end of a sentence or a long pause, unless joined
/// there by hand.
fn gaps(words: &[Shown], breaks: &[CaptionBreak], source_duration_ms: f64) -> Vec<Option<bool>> {
  let share = |word: &Shown| (word.source_start_ms + word.source_end_ms) / 2.0 / source_duration_ms;
  words
    .iter()
    .enumerate()
    .map(|(index, word)| {
      let Some(before) = index.checked_sub(1).map(|before| &words[before]) else {
        return Some(true);
      };
      if !word.follows {
        return Some(true);
      }
      let (from, to) = (share(before), share(word));
      let placed = breaks
        .iter()
        .find(|caption_break| from < caption_break.source_at && caption_break.source_at <= to);
      match placed.map(|caption_break| caption_break.kind) {
        Some(CaptionBreakKind::Split) => Some(true),
        Some(CaptionBreakKind::Join) => Some(false),
        None => {
          let pause = word.output_start_ms - before.output_end_ms;
          (pause >= LONG_PAUSE_MS || break_score(&before.said, 0.0) == SENTENCE_END).then_some(true)
        }
      }
    })
    .collect()
}

/// Lays the caption `words` out in lines as the style arranges them.
pub(crate) fn lines(
  words: Range<usize>,
  style: &CaptionStyle,
  max_width: f64,
  widths: &Widths,
) -> Vec<Range<usize>> {
  let emphasis = widths.emphasis(&words);
  match style.arrangement {
    CaptionArrangement::Lines => in_lines(words, style, max_width, widths, emphasis),
    CaptionArrangement::Stack | CaptionArrangement::Staircase => {
      stacked(words, max_width, widths, emphasis)
    }
  }
}

/// In at most the style's lines: on one line where they fit on one,
/// otherwise split where the longer line is shortest.
fn in_lines(
  words: Range<usize>,
  style: &CaptionStyle,
  max_width: f64,
  widths: &Widths,
  emphasis: Emphasis,
) -> Vec<Range<usize>> {
  let width = |line: Range<usize>| widths.of(&line, emphasis);
  if style.line_count() < 2 || words.len() < 2 || width(words.clone()) <= max_width {
    return vec![words];
  }
  let longer = |at: usize| width(words.start..at).max(width(at..words.end));
  let split = (words.start + 1..words.end)
    .min_by(|&a, &b| longer(a).total_cmp(&longer(b)))
    .unwrap_or(words.end);
  vec![words.start..split, split..words.end]
}

/// A word or two a line, in order, with each word set apart on a line of
/// its own; two words too wide together take a line each.
fn stacked(
  words: Range<usize>,
  max_width: f64,
  widths: &Widths,
  emphasis: Emphasis,
) -> Vec<Range<usize>> {
  let mut lines: Vec<Range<usize>> = Vec::new();
  let pairs = |lines: &mut Vec<Range<usize>>, run: Range<usize>| {
    let mut start = run.start;
    while start < run.end {
      let pair = start..(start + STACK_WORDS).min(run.end);
      let line = if widths.of(&pair, emphasis) <= max_width {
        pair
      } else {
        start..start + 1
      };
      start = line.end;
      lines.push(line);
    }
  };
  let mut run = words.start;
  for index in words.clone() {
    if widths.is_emphasized(emphasis, index) {
      pairs(&mut lines, run..index);
      lines.push(index..index + 1);
      run = index + 1;
    }
  }
  pairs(&mut lines, run..words.end);
  lines
}

/// Whether `words` fit one caption: within the word limit, and set in its
/// lines with none wider than `max_width`. One word always fits.
fn fits(words: Range<usize>, style: &CaptionStyle, max_width: f64, widths: &Widths) -> bool {
  if words.len() <= 1 {
    return true;
  }
  let emphasis = widths.emphasis(&words);
  words.len() <= style.max_words as usize
    && lines(words, style, max_width, widths)
      .iter()
      .all(|line| widths.of(line, emphasis) <= max_width)
}

/// How strongly the gap before `words[index]` reads as an ending.
fn score_before(words: &[Shown], index: usize) -> u8 {
  let before = &words[index - 1];
  break_score(
    &before.said,
    words[index].output_start_ms - before.output_end_ms,
  )
}

/// The words each caption shows, as index ranges into `words`.
pub(crate) fn captions(
  words: &[Shown],
  style: &CaptionStyle,
  breaks: &[CaptionBreak],
  source_duration_ms: f64,
  max_width: f64,
  widths: &Widths,
) -> Vec<Range<usize>> {
  let gaps = gaps(words, breaks, source_duration_ms);
  let may_end = |index: usize| gaps[index] != Some(false);
  let mut captions = Vec::new();
  let mut start = 0;
  while start < words.len() {
    // The run a caption may come from ends where one must start.
    let run_end = (start + 1..words.len())
      .find(|&index| gaps[index] == Some(true))
      .unwrap_or(words.len());
    let mut end = start + 1;
    while end < run_end && fits(start..end + 1, style, max_width, widths) {
      end += 1;
    }
    if end < run_end {
      // The run is split into as few captions as fit, of about the same size,
      // so the last is never a word left over; each ends at the gap near
      // that size that reads most like an ending, the nearest of equals.
      let fit = end - start;
      let left = run_end - start;
      let target = start + left.div_ceil(left.div_ceil(fit));
      let slack = (fit / 4).max(1);
      let near = target.saturating_sub(slack).max(start + 1)..=(target + slack).min(end);
      if let Some(best) = near.filter(|&index| may_end(index)).max_by_key(|&index| {
        (
          score_before(words, index),
          usize::MAX - index.abs_diff(target),
          index,
        )
      }) {
        end = best;
      }
      // A gap joined by hand never ends a caption: it ends at the nearest
      // gap before that may, or else the first one after.
      if !may_end(end) {
        end = (start + 1..end)
          .rev()
          .find(|&index| may_end(index))
          .or_else(|| (end + 1..run_end).find(|&index| may_end(index)))
          .unwrap_or(run_end);
      }
    }
    captions.push(start..end);
    start = end;
  }
  captions
}
