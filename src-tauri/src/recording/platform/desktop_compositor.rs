// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GPU composition for macOS regions that cross display boundaries. Each
//! display's IOSurface-backed frame is sampled in place, and the pieces are
//! drawn straight into the IOSurface of a new buffer for the encoder.

use cidre::{arc, cf, cv};

use crate::desktop_capture::{CapturePiece, CapturePlan, FrameSynchronizer, PixelRect};
use crate::gpu::Gpu;
use crate::recording::desktop_canvas::{DesktopCanvas, PieceSource};

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
      compositor: DesktopCompositor::new(plan.width, plan.height, plan.pieces.len())?,
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
      .map(|(frame, piece)| {
        let pixels = &frame
          .as_ref()
          .expect("the synchronizer waits for every source")
          .1;
        (cropped_source(*piece), &**pixels)
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

/// ScreenCaptureKit has already applied `source_pixels` as its source
/// rectangle, so the delivered surface begins at this crop's origin.
fn cropped_source(piece: CapturePiece) -> CapturePiece {
  CapturePiece {
    source_pixels: PixelRect {
      x: 0,
      y: 0,
      ..piece.source_pixels
    },
    ..piece
  }
}

struct DesktopCompositor {
  gpu: &'static Gpu,
  width: u32,
  height: u32,
  drawing: DesktopCanvas,
}

impl DesktopCompositor {
  fn new(width: u32, height: u32, pieces: usize) -> Result<Self, String> {
    if width == 0 || height == 0 {
      return Err("The desktop canvas is empty".to_owned());
    }
    let gpu = crate::gpu::shared()?;
    Ok(Self {
      gpu,
      width,
      height,
      drawing: DesktopCanvas::new(gpu, width, height, pieces),
    })
  }

  /// Draws every piece into a new buffer and waits for the GPU: the encoder
  /// reads the buffer as soon as it is returned.
  fn compose(
    &self,
    frames: &[(CapturePiece, &cv::PixelBuf)],
  ) -> Result<arc::R<cv::PixelBuf>, String> {
    let output = canvas_buffer(self.width, self.height)?;
    let gpu = self.gpu;
    let canvas = crate::gpu::macos::bgra_buffer_texture(gpu, &output, "desktop canvas")?;
    let textures = frames
      .iter()
      .map(|(_, pixels)| crate::gpu::macos::bgra_buffer_texture(gpu, pixels, "desktop source"))
      .collect::<Result<Vec<_>, String>>()?;
    let sources = frames
      .iter()
      .zip(&textures)
      .map(|((piece, _), texture)| PieceSource {
        piece: *piece,
        texture,
      })
      .collect::<Vec<_>>();
    let submission = self
      .drawing
      .draw(&canvas.create_view(&Default::default()), &sources)?;
    gpu
      .device
      .poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: None,
      })
      .map_err(|error| format!("The GPU did not finish desktop composition: {error}"))?;
    Ok(output)
  }
}

/// A BGRA buffer the GPU can draw into and the encoder can read.
fn canvas_buffer(width: u32, height: u32) -> Result<arc::R<cv::PixelBuf>, String> {
  let empty = cf::Dictionary::with_keys_values(&[], &[])
    .ok_or_else(|| "CoreVideo could not describe the desktop canvas".to_owned())?;
  let attributes = cf::Dictionary::with_keys_values(
    &[cv::pixel_buffer::keys::io_surf_props().as_type_ref()],
    &[empty.as_type_ref()],
  )
  .ok_or_else(|| "CoreVideo could not describe the desktop canvas".to_owned())?;
  cv::PixelBuf::new(
    width as usize,
    height as usize,
    cv::PixelFormat::_32_BGRA,
    Some(&attributes),
  )
  .map_err(|error| format!("CoreVideo could not allocate the desktop canvas: {error:?}"))
}

#[cfg(test)]
mod tests;
