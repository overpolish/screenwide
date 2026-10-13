// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How captions look and move. A style is a set of options, and a template
//! is a style the app ships under a name: choosing one takes up its options,
//! or the ones it was last saved with, and any of them can change after.
//! Everything here is a template's to set; what is said and when stays in
//! the recording's settings, whatever the template.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::CaptionPosition;

/// A caption template.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionTemplate {
  /// Type on a dark rounded box: plain subtitles that read over anything.
  #[default]
  Box,
  /// Outlined type and no box, keeping the picture clear.
  Clean,
  /// Big type a few words at a time, each caption popping in from three
  /// quarters of its size past its own and settling, and swapped out at
  /// once for the next.
  Pop,
  /// The word being said on a bright pill of its own.
  Highlight,
  /// Each word appearing as it is said, so the caption builds up with the
  /// voice.
  Reveal,
}

/// How many lines one caption may take.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionLines {
  One,
  #[default]
  Two,
}

/// What sits behind a caption's type.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionBackground {
  #[default]
  None,
  /// A rounded box behind every line, in the background colour.
  Box,
  /// A pill behind the word being said, in the highlight colour.
  Word,
}

/// How a caption comes on.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionEntrance {
  /// All at once.
  #[default]
  None,
  /// Growing in past its size and settling.
  Pop,
  /// A word at a time, as each is said.
  Words,
}

/// The face caption type is set in: a family installed on the system, or
/// none for the app's own Inter, at a weight on the CSS scale of 100 to 900.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionFont {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub family: Option<String>,
  pub weight: u16,
}

impl Default for CaptionFont {
  fn default() -> Self {
    Self {
      family: None,
      weight: 600,
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionStyle {
  /// The template the style was taken from, which its tile shows chosen.
  pub template: CaptionTemplate,
  pub font: CaptionFont,
  /// Type size, in percent of the size a caption is set at by default.
  pub size_percent: f64,
  pub all_caps: bool,
  /// The most words one caption holds, across its lines.
  pub max_words: u32,
  pub lines: CaptionLines,
  /// Where on the picture the caption sits, and so how its lines line up.
  pub position: CaptionPosition,
  /// The type's colour, as `#rrggbb` or `#rrggbbaa`.
  pub text_color: String,
  /// The colour that marks the word being said: its type, or the pill
  /// behind it.
  pub highlight_color: String,
  /// The box's colour, where there is one.
  pub background_color: String,
  /// Colour the word being said, where no pill marks it already.
  pub highlight_words: bool,
  /// A soft dark outline around the type, so it reads without a box.
  pub outline: bool,
  pub background: CaptionBackground,
  pub entrance: CaptionEntrance,
}

pub const MIN_WORDS: u32 = 1;
pub const MAX_WORDS: u32 = 20;
pub const MIN_SIZE_PERCENT: f64 = 25.0;
pub const MAX_SIZE_PERCENT: f64 = 400.0;
pub const MIN_WEIGHT: u16 = 100;
pub const MAX_WEIGHT: u16 = 900;

const WHITE: &str = "#ffffff";
/// A warm yellow, which reads on white type and on any picture.
const YELLOW: &str = "#ffd60a";
/// Black, a little see-through.
const SHADE: &str = "#0000009e";

impl Default for CaptionStyle {
  fn default() -> Self {
    CaptionTemplate::Box.style()
  }
}

impl CaptionTemplate {
  pub const ALL: [Self; 5] = [
    Self::Box,
    Self::Clean,
    Self::Pop,
    Self::Highlight,
    Self::Reveal,
  ];

  /// The style the template ships with.
  pub fn style(self) -> CaptionStyle {
    let plain = CaptionStyle {
      template: self,
      font: CaptionFont::default(),
      size_percent: 100.0,
      all_caps: false,
      max_words: 8,
      lines: CaptionLines::Two,
      position: CaptionPosition::Bottom,
      text_color: WHITE.to_owned(),
      highlight_color: YELLOW.to_owned(),
      background_color: SHADE.to_owned(),
      highlight_words: false,
      outline: true,
      background: CaptionBackground::None,
      entrance: CaptionEntrance::None,
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
          family: None,
          weight: 800,
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
          family: None,
          weight: 800,
        },
        size_percent: 120.0,
        all_caps: true,
        max_words: 4,
        lines: CaptionLines::One,
        background: CaptionBackground::Word,
        ..plain
      },
      Self::Reveal => CaptionStyle {
        size_percent: 110.0,
        max_words: 6,
        entrance: CaptionEntrance::Words,
        ..plain
      },
    }
  }
}

/// `color`'s red, green, blue and alpha, each from 0 to 1, if it is written
/// `#rrggbb` or `#rrggbbaa`.
pub fn rgba(color: &str) -> Option<[f32; 4]> {
  let digits = color.strip_prefix('#')?;
  if !matches!(digits.len(), 6 | 8) || !digits.is_ascii() {
    return None;
  }
  let channel = |at: usize| u8::from_str_radix(digits.get(at..at + 2)?, 16).ok();
  let alpha = if digits.len() == 8 { channel(6)? } else { 255 };
  Some([channel(0)?, channel(2)?, channel(4)?, alpha].map(|value| f32::from(value) / 255.0))
}

impl CaptionStyle {
  /// The style with every number brought into its range and every colour
  /// readable, so a file edited by hand still draws.
  pub fn normalized(&self) -> Self {
    let color = |value: &str, fallback: &str| {
      if rgba(value).is_some() {
        value
      } else {
        fallback
      }
      .to_owned()
    };
    Self {
      font: CaptionFont {
        family: self
          .font
          .family
          .clone()
          .filter(|family| !family.trim().is_empty()),
        weight: self.font.weight.clamp(MIN_WEIGHT, MAX_WEIGHT),
      },
      size_percent: if self.size_percent.is_finite() {
        self.size_percent.clamp(MIN_SIZE_PERCENT, MAX_SIZE_PERCENT)
      } else {
        100.0
      },
      max_words: self.max_words.clamp(MIN_WORDS, MAX_WORDS),
      text_color: color(&self.text_color, WHITE),
      highlight_color: color(&self.highlight_color, YELLOW),
      background_color: color(&self.background_color, SHADE),
      ..self.clone()
    }
  }

  /// Whether every option is one a style can hold as written.
  pub fn is_valid(&self) -> bool {
    self == &self.normalized()
  }

  pub fn line_count(&self) -> usize {
    match self.lines {
      CaptionLines::One => 1,
      CaptionLines::Two => 2,
    }
  }
}
