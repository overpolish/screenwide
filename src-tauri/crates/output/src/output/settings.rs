// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a layer's settings say of themselves.

use super::{ScreenshotOutputSettings, MAX_OUTPUT_PIXELS};

impl ScreenshotOutputSettings {
  /// Whether this carries a placement at all. Settings written before
  /// placement was measured in output pixels deserialize without one, and are
  /// discarded rather than converted.
  pub fn has_placement(&self) -> bool {
    self.crop_width > 0.0 && self.crop_height > 0.0 && self.image_width > 0.0
  }

  /// How strong the shadow is drawn, zero (none) to one: whole wherever it
  /// is on, but while the preview fades it in or out.
  pub fn shadow_strength(&self) -> f32 {
    if self.drop_shadow {
      self
        .drop_shadow_strength
        .map_or(1.0, |strength| strength.clamp(0.0, 1.0) as f32)
    } else {
      0.0
    }
  }
}

pub fn output_dimensions(settings: &ScreenshotOutputSettings) -> Result<(u32, u32), String> {
  if settings.width < 64
    || settings.height < 64
    || u64::from(settings.width) * u64::from(settings.height) > MAX_OUTPUT_PIXELS
  {
    return Err("The screenshot output dimensions are not valid".to_owned());
  }
  Ok((settings.width, settings.height))
}
