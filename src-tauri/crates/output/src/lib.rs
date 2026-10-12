// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a still or a recording's frame is composed into: the canvas, its
//! background (a colour, a picture or a mesh gradient), the source's crop and
//! placement on it, and the annotations over it. The compositors read this;
//! the app captures the source and runs them.

mod background_image;
mod capture;
pub mod encoding;
#[cfg(any(test, feature = "test-support"))]
pub mod fixtures;
mod hex_colour;
#[cfg(target_os = "macos")]
mod macos {
  pub(crate) mod image_decode;
}
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub mod mesh;
pub mod mesh_generator;
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod mesh_gpu;
mod output;
mod placement;
mod recenter;
mod source_crop;

pub use background_image::{
  background_image_canvas, background_image_swatch, load as load_background_image,
};
pub use capture::Capture;
pub use encoding::{encode_png, rounded_corners};
pub use hex_colour::parse_hex_colour;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use mesh::validate_mesh;
pub use mesh::MeshGradientPoint;
pub use mesh_generator::default_generator as default_mesh_generator;
#[cfg(target_os = "windows")]
pub use mesh_generator::generator_seed_shift;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use mesh_generator::{generator_palette, mesh_generator};
#[cfg(not(target_os = "macos"))]
pub use output::compose_screenshot;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use output::output_dimensions;
pub use output::{CropPreviewRect, ScreenshotOutputSettings};
pub use placement::{output_placement, OutputPlacement};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use recenter::{colour_f32, foreground_bounds_f32, optional_colour_f32};
use screenwide_imaging::CapturedImage;
pub use source_crop::NormalizedSourceRect;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn validate_output_settings(
  source_width: u32,
  source_height: u32,
  settings: &ScreenshotOutputSettings,
) -> Result<(), String> {
  output_placement(source_width, source_height, settings)?;
  parse_hex_colour(&settings.background_color)?;
  if let Some(colour) = settings.recenter_inset_color.as_deref() {
    parse_hex_colour(colour)?;
  }
  if !settings.radius_percent.is_finite()
    || !(0.0..=50.0).contains(&settings.radius_percent)
    || !settings.background_radius_percent.is_finite()
    || !(0.0..=50.0).contains(&settings.background_radius_percent)
  {
    return Err("The output corner radius is not valid".to_owned());
  }
  match settings.background_type.as_str() {
    // A picture that cannot be loaded falls back to the solid colour at
    // render time, so a missing file is not a settings error.
    "image" | "solid" => Ok(()),
    "mesh" => mesh::validate_mesh(
      &settings.mesh_generator,
      &settings.mesh_colors,
      &settings.mesh_points,
      settings.mesh_warp_percent,
    ),
    _ => Err("The output background type is not valid".to_owned()),
  }
}
