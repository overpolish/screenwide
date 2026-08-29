// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::{cf, cv};

use super::*;
use crate::desktop_capture::{DesktopRect, PixelRect};

fn buffer(width: usize, height: usize, bgra: [u8; 4]) -> arc::R<cv::PixelBuf> {
  let empty = cf::Dictionary::with_keys_values(&[], &[]).unwrap();
  let attributes = cf::Dictionary::with_keys_values(
    &[cv::pixel_buffer::keys::io_surf_props().as_type_ref()],
    &[empty.as_type_ref()],
  )
  .unwrap();
  let mut pixels = cv::PixelBuf::new(width, height, cv::PixelFormat::_32_BGRA, Some(&attributes))
    .expect("a BGRA test surface");
  assert!(unsafe { pixels.lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT) }.is_ok());
  let stride = pixels.bytes_per_row();
  let base = unsafe { pixels.base_address_mut() }.cast::<u8>();
  for y in 0..height {
    for x in 0..width {
      unsafe {
        base
          .add(y * stride + x * 4)
          .copy_from_nonoverlapping(bgra.as_ptr(), 4)
      };
    }
  }
  assert!(unsafe { pixels.unlock_lock_base_addr(cv::pixel_buffer::LockFlags::DEFAULT) }.is_ok());
  pixels
}

fn plan() -> CapturePlan {
  CapturePlan {
    desktop_region: DesktopRect {
      x: 0.0,
      y: 0.0,
      width: 4.0,
      height: 2.0,
    },
    width: 4,
    height: 2,
    output_scale: 1.0,
    pieces: vec![
      CapturePiece {
        display_id: 1,
        source_pixels: PixelRect {
          x: 0,
          y: 0,
          width: 2,
          height: 2,
        },
        destination: PixelRect {
          x: 0,
          y: 0,
          width: 2,
          height: 2,
        },
      },
      CapturePiece {
        display_id: 2,
        source_pixels: PixelRect {
          x: 0,
          y: 0,
          width: 2,
          height: 2,
        },
        destination: PixelRect {
          x: 2,
          y: 0,
          width: 2,
          height: 2,
        },
      },
    ],
  }
}

#[test]
fn native_piece_samples_from_the_already_cropped_stream_origin() {
  let piece = CapturePiece {
    display_id: 1,
    source_pixels: PixelRect {
      x: 600,
      y: 300,
      width: 800,
      height: 400,
    },
    destination: PixelRect {
      x: 25,
      y: 10,
      width: 400,
      height: 200,
    },
  };
  let native = NativePiece::from_cropped_source(piece);
  assert_eq!((native.source_x, native.source_y), (0, 0));
  assert_eq!((native.source_width, native.source_height), (800, 400));
  assert_eq!((native.destination_x, native.destination_y), (25, 10));
}

#[test]
fn waits_for_every_source_then_composes_the_shared_canvas() {
  let mut coordinator = DesktopFrameCoordinator::new(&plan()).unwrap();
  let blue = buffer(2, 2, [255, 0, 0, 255]);
  let red = buffer(2, 2, [0, 0, 255, 255]);
  assert!(coordinator.update(0, 10, &blue).unwrap().is_none());
  let mut composed = coordinator.update(1, 12, &red).unwrap().unwrap();
  assert_eq!(composed.timestamp_ns, 12);
  assert_eq!((composed.buffer.width(), composed.buffer.height()), (4, 2));
  assert!(unsafe {
    composed
      .buffer
      .lock_base_addr(cv::pixel_buffer::LockFlags::READ_ONLY)
  }
  .is_ok());
  let base = unsafe { composed.buffer.base_address() }.cast::<u8>();
  let left = unsafe { std::slice::from_raw_parts(base, 4) };
  let right = unsafe { std::slice::from_raw_parts(base.add(3 * 4), 4) };
  assert_eq!(left, [255, 0, 0, 255]);
  assert_eq!(right, [0, 0, 255, 255]);
  assert!(unsafe {
    composed
      .buffer
      .unlock_lock_base_addr(cv::pixel_buffer::LockFlags::READ_ONLY)
  }
  .is_ok());
}

#[test]
fn a_stale_surface_cannot_replace_the_latest_source_pixels() {
  let mut coordinator = DesktopFrameCoordinator::new(&plan()).unwrap();
  let blue = buffer(2, 2, [255, 0, 0, 255]);
  let green = buffer(2, 2, [0, 255, 0, 255]);
  let red = buffer(2, 2, [0, 0, 255, 255]);
  coordinator.update(0, 10, &blue).unwrap();
  coordinator.update(1, 10, &red).unwrap();
  assert!(coordinator.update(0, 9, &green).unwrap().is_none());
  let mut composed = coordinator.update(1, 11, &red).unwrap().unwrap();
  assert!(unsafe {
    composed
      .buffer
      .lock_base_addr(cv::pixel_buffer::LockFlags::READ_ONLY)
  }
  .is_ok());
  let left = unsafe { std::slice::from_raw_parts(composed.buffer.base_address().cast::<u8>(), 4) };
  assert_eq!(left, [255, 0, 0, 255]);
  assert!(unsafe {
    composed
      .buffer
      .unlock_lock_base_addr(cv::pixel_buffer::LockFlags::READ_ONLY)
  }
  .is_ok());
}
