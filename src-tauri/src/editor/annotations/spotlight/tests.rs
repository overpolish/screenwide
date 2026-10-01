// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{light, prepare_spotlight, shade};
use super::native::{blur_holes, draw_points};
use super::reveal::{spotlight_reveal_window, SPOTLIGHT_FADE_IN_MS, SPOTLIGHT_FADE_OUT_MS};
use crate::editor::annotations::native::NativeAnnotation;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::spotlight::model::default_spotlight_style;
use crate::editor::annotations::{AnnotationKind, AnnotationPoint};

/// A 100 by 50 box at (100, 100): its shorter side is fifty pixels.
fn boxed(radius: f32, softness: f32) -> crate::editor::annotations::geometry::ArrowGeometry {
  prepare_spotlight([200.0, 150.0], [100.0, 100.0], radius, softness)
}

#[test]
fn a_soft_edge_fades_in_from_the_box_and_never_past_it() {
  // Ten percent of fifty pixels.
  let geometry = boxed(0.0, 10.0);
  assert_eq!(geometry.width, 5.0);
  assert_eq!(light([150.0, 125.0], &geometry, 1.0), 1.0);
  // In from the edge by more than the fade: whole.
  assert_eq!(light([194.0, 125.0], &geometry, 1.0), 1.0);
  let halfway = light([197.5, 125.0], &geometry, 1.0);
  assert!(halfway > 0.4 && halfway < 0.6, "{halfway}");
  // Past the edge by half a pixel: none, however soft.
  assert_eq!(light([200.5, 125.0], &geometry, 1.0), 0.0);
}

#[test]
fn a_spotlight_without_softness_still_has_an_antialiased_edge() {
  let geometry = boxed(0.0, 0.0);
  assert_eq!(light([200.0, 125.0], &geometry, 1.0), 0.5);
  assert_eq!(light([201.0, 125.0], &geometry, 1.0), 0.0);
}

#[test]
fn rounded_corners_leave_the_corner_itself_in_the_shade() {
  let geometry = boxed(50.0, 0.0);
  assert_eq!(light([101.0, 101.0], &geometry, 1.0), 0.0);
  assert_eq!(light([150.0, 101.0], &geometry, 1.0), 1.0);
}

#[test]
fn two_spotlights_light_two_places_without_darkening_each_other() {
  // Outside both: the whole shade. Inside either: none of it.
  assert_eq!(shade([(1.0, 0.0), (1.0, 0.0)]), 1.0);
  assert_eq!(shade([(1.0, 1.0), (1.0, 0.0)]), 0.0);
  assert_eq!(shade([(1.0, 0.0), (1.0, 1.0)]), 0.0);
}

#[test]
fn a_spotlight_fading_in_lights_its_box_only_as_far_as_it_has_arrived() {
  // One spotlight whole, one a fifth of the way in: its box is lifted a
  // fifth of the shade the first casts.
  let lit = shade([(1.0, 0.0), (0.2, 1.0)]);
  assert!((lit - 0.8).abs() < 1e-6, "{lit}");
  // Alone and half arrived, the shade around it is half as deep.
  assert_eq!(shade([(0.5, 0.0)]), 0.5);
  assert_eq!(shade([(0.5, 1.0)]), 0.0);
}

#[test]
fn a_spotlight_fades_in_and_out_on_its_own_time() {
  let at = |elapsed| spotlight_reveal_window(elapsed, 5_000.0, Default::default(), 1.0);
  assert_eq!(at(0.0).opacity, 0.0);
  assert_eq!(at(SPOTLIGHT_FADE_IN_MS).opacity, 1.0);
  assert_eq!(at(5_000.0 - SPOTLIGHT_FADE_OUT_MS).opacity, 1.0);
  assert_eq!(at(5_000.0).opacity, 0.0);
  // A fade is not smeared over the exposure: the frame before is the same.
  let half = at(SPOTLIGHT_FADE_IN_MS / 2.0);
  assert_eq!(half.previous[3], half.opacity);
}

/// The editor reaches a fresh spotlight's clip back by its fade, and
/// TypeScript cannot read the constant, so it keeps its own copy. This is
/// what stops the two from drifting.
#[test]
fn the_editor_places_a_spotlight_by_its_fade() {
  const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/features/editor/annotations/annotation-kinds.ts"
  ));
  let declared = SOURCE
    .lines()
    .find_map(|line| {
      line
        .trim()
        .strip_prefix("const ANNOTATION_SPOTLIGHT_DRAW_IN_MS = ")
    })
    .and_then(|value| value.trim_end_matches(';').parse::<f32>().ok());
  assert_eq!(declared, Some(SPOTLIGHT_FADE_IN_MS));
}

fn record(start: [f32; 2], end: [f32; 2], blur: bool, presence: f32) -> NativeAnnotation {
  let style = crate::editor::annotations::AnnotationStyle {
    blur,
    ..default_spotlight_style()
  };
  let point = |[x, y]: [f32; 2]| AnnotationPoint {
    x: f64::from(x),
    y: f64::from(y),
  };
  let [p0, p1, p2] = draw_points(point(start), point(end), &style);
  NativeAnnotation {
    kind: AnnotationKind::Spotlight.raw(),
    flags: super::native::flags(&style),
    p0,
    p1,
    p2,
    reveal: AnnotationReveal {
      opacity: presence,
      ..AnnotationReveal::WHOLE
    },
    ..NativeAnnotation::default()
  }
}

#[test]
fn only_a_spotlight_that_blurs_asks_for_the_blur_pass_but_every_one_cuts_it() {
  let plain = record([0.0, 0.0], [10.0, 10.0], false, 1.0);
  assert!(blur_holes(&[plain], 100, 100).is_none());
  let blurring = record([20.0, 20.0], [40.0, 30.0], true, 0.6);
  let (strength, holes) = blur_holes(&[plain, blurring], 100, 100).unwrap();
  assert_eq!(strength, 0.6);
  // Four points a hole, both holes.
  assert_eq!(holes.len(), 8);
  assert_eq!(holes[4], [20.0, 20.0]);
  assert_eq!(holes[5], [40.0, 30.0]);
  // Measured against the blur's own presence, so its light is clear.
  assert_eq!(holes[7], [1.0, 0.0]);
}
