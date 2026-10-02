// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A custom scene's layout kept under a name for any recording. The twin of
//! `SceneTemplate` in
//! `src/features/editor/recording/scenes/recording-scene-template.ts`, which
//! fits it to the canvas it is chosen on; this side only keeps it sound.

use serde::{Deserialize, Serialize};

use super::framing::MAX_ZOOM;
use super::model::{SceneBoxes, SceneRadius};

/// How far a template zooms into each pane; unset shows the whole picture.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SceneZoom {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub screen: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneTemplate {
  pub id: String,
  pub name: String,
  /// The width over the height of the canvas the layout was made on, which
  /// its boxes are shares of.
  pub canvas_aspect: f64,
  pub boxes: SceneBoxes,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub zoom: Option<SceneZoom>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub radius: Option<SceneRadius>,
}

impl SceneTemplate {
  /// Whether the template holds a layout a scene can take: named, made on a
  /// canvas with a shape, its boxes, zooms and radii each within what a
  /// custom scene allows.
  pub(crate) fn is_valid(&self) -> bool {
    !self.id.is_empty()
      && !self.name.is_empty()
      && self.canvas_aspect.is_finite()
      && self.canvas_aspect > 0.0
      && self.boxes.is_valid()
      && self.radius.is_none_or(SceneRadius::is_valid)
      && self.zoom.is_none_or(|zoom| {
        [zoom.screen, zoom.camera]
          .into_iter()
          .flatten()
          .all(|zoom| (1.0..=MAX_ZOOM).contains(&zoom))
      })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn template() -> SceneTemplate {
    serde_json::from_value(serde_json::json!({
      "id": "t", "name": "Template 1", "canvasAspect": 1.777,
      "boxes": {
        "screen": { "x": 0.05, "y": 0.1, "width": 0.6, "height": 0.6 },
        "camera": { "x": 0.7, "y": 0.3, "width": 0.25, "height": 0.4 }
      },
      "zoom": { "screen": 2.0 },
      "radius": { "camera": 50.0 }
    }))
    .unwrap()
  }

  #[test]
  fn a_saved_layout_is_kept_as_the_webview_wrote_it() {
    let saved = template();
    assert!(saved.is_valid());
    assert_eq!(
      serde_json::from_value::<SceneTemplate>(serde_json::to_value(&saved).unwrap()).unwrap(),
      saved
    );
  }

  #[test]
  fn a_layout_no_scene_could_take_is_refused() {
    let mut lost = template();
    lost.boxes.screen.x = 3.0;
    let mut flat = template();
    flat.canvas_aspect = 0.0;
    let mut zoomed_out = template();
    zoomed_out.zoom = Some(SceneZoom {
      screen: Some(0.5),
      camera: None,
    });
    let mut unnamed = template();
    unnamed.name.clear();
    for template in [lost, flat, zoomed_out, unnamed] {
      assert!(!template.is_valid(), "{template:?}");
    }
  }
}
