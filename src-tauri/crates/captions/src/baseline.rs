// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What each style option is where nothing says otherwise: the base every
//! template manifest sets its own options over, and what an option missing
//! from a style saved before it existed reads as. `templates/base.json`
//! writes it out for anything reading the manifests outside Rust, and a test
//! keeps the two alike.

use crate::font::CaptionFont;
use crate::motion::CaptionMotion;
use crate::size::CaptionSize;
use crate::style::{
  CaptionAccent, CaptionArrangement, CaptionBackground, CaptionLines, CaptionStyle, BLACK, SHADE,
  WHITE, YELLOW,
};
use crate::templates::CaptionTemplateId;
use screenwide_placement::Placement;

impl Default for CaptionStyle {
  fn default() -> Self {
    Self {
      template: CaptionTemplateId::default(),
      font: CaptionFont::default(),
      size_percent: 100.0,
      size: CaptionSize::Medium,
      all_caps: false,
      max_words: 8,
      lines: CaptionLines::Two,
      arrangement: CaptionArrangement::Lines,
      position: Placement::Bottom,
      text_color: WHITE.to_owned(),
      highlight_color: YELLOW.to_owned(),
      background_color: SHADE.to_owned(),
      highlight_words: false,
      reveal_words: false,
      visible_words: 0,
      outline: true,
      outline_color: BLACK.to_owned(),
      background: CaptionBackground::None,
      motion: CaptionMotion::default(),
      accent: CaptionAccent::None,
      // The serif italic accents are set in, which reads small beside a sans.
      accent_font: CaptionFont {
        family: Some("Instrument Serif".to_owned()),
        weight: 400,
        italic: true,
      },
      accent_size_percent: 120.0,
      accent_size: CaptionSize::Medium,
      accent_color: YELLOW.to_owned(),
      shadow: None,
    }
  }
}
