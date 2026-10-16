// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The type's colour as it is drawn: the one a style names, or black or
//! white on its background colour where it names [`AUTO_INK`].

use super::{rgba, TextStyle, BLACK, WHITE};

/// The text colour that is black or white, whichever reads on the
/// background colour.
pub const AUTO_INK: &str = "auto";

/// Type that reads on `color` by the counter's rule, which a text box has
/// always used: black on a light colour, white on a dark one.
fn auto_ink(color: &str) -> &'static str {
  let [red, green, blue, _] = rgba(color).unwrap_or([0.0; 4]);
  if 0.2126 * red + 0.7152 * green + 0.0722 * blue > 0.6 {
    BLACK
  } else {
    WHITE
  }
}

impl TextStyle {
  /// The type's colour, `#rrggbb` or `#rrggbbaa`, with [`AUTO_INK`] read
  /// against the background colour.
  pub fn ink(&self) -> &str {
    if self.text_color == AUTO_INK {
      auto_ink(&self.background_color)
    } else {
      &self.text_color
    }
  }
}
