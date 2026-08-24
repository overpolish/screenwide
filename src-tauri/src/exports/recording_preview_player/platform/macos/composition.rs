// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::cursor::GpuCursorPreview;
use crate::{
  exports::{cursor_effects::GpuCursor, media_preview, CameraOverlaySettings},
  screenshots::{self, CapturedImage, ScreenshotOutputSettings, StillOverlay},
};

pub(super) fn gpu_still_overlay(
  screen: &CapturedImage,
  output: &ScreenshotOutputSettings,
  cursor: Option<&GpuCursorPreview>,
  camera: Option<&CapturedImage>,
  camera_overlay: Option<CameraOverlaySettings>,
  camera_drop_shadow: bool,
  camera_on_top: bool,
) -> Result<(Option<GpuCursor>, Option<StillOverlay>), String> {
  let placement = screenshots::output_placement(screen.width, screen.height, output)?;
  let cursor = cursor.map(|preview| {
    let scale_x = f64::from(placement.image_width) / f64::from(preview.canvas_width.max(1));
    let scale_y = f64::from(placement.image_height) / f64::from(preview.canvas_height.max(1));
    let mut cursor = preview.cursor;
    cursor.x = (placement.image_x + f64::from(cursor.x) * scale_x) as f32;
    cursor.y = (placement.image_y + f64::from(cursor.y) * scale_y) as f32;
    cursor.width *= scale_x as f32;
    cursor.height *= scale_y as f32;
    cursor.hotspot_x *= scale_x as f32;
    cursor.hotspot_y *= scale_y as f32;
    cursor.blur_delta_x *= scale_x as f32;
    cursor.blur_delta_y *= scale_y as f32;
    cursor
  });
  let overlay = camera_overlay
    .map(|settings| {
      camera_still_overlay(camera, output, settings, camera_drop_shadow, camera_on_top)
    })
    .transpose()?;
  Ok((cursor, overlay))
}

pub(super) fn encoded_jpeg(image: &CapturedImage) -> Result<Vec<u8>, String> {
  let rgba = image::RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
    .ok_or_else(|| "The native preview compositor returned invalid pixels".to_owned())?;
  let mut bytes = Vec::new();
  image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 85)
    .encode_image(&rgba)
    .map_err(|error| error.to_string())?;
  Ok(bytes)
}

pub(super) fn camera_still_overlay(
  camera: Option<&CapturedImage>,
  output: &ScreenshotOutputSettings,
  settings: CameraOverlaySettings,
  camera_drop_shadow: bool,
  camera_on_top: bool,
) -> Result<StillOverlay, String> {
  let camera = camera.ok_or_else(|| "Camera pixels are missing from the preview".to_owned())?;
  let geometry = media_preview::bake_geometry(media_preview::BakedVideoExportOptions {
    camera_drop_shadow,
    camera_height: camera.height,
    camera_width: camera.width,
    overlay: settings,
    screen_height: output.height,
    screen_width: output.width,
    video: media_preview::VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  })?;
  Ok(StillOverlay {
    camera_crop_x: geometry.crop_x,
    camera_crop_y: geometry.crop_y,
    camera_crop_width: geometry.crop_width,
    camera_crop_height: geometry.crop_height,
    camera_frame_x: geometry.frame_x,
    camera_frame_y: geometry.frame_y,
    camera_frame_width: geometry.frame_width,
    camera_frame_height: geometry.frame_height,
    camera_radius: geometry.radius,
    camera_source_width: camera.width,
    camera_source_height: camera.height,
    camera_drop_shadow: u32::from(camera_drop_shadow),
    camera_on_top: u32::from(camera_on_top),
    ..Default::default()
  })
}

pub(super) fn decoded_rgba(encoded: &[u8]) -> Result<CapturedImage, String> {
  let rgba = image::load_from_memory(encoded)
    .map_err(|error| error.to_string())?
    .into_rgba8();
  let (width, height) = rgba.dimensions();
  Ok(CapturedImage {
    height,
    rgba: rgba.into_raw(),
    width,
  })
}
