// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a recording's captions are asked to be: the settings that hold
//! whatever the template, their look, and the breaks placed by hand. The
//! edit keeps them all, so they undo with it and travel with the project.

use crate::captions::Caption;
use crate::layout::TextLayout;
use crate::style::TextStyle;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a caption shows of a word found to be profanity.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionProfanity {
  /// The first letter, the rest starred out.
  #[default]
  Mask,
  /// Every letter starred out.
  Hide,
  /// Nothing: the word is left out.
  Remove,
}

/// What the captions say and when, whatever they look like.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionSettings {
  /// A caption stays up until the next one starts, through any silence
  /// short of a stretch with nothing said.
  pub no_gaps: bool,
  pub remove_punctuation: bool,
  pub hide_fillers: bool,
  pub censor_profanity: bool,
  pub profanity: CaptionProfanity,
}

/// What a break placed by hand does to the gap between two words.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionBreakKind {
  /// A new caption starts here.
  Split,
  /// The words either side stay in one caption.
  Join,
}

/// A break placed by hand between two words, at a moment in the recording
/// as a share of its length: anywhere between the middle of the word before
/// and the middle of the word after names that gap, so a break survives a
/// word's edges settling differently.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionBreak {
  pub source_at: f64,
  pub kind: CaptionBreakKind,
}

/// Whether a word chosen by hand shows in the captions, whatever the rules
/// would do with it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionWordKind {
  /// Left out, as a filler the rules missed.
  Hidden,
  /// Shown as said, as a word the rules took for a filler or for profanity
  /// to remove.
  Shown,
}

/// A word hidden from the captions or shown in them by hand, at a moment in
/// the recording as a share of its length: anywhere within the word names
/// it, as a break names its gap.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionWordChoice {
  pub source_at: f64,
  pub kind: CaptionWordKind,
}

/// Whether a word chosen by hand is set apart in the accent, whatever the
/// rule would pick.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionEmphasisKind {
  /// Set apart, in place of the word the rule picks in its caption.
  Emphasized,
  /// Never set apart, so the rule picks another.
  Plain,
}

/// A word set apart by hand or kept plain, at a moment in the recording as
/// a share of its length, as a word choice is placed.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionEmphasisChoice {
  pub source_at: f64,
  pub kind: CaptionEmphasisKind,
}

/// What `choices`, sorted by `source_at` of each, asks of the word said
/// from `start` to `end`, both shares of the recording: the choice placed
/// within it, if any.
pub(crate) fn word_choice<T: Copy>(
  choices: &[T],
  source_at: impl Fn(&T) -> f64,
  start: f64,
  end: f64,
) -> Option<T> {
  let at = choices.partition_point(|choice| source_at(choice) < start);
  choices
    .get(at)
    .filter(|choice| source_at(choice) <= end)
    .copied()
}

/// A recording's captions as the edit keeps them.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingCaptions {
  /// Whether the captions show. Turning them off keeps everything else.
  pub enabled: bool,
  pub settings: CaptionSettings,
  /// How they look and move.
  pub style: TextStyle,
  /// The layout options set by hand, which `style` already holds: kept as
  /// they are when another template is chosen.
  #[serde(skip_serializing_if = "TextLayout::is_empty")]
  #[ts(optional, as = "Option<TextLayout>")]
  pub layout: TextLayout,
  /// Sorted by `source_at`, at most one at a moment.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<CaptionBreak>>")]
  pub breaks: Vec<CaptionBreak>,
  /// Sorted by `source_at`, at most one at a moment.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<CaptionWordChoice>>")]
  pub word_choices: Vec<CaptionWordChoice>,
  /// Sorted by `source_at`, at most one at a moment.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<CaptionEmphasisChoice>>")]
  pub emphasis_choices: Vec<CaptionEmphasisChoice>,
  /// Captions taken out of the video by hand, each named by its id: where
  /// its first word starts in the source, in milliseconds. Whichever caption
  /// holds the word said then is left out, however the settings regroup the
  /// words. Sorted, none twice.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<u64>>")]
  pub hidden: Vec<u64>,
}

impl Default for RecordingCaptions {
  /// Off, saying everything, in the default template as it ships.
  fn default() -> Self {
    Self {
      enabled: false,
      settings: CaptionSettings::default(),
      style: crate::templates::TextTemplateId::default().shipped_style(),
      layout: TextLayout::default(),
      breaks: Vec::new(),
      word_choices: Vec::new(),
      emphasis_choices: Vec::new(),
      hidden: Vec::new(),
    }
  }
}

/// More than any recording has gaps between words: a cap on what a damaged
/// file may ask to be read, of breaks and of word choices alike.
const MAX_BREAKS: usize = 100_000;

/// Whether `moments` are shares of the recording, in order, none twice, and
/// no more than a damaged file could hold.
fn are_ordered(moments: impl ExactSizeIterator<Item = f64>) -> bool {
  let count = moments.len();
  let mut previous = -1.0;
  for at in moments {
    if !(0.0..=1.0).contains(&at) || at <= previous {
      return false;
    }
    previous = at;
  }
  count <= MAX_BREAKS
}

impl RecordingCaptions {
  pub fn validate(&self) -> Result<(), String> {
    if !self.style.is_valid() {
      return Err("The timeline records an invalid caption style".to_owned());
    }
    if !are_ordered(
      self
        .breaks
        .iter()
        .map(|caption_break| caption_break.source_at),
    ) {
      return Err("The timeline records invalid caption breaks".to_owned());
    }
    if !are_ordered(self.word_choices.iter().map(|choice| choice.source_at)) {
      return Err("The timeline records invalid caption word choices".to_owned());
    }
    if !are_ordered(self.emphasis_choices.iter().map(|choice| choice.source_at)) {
      return Err("The timeline records invalid caption emphasis choices".to_owned());
    }
    if self.hidden.len() > MAX_BREAKS || !self.hidden.is_sorted_by(|a, b| a < b) {
      return Err("The timeline records invalid hidden captions".to_owned());
    }
    Ok(())
  }

  /// `built` without every caption that holds a hidden word: one whose start,
  /// rounded as a caption's id is, a hidden id names. Matched word by word
  /// rather than by span, as the caption before often ends just as the
  /// hidden one starts. Done once the captions are timed, so those either
  /// side keep their times rather than taking up the hidden one's.
  pub(crate) fn without_hidden(&self, mut built: Vec<Caption>) -> Vec<Caption> {
    if self.hidden.is_empty() {
      return built;
    }
    built.retain(|caption| {
      !caption.lines.iter().flatten().any(|word| {
        self
          .hidden
          .binary_search(&(word.source_start_ms.round() as u64))
          .is_ok()
      })
    });
    built
  }
}
