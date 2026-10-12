// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use cidre::cf;

/// The scaler wraps a frame's IOSurface in place, which is what the camera
/// itself delivers.
fn pixel_buffer(width: usize, height: usize, format: cv::PixelFormat) -> arc::R<cv::PixelBuf> {
  let empty = cf::Dictionary::with_keys_values(&[], &[]).expect("an empty attribute dictionary");
  let attributes = cf::Dictionary::with_keys_values(
    &[cv::pixel_buffer::keys::io_surf_props().as_type_ref()],
    &[empty.as_type_ref()],
  )
  .expect("the pixel buffer attributes");
  cv::PixelBuf::new(width, height, format, Some(&attributes)).expect("a test pixel buffer")
}

/// The guard only borrows the buffer for its own lifetime, which leaves the
/// plane accessors usable while the base address stays locked.
struct Lock(*mut cv::PixelBuf);

impl Drop for Lock {
  fn drop(&mut self) {
    let result = unsafe { (*self.0).unlock_lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT) };
    assert!(result.is_ok(), "the test buffer unlocks");
  }
}

fn lock(buffer: &mut cv::PixelBuf) -> Lock {
  let result = unsafe { buffer.lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT) };
  assert!(result.is_ok(), "the test buffer locks");
  Lock(buffer)
}

fn assert_every_pixel_is(pixels: &[u8], expected: [u8; 3]) {
  let (texels, _) = pixels.as_chunks::<4>();
  for pixel in texels {
    assert_eq!(pixel[3], 255);
    for (actual, expected) in pixel[..3].iter().zip(expected) {
      assert!(
        actual.abs_diff(expected) <= 2,
        "expected {expected} but the scaler produced {actual}"
      );
    }
  }
}

#[test]
fn scales_a_bgra_frame_to_the_thumbnail_size() {
  let mut scaler = Scaler::create().expect("a thumbnail scaler");
  let mut buffer = pixel_buffer(640, 480, cv::PixelFormat::_32_BGRA);
  {
    let _lock = lock(&mut buffer);
    let stride = buffer.plane_bytes_per_row(0);
    let base = buffer.plane_base_address(0).cast_mut();
    for row in 0..480 {
      for column in 0..640 {
        // Blue, green, red, alpha as CoreVideo stores 32BGRA.
        let pixel = unsafe { base.add(row * stride + column * 4) };
        unsafe { pixel.copy_from_nonoverlapping([50_u8, 100, 200, 255].as_ptr(), 4) };
      }
    }
  }

  let (width, height) = thumbnail_size(&buffer).expect("a thumbnail size");
  assert_eq!((width, height), (72, 54));
  let mut pixels = vec![0_u8; usize::from(width) * usize::from(height) * 4];
  assert!(scaler.thumbnail(&buffer, width, height, &mut pixels));
  assert_every_pixel_is(&pixels, [200, 100, 50]);
}

#[test]
fn converts_a_video_range_biplanar_frame_to_rgba() {
  let mut scaler = Scaler::create().expect("a thumbnail scaler");
  let mut buffer = pixel_buffer(640, 480, cv::PixelFormat::_420V);
  {
    let _lock = lock(&mut buffer);
    // BT.709 video range encoding of red 200, green 100, blue 50.
    let luma_stride = buffer.plane_bytes_per_row(0);
    let luma = buffer.plane_base_address(0).cast_mut();
    for row in 0..480 {
      unsafe { luma.add(row * luma_stride).write_bytes(117, 640) };
    }
    let chroma_stride = buffer.plane_bytes_per_row(1);
    let chroma = buffer.plane_base_address(1).cast_mut();
    for row in 0..240 {
      for column in 0..320 {
        let pixel = unsafe { chroma.add(row * chroma_stride + column * 2) };
        unsafe { pixel.copy_from_nonoverlapping([96_u8, 174].as_ptr(), 2) };
      }
    }
  }

  let (width, height) = thumbnail_size(&buffer).expect("a thumbnail size");
  let mut pixels = vec![0_u8; usize::from(width) * usize::from(height) * 4];
  assert!(scaler.thumbnail(&buffer, width, height, &mut pixels));
  assert_every_pixel_is(&pixels, [200, 100, 50]);
}

/// The thumbnail is the frame the right way up and the right way round.
#[test]
fn keeps_the_frame_orientation() {
  let mut scaler = Scaler::create().expect("a thumbnail scaler");
  let mut buffer = pixel_buffer(640, 480, cv::PixelFormat::_32_BGRA);
  {
    let _lock = lock(&mut buffer);
    let stride = buffer.plane_bytes_per_row(0);
    let base = buffer.plane_base_address(0).cast_mut();
    for row in 0..480 {
      for column in 0..640 {
        // Red across the top-left quarter, blue everywhere else.
        let bgra = if row < 240 && column < 320 {
          [0_u8, 0, 255, 255]
        } else {
          [255, 0, 0, 255]
        };
        let pixel = unsafe { base.add(row * stride + column * 4) };
        unsafe { pixel.copy_from_nonoverlapping(bgra.as_ptr(), 4) };
      }
    }
  }

  let (width, height) = thumbnail_size(&buffer).expect("a thumbnail size");
  let mut pixels = vec![0_u8; usize::from(width) * usize::from(height) * 4];
  assert!(scaler.thumbnail(&buffer, width, height, &mut pixels));
  let at = |x: u16, y: u16| {
    let start = (usize::from(y) * usize::from(width) + usize::from(x)) * 4;
    [pixels[start], pixels[start + 1], pixels[start + 2]]
  };
  assert_eq!(at(0, 0), [255, 0, 0], "the top-left is red");
  assert_eq!(at(width - 1, 0), [0, 0, 255], "the top-right is blue");
  assert_eq!(at(0, height - 1), [0, 0, 255], "the bottom-left is blue");
}

#[test]
fn rejects_an_unsupported_pixel_format() {
  let mut scaler = Scaler::create().expect("a thumbnail scaler");
  let buffer = pixel_buffer(64, 64, cv::PixelFormat::_32_ARGB);
  let mut pixels = vec![0_u8; 48 * 30 * 4];
  assert!(!scaler.thumbnail(&buffer, 48, 30, &mut pixels));
}
