// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Converts the webview's recording selection into the shared native OSC model.

use crate::exports::preview_platform::{PreviewSelection, PreviewSurfaceRect};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecordingPreviewSelection {
  #[serde(default)]
  crop_mode: bool,
  #[serde(default)]
  image: Option<PreviewSurfaceRect>,
  layer_id: Option<u32>,
  #[serde(default)]
  maximum_scale: Option<f64>,
  #[serde(default)]
  minimum_scale: Option<f64>,
  pane_index: u32,
  radius_percent: f64,
  #[serde(default)]
  recenter_bounds: Option<PreviewSurfaceRect>,
  #[serde(default)]
  recenter_mode: bool,
  rect: PreviewSurfaceRect,
}

impl RecordingPreviewSelection {
  pub(super) fn is_recenter(&self) -> bool {
    self.recenter_mode
  }

  pub(super) fn into_native(self) -> PreviewSelection {
    let radius_disabled = self.recenter_mode
      || self
        .layer_id
        .is_some_and(|layer_id| layer_id == u32::MAX || layer_id == u32::MAX - 1);
    PreviewSelection {
      recenter_height: self.recenter_bounds.map_or(0.0, |bounds| bounds.height),
      recenter_width: self.recenter_bounds.map_or(0.0, |bounds| bounds.width),
      recenter_x: self.recenter_bounds.map_or(0.0, |bounds| bounds.x),
      recenter_y: self.recenter_bounds.map_or(0.0, |bounds| bounds.y),
      recenter_mode: u32::from(self.recenter_mode),
      crop_mode: u32::from(self.crop_mode),
      image_height: self.image.map_or(0.0, |image| image.height),
      image_width: self.image.map_or(0.0, |image| image.width),
      image_x: self.image.map_or(0.0, |image| image.x),
      image_y: self.image.map_or(0.0, |image| image.y),
      layer_id: self.layer_id.unwrap_or(self.pane_index),
      maximum_scale: self
        .maximum_scale
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .unwrap_or(0.0),
      minimum_scale: self
        .minimum_scale
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .unwrap_or(0.0),
      radius_disabled: u32::from(radius_disabled),
      pane_index: self.pane_index,
      x: self.rect.x,
      y: self.rect.y,
      width: self.rect.width,
      height: self.rect.height,
      radius_percent: if self.recenter_mode {
        0.0
      } else {
        self.radius_percent
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn selection(recenter_mode: bool) -> RecordingPreviewSelection {
    RecordingPreviewSelection {
      crop_mode: false,
      image: None,
      layer_id: None,
      maximum_scale: None,
      minimum_scale: None,
      pane_index: 0,
      radius_percent: 18.0,
      recenter_bounds: None,
      recenter_mode,
      rect: PreviewSurfaceRect {
        height: 80.0,
        width: 120.0,
        x: 10.0,
        y: 20.0,
      },
    }
  }

  #[test]
  fn recenter_osc_is_square_without_changing_other_selection_radii() {
    let recenter = selection(true).into_native();
    assert_eq!(recenter.radius_disabled, 1);
    assert_eq!(recenter.radius_percent, 0.0);

    let regular = selection(false).into_native();
    assert_eq!(regular.radius_disabled, 0);
    assert_eq!(regular.radius_percent, 18.0);

    let mut keyboard = selection(false);
    keyboard.layer_id = Some(u32::MAX - 1);
    assert_eq!(keyboard.into_native().radius_disabled, 1);
  }

  #[test]
  fn keyboard_resize_limits_are_forwarded_and_invalid_values_are_ignored() {
    let mut keyboard = selection(false);
    keyboard.maximum_scale = Some(1.75);
    keyboard.minimum_scale = Some(0.5);
    let native = keyboard.into_native();
    assert_eq!(native.maximum_scale, 1.75);
    assert_eq!(native.minimum_scale, 0.5);

    let mut invalid = selection(false);
    invalid.maximum_scale = Some(f64::NAN);
    invalid.minimum_scale = Some(-1.0);
    let native = invalid.into_native();
    assert_eq!(native.maximum_scale, 0.0);
    assert_eq!(native.minimum_scale, 0.0);
  }
}
