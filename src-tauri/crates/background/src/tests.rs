// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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
