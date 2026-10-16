// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The face styled type is set in, as a style names it.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MIN_WEIGHT: u16 = 100;
pub const MAX_WEIGHT: u16 = 900;

/// The face caption type is set in: a family installed on the system or
/// carried by the app, or none for the app's own Inter, at a weight on the
/// CSS scale of 100 to 900, upright or italic.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct TextFont {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub family: Option<String>,
  pub weight: u16,
  /// Set in the family's italic, where it has one.
  pub italic: bool,
}

impl Default for TextFont {
  fn default() -> Self {
    Self {
      family: None,
      weight: 600,
      italic: false,
    }
  }
}

impl TextFont {
  pub(crate) fn normalized(&self) -> Self {
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
