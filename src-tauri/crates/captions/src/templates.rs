// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The caption templates the app ships: each a style under a name. Those
//! set in a family other than Inter use one the app carries, so they look
//! the same on every system. Whether the word being said is highlighted or
//! the words are revealed as they are said is every template's to change;
//! each starts with what suits it.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::CaptionPosition;
use crate::size::CaptionSize;
use crate::style::{
  CaptionAccent, CaptionArrangement, CaptionBackground, CaptionEntrance, CaptionFont, CaptionLines,
  CaptionStyle, BLACK, SHADE, WHITE, YELLOW,
};

/// A caption template.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionTemplate {
  /// Type on a dark rounded box: plain subtitles that read over anything.
  #[default]
  Box,
  /// Outlined type and no box, keeping the picture clear. Also what the
  /// templates no longer shipped are read as: a style travels whole beside
  /// its template, so only which tile shows chosen changes.
  #[serde(alias = "reveal", alias = "subtitle", alias = "duet")]
  Clean,
  /// Big type a few words at a time, each caption popping in where it sits,
  /// from three quarters of its size past it and settling, and swapped out
  /// at once for the next.
  Pop,
  /// The word being said on a bright pill of its own.
  Highlight,
  /// Heavy capitals, the word being said lit up in its colour.
  Karaoke,
  /// A condensed display face a word or two at a time, popping in, the
  /// word being said in yellow.
  Punch,
  /// A quiet sans with one word of each caption set in a serif italic.
  Editorial,
  /// Heavy capitals with one word of each caption set far larger.
  Supersize,
  /// A word or two a line, one beneath another as they are said, the key
  /// word set large on a line of its own.
  Stack,
  /// Lines that each start a step further along, building as they are said.
  Staircase,
}

/// The serif italic accents are set in, which reads small beside a sans.
fn serif_italic() -> CaptionFont {
  CaptionFont {
    family: Some("Instrument Serif".to_owned()),
    weight: 400,
    italic: true,
  }
}

impl CaptionTemplate {
  pub const ALL: [Self; 10] = [
    Self::Box,
    Self::Clean,
    Self::Pop,
    Self::Highlight,
    Self::Karaoke,
    Self::Punch,
    Self::Editorial,
    Self::Supersize,
    Self::Stack,
    Self::Staircase,
  ];

  /// The style the template ships with.
  pub fn style(self) -> CaptionStyle {
    let plain = CaptionStyle {
      template: self,
      font: CaptionFont::default(),
      size_percent: 100.0,
      size: CaptionSize::Medium,
      all_caps: false,
      max_words: 8,
      lines: CaptionLines::Two,
      arrangement: CaptionArrangement::Lines,
      position: CaptionPosition::Bottom,
      text_color: WHITE.to_owned(),
      highlight_color: YELLOW.to_owned(),
      background_color: SHADE.to_owned(),
      highlight_words: false,
      reveal_words: false,
      outline: true,
      outline_color: BLACK.to_owned(),
      background: CaptionBackground::None,
      entrance: CaptionEntrance::None,
      accent: CaptionAccent::None,
      accent_font: serif_italic(),
      accent_size_percent: 120.0,
      accent_size: CaptionSize::Medium,
      accent_color: YELLOW.to_owned(),
    };
    match self {
      Self::Box => CaptionStyle {
        outline: false,
        background: CaptionBackground::Box,
        ..plain
      },
      Self::Clean => plain,
      Self::Pop => CaptionStyle {
        font: CaptionFont {
          weight: 800,
          ..CaptionFont::default()
        },
        size_percent: 140.0,
        all_caps: true,
        max_words: 3,
        lines: CaptionLines::One,
        entrance: CaptionEntrance::Pop,
        ..plain
      },
      Self::Highlight => CaptionStyle {
        font: CaptionFont {
          weight: 800,
          ..CaptionFont::default()
        },
        size_percent: 120.0,
        all_caps: true,
        max_words: 4,
        lines: CaptionLines::One,
        background: CaptionBackground::Word,
        highlight_words: true,
        ..plain
      },
      Self::Karaoke => CaptionStyle {
        font: CaptionFont::upright("Montserrat", 800),
        size_percent: 110.0,
        all_caps: true,
        max_words: 6,
        highlight_color: "#4ade80".to_owned(),
        highlight_words: true,
        ..plain
      },
      Self::Punch => CaptionStyle {
        font: CaptionFont::upright("Anton", 400),
        size_percent: 170.0,
        all_caps: true,
        max_words: 2,
        lines: CaptionLines::One,
        position: CaptionPosition::Center,
        highlight_words: true,
        entrance: CaptionEntrance::Pop,
        ..plain
      },
      Self::Editorial => CaptionStyle {
        font: CaptionFont::upright("DM Sans", 600),
        size_percent: 110.0,
        max_words: 6,
        accent: CaptionAccent::Word,
        accent_size_percent: 130.0,
        accent_color: WHITE.to_owned(),
        ..plain
      },
      Self::Supersize => CaptionStyle {
        font: CaptionFont::upright("Montserrat", 900),
        size_percent: 110.0,
        all_caps: true,
        max_words: 5,
        entrance: CaptionEntrance::Pop,
        accent: CaptionAccent::Word,
        accent_font: CaptionFont::upright("Montserrat", 900),
        accent_size_percent: 170.0,
        ..plain
      },
      Self::Stack => CaptionStyle {
        font: CaptionFont::upright("Montserrat", 800),
        size_percent: 110.0,
        all_caps: true,
        max_words: 6,
        arrangement: CaptionArrangement::Stack,
        position: CaptionPosition::Center,
        reveal_words: true,
        accent: CaptionAccent::Word,
        accent_font: CaptionFont::upright("Anton", 400),
        accent_size_percent: 200.0,
        ..plain
      },
      Self::Staircase => CaptionStyle {
        font: CaptionFont::upright("DM Sans", 700),
        size_percent: 115.0,
        max_words: 6,
        arrangement: CaptionArrangement::Staircase,
        position: CaptionPosition::Center,
        reveal_words: true,
        accent: CaptionAccent::Word,
        accent_size_percent: 135.0,
        accent_color: WHITE.to_owned(),
        ..plain
      },
    }
  }
}
