// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// A crop rectangle expressed as fractions of the captured source image.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedSourceRect {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}

impl NormalizedSourceRect {
  pub fn is_valid(self) -> bool {
    self.x.is_finite()
      && self.y.is_finite()
      && self.width.is_finite()
      && self.height.is_finite()
      && self.x >= 0.0
      && self.y >= 0.0
      && self.width > 0.0
      && self.height > 0.0
      && self.x + self.width <= 1.0
      && self.y + self.height <= 1.0
  }

  pub fn validate(self) -> Result<(), String> {
    self
      .is_valid()
      .then_some(())
      .ok_or_else(|| "The source crop is not valid".to_owned())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::screenshots::{test_output_settings, ScreenshotOutputSettings};

  #[test]
  fn source_crop_serializes_and_round_trips() {
    let mut settings = test_output_settings(640, 480);
    settings.source_crop = NormalizedSourceRect {
      height: 0.75,
      width: 0.8,
      x: 0.1,
      y: 0.125,
    };

    let serialized = serde_json::to_value(&settings).unwrap();
    assert_eq!(serialized["sourceCrop"]["x"], 0.1);
    assert_eq!(serialized["sourceCrop"]["width"], 0.8);

    let round_tripped: ScreenshotOutputSettings = serde_json::from_value(serialized).unwrap();
    assert_eq!(round_tripped, settings);
  }
}
