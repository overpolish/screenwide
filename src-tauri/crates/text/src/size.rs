// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How large captions are drawn: a template sets its type size and its key
//! word's, and a step taken from them on the Layout tab sets them smaller or
//! larger. The step is kept apart from the sizes it scales, so it means the
//! same whichever template it is taken to.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::style::{
  TextStyle, MAX_ACCENT_SIZE_PERCENT, MAX_SIZE_PERCENT, MIN_ACCENT_SIZE_PERCENT, MIN_SIZE_PERCENT,
};

/// A step of a caption's type size, or of its key word's, from the size its
/// template sets, which is Medium.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum TextSize {
  Small,
  #[default]
  Medium,
  Large,
  ExtraLarge,
}

impl TextSize {
  /// How many times its template's size the type is set at.
  fn type_factor(self) -> f64 {
    match self {
      Self::Small => 0.8,
      Self::Medium => 1.0,
      Self::Large => 1.25,
      Self::ExtraLarge => 1.5,
    }
  }

  /// How many times as much larger than the caption's type than its
  /// template sets it a key word is set. Scaling the key word's whole size
  /// instead would put it below the words around it at Small, which no
  /// longer sets it apart.
  fn accent_factor(self) -> f64 {
    match self {
      Self::Small => 0.5,
      Self::Medium => 1.0,
      Self::Large => 1.5,
      Self::ExtraLarge => 2.0,
    }
  }
}

impl TextStyle {
  /// The type size the captions are drawn at, in percent of the size a
  /// caption is set at by default.
  pub fn drawn_size_percent(&self) -> f64 {
    let style = self.normalized();
    (style.size_percent * style.size.type_factor()).clamp(MIN_SIZE_PERCENT, MAX_SIZE_PERCENT)
  }

  /// The key word's size, in percent of the caption's type.
  pub(crate) fn drawn_accent_percent(&self) -> f64 {
    let style = self.normalized();
    (100.0 + (style.accent_size_percent - 100.0) * style.accent_size.accent_factor())
      .clamp(MIN_ACCENT_SIZE_PERCENT, MAX_ACCENT_SIZE_PERCENT)
  }
}

#[cfg(test)]
mod tests {
  use super::TextSize;
  use crate::templates::{bundled_templates, TextTemplateId};

  const STEPS: [TextSize; 4] = [
    TextSize::Small,
    TextSize::Medium,
    TextSize::Large,
    TextSize::ExtraLarge,
  ];

  #[test]
  fn each_step_sets_the_type_larger_from_the_templates_own_size() {
    let mut style = TextTemplateId::from("supersize").shipped_style();
    let sizes = STEPS.map(|step| {
      style.size = step;
      style.drawn_size_percent()
    });
    assert!(sizes.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
      sizes[1],
      TextTemplateId::from("supersize")
        .shipped_style()
        .size_percent
    );
  }

  #[test]
  fn a_key_word_stays_larger_than_the_caption_at_every_step() {
    for template in bundled_templates() {
      let mut style = template.style.clone();
      let sizes = STEPS.map(|step| {
        style.accent_size = step;
        style.drawn_accent_percent()
      });
      assert!(sizes[0] > 100.0, "{}", template.id);
      assert!(
        sizes.windows(2).all(|pair| pair[0] < pair[1]),
        "{}",
        template.id
      );
    }
  }
}
