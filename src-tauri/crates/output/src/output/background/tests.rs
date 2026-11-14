// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::fixtures::test_output_settings;

fn turn(push: f64, share: f64) -> BackgroundTurn {
  BackgroundTurn {
    outgoing: CanvasBackground::worn_on(
      &test_output_settings(100, 100),
      &Background::Solid {
        color: "#000000".to_owned(),
      },
    ),
    share,
    centre: (0.5, 0.5),
    push,
  }
}

fn close(got: (f64, f64), want: (f64, f64)) {
  assert!(
    (got.0 - want.0).abs() < 1e-9 && (got.1 - want.1).abs() < 1e-9,
    "{got:?} against {want:?}"
  );
}

#[test]
fn a_zoom_in_grows_both_backgrounds_and_lands_the_new_one_at_its_size() {
  close(turn(0.06, 0.0).scales(), (1.0, 1.0 / 1.06));
  close(turn(0.06, 1.0).scales(), (1.06, 1.0));
}

#[test]
fn a_zoom_out_shrinks_both_backgrounds_and_lands_the_new_one_at_its_size() {
  close(turn(-0.06, 0.0).scales(), (1.0, 1.06));
  close(turn(-0.06, 1.0).scales(), (1.0 / 1.06, 1.0));
}

#[test]
fn a_move_alone_scales_neither_background() {
  close(turn(0.0, 0.4).scales(), (1.0, 1.0));
}

#[test]
fn a_worn_colour_keeps_the_mesh_for_later() {
  let mut canvas = test_output_settings(100, 100);
  canvas.background_type = "mesh".to_owned();
  let mesh = canvas.background();
  canvas.wear_background(&Background::Solid {
    color: "#ABCDEF".to_owned(),
  });
  assert_eq!(
    canvas.background(),
    Background::Solid {
      color: "#ABCDEF".to_owned()
    }
  );
  canvas.wear_background(&mesh);
  assert_eq!(canvas.background(), mesh);
}

#[test]
fn a_composition_without_blobs_keeps_the_canvas_blobs() {
  let canvas = test_output_settings(100, 100);
  let worn = CanvasBackground::worn_on(
    &canvas,
    &Background::Mesh {
      colors: canvas.mesh_colors.clone(),
      seed: 4,
      warp_percent: 8.0,
      generator: "mesh".to_owned(),
      locked_colors: Vec::new(),
      points: Vec::new(),
    },
  );
  assert_eq!(worn.background_type, "mesh");
  assert_eq!(worn.mesh_points, canvas.mesh_points);
  assert_eq!(worn.mesh_seed, 4);
}
