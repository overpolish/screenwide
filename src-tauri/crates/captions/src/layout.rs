// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How large a caption is and the layout options set by hand. A template
//! brings its own case, words and lines per caption, and place, and the
//! sizes the size steps scale; one set here stays as set whichever template
//! is chosen after, and one left unset follows the template.

use screenwide_placement::Placement;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::size::CaptionSize;
use crate::style::{CaptionLines, CaptionStyle};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionLayout {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub size: Option<CaptionSize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub accent_size: Option<CaptionSize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub all_caps: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub max_words: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub lines: Option<CaptionLines>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub position: Option<Placement>,
}

impl CaptionLayout {
  pub fn is_empty(&self) -> bool {
    *self == Self::default()
  }

  /// Every layout option `style` holds, as though each were set.
  pub fn of(style: &CaptionStyle) -> Self {
    Self {
      size: Some(style.size),
      accent_size: Some(style.accent_size),
      all_caps: Some(style.all_caps),
      max_words: Some(style.max_words),
      lines: Some(style.lines),
      position: Some(style.position),
    }
  }

  /// `style` with the options set here in place of its own.
  pub fn applied_to(&self, style: CaptionStyle) -> CaptionStyle {
    CaptionStyle {
      size: self.size.unwrap_or(style.size),
      accent_size: self.accent_size.unwrap_or(style.accent_size),
      all_caps: self.all_caps.unwrap_or(style.all_caps),
      max_words: self.max_words.unwrap_or(style.max_words),
      lines: self.lines.unwrap_or(style.lines),
      position: self.position.unwrap_or(style.position),
      ..style
    }
  }
}

#[cfg(test)]
mod tests {
  use super::CaptionLayout;
  use crate::style::CaptionLines;
  use crate::templates::CaptionTemplateId;

  #[test]
  fn options_set_by_hand_outlast_the_template_and_the_rest_follow_it() {
    let layout = CaptionLayout {
      max_words: Some(5),
      ..CaptionLayout::default()
    };
    let pop = layout.applied_to(CaptionTemplateId::from("pop").shipped_style());
    assert_eq!(pop.max_words, 5);
    assert_eq!(pop.lines, CaptionLines::One);
    assert!(pop.all_caps);
    assert_eq!(pop.size_percent, 140.0);
  }
}
