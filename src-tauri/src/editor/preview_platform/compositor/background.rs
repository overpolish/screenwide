// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the canvas background reads: the classic mesh's points and colours,
//! or a ported generator's palette and seed shift.

use super::*;

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

pub(super) fn background_constants(
  settings: &ScreenshotOutputSettings,
) -> Result<BackgroundConstants, String> {
  let mesh = settings.background_type == "mesh";
  // A chosen picture is a background of its own: the shader fills it to the
  // canvas from `background_image`, falling back to the solid colour when
  // the file behind it cannot be read.
  if !mesh && !matches!(settings.background_type.as_str(), "image" | "solid") {
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
    &settings.mesh_generator,
    &settings.mesh_colors,
    &settings.mesh_points,
    settings.mesh_warp_percent,
  )?;
  let generator = mesh_generator(&settings.mesh_generator)
    .ok_or("The screenshot mesh background is not valid")?;
  background.generator = generator.id;
  background.palette_size = generator.color_count as u32;
  background.speed = generator.speed;
  if generator.id == 0 {
    for (index, point) in settings.mesh_points.iter().enumerate() {
      let radians = point.rotation.to_radians();
      background.points[index * 2] = [
        (point.x / 100.0) as f32,
        (point.y / 100.0) as f32,
        (point.radius_x / 100.0) as f32,
        (point.radius_y / 100.0) as f32,
      ];
      background.points[index * 2 + 1] = [radians.cos() as f32, radians.sin() as f32, 0.0, 0.0];
      background.colors[index] = colour_f32(&settings.mesh_colors[index])?;
    }
    background.base = colour_f32(
      settings
        .mesh_colors
        .last()
        .expect("a validated mesh has a base colour"),
    )?;
  } else {
    background.colors = generator_palette(&settings.mesh_colors)?;
    // Ported generators reuse the classic mesh point slot for their seed
    // domain shift; they do not read classic points.
    background.points[0] = generator_shift(settings.mesh_seed);
  }
  Ok(background)
}
