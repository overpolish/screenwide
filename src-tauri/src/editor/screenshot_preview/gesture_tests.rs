// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::preview_platform::{SelectionGestureOperation, SelectionGesturePhase};
use super::state::PreviewManager;
use crate::editor::screenshot_model::ScreenshotWorkspaceItemOutput;
use crate::editor::ScreenshotWorkspaceOutputSettings;
use crate::screenshots::test_output_settings;

#[test]
fn moving_an_item_keeps_canvas_presentation_fields() {
  let mut canvas = test_output_settings(1_000, 800);
  canvas.background_color = "#112233".to_owned();
  canvas.background_radius_percent = 12.0;
  canvas.background_type = "mesh".to_owned();
  canvas.mesh_seed = 1234;
  canvas.mesh_colors = vec!["#112233".to_owned(), "#445566".to_owned()];
  let mut item = test_output_settings(800, 600);
  item.background_color = "#AABBCC".to_owned();
  item.radius_percent = 31.0;
  item.mesh_seed = 5678;
  let snapshot = ScreenshotWorkspaceOutputSettings {
    canvas,
    items: vec![ScreenshotWorkspaceItemOutput {
      id: 1,
      output: item,
    }],
  };
  let mut manager = PreviewManager {
    react_output: Some(snapshot.clone()),
    ..Default::default()
  };

  manager
    .handle_selection_gesture(
      SelectionGesturePhase::Begin,
      0,
      SelectionGestureOperation::Move,
      0,
      1.0,
      0.0,
      0.0,
    )
    .unwrap();
  manager
    .handle_selection_gesture(
      SelectionGesturePhase::Update,
      0,
      SelectionGestureOperation::Move,
      0,
      1.0,
      20.0,
      0.0,
    )
    .unwrap();

  let next = manager.output.unwrap();
  assert_eq!(next.canvas.background_color, "#112233");
  assert_eq!(next.canvas.background_radius_percent, 12.0);
  assert_eq!(next.canvas.background_type, "mesh");
  assert_eq!(next.canvas.mesh_seed, 1234);
  assert_eq!(next.canvas.mesh_colors, snapshot.canvas.mesh_colors);
  assert_eq!(next.items[0].output.background_color, "#AABBCC");
  assert_eq!(next.items[0].output.radius_percent, 31.0);
  assert_ne!(next.items[0].output.crop_x, snapshot.items[0].output.crop_x);
}
