// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A length on a layer, its corners' radius or its padding. The looks are a
//! share of the canvas's shorter side, so a look rounds and pads a full
//! display and a small region alike, and a screen and a camera given the
//! same look have the same corners, whatever was captured and however large
//! each is drawn. A camera drawn as a circle is a share of its own frame,
//! so it stays one whatever its crop. Settings saved before lengths had a
//! unit hold a bare share of the frame, read as a percent; settings saved
//! while they were points of the capture read each point as a pixel of a
//! 1080 high canvas, near the look it stood for, until the editor snaps
//! them to a look.

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Measure {
  /// Percent of the canvas's shorter side.
  Canvas(f64),
  /// Percent of the frame's shorter side.
  Percent(f64),
}

impl Default for Measure {
  fn default() -> Self {
    Self::Canvas(0.0)
  }
}

/// The canvas height a saved point was drawn on, near enough, when the looks
/// were points: a full display drawn on a 1080 high canvas is about a pixel
/// per point.
const LEGACY_POINTS_CANVAS: f64 = 1080.0;

impl Measure {
  /// The length in canvas pixels on a frame whose shorter side is `frame`
  /// canvas pixels, on a canvas whose shorter side is `canvas`.
  pub fn pixels(self, frame: f64, canvas: f64) -> f64 {
    match self {
      Self::Canvas(percent) => canvas.max(0.0) * percent / 100.0,
      Self::Percent(percent) => frame.max(0.0) * percent / 100.0,
    }
    .max(0.0)
  }

  /// As a corner radius: never more than half the frame's shorter side, so
  /// a radius too big for a thin frame rounds its ends fully instead.
  pub fn radius_pixels(self, frame: f64, canvas: f64) -> f64 {
    self.pixels(frame, canvas).min(frame.max(0.0) / 2.0)
  }

  /// As a corner radius in percent of the frame's shorter side, as a
  /// selection's outline is rounded: the outline is drawn in the view's
  /// pixels, where the canvas's are not known.
  pub fn radius_frame_percent(self, frame: f64, canvas: f64) -> f64 {
    if frame > 0.0 {
      self.radius_pixels(frame, canvas) / frame * 100.0
    } else {
      0.0
    }
  }

  /// Whether the length is a number, not negative, and a percent of the
  /// frame no more than `max_percent`.
  pub fn is_valid(self, max_percent: f64) -> bool {
    match self {
      Self::Canvas(percent) => percent.is_finite() && percent >= 0.0,
      Self::Percent(percent) => percent.is_finite() && (0.0..=max_percent).contains(&percent),
    }
  }

  /// `share` of the way from this length to `to`. Lengths in different
  /// units cannot be mixed, so the change happens halfway.
  pub fn between(self, to: Self, share: f64) -> Self {
    match (self, to) {
      (Self::Canvas(from), Self::Canvas(to)) => Self::Canvas(from + (to - from) * share),
      (Self::Percent(from), Self::Percent(to)) => Self::Percent(from + (to - from) * share),
      _ if share < 0.5 => self,
      _ => to,
    }
  }

  pub fn is_zero(self) -> bool {
    match self {
      Self::Canvas(value) | Self::Percent(value) => value <= 0.0,
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum MeasureFields {
  Canvas(f64),
  Percent(f64),
  Points(f64),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Saved {
  Bare(f64),
  Measure(MeasureFields),
}

impl Saved {
  fn read(self, bare_percent: f64) -> Measure {
    match self {
      Self::Bare(value) => Measure::Percent(value * bare_percent),
      Self::Measure(MeasureFields::Canvas(percent)) => Measure::Canvas(percent),
      Self::Measure(MeasureFields::Percent(percent)) => Measure::Percent(percent),
      Self::Measure(MeasureFields::Points(points)) => {
        Measure::Canvas(points / LEGACY_POINTS_CANVAS * 100.0)
      }
    }
  }
}

/// Reads a length, or a bare percent saved before lengths had a unit.
pub fn measure_or_bare_percent<'de, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<Measure, D::Error> {
  Ok(Saved::deserialize(deserializer)?.read(1.0))
}

/// Reads an optional length, or a bare percent saved before lengths had a
/// unit.
pub fn optional_measure_or_bare_percent<'de, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<Option<Measure>, D::Error> {
  Ok(Option::<Saved>::deserialize(deserializer)?.map(|saved| saved.read(1.0)))
}

/// Reads an optional length, or a bare share (a fraction, not a percent)
/// saved before lengths had a unit, as a padding was.
pub fn optional_measure_or_bare_share<'de, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<Option<Measure>, D::Error> {
  Ok(Option::<Saved>::deserialize(deserializer)?.map(|saved| saved.read(100.0)))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Deserialize)]
  #[serde(rename_all = "camelCase")]
  struct Held {
    #[serde(alias = "radiusPercent", deserialize_with = "measure_or_bare_percent")]
    radius: Measure,
    #[serde(default, deserialize_with = "optional_measure_or_bare_share")]
    inset: Option<Measure>,
  }

  #[test]
  fn bare_values_saved_before_units_read_as_percents() {
    let held: Held = serde_json::from_str(r#"{ "radiusPercent": 12, "inset": 0.06 }"#).unwrap();
    assert_eq!(held.radius, Measure::Percent(12.0));
    assert_eq!(held.inset, Some(Measure::Percent(6.0)));
  }

  #[test]
  fn points_read_as_pixels_of_a_1080_high_canvas() {
    let held: Held =
      serde_json::from_str(r#"{ "radius": { "points": 27 }, "inset": { "points": 54 } }"#).unwrap();
    let canvas = |measure: Measure| match measure {
      Measure::Canvas(percent) => percent,
      Measure::Percent(_) => panic!("a point is not a share of the frame"),
    };
    assert!((canvas(held.radius) - 2.5).abs() < 1e-9);
    assert!((canvas(held.inset.unwrap()) - 5.0).abs() < 1e-9);
  }

  #[test]
  fn a_measure_reads_back_as_written() {
    let written = serde_json::to_string(&Measure::Canvas(3.0)).unwrap();
    let held: Held =
      serde_json::from_str(&format!(r#"{{ "radius": {written}, "inset": {written} }}"#)).unwrap();
    assert_eq!(held.radius, Measure::Canvas(3.0));
    assert_eq!(held.inset, Some(Measure::Canvas(3.0)));
  }

  #[test]
  fn a_canvas_share_ignores_the_frame_and_a_radius_never_passes_half_the_frame() {
    assert_eq!(Measure::Canvas(3.0).radius_pixels(400.0, 1080.0), 32.4);
    assert_eq!(Measure::Canvas(3.0).radius_pixels(900.0, 1080.0), 32.4);
    assert_eq!(Measure::Canvas(3.0).radius_pixels(30.0, 1080.0), 15.0);
    assert_eq!(Measure::Canvas(3.0).pixels(30.0, 1080.0), 32.4);
    assert_eq!(Measure::Percent(50.0).radius_pixels(300.0, 1080.0), 150.0);
    assert_eq!(
      Measure::Canvas(3.0).radius_frame_percent(200.0, 1000.0),
      15.0
    );
  }
}
