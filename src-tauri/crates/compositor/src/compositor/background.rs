// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the canvas background reads: the classic mesh's points and colours,
//! or a ported generator's palette and seed shift.

use super::*;
use screenwide_output::{CanvasBackground, MeshGradientPoint};

/// The fields a background is drawn from, borrowed from the canvas or from
/// the background giving way on it.
pub(super) struct Ground<'a> {
  pub(super) kind: &'a str,
  pub(super) generator: &'a str,
  pub(super) colors: &'a [String],
  pub(super) points: &'a [MeshGradientPoint],
  pub(super) seed: u32,
  pub(super) warp_percent: f64,
}

impl<'a> From<&'a ScreenshotOutputSettings> for Ground<'a> {
  fn from(settings: &'a ScreenshotOutputSettings) -> Self {
    Self {
      kind: &settings.background_type,
      generator: &settings.mesh_generator,
      colors: &settings.mesh_colors,
      points: &settings.mesh_points,
      seed: settings.mesh_seed,
      warp_percent: settings.mesh_warp_percent,
    }
  }
}

impl<'a> From<&'a CanvasBackground> for Ground<'a> {
  fn from(background: &'a CanvasBackground) -> Self {
    Self {
      kind: &background.background_type,
      generator: &background.mesh_generator,
      colors: &background.mesh_colors,
      points: &background.mesh_points,
      seed: background.mesh_seed,
      warp_percent: background.mesh_warp_percent,
    }
  }
}

pub(super) struct BackgroundConstants {
  pub(super) mesh: bool,
  pub(super) points: [[f32; 4]; 8],
  pub(super) colors: [[f32; 4]; 4],
  /// The classic mesh's base colour, under its blobs.
  pub(super) base: [f32; 4],
  /// Zero is the app's own blob mesh, which alone reads the points and the
  /// warp; every other id names a generator that reads only its palette and
  /// the seed.
  pub(super) generator: u32,
  pub(super) palette_size: u32,
  /// The classic mesh reads the seconds as they come, so a canvas with no
  /// mesh at all carries its speed rather than a zero.
  pub(super) speed: f32,
}

pub(super) fn background_constants(ground: Ground<'_>) -> Result<BackgroundConstants, String> {
  let mesh = ground.kind == "mesh";
  // A chosen picture is a background of its own: the shader fills it to the
  // canvas from `background_image`, falling back to the solid colour when
  // the file behind it cannot be read.
  if !mesh && !matches!(ground.kind, "image" | "solid") {
    return Err("The screenshot background type is not valid".to_owned());
  }
  let mut background = BackgroundConstants {
    mesh,
    points: [[0.0; 4]; 8],
    colors: [[0.0; 4]; 4],
    base: [0.0; 4],
    generator: 0,
    palette_size: 0,
    speed: 1.0,
  };
  if !mesh {
    return Ok(background);
  }
  validate_mesh(
    ground.generator,
    ground.colors,
    ground.points,
    ground.warp_percent,
  )?;
  let generator =
    mesh_generator(ground.generator).ok_or("The screenshot mesh background is not valid")?;
  background.generator = generator.id;
  background.palette_size = generator.color_count as u32;
  background.speed = generator.speed;
  if generator.id == 0 {
    for (index, point) in ground.points.iter().enumerate() {
      let radians = point.rotation.to_radians();
      background.points[index * 2] = [
        (point.x / 100.0) as f32,
        (point.y / 100.0) as f32,
        (point.radius_x / 100.0) as f32,
        (point.radius_y / 100.0) as f32,
      ];
      background.points[index * 2 + 1] = [radians.cos() as f32, radians.sin() as f32, 0.0, 0.0];
      background.colors[index] = colour_f32(&ground.colors[index])?;
    }
    background.base = colour_f32(
      ground
        .colors
        .last()
        .expect("a validated mesh has a base colour"),
    )?;
  } else {
    background.colors = generator_palette(ground.colors)?;
    // Ported generators reuse the classic mesh point slot for their seed
    // domain shift; they do not read classic points.
    background.points[0] = generator_shift(ground.seed);
  }
  Ok(background)
}
