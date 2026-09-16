// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn with_color(color: &str) -> AnnotateSettings {
  AnnotateSettings {
    default_color: color.to_owned(),
    ..AnnotateSettings::default()
  }
}

#[test]
fn a_colour_the_compositor_cannot_read_is_refused() {
  assert!(validated(with_color("red")).is_err());
  assert!(validated(with_color("#fc0")).is_err());
  assert!(validated(with_color("#gggggg")).is_err());
}

#[test]
fn a_colour_is_kept_in_the_palettes_own_spelling() {
  assert_eq!(
    validated(with_color("#FFCC00")).unwrap().default_color,
    "#ffcc00"
  );
  assert_eq!(
    validated(with_color("#ffcc0080")).unwrap().default_color,
    "#ffcc0080"
  );
}

#[test]
fn a_width_outside_the_stroke_presets_is_refused() {
  for width in [0.0, 4.0, 64.0, f64::NAN, f64::INFINITY] {
    assert!(validated(AnnotateSettings {
      default_width: width,
      ..AnnotateSettings::default()
    })
    .is_err());
  }
  assert!(validated(AnnotateSettings {
    default_width: 48.0,
    ..AnnotateSettings::default()
  })
  .is_ok());
}

#[test]
fn a_toolbar_position_that_is_not_a_place_is_refused() {
  for (x, y) in [(f64::NAN, 0.0), (0.0, f64::INFINITY)] {
    assert!(validated(AnnotateSettings {
      toolbar_position: Some(ToolbarPosition {
        display_id: 1,
        x,
        y
      }),
      ..AnnotateSettings::default()
    })
    .is_err());
  }
  assert!(validated(AnnotateSettings {
    toolbar_position: Some(ToolbarPosition {
      display_id: 1,
      x: -12.0,
      y: 8.0
    }),
    ..AnnotateSettings::default()
  })
  .is_ok());
}

/// A stored file written before the toolbar existed still loads, and the
/// arrow it dresses keeps its head.
#[test]
fn settings_without_the_newer_fields_take_their_defaults() {
  let settings: AnnotateSettings =
    serde_json::from_str(r##"{"enabled":true,"defaultColor":"#ffcc00","defaultWidth":8.0}"##)
      .unwrap();
  assert_eq!(settings.default_head, AnnotationHead::End);
  assert_eq!(settings.toolbar_position, None);
}
