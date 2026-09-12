// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

use crate::screenshots::default_mesh_generator as default_generator;

/// One blob of a mesh gradient, in shares of the canvas.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundMeshPoint {
  pub radius_x: f64,
  pub radius_y: f64,
  pub rotation: f64,
  pub x: f64,
  pub y: f64,
}

/// What sits behind the picture, as a saved preset carries it.
///
/// The three kinds are the three ways a canvas is filled: one colour, a mesh
/// of them, or a picture of your own. The editor spreads the same values
/// across the flat fields of an output canvas; a preset keeps them together,
/// since a preset is one background rather than part of one.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Background {
  #[serde(rename_all = "camelCase")]
  Solid { color: String },
  #[serde(rename_all = "camelCase")]
  Image { path: String },
  #[serde(rename_all = "camelCase")]
  Mesh {
    colors: Vec<String>,
    seed: u32,
    warp_percent: f64,
    /// Which picture this paints. Presets saved before the ported generators
    /// existed carry none, and are the app's own blob mesh.
    #[serde(default = "default_generator")]
    generator: String,
    #[serde(default)]
    locked_colors: Vec<bool>,
    #[serde(default)]
    points: Vec<BackgroundMeshPoint>,
  },
}

/// A background under a name, as the background picker offers it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundPreset {
  pub background: Background,
  pub id: String,
  pub name: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trips_every_kind() {
    let presets = vec![
      BackgroundPreset {
        background: Background::Solid {
          color: "#FFFFFF".to_owned(),
        },
        id: "one".to_owned(),
        name: "Paper".to_owned(),
      },
      BackgroundPreset {
        background: Background::Image {
          path: "/pictures/desk.png".to_owned(),
        },
        id: "two".to_owned(),
        name: "Desk".to_owned(),
      },
      BackgroundPreset {
        background: Background::Mesh {
          colors: vec!["#000000".to_owned(), "#FFFFFF".to_owned()],
          generator: "mesh".to_owned(),
          locked_colors: vec![false, true],
          points: vec![BackgroundMeshPoint {
            radius_x: 50.0,
            radius_y: 40.0,
            rotation: 12.0,
            x: 30.0,
            y: 70.0,
          }],
          seed: 7,
          warp_percent: 9.0,
        },
        id: "three".to_owned(),
        name: "Fog".to_owned(),
      },
    ];
    let json = serde_json::to_string(&presets).expect("presets serialize");
    assert!(json.contains("\"kind\":\"mesh\""));
    assert!(json.contains("\"warpPercent\""));
    assert!(json.contains("\"lockedColors\""));
    assert!(json.contains("\"generator\":\"mesh\""));
    let restored: Vec<BackgroundPreset> = serde_json::from_str(&json).expect("presets restore");
    assert_eq!(restored, presets);
  }

  #[test]
  fn reads_a_mesh_saved_without_its_geometry() {
    let restored: Background =
      serde_json::from_str(r##"{"kind":"mesh","colors":["#112233"],"seed":3,"warpPercent":8.0}"##)
        .expect("mesh restores");
    assert_eq!(
      restored,
      Background::Mesh {
        colors: vec!["#112233".to_owned()],
        generator: "mesh".to_owned(),
        locked_colors: Vec::new(),
        points: Vec::new(),
        seed: 3,
        warp_percent: 8.0,
      }
    );
  }
}
