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

/// The press that ends the typing may go straight on to a gesture, and a
/// gesture begins on React's layout, which trails the typing by a round trip.
/// What was typed has to survive it.
#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn a_gesture_straight_after_typing_keeps_what_was_typed() {
  use super::super::preview_platform::AnnotationTextPhase;
  use crate::editor::annotations::text::new_text;
  use crate::editor::annotations::{AnnotationPoint, AnnotationShape};

  let mut item = test_output_settings(800, 600);
  item.annotations = vec![new_text(
    "t".to_owned(),
    AnnotationPoint { x: 10.0, y: 10.0 },
    None,
    0.0,
  )];
  let mut manager = PreviewManager {
    react_output: Some(ScreenshotWorkspaceOutputSettings {
      canvas: test_output_settings(1_000, 800),
      items: vec![ScreenshotWorkspaceItemOutput {
        id: 1,
        output: item,
      }],
    }),
    ..Default::default()
  };
  for (phase, text, revision) in [
    (AnnotationTextPhase::Open, "", 0),
    (AnnotationTextPhase::Change, "Hello", 1),
    (AnnotationTextPhase::End, "Hello", 2),
  ] {
    assert!(manager
      .handle_annotation_text(phase, 0, 0, text, revision)
      .is_some());
  }
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

  let output = manager.output.unwrap();
  let AnnotationShape::Text { text, .. } = &output.items[0].output.annotations[0].shape else {
    panic!("the box is gone");
  };
  assert_eq!(text, "Hello");
}
