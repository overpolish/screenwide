// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A length on a layer, its corners' radius or its padding. A capture's are
//! points of the capture, as its annotations are, so a look pads and rounds
//! a short strip as much as a full screen and matches the corners of the
//! windows in it. A camera's corners are a share of its frame, since a camera
//! drawn as a circle stays one whatever its crop. Settings saved before
//! lengths had a unit hold a bare share, read as a percent, which draws as
//! it did until the editor snaps it to a look in points.

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Measure {
  /// Points of the capture, drawn at the layer's canvas pixels per point.
  Points(f64),
  /// Percent of the frame's shorter side.
  Percent(f64),
}

impl Default for Measure {
  fn default() -> Self {
    Self::Points(0.0)
  }
}

impl Measure {
  /// The length in canvas pixels on a frame whose shorter side is
  /// `shortest` canvas pixels, where a point of the capture is
  /// `pixels_per_point` canvas pixels.
  pub fn pixels(self, shortest: f64, pixels_per_point: f64) -> f64 {
    match self {
      Self::Points(points) => points * pixels_per_point.max(0.0),
      Self::Percent(percent) => shortest.max(0.0) * percent / 100.0,
    }
    .max(0.0)
  }

  /// As a corner radius: never more than half the shorter side, so a radius
  /// too big for a thin frame rounds its ends fully instead.
  pub fn radius_pixels(self, shortest: f64, pixels_per_point: f64) -> f64 {
    self
      .pixels(shortest, pixels_per_point)
      .min(shortest.max(0.0) / 2.0)
  }

  /// Whether the length is a number, not negative, and a percent no more
  /// than `max_percent`.
  pub fn is_valid(self, max_percent: f64) -> bool {
    match self {
      Self::Points(points) => points.is_finite() && points >= 0.0,
      Self::Percent(percent) => percent.is_finite() && (0.0..=max_percent).contains(&percent),
    }
  }

  pub fn is_zero(self) -> bool {
    match self {
      Self::Points(value) | Self::Percent(value) => value <= 0.0,
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum MeasureFields {
  Points(f64),
  Percent(f64),
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
      Self::Measure(MeasureFields::Points(points)) => Measure::Points(points),
      Self::Measure(MeasureFields::Percent(percent)) => Measure::Percent(percent),
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
  fn a_measure_reads_back_as_written() {
    let written = serde_json::to_string(&Measure::Points(10.0)).unwrap();
    let held: Held =
      serde_json::from_str(&format!(r#"{{ "radius": {written}, "inset": {written} }}"#)).unwrap();
    assert_eq!(held.radius, Measure::Points(10.0));
    assert_eq!(held.inset, Some(Measure::Points(10.0)));
  }

  #[test]
  fn points_follow_the_drawn_scale_and_a_radius_never_passes_half_the_frame() {
    assert_eq!(Measure::Points(10.0).radius_pixels(400.0, 2.0), 20.0);
    assert_eq!(Measure::Points(24.0).radius_pixels(30.0, 2.0), 15.0);
    assert_eq!(Measure::Points(24.0).pixels(30.0, 2.0), 48.0);
    assert_eq!(Measure::Percent(50.0).radius_pixels(300.0, 2.0), 150.0);
  }
}
