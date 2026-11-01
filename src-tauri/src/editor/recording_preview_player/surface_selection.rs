// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Converts the webview's recording selection into the shared native OSC model.

use screenwide_compositor::PreviewSurfaceRect;
use screenwide_scenes::SceneRotation;
use screenwide_workspace::editing::{PreviewSelection, FRAMED_FIXED, FRAMED_FREE};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecordingPreviewSelection {
  #[serde(default)]
  crop_mode: bool,
  /// A crop window given a shape, which its handles keep as they are pulled.
  #[serde(default)]
  crop_keeps_shape: bool,
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
  /// Set where the select tool turns this pane with the rotation gizmo, the
  /// screen: how it is turned now, by its own turn or by a scene's.
  #[serde(default)]
  rotation: Option<SceneRotation>,
}

impl RecordingPreviewSelection {
  pub(super) fn into_native(self) -> PreviewSelection {
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
      crop_keeps_shape: u32::from(self.crop_keeps_shape),
      image_height: self.image.map_or(0.0, |image| image.height),
      image_width: self.image.map_or(0.0, |image| image.width),
      image_x: self.image.map_or(0.0, |image| image.x),
      image_y: self.image.map_or(0.0, |image| image.y),
      layer_id: self.layer_id.unwrap_or(self.pane_index),
      pane_index: self.pane_index,
      x: self.rect.x,
      y: self.rect.y,
      width: self.rect.width,
      height: self.rect.height,
      radius_percent: self.radius_percent,
      ..PreviewSelection::default()
    }
    .turned_by(self.rotation)
  }
}
