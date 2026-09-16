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
