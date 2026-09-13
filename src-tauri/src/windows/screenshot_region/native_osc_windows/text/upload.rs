// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Coverage becomes premultiplied RGBA: the shader divides by alpha again, so
/// the ink colour survives the round trip unchanged.
pub(super) fn premultiply(coverage: &[u8], ink: [f32; 3]) -> Vec<u8> {
  let mut rgba = vec![0_u8; coverage.len() * 4];
  for (pixel, coverage) in rgba.chunks_exact_mut(4).zip(coverage) {
    let alpha = f32::from(*coverage) * LABEL_ALPHA;
    pixel[0] = (ink[0] * alpha).round() as u8;
    pixel[1] = (ink[1] * alpha).round() as u8;
    pixel[2] = (ink[2] * alpha).round() as u8;
    pixel[3] = alpha.round() as u8;
  }
  rgba
}

pub(super) fn build_label(
  device: &ID3D11Device,
  text: &str,
  scale: f64,
  font_size: f64,
  line_height: f64,
  ink: [f32; 3],
  mono: bool,
) -> Option<TextTexture> {
  if text.is_empty() || scale <= 0.0 {
    return None;
  }
  let raster_scale = scale * f64::from(SUPERSAMPLE);
  let context = Context::new(font_size, raster_scale, mono)?;
  let (extent_x, extent_y) = context.measure(text)?;
  let point_width = (f64::from(extent_x) / raster_scale).ceil().max(1.0);
  let point_height = line_height.ceil().max(1.0);
  let pixel_width = (point_width * scale).round().max(1.0) as i32;
  let pixel_height = (point_height * scale).round().max(1.0) as i32;
  // The glyph cell is centred in the line box, which is what gives every
  // control the same optical baseline as its React peer.
  let top = (pixel_height * SUPERSAMPLE - extent_y) / 2;
  let coverage = context.coverage((pixel_width, pixel_height), &[(text.to_owned(), 0, top)])?;
  drop(context);
  let view = upload(
    device,
    &premultiply(&coverage, ink),
    pixel_width,
    pixel_height,
  )?;
  Some(TextTexture {
    view,
    size: Size {
      width: point_width,
      height: point_height,
    },
    atlas: None,
  })
}

pub(super) fn upload(
  device: &ID3D11Device,
  rgba: &[u8],
  width: i32,
  height: i32,
) -> Option<ID3D11ShaderResourceView> {
  super::super::surface::upload_rgba(device, rgba, width as u32, height as u32)
    .inspect_err(|error| eprintln!("The Windows region OSC could not upload text: {error}"))
    .ok()
}
