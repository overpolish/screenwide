// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captions made from a recording's transcript: the words as they read after
//! the Transcript panel's corrections, placed where the edited video plays
//! them, grouped into captions of a line or two, and timed on the edited
//! video word by word. The editor's lane, the subtitle files an export
//! writes and the captions drawn into the picture all come from here.

mod draw;
mod layout;
mod segment;
mod settings;
mod style;
mod subtitles;
mod text;
mod timing;
mod words;

use std::collections::HashSet;

use screenwide_lexicon::{find_fillers, find_profanity, ProfanityLexicon, Unit};
use screenwide_timeline::{output_duration_us, TimelineRange};
use serde::Serialize;
use ts_rs::TS;

pub use draw::{caption_pieces, CaptionPiece, CaptionPieceKind, CaptionTrack};
pub use layout::{CaptionLayout, CaptionPosition};
pub use settings::{
  CaptionBreak, CaptionBreakKind, CaptionProfanity, CaptionSettings, CaptionWordChoice,
  CaptionWordKind, RecordingCaptions,
};
pub use style::{
  rgba, CaptionBackground, CaptionEntrance, CaptionFont, CaptionLines, CaptionStyle,
  CaptionTemplate, MAX_SIZE_PERCENT, MAX_WEIGHT, MAX_WORDS, MIN_SIZE_PERCENT, MIN_WEIGHT,
  MIN_WORDS,
};
pub use subtitles::{srt, subtitles, vtt, SubtitleFile};
pub use words::{Correction, HeardWord};

use segment::{Shown, Widths};

/// One word of a caption, timed on the edited video and in the source.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionWord {
  pub text: String,
  pub start_ms: f64,
  pub end_ms: f64,
  pub source_start_ms: f64,
  pub source_end_ms: f64,
}

/// One caption: the words it shows, a line at a time, and when it shows on
/// the edited video, in milliseconds.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Caption {
  /// Where its first word starts in the source, in milliseconds: the same
  /// caption keeps it however the settings change what it holds.
  pub id: u64,
  pub start_ms: f64,
  pub end_ms: f64,
  /// Its lines' text, a line break between each.
  pub text: String,
  pub lines: Vec<Vec<CaptionWord>>,
  /// Whether a space parts its words, as most languages write them.
  pub spaced: bool,
}

/// What a recording's captions are made from.
pub struct CaptionSource<'a> {
  pub words: &'a [HeardWord],
  /// The language the transcriber heard, as its code.
  pub language: Option<&'a str>,
  /// Sorted by start and never overlapping.
  pub corrections: &'a [Correction],
  /// Folded text marked "Not profanity".
  pub allowed_words: &'a [String],
  /// What the edited video plays of the source, in the order it plays.
  pub ranges: &'a [TimelineRange],
  pub source_duration_ms: f64,
}

/// The size of the picture captions are set in, in pixels.
#[derive(Clone, Copy, Debug)]
pub struct CaptionFrame {
  pub width: u32,
  pub height: u32,
}

/// Caption type at 100%, as a share of the picture's shorter side: the same
/// size on a wide video and a tall one.
const TYPE_SHARE: f64 = 0.05;
/// How much of the picture's width a caption line may take.
const LINE_WIDTH_SHARE: f64 = 0.8;
/// The longest a caption line runs, in ems of its type: about the 42
/// characters subtitle guidelines read comfortably in one glance.
const LINE_EMS: f64 = 23.0;

/// The type size captions are set at in `frame`, in its pixels.
pub fn type_size_px(style: &CaptionStyle, frame: CaptionFrame) -> f64 {
  f64::from(frame.width.min(frame.height)) * TYPE_SHARE * style.normalized().size_percent / 100.0
}

/// The widest a caption line may be set in `frame` at `type_size`, both in
/// its pixels.
pub fn max_line_width(frame: CaptionFrame, type_size: f64) -> f64 {
  (f64::from(frame.width) * LINE_WIDTH_SHARE).min(type_size * LINE_EMS)
}

/// Which words each kept token is, by index: a filler, and profanity.
fn marks(
  tokens: &[words::Token],
  source: &CaptionSource,
  settings: &CaptionSettings,
) -> (Vec<bool>, Vec<bool>) {
  let units: Vec<Unit> = tokens
    .iter()
    .map(|token| Unit {
      text: &token.text,
      start_ms: token.source_start_ms,
      end_ms: token.source_end_ms,
      removed: token.played.is_none(),
    })
    .collect();
  let mut fillers = vec![false; tokens.len()];
  if settings.hide_fillers {
    for found in find_fillers(&units, source.language) {
      fillers[found.first..=found.last].fill(true);
    }
  }
  let mut profanity = vec![false; tokens.len()];
  if settings.censor_profanity {
    let lexicon = ProfanityLexicon::for_language(source.language);
    let allowed: HashSet<String> = source.allowed_words.iter().cloned().collect();
    for found in find_profanity(&units, &lexicon, &allowed) {
      profanity[found.first..=found.last].fill(true);
    }
  }
  (fillers, profanity)
}

/// The words the captions show, in the order the edited video plays them:
/// those the rules keep, less the ones `choices` hides, plus the ones it
/// shows as said.
fn shown(
  source: &CaptionSource,
  settings: &CaptionSettings,
  choices: &[CaptionWordChoice],
) -> Vec<Shown> {
  let tokens = words::tokens(source.words, source.corrections, source.ranges);
  let (fillers, profanity) = marks(&tokens, source, settings);
  let share = |ms: f64| ms / source.source_duration_ms;
  let mut played: Vec<usize> = (0..tokens.len())
    .filter(|&index| tokens[index].played.is_some())
    .collect();
  // Tokens are in source order, so the next played index after a token is
  // the word said next of those the edit keeps.
  let said_next: Vec<Option<usize>> = {
    let mut next = vec![None; tokens.len()];
    for pair in played.windows(2) {
      next[pair[0]] = Some(pair[1]);
    }
    next
  };
  played.sort_by(|&a, &b| {
    let (a_range, ..) = tokens[a].played.unwrap_or_default();
    let (b_range, ..) = tokens[b].played.unwrap_or_default();
    a_range.cmp(&b_range).then(
      tokens[a]
        .source_start_ms
        .total_cmp(&tokens[b].source_start_ms),
    )
  });
  let mut shown = Vec::new();
  let mut follows = false;
  let mut previous: Option<usize> = None;
  for index in played {
    let token = &tokens[index];
    if let Some(previous) = previous {
      follows &= said_next[previous] == Some(index);
    }
    previous = Some(index);
    let choice = settings::word_choice(
      choices,
      share(token.source_start_ms),
      share(token.source_end_ms),
    );
    let ruled = choice != Some(CaptionWordKind::Shown);
    let text = text::caption_text(
      &token.text,
      settings,
      ruled && fillers[index],
      ruled && profanity[index],
    );
    let Some(text) = text.filter(|_| choice != Some(CaptionWordKind::Hidden)) else {
      continue;
    };
    let (_, output_start_ms, output_end_ms) = token.played.unwrap_or_default();
    shown.push(Shown {
      text,
      said: token.text.clone(),
      source_start_ms: token.source_start_ms,
      source_end_ms: token.source_end_ms,
      output_start_ms,
      output_end_ms,
      follows,
    });
    follows = true;
  }
  shown
}

/// The recording's captions, in the order the edited video shows them.
/// `measure` gives a text's width set at a type size, both in pixels.
pub fn build(
  source: &CaptionSource,
  captions: &RecordingCaptions,
  frame: CaptionFrame,
  measure: &mut dyn FnMut(&str, f64) -> f64,
) -> Vec<Caption> {
  if source.source_duration_ms <= 0.0 {
    return Vec::new();
  }
  let settings = captions.settings;
  let style = captions.style.normalized();
  let mut shown = shown(source, &settings, &captions.word_choices);
  if style.all_caps {
    for word in &mut shown {
      word.text = word.text.to_uppercase();
    }
  }
  let gap = text::word_gap(source.language);
  let size = type_size_px(&style, frame);
  let widths = Widths::new(&shown, gap, &mut |text| measure(text, size));
  let max_width = max_line_width(frame, size);
  let mut built: Vec<Caption> = segment::captions(
    &shown,
    &style,
    &captions.breaks,
    source.source_duration_ms,
    max_width,
    &widths,
  )
  .into_iter()
  .map(|words| {
    let lines: Vec<Vec<CaptionWord>> = segment::lines(words.clone(), &style, max_width, &widths)
      .into_iter()
      .map(|line| {
        shown[line]
          .iter()
          .map(|word| CaptionWord {
            text: word.text.clone(),
            start_ms: word.output_start_ms,
            end_ms: word.output_end_ms,
            source_start_ms: word.source_start_ms,
            source_end_ms: word.source_end_ms,
          })
          .collect()
      })
      .collect();
    let text = lines
      .iter()
      .map(|line| {
        line
          .iter()
          .map(|word| word.text.as_str())
          .collect::<Vec<_>>()
          .join(gap)
      })
      .collect::<Vec<_>>()
      .join("\n");
    Caption {
      id: shown[words.start].source_start_ms.round() as u64,
      start_ms: shown[words.start].output_start_ms,
      end_ms: shown[words.end - 1].output_end_ms,
      text,
      lines,
      spaced: !gap.is_empty(),
    }
  })
  .collect();
  let length_ms = output_duration_us(source.ranges).map_or(f64::INFINITY, |us| us as f64 / 1_000.0);
  timing::timed(&mut built, &settings, length_ms);
  built
}

#[cfg(test)]
mod draw_tests;
#[cfg(test)]
mod tests;
