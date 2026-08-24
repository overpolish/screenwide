// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{GpuArtwork, GpuCursor};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct NativeGpuCursor {
  pub blur_delta_x: f32,
  pub blur_delta_y: f32,
  pub height: f32,
  pub hotspot_x: f32,
  pub hotspot_y: f32,
  pub rotation_radians: f32,
  pub scale: f32,
  pub width: f32,
  pub x: f32,
  pub y: f32,
  pub style: u32,
  pub clip_at_video_edge: u32,
  pub visible: u32,
}

impl From<Option<GpuCursor>> for NativeGpuCursor {
  fn from(cursor: Option<GpuCursor>) -> Self {
    cursor.map_or_else(Self::default, |cursor| Self {
      blur_delta_x: cursor.blur_delta_x,
      blur_delta_y: cursor.blur_delta_y,
      height: cursor.height,
      hotspot_x: cursor.hotspot_x,
      hotspot_y: cursor.hotspot_y,
      rotation_radians: cursor.rotation_radians,
      scale: cursor.scale,
      width: cursor.width,
      x: cursor.x,
      y: cursor.y,
      style: cursor.style,
      clip_at_video_edge: u32::from(cursor.clip_at_video_edge),
      visible: 1,
    })
  }
}

#[repr(C)]
pub(crate) struct NativeGpuArtwork {
  pub pixels: *const u8,
  pub width: u32,
  pub height: u32,
  pub design_width: f32,
  pub design_height: f32,
  pub origin_x: f32,
  pub origin_y: f32,
  pub use_design: u32,
  pub clip_local_box: u32,
  pub supersample: u32,
}

impl From<&GpuArtwork> for NativeGpuArtwork {
  fn from(artwork: &GpuArtwork) -> Self {
    Self {
      pixels: artwork.pixels.as_ptr(),
      width: artwork.width,
      height: artwork.height,
      design_width: artwork.design_width,
      design_height: artwork.design_height,
      origin_x: artwork.origin_x,
      origin_y: artwork.origin_y,
      use_design: u32::from(artwork.use_design),
      clip_local_box: u32::from(artwork.clip_local_box),
      supersample: u32::from(artwork.supersample),
    }
  }
}
