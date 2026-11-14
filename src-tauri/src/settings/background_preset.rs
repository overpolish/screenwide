// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub(crate) use screenwide_background::{Background, BackgroundMeshPoint};

/// A background under a name, as the background picker offers it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
}
