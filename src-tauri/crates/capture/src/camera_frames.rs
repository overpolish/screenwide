// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Camera-frame conversion for the live preview.
//!
//! Nokhwa exposes the camera's native bytes. MJPEG is deliberately left
//! compressed; YUYV and NV12 need this conversion before the browser can
//! display them. It lives in this crate, which is optimized in dev builds:
//! unoptimized, a 1080p frame takes longer to convert than the camera takes
//! to deliver the next one.

use image::{imageops, RgbaImage};
use nokhwa::{pixel_format::RgbAFormat, utils::FrameFormat, Buffer};
use rayon::{
  iter::{IndexedParallelIterator, ParallelIterator},
  slice::ParallelSliceMut,
};
use yuv::{YuvBiPlanarImage, YuvConversionMode, YuvPackedImage, YuvRange, YuvStandardMatrix};

/// The frame as RGBA at `target` size, scaled down when the camera's mode is
/// larger.
pub fn preview_rgba(frame: &Buffer, target: (u32, u32)) -> Result<RgbaImage, String> {
  let resolution = frame.resolution();
  let (width, height) = (resolution.width(), resolution.height());
  let rgba = match frame.source_frame_format() {
    FrameFormat::YUYV => yuyv_to_rgba(frame.buffer(), width, height)?,
    FrameFormat::NV12 => nv12_to_rgba(frame.buffer(), width, height)?,
    _ => {
      return frame
        .decode_image::<RgbAFormat>()
        .map_err(|error| error.to_string())
        .map(|image| scaled(image, target))
    }
  };
  RgbaImage::from_raw(width, height, rgba)
    .map(|image| scaled(image, target))
    .ok_or_else(|| "The camera preview produced an incomplete image".to_owned())
}

fn scaled(image: RgbaImage, (width, height): (u32, u32)) -> RgbaImage {
  if image.dimensions() == (width, height) {
    image
  } else {
    imageops::resize(&image, width, height, imageops::FilterType::Triangle)
  }
}

pub fn yuyv_to_rgba(buffer: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
  let packed_stride = width
    .checked_mul(2)
    .ok_or_else(|| "The camera preview row is too wide".to_owned())?;
  let height_usize = usize::try_from(height).map_err(|error| error.to_string())?;
  if height_usize == 0 || !buffer.len().is_multiple_of(height_usize) {
    return Err("The camera preview has an invalid row layout".to_owned());
  }
  // CoreVideo aligns rows for some native dimensions (1552 square is one
  // example). Treating that padding as the next row is what produced the
  // diagonal, repeated bands in the preview.
  let yuyv_stride = buffer.len() / height_usize;
  if yuyv_stride < packed_stride as usize {
    return Err("The camera preview row is incomplete".to_owned());
  }
  let rgba_stride = width * 4;
  let mut rgba = vec![0_u8; (width * height * 4) as usize];

  rgba
    .par_chunks_mut(rgba_stride as usize)
    .enumerate()
    .for_each(|(row_index, row_rgba)| {
      let input_offset = row_index * yuyv_stride;
      let input = &buffer[input_offset..input_offset + packed_stride as usize];
      let packed = YuvPackedImage {
        yuy: input,
        yuy_stride: packed_stride,
        width,
        height: 1,
      };
      let _ = yuv::yuyv422_to_rgba(
        &packed,
        row_rgba,
        rgba_stride,
        YuvRange::Full,
        YuvStandardMatrix::Bt601,
      );
    });

  Ok(rgba)
}

/// Media Foundation hands NV12 over as one contiguous buffer: the luma plane,
/// then the interleaved chroma plane at half height, both `width` bytes per
/// row. Studio-range BT.601 is what Nokhwa's own decoder assumed for these
/// frames and what the recording's encoder writes.
pub fn nv12_to_rgba(buffer: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
  if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
    return Err("The camera preview has odd NV12 dimensions".to_owned());
  }
  let luma = width as usize * height as usize;
  if buffer.len() != luma + luma / 2 {
    return Err("The camera preview has an incomplete NV12 frame".to_owned());
  }
  let (y_plane, uv_plane) = buffer.split_at(luma);
  let image = YuvBiPlanarImage {
    y_plane,
    y_stride: width,
    uv_plane,
    uv_stride: width,
    width,
    height,
  };
  let mut rgba = vec![0_u8; luma * 4];
  yuv::yuv_nv12_to_rgba(
    &image,
    &mut rgba,
    width * 4,
    YuvRange::Limited,
    YuvStandardMatrix::Bt601,
    YuvConversionMode::Balanced,
  )
  .map_err(|error| error.to_string())?;
  Ok(rgba)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ignores_core_video_row_padding() {
    let packed = [128, 128, 128, 128];
    let padded = [128, 128, 128, 128, 0, 0, 0, 0];

    assert_eq!(
      yuyv_to_rgba(&padded, 2, 1).unwrap(),
      yuyv_to_rgba(&packed, 2, 1).unwrap()
    );
  }

  #[test]
  fn reads_nv12_as_studio_range() {
    // 16 and 235 are studio black and white; neutral chroma sits at 128.
    let frame = [16, 235, 16, 235, 128, 128];
    let rgba = nv12_to_rgba(&frame, 2, 2).unwrap();
    assert_eq!(&rgba[0..3], &[0, 0, 0]);
    assert_eq!(&rgba[4..7], &[255, 255, 255]);
  }
}
