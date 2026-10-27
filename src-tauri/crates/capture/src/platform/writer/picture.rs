// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's picture readings (`screenwide-activity`), taken from each
//! frame as it is written, so a recording's readings are ready when it stops.
//! A frame is shrunk only when a reading falls due, ten times a second, by
//! reading a lattice of its pixels in place: no copy of the frame is made.

use cidre::cv;
use screenwide_activity::{grey_from_bgra, grey_from_luma, PictureTap};

use super::*;

#[link(name = "CoreVideo", kind = "framework")]
extern "C" {
  fn CVPixelBufferGetBaseAddress(buffer: &cv::PixelBuf) -> *const u8;
}

impl Writer {
  /// The readings for the movie at `path`, or none if they cannot be
  /// written: they help the editor, and never stop a recording.
  pub(super) fn picture_tap(path: &std::path::Path) -> Option<PictureTap> {
    PictureTap::create(path)
      .inspect_err(|error| eprintln!("Not reading this recording's picture: {error}"))
      .ok()
  }

  pub(super) fn read_picture(&mut self, frame: &Frame, pts_ns: i64) {
    let Some(tap) = self.picture.as_mut() else {
      return;
    };
    if let Err(error) = tap.frame(pts_ns, || grey(&frame.buf)) {
      eprintln!("Stopped reading this recording's picture: {error}");
      self.picture = None;
    }
  }

  pub(super) fn finish_picture(&mut self, end_ns: i64) {
    if let Some(tap) = self.picture.take() {
      if let Err(error) = tap.finish(end_ns) {
        eprintln!("Could not finish this recording's picture readings: {error}");
      }
    }
  }
}

/// The frame as grey cells: a screen's BGRA, or the luma plane of a
/// camera's YUV. Other formats are not read.
fn grey(buffer: &arc::R<cv::PixelBuf>) -> Option<Vec<u8>> {
  let mut buffer = buffer.retained();
  let flags = cv::pixel_buffer::LockFlags::READ_ONLY;
  let format = buffer.pixel_format();
  let is_bgra = format == cv::PixelFormat::_32_BGRA;
  let is_yuv = buffer.is_planar() && buffer.plane_count() == 2;
  if !is_bgra && !is_yuv {
    return None;
  }
  // SAFETY: the buffer is retained for this call, locked read-only while
  // its bytes are read, and the slice never outlives the lock.
  unsafe {
    buffer.lock_base_addr(flags).result().ok()?;
    let grey = if is_bgra {
      let (width, height, stride) = (buffer.width(), buffer.height(), buffer.bytes_per_row());
      let base = CVPixelBufferGetBaseAddress(&buffer);
      (!base.is_null()).then(|| {
        let pixels = std::slice::from_raw_parts(base, stride * height);
        grey_from_bgra(pixels, width, height, stride)
      })
    } else {
      let (width, height) = (buffer.plane_width(0), buffer.plane_height(0));
      let stride = buffer.plane_bytes_per_row(0);
      let base = buffer.plane_base_address(0);
      (!base.is_null()).then(|| {
        let plane = std::slice::from_raw_parts(base, stride * height);
        grey_from_luma(plane, width, height, stride)
      })
    };
    let _ = buffer.unlock_lock_base_addr(flags);
    grey
  }
}
