// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The buffers a format beside the loop's own is drawn into, and the thread
//! that reads each one back and writes it to the format's x264 run.

use std::ffi::c_void;
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};

use cidre::{arc, cf, cv};

/// A buffer the GPU draws a frame into and the writer then reads.
///
/// SAFETY: a `CVPixelBuffer` is a reference-counted Core Foundation object
/// that may be retained and released on any thread. The loop hands a buffer
/// to the GPU and then to the writer, and takes it back only once the writer
/// has said it is free, so only one side touches its pixels at a time.
pub(super) struct Buffer(pub(super) arc::R<cv::PixelBuf>);

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

pub(super) enum Written {
  /// The frame drawn into this buffer.
  Frame(usize),
  /// The frame before, again: the loop shows a frame twice where the
  /// timeline leaves a moment out.
  Again,
}

/// An NV12 buffer a format is drawn into, backed by an IOSurface the GPU can
/// open, as the loop's own are.
pub(super) fn nv12_buffer(width: u32, height: u32) -> Result<Buffer, String> {
  let empty = cf::Dictionary::with_keys_values(&[], &[])
    .ok_or_else(|| "CoreVideo could not describe a format's frames".to_owned())?;
  let attributes = cf::Dictionary::with_keys_values(
    &[
      cv::pixel_buffer::keys::io_surf_props().as_type_ref(),
      cv::pixel_buffer::keys::metal_compatibility().as_type_ref(),
    ],
    &[empty.as_type_ref(), cf::Boolean::value_true().as_type_ref()],
  )
  .ok_or_else(|| "CoreVideo could not describe a format's frames".to_owned())?;
  cv::PixelBuf::new(
    width as usize,
    height as usize,
    cv::PixelFormat::_420V,
    Some(&attributes),
  )
  .map(Buffer)
  .map_err(|error| format!("CoreVideo could not make a format's frames: {error:?}"))
}

/// Writes each frame it is given to x264, its rows packed as raw NV12 has
/// them, and gives the buffer back once read.
pub(super) fn write_frames(
  mut input: std::process::ChildStdin,
  buffers: &[Buffer],
  width: usize,
  frames: &Receiver<Written>,
  free: &Sender<usize>,
) -> Result<(), String> {
  let mut packed = Vec::new();
  for written in frames {
    if let Written::Frame(index) = written {
      pack(&buffers[index].0, width, &mut packed)?;
      // The loop may be gone already; the frame is still written.
      let _ = free.send(index);
    }
    input
      .write_all(&packed)
      .map_err(|error| format!("the encoder stopped taking frames: {error}"))?;
  }
  Ok(())
}

/// `buffer`'s two planes into `packed`, their rows packed: IOSurface rows
/// are padded past the picture, and raw video has none.
fn pack(buffer: &cv::PixelBuf, width: usize, packed: &mut Vec<u8>) -> Result<(), String> {
  // The `CVPixelBufferRef` itself: cidre's buffers are the Core Foundation
  // object they wrap.
  let pointer = std::ptr::from_ref(buffer).cast::<c_void>();
  // SAFETY: the buffer is alive for the call, and its base address is only
  // read, between this lock and the unlock below on every path. The GPU
  // finished drawing into it before it was handed on.
  if unsafe { CVPixelBufferLockBaseAddress(pointer, LOCK_READ_ONLY) } != 0 {
    return Err("a drawn frame could not be read".to_owned());
  }
  packed.clear();
  for plane in 0..2 {
    let rows = unsafe { CVPixelBufferGetHeightOfPlane(pointer, plane) };
    let stride = unsafe { CVPixelBufferGetBytesPerRowOfPlane(pointer, plane) };
    let base = unsafe { CVPixelBufferGetBaseAddressOfPlane(pointer, plane) };
    let row_bytes = if plane == 0 {
      width
    } else {
      width.div_ceil(2) * 2
    };
    for row in 0..rows {
      // SAFETY: each row lies within the locked plane: `stride` is at least
      // `row_bytes`, and `rows` is the plane's height.
      packed.extend_from_slice(unsafe {
        std::slice::from_raw_parts(base.add(row * stride), row_bytes)
      });
    }
  }
  unsafe { CVPixelBufferUnlockBaseAddress(pointer, LOCK_READ_ONLY) };
  Ok(())
}

/// `kCVPixelBufferLock_ReadOnly`.
const LOCK_READ_ONLY: u64 = 1;

unsafe extern "C" {
  fn CVPixelBufferLockBaseAddress(buffer: *const c_void, flags: u64) -> i32;
  fn CVPixelBufferUnlockBaseAddress(buffer: *const c_void, flags: u64) -> i32;
  fn CVPixelBufferGetBaseAddressOfPlane(buffer: *const c_void, plane: usize) -> *const u8;
  fn CVPixelBufferGetBytesPerRowOfPlane(buffer: *const c_void, plane: usize) -> usize;
  fn CVPixelBufferGetHeightOfPlane(buffer: *const c_void, plane: usize) -> usize;
}
