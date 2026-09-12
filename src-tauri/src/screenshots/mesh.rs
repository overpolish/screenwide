// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

use super::mesh_generator::{mesh_generator, MAXIMUM_GENERATOR_COLORS};
use super::parse_hex_colour;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MeshGradientPoint {
  pub radius_x: f64,
  pub radius_y: f64,
  pub rotation: f64,
  pub x: f64,
  pub y: f64,
}

/// `seconds` is where the canvas is on its timeline: a generator drifts with
/// it, the way the native backends drift a recording's background. A thumbnail
/// and a screenshot export are stills, so they pass zero.
#[allow(clippy::too_many_arguments)]
pub(super) fn mesh_canvas(
  width: u32,
  height: u32,
  generator: &str,
  colors: &[String],
  points: &[MeshGradientPoint],
  seed: u32,
  warp_percent: f64,
  seconds: f64,
) -> Result<image::RgbaImage, String> {
  validate_mesh(generator, colors, points, warp_percent)?;
  let generator = mesh_generator(generator).ok_or_else(invalid)?;
  let parsed = colors
    .iter()
    .map(|color| parse_hex_colour(color))
    .collect::<Result<Vec<_>, _>>()?;
  super::mesh_gpu::render(
    width,
    height,
    generator,
    &parsed,
    points,
    seed,
    warp_percent,
    seconds,
  )
}

fn invalid() -> String {
  "The screenshot mesh background is not valid".to_owned()
}

/// What a mesh background must carry to be painted.
///
/// The original generator is the strict one: a blob for every colour past the
/// first, and a warp inside the range the editor offers. A ported generator
/// reads neither, so it is checked only for a palette its shader has room
/// for, and settings that still carry blobs from a spell on the original keep
/// them untouched.
pub(crate) fn validate_mesh(
  generator: &str,
  colors: &[String],
  points: &[MeshGradientPoint],
  warp_percent: f64,
) -> Result<(), String> {
  let generator = mesh_generator(generator).ok_or_else(invalid)?;
  if generator.id != 0 {
    // A generator reads the first few colours and ignores the rest, so a
    // palette carried over from another generator is drawn rather than
    // refused: the canvas is mid-change, not broken.
    return if (1..=MAXIMUM_GENERATOR_COLORS).contains(&colors.len()) {
      Ok(())
    } else {
      Err(invalid())
    };
  }
  // One base colour plus up to four control-point colours. More than five
  // becomes visually muddy rather than adding useful variation.
  let unusable = !(3..=4).contains(&points.len())
    || colors.len() != points.len() + 1
    || !warp_percent.is_finite()
    || !(0.0..=20.0).contains(&warp_percent)
    || points.iter().any(|point| {
      !point.x.is_finite()
        || !point.y.is_finite()
        || !point.radius_x.is_finite()
        || !point.radius_y.is_finite()
        || !point.rotation.is_finite()
        || !(-25.0..=125.0).contains(&point.x)
        || !(-25.0..=125.0).contains(&point.y)
        || !(20.0..=120.0).contains(&point.radius_x)
        || !(20.0..=120.0).contains(&point.radius_y)
        || !(-360.0..=360.0).contains(&point.rotation)
    });
  if unusable {
    Err(invalid())
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The mesh carried a six-digit-only parser of its own for a while. It now
  /// shares the screenshot one, so the short forms expand here as well.
  #[test]
  fn a_mesh_colour_accepts_the_short_forms() {
    assert_eq!(parse_hex_colour("#1a3").unwrap(), [17, 170, 51, 255]);
    assert_eq!(parse_hex_colour("1a3").unwrap(), [17, 170, 51, 255]);
    assert_eq!(parse_hex_colour("#11aa33").unwrap(), [17, 170, 51, 255]);
    assert_eq!(parse_hex_colour("#12").unwrap(), [18, 18, 18, 255]);
    assert!(parse_hex_colour("#12345").is_err());
  }
}
