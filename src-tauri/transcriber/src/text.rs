// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What reading text answers with: each line found in the picture, in reading
//! order, measured in the picture's own pixels.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextLine {
  pub text: String,
  /// How sure the reading is, 0 to 1.
  pub confidence: f32,
  /// The line's box: left, top, width and height.
  pub bounds: [f32; 4],
  /// Where each character of `text` runs across the line, in order: its left
  /// and right edges. A character the model read as one with its neighbour
  /// shares its box.
  pub characters: Vec<[f32; 2]>,
}
