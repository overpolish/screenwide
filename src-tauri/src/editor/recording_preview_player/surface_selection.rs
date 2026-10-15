// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Converts the webview's recording selection into the shared native OSC model.

use crate::editor::preview_platform::{
  is_overlay_layer, PreviewSelection, FRAMED_FIXED, FRAMED_FREE,
};
use screenwide_compositor::PreviewSurfaceRect;
use screenwide_scenes::SceneRotation;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecordingPreviewSelection {
  #[serde(default)]
  crop_mode: bool,
  #[serde(default)]
  framed: bool,
  #[serde(default)]
  image: Option<PreviewSurfaceRect>,
  /// A pane a scene places but leaves free, a custom scene's box.
  #[serde(default)]
  in_scene: bool,
  layer_id: Option<u32>,
  pane_index: u32,
  radius_percent: f64,
  #[serde(default)]
  recenter_bounds: Option<PreviewSurfaceRect>,
  rect: PreviewSurfaceRect,
  /// Set where a scene turns this pane and the select tool turns it with the
  /// rotation gizmo: how it is turned now.
  #[serde(default)]
  rotation: Option<SceneRotation>,
}

impl RecordingPreviewSelection {
  pub(super) fn into_native(self) -> PreviewSelection {
    let radius_disabled = self
      .layer_id
      .is_some_and(|layer_id| layer_id == u32::MAX || is_overlay_layer(layer_id));
    PreviewSelection {
      framed: if self.framed {
        FRAMED_FIXED
      } else if self.in_scene {
        FRAMED_FREE
      } else {
        0
      },
      recenter_height: self.recenter_bounds.map_or(0.0, |bounds| bounds.height),
      recenter_width: self.recenter_bounds.map_or(0.0, |bounds| bounds.width),
      recenter_x: self.recenter_bounds.map_or(0.0, |bounds| bounds.x),
      recenter_y: self.recenter_bounds.map_or(0.0, |bounds| bounds.y),
      crop_mode: u32::from(self.crop_mode),
      image_height: self.image.map_or(0.0, |image| image.height),
      image_width: self.image.map_or(0.0, |image| image.width),
      image_x: self.image.map_or(0.0, |image| image.x),
      image_y: self.image.map_or(0.0, |image| image.y),
      layer_id: self.layer_id.unwrap_or(self.pane_index),
      radius_disabled: u32::from(radius_disabled),
      pane_index: self.pane_index,
      x: self.rect.x,
      y: self.rect.y,
      width: self.rect.width,
      height: self.rect.height,
      radius_percent: self.radius_percent,
      turnable: u32::from(self.rotation.is_some()),
      rotation_x: self.rotation.map_or(0.0, |rotation| rotation.x),
      rotation_y: self.rotation.map_or(0.0, |rotation| rotation.y),
      rotation_z: self.rotation.map_or(0.0, |rotation| rotation.z),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn selection() -> RecordingPreviewSelection {
    RecordingPreviewSelection {
      crop_mode: false,
      framed: false,
      image: None,
      in_scene: false,
      layer_id: None,
      pane_index: 0,
      radius_percent: 18.0,
      recenter_bounds: None,
      rotation: None,
      rect: PreviewSurfaceRect {
        height: 80.0,
        width: 120.0,
        x: 10.0,
        y: 20.0,
      },
    }
  }

  #[test]
  fn overlay_osc_is_square_without_changing_other_selection_radii() {
    let regular = selection().into_native();
    assert_eq!(regular.radius_disabled, 0);
    assert_eq!(regular.radius_percent, 18.0);

    let mut keyboard = selection();
    keyboard.layer_id = Some(crate::editor::preview_platform::KEYBOARD_LAYER_ID);
    assert_eq!(keyboard.into_native().radius_disabled, 1);
  }
}
