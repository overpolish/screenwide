// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn rect(x: f64, y: f64, width: f64, height: f64) -> PreviewSurfaceRect {
  PreviewSurfaceRect {
    height,
    width,
    x,
    y,
  }
}

#[test]
fn a_resized_pane_pushes_its_row_and_keeps_the_gaps() {
  let starts = vec![
    (0, rect(0.0, 0.0, 100.0, 100.0)),
    (1, rect(110.0, 0.0, 100.0, 50.0)),
    (2, rect(220.0, 0.0, 100.0, 100.0)),
  ];

  let reflowed = reflow_workspace_panes(&starts, 1, rect(110.0, -50.0, 200.0, 200.0));

  // The row keeps its 10pt gaps around the grown canvas, and every pane
  // stays centred on the row.
  assert_eq!(
    reflowed
      .iter()
      .map(|(index, rect)| (*index, rect.x, rect.y, rect.width, rect.height))
      .collect::<Vec<_>>(),
    vec![
      (0, 0.0, 0.0, 100.0, 100.0),
      (1, 110.0, -50.0, 200.0, 200.0),
      (2, 320.0, 0.0, 100.0, 100.0),
    ]
  );
}

#[test]
fn a_mismatched_canvas_is_centred_in_its_box_and_a_matching_one_is_untouched() {
  let fitted = aspect_fit_rect(rect(10.0, 20.0, 200.0, 100.0), (100, 100));
  assert_eq!(
    (fitted.x, fitted.y, fitted.width, fitted.height),
    (60.0, 20.0, 100.0, 100.0)
  );

  let box_rect = rect(10.0, 20.0, 200.0, 100.0);
  let unchanged = aspect_fit_rect(box_rect, (1_920, 960));
  assert_eq!(
    (unchanged.x, unchanged.y, unchanged.width, unchanged.height),
    (box_rect.x, box_rect.y, box_rect.width, box_rect.height)
  );
}

#[test]
fn a_press_on_an_uncropped_body_draws_only_a_crop_that_can_be_drawn_afresh() {
  let whole = PreviewSelection {
    crop_mode: 1,
    width: 1.0,
    height: 1.0,
    image_width: 1.0,
    image_height: 1.0,
    ..Default::default()
  };
  let pressed = |selection: PreviewSelection| {
    let state = SurfaceState {
      selection: Some(selection),
      ..Default::default()
    };
    crop_press_draws(&state, Some((selection, 0)))
  };
  assert!(pressed(whole));
  // A scene's fixed window and a crop given a shape are never drawn afresh,
  // so the press is theirs to move rather than a pan of the whole view.
  assert!(!pressed(PreviewSelection {
    framed: FRAMED_FIXED,
    ..whole
  }));
  assert!(!pressed(PreviewSelection {
    crop_keeps_shape: 1,
    ..whole
  }));
}
