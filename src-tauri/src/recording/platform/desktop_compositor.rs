// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::{c_char, c_void};

use cidre::{arc, cv};

use crate::desktop_capture::{CapturePiece, CapturePlan, FrameSynchronizer, PixelRect};

#[repr(C)]
#[derive(Clone, Copy)]
struct NativePiece {
  source_x: u32,
  source_y: u32,
  source_width: u32,
  source_height: u32,
  destination_x: u32,
  destination_y: u32,
  destination_width: u32,
  destination_height: u32,
}

#[repr(C)]
struct NativeFrame {
  pixels: *const c_void,
  piece: NativePiece,
}

pub(super) struct ComposedFrame {
  pub buffer: arc::R<cv::PixelBuf>,
  pub timestamp_ns: i64,
}

pub(super) struct DesktopFrameCoordinator {
  compositor: DesktopCompositor,
  pieces: Vec<CapturePiece>,
  latest: Vec<Option<(i64, arc::R<cv::PixelBuf>)>>,
  synchronizer: FrameSynchronizer,
}

impl DesktopFrameCoordinator {
  pub fn new(plan: &CapturePlan) -> Result<Self, String> {
    Ok(Self {
      compositor: DesktopCompositor::new(plan.width, plan.height)?,
      pieces: plan.pieces.clone(),
      latest: vec![None; plan.pieces.len()],
      synchronizer: FrameSynchronizer::new(plan.pieces.len())?,
    })
  }

  pub fn update(
    &mut self,
    source_index: usize,
    timestamp_ns: i64,
    pixels: &cv::PixelBuf,
  ) -> Result<Option<ComposedFrame>, String> {
    let slot = self
      .latest
      .get_mut(source_index)
      .ok_or_else(|| "A frame arrived from an unknown desktop source".to_owned())?;
    if pixels.pixel_format() != cv::PixelFormat::_32_BGRA {
      return Err("A desktop capture source did not provide BGRA pixels".to_owned());
    }
    if timestamp_ns < 0 {
      return Err("A desktop frame has an invalid timestamp".to_owned());
    }
    if slot
      .as_ref()
      .is_some_and(|(latest_ns, _)| timestamp_ns <= *latest_ns)
    {
      return Ok(None);
    }
    let piece = self
      .pieces
      .get(source_index)
      .expect("latest frames and pieces have the same length");
    if pixels.width() < piece.source_pixels.width as usize
      || pixels.height() < piece.source_pixels.height as usize
    {
      return Err("A desktop capture source returned an undersized surface".to_owned());
    }
    *slot = Some((timestamp_ns, pixels.retained()));
    let Some(tick) = self.synchronizer.update(source_index, timestamp_ns)? else {
      return Ok(None);
    };
    let frames = self
      .latest
      .iter()
      .zip(&self.pieces)
      .map(|(frame, piece)| NativeFrame {
        pixels: frame
          .as_ref()
          .expect("the synchronizer waits for every source")
          .1
          .as_ref() as *const cv::PixelBuf as *const c_void,
        piece: NativePiece::from_cropped_source(*piece),
      })
      .collect::<Vec<_>>();
    Ok(Some(ComposedFrame {
      buffer: self.compositor.compose(&frames)?,
      timestamp_ns: tick.output_ns,
    }))
  }
}

// A coordinator is constructed and consumed by the dedicated composition
// worker. Retained CoreVideo buffers reach it by an ownership-transferring
// bounded channel and no other thread accesses them afterwards.
unsafe impl Send for DesktopFrameCoordinator {}

impl NativePiece {
  fn from_cropped_source(piece: CapturePiece) -> Self {
    let PixelRect {
      x: _,
      y: _,
      width: source_width,
      height: source_height,
    } = piece.source_pixels;
    let PixelRect {
      x: destination_x,
      y: destination_y,
      width: destination_width,
      height: destination_height,
    } = piece.destination;
    Self {
      // ScreenCaptureKit has already applied `source_pixels` as its source
      // rectangle, so the delivered texture begins at this crop's origin.
      source_x: 0,
      source_y: 0,
      source_width,
      source_height,
      destination_x,
      destination_y,
      destination_width,
      destination_height,
    }
  }
}

struct DesktopCompositor(*mut c_void);

impl DesktopCompositor {
  fn new(width: u32, height: u32) -> Result<Self, String> {
    let mut error = [0 as c_char; 256];
    let handle = unsafe {
      screenwide_desktop_compositor_create(width, height, error.as_mut_ptr(), error.len())
    };
    if handle.is_null() {
      return Err(error_text(&error));
    }
    Ok(Self(handle))
  }

  fn compose(&self, frames: &[NativeFrame]) -> Result<arc::R<cv::PixelBuf>, String> {
    let mut error = [0 as c_char; 256];
    let pixels = unsafe {
      screenwide_desktop_compositor_compose(
        self.0,
        frames.as_ptr(),
        frames.len(),
        error.as_mut_ptr(),
        error.len(),
      )
    };
    if pixels.is_null() {
      return Err(error_text(&error));
    }
    Ok(unsafe { arc::R::from_raw(pixels.cast::<cv::PixelBuf>()) })
  }
}

impl Drop for DesktopCompositor {
  fn drop(&mut self) {
    unsafe { screenwide_desktop_compositor_destroy(self.0) };
  }
}

fn error_text(error: &[c_char]) -> String {
  let bytes = error.iter().map(|byte| *byte as u8).collect::<Vec<_>>();
  String::from_utf8_lossy(bytes.split(|byte| *byte == 0).next().unwrap_or_default()).into_owned()
}

unsafe extern "C" {
  fn screenwide_desktop_compositor_create(
    width: u32,
    height: u32,
    error_text: *mut c_char,
    error_capacity: usize,
  ) -> *mut c_void;
  fn screenwide_desktop_compositor_compose(
    compositor: *mut c_void,
    frames: *const NativeFrame,
    frame_count: usize,
    error_text: *mut c_char,
    error_capacity: usize,
  ) -> *mut c_void;
  fn screenwide_desktop_compositor_destroy(compositor: *mut c_void);
}

#[cfg(test)]
mod tests;
