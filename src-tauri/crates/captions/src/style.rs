// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How captions look and move. A style is a set of options, and a template
//! is a style under an id: choosing one takes up its options, or the ones it
//! was last saved with, and any of them can change after. Everything here is
//! a template's to set; what is said and when stays in the recording's
//! settings, whatever the template. What an option is where nothing sets it
//! is in `baseline`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::CaptionPosition;
use crate::motion::CaptionMotion;
use crate::size::CaptionSize;
use crate::templates::CaptionTemplateId;

/// How many lines one caption may take, where its words are set in lines.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionLines {
  One,
  #[default]
  Two,
}

/// How a caption's words are broken into lines and set out.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionArrangement {
  /// In as many lines as the style allows, each lined up as the caption's
  /// place has it.
  #[default]
  Lines,
  /// A word or two a line, one beneath another, with a word set apart on a
  /// line of its own.
  Stack,
  /// Broken as a stack is, each line starting a step further along than the
  /// one above.
  Staircase,
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
  /// A pill behind the word being said, in the highlight colour, where the
  /// word being said is highlighted.
  Word,
}

/// Whether a template sets a key word of each caption apart in the
/// accent's face, size and colour: the template's to say, not the user's.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionAccent {
  /// Every word set alike. Lines taking turns in the accent, before the
  /// accent was only ever a key word, are read as this.
  #[default]
  #[serde(alias = "lines")]
  None,
  /// One word of each caption: the longest, or the ones chosen by hand.
  Word,
}

/// Which of a style's two faces a word is set in.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CaptionFace {
  Main,
  Accent,
}

/// The face caption type is set in: a family installed on the system or
/// carried by the app, or none for the app's own Inter, at a weight on the
/// CSS scale of 100 to 900, upright or italic.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionFont {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub family: Option<String>,
  pub weight: u16,
  /// Set in the family's italic, where it has one.
  pub italic: bool,
}

impl Default for CaptionFont {
  fn default() -> Self {
    Self {
      family: None,
      weight: 600,
      italic: false,
    }
  }
}

impl CaptionFont {
  fn normalized(&self) -> Self {
    Self {
      family: self
        .family
        .clone()
        .filter(|family| !family.trim().is_empty()),
      weight: self.weight.clamp(MIN_WEIGHT, MAX_WEIGHT),
      italic: self.italic,
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionStyle {
  /// The template the style was taken from, which its tile shows chosen.
  pub template: CaptionTemplateId,
  pub font: CaptionFont,
  /// Type size, in percent of the size a caption is set at by default: the
  /// template's, which `size` steps from.
  pub size_percent: f64,
  /// The step the type is drawn at from `size_percent`.
  pub size: CaptionSize,
  pub all_caps: bool,
  /// The most words one caption holds, across its lines.
  pub max_words: u32,
  pub lines: CaptionLines,
  pub arrangement: CaptionArrangement,
  /// Where on the picture the caption sits, and so how its lines line up.
  pub position: CaptionPosition,
  /// The type's colour, as `#rrggbb` or `#rrggbbaa`.
  pub text_color: String,
  /// The colour that marks the word being said: its type, or the pill
  /// behind it.
  pub highlight_color: String,
  /// The box's colour, where there is one.
  pub background_color: String,
  /// Mark the word being said: its type in the highlight colour, or the
  /// pill behind it where the template sets it on one.
  pub highlight_words: bool,
  /// Show each word as it is said, so the caption builds up with the voice.
  pub reveal_words: bool,
  /// A soft outline around the type, so it reads without a box.
  pub outline: bool,
  /// The outline's colour, as `#rrggbb`; it is always drawn a little
  /// see-through, so it softens the type's edge rather than ringing it.
  pub outline_color: String,
  pub background: CaptionBackground,
  /// How it comes on: the caption whole, and each word as it is said. An
  /// older style's `entrance`, a preset's name, reads as the same motion.
  #[serde(
    alias = "entrance",
    deserialize_with = "crate::motion::deserialize_motion"
  )]
  pub motion: CaptionMotion,
  /// Which words are set apart in the accent.
  pub accent: CaptionAccent,
  pub accent_font: CaptionFont,
  /// The accent's type size, in percent of the caption's: the template's,
  /// which `accent_size` steps from.
  pub accent_size_percent: f64,
  /// The step the accent is drawn at from `accent_size_percent`.
  pub accent_size: CaptionSize,
  /// The accent's colour, as `#rrggbb` or `#rrggbbaa`.
  pub accent_color: String,
}

pub const MIN_WORDS: u32 = 1;
pub const MAX_WORDS: u32 = 20;
pub const MIN_SIZE_PERCENT: f64 = 25.0;
pub const MAX_SIZE_PERCENT: f64 = 400.0;
pub const MIN_ACCENT_SIZE_PERCENT: f64 = 50.0;
pub const MAX_ACCENT_SIZE_PERCENT: f64 = 300.0;
pub const MIN_WEIGHT: u16 = 100;
pub const MAX_WEIGHT: u16 = 900;

pub(crate) const WHITE: &str = "#ffffff";
/// A warm yellow, which reads on white type and on any picture.
pub(crate) const YELLOW: &str = "#ffd60a";
/// Black, a little see-through.
pub(crate) const SHADE: &str = "#0000009e";
/// Black: the outline is drawn see-through already.
pub(crate) const BLACK: &str = "#000000";

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

/// `value` within `min..=max`, or `fallback` where it is no number at all.
fn clamped(value: f64, (min, max): (f64, f64), fallback: f64) -> f64 {
  if value.is_finite() {
    value.clamp(min, max)
  } else {
    fallback
  }
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
      font: self.font.normalized(),
      size_percent: clamped(
        self.size_percent,
        (MIN_SIZE_PERCENT, MAX_SIZE_PERCENT),
        100.0,
      ),
      max_words: self.max_words.clamp(MIN_WORDS, MAX_WORDS),
      text_color: color(&self.text_color, WHITE),
      highlight_color: color(&self.highlight_color, YELLOW),
      background_color: color(&self.background_color, SHADE),
      outline_color: color(&self.outline_color, BLACK),
      accent_font: self.accent_font.normalized(),
      accent_size_percent: clamped(
        self.accent_size_percent,
        (MIN_ACCENT_SIZE_PERCENT, MAX_ACCENT_SIZE_PERCENT),
        100.0,
      ),
      accent_color: color(&self.accent_color, WHITE),
      motion: self.motion.normalized(),
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

  /// The font `face` is set in.
  pub fn font_of(&self, face: CaptionFace) -> &CaptionFont {
    match face {
      CaptionFace::Main => &self.font,
      CaptionFace::Accent => &self.accent_font,
    }
  }

  /// How many times the caption's type size `face` is set at.
  pub fn scale_of(&self, face: CaptionFace) -> f64 {
    match face {
      CaptionFace::Main => 1.0,
      CaptionFace::Accent => self.drawn_accent_percent() / 100.0,
    }
  }
}
