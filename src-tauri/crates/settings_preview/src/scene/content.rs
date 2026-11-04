// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a recording scene records, and the canvas it is drawn on: the
//! picture filling the box edge to edge, as a recording with no frame of its
//! own fills its video, so only what the setting changes moves.

use screenwide_compositor::compositor::FORMAT;
use screenwide_imaging::CapturedImage;
use screenwide_output::mesh_generator::default_generator;
use screenwide_output::{NormalizedSourceRect, ScreenshotOutputSettings};

use super::PictureBox;

/// The page's pictures, every one the same size: the screen, then where a
/// scene has one, the same screen with what the setting leaves out of a
/// recording on it, such as Screenwide's windows or the desktop's icons.
pub(super) struct Pictures {
  pub screen: CapturedImage,
  pub layered: Option<CapturedImage>,
}

impl Pictures {
  pub(super) fn new(mut pictures: Vec<CapturedImage>) -> Result<Self, String> {
    if pictures.is_empty() || pictures.len() > 2 {
      return Err("A recording preview takes a screen and at most one more picture".to_owned());
    }
    let layered = (pictures.len() == 2).then(|| pictures.remove(1));
    let screen = pictures.remove(0);
    if layered
      .as_ref()
      .is_some_and(|other| (other.width, other.height) != (screen.width, screen.height))
    {
      return Err("A recording preview's pictures are not the same size".to_owned());
    }
    Ok(Self { screen, layered })
  }
}

/// The settings that lay a `source`-pixel picture, `points` in size, over a
/// `canvas`-pixel canvas: covering it, centred, cut at its edges.
pub(super) fn canvas_settings(
  source: (u32, u32),
  points: (f64, f64),
  canvas: (u32, u32),
) -> ScreenshotOutputSettings {
  let (source_width, source_height) = (f64::from(source.0), f64::from(source.1));
  let (width, height) = (f64::from(canvas.0), f64::from(canvas.1));
  let scale = (width / source_width).max(height / source_height);
  let image_width = source_width * scale;
  let image_height = source_height * scale;
  ScreenshotOutputSettings {
    annotations: Vec::new(),
    background_color: "#000000".to_owned(),
    background_image_path: None,
    background_type: "solid".to_owned(),
    background_radius_percent: 0.0,
    capture_width_points: points.0,
    capture_scale: source_width / points.0.max(1.0),
    capture_corner_radius: 0.0,
    crop_height: height,
    crop_preview: None,
    crop_width: width,
    crop_x: 0.0,
    crop_y: 0.0,
    drop_shadow: false,
    drop_shadow_strength: None,
    height: canvas.1,
    image_width,
    image_x: (width - image_width) / 2.0,
    image_y: (height - image_height) / 2.0,
    inset: None,
    legacy_mode: None,
    scene_motion: None,
    scene_opacity: None,
    scene_camera_front: None,
    scene_window: None,
    camera_shadow_strength: None,
    mesh_colors: Vec::new(),
    mesh_generator: default_generator(),
    mesh_locked_colors: Vec::new(),
    mesh_points: Vec::new(),
    mesh_seed: 0,
    mesh_warp_percent: 0.0,
    radius_percent: 0.0,
    recenter_inset_color: None,
    rotation: None,
    source_crop: NormalizedSourceRect {
      height: 1.0,
      width: 1.0,
      x: 0.0,
      y: 0.0,
    },
    width: canvas.0,
  }
}

/// The settings that lay the `kept` part of a `source`-pixel picture,
/// `points` in size, over a `canvas`-pixel canvas as `canvas_settings` lays
/// the whole one: the rest falls outside the canvas, as it falls outside a
/// recording that keeps only that part.
pub(super) fn kept_settings(
  source: (u32, u32),
  points: (f64, f64),
  canvas: (u32, u32),
  kept: &PictureBox,
) -> ScreenshotOutputSettings {
  let mut settings = canvas_settings(source, points, canvas);
  let pixels_per_point = f64::from(source.0) / points.0.max(1.0);
  let (kept_width, kept_height) = (
    kept.width * pixels_per_point,
    kept.height * pixels_per_point,
  );
  if kept_width <= 0.0 || kept_height <= 0.0 {
    return settings;
  }
  let (width, height) = (f64::from(canvas.0), f64::from(canvas.1));
  let scale = (width / kept_width).max(height / kept_height);
  settings.image_width = f64::from(source.0) * scale;
  settings.image_x = (width - kept_width * scale) / 2.0 - kept.x * pixels_per_point * scale;
  settings.image_y = (height - kept_height * scale) / 2.0 - kept.y * pixels_per_point * scale;
  settings
}

/// A canvas the compositor draws into and the frame is copied or read from.
pub(crate) fn canvas_texture(
  gpu: &screenwide_gpu::Gpu,
  (width, height): (u32, u32),
) -> wgpu::Texture {
  gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide settings preview canvas"),
    size: wgpu::Extent3d {
      width: width.max(1),
      height: height.max(1),
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  })
}
