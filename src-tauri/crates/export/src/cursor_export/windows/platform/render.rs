// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::cursor_export::frame_clock;
use crate::decode::windows::gpu_decoder::GpuFrame;

/// A decoder and the frame in hand, with the next read ahead, so the frame
/// showing at any moment is the last one at or before it.
struct Reader {
  reader: GpuVideoReader,
  /// The first frame's timestamp, which every frame is measured from.
  origin_100ns: i64,
  current: Option<GpuFrame>,
  next: Option<GpuFrame>,
}

impl Reader {
  fn open(path: &Path, device: &ID3D11Device) -> Result<Self, String> {
    let mut reader = GpuVideoReader::open(path, 0, device.clone())?;
    let current = reader.next_frame()?;
    let next = reader.next_frame()?;
    Ok(Self {
      reader,
      origin_100ns: current.as_ref().map_or(0, |frame| frame.timestamp_100ns),
      current,
      next,
    })
  }

  fn time_us(&self, frame: &GpuFrame) -> u64 {
    u64::try_from(
      frame
        .timestamp_100ns
        .saturating_sub(self.origin_100ns)
        .max(0)
        / 10,
    )
    .unwrap_or_default()
  }

  /// Reads again from a little before `source_us`.
  fn restart_at(&mut self, source_us: u64) -> Result<(), String> {
    let start_us = source_us.saturating_sub(frame_clock::REWIND_PREROLL_US);
    let origin_ms = u64::try_from(self.origin_100ns.max(0) / 10_000).unwrap_or_default();
    self.reader.seek(origin_ms + start_us / 1_000, false)?;
    self.current = self.reader.next_frame()?;
    self.next = self.reader.next_frame()?;
    Ok(())
  }

  /// Moves to the frame showing at `source_us`, reading on to it or, where
  /// the shared clock says seeking costs less, starting again near it.
  fn show(&mut self, source_us: u64, frame_us: u64) -> Result<(), String> {
    let held = self.current.as_ref().map(|frame| self.time_us(frame));
    let next = self.next.as_ref().map(|frame| self.time_us(frame));
    if held.is_some_and(|held| frame_clock::should_reposition(source_us, held, next, frame_us)) {
      self.restart_at(source_us)?;
    }
    while self
      .next
      .as_ref()
      .is_some_and(|frame| self.time_us(frame) <= source_us)
    {
      self.current = self.next.take();
      self.next = self.reader.next_frame()?;
    }
    Ok(())
  }

  fn frame(&self) -> Option<(&ID3D11Texture2D, u32)> {
    self
      .current
      .as_ref()
      .map(|frame| (&frame.texture, frame.subresource))
  }
}

/// Draws and encodes `request`'s edited video into the first of `paths`,
/// and each of `beside` into the path after it from the same decoded
/// frames, or only `stretch` of it, on the shared output clock. Answers how
/// the run ended and how many frames it wrote.
pub(super) fn render_video<'r>(
  request: &mut CursorExportRequest<'r>,
  beside: &[CursorExportRequest<'r>],
  paths: &[PathBuf],
  stretch: Option<&media_preview::PreviewStretch>,
) -> Result<(ExportRunResult, u64), String> {
  screenwide_output::validate_output_settings(request.width, request.height, request.output)?;
  for format in beside {
    screenwide_output::validate_output_settings(format.width, format.height, format.output)?;
  }
  let formats = std::iter::once(&*request).chain(beside).collect::<Vec<_>>();
  if paths.len() != formats.len() {
    return Err("The export has the wrong number of files to write".to_owned());
  }
  let output_sizes = formats
    .iter()
    .map(|format| screenwide_output::output_dimensions(format.output))
    .collect::<Result<Vec<_>, _>>()?;
  let canvas = WindowsExportCanvas::new(
    (request.width, request.height),
    request
      .camera
      .map(|(_, options)| (options.camera_width, options.camera_height)),
    &output_sizes,
  )?;
  let device = canvas.device();
  let mut screen = Reader::open(request.screen, device)?;
  let mut camera = request
    .camera
    .map(|(path, _)| Reader::open(path, device))
    .transpose()?;
  let mut draws = formats
    .iter()
    .map(|format| {
      FrameDraw::new(
        format,
        canvas.gpu(),
        stretch.map(media_preview::PreviewStretch::source_span_ms),
      )
    })
    .collect::<Result<Vec<_>, _>>()?;
  let fps = frame_clock::frame_rate(
    media_preview::recording_info(request.screen).and_then(|info| info.frames_per_second),
  );
  let mut sinks = formats
    .iter()
    .zip(&output_sizes)
    .zip(paths)
    .map(|((format, &size), path)| {
      video_file(
        path,
        device,
        (size, fps),
        super::super::super::video_bitrate(size.0, size.1, format.video.compression),
      )
    })
    .collect::<Result<Vec<_>, _>>()?;
  let ranges = match stretch {
    Some(stretch) => stretch.ranges.clone(),
    None => request
      .timeline
      .map_or_else(Vec::new, |timeline| timeline.ranges().to_vec()),
  };
  let duration_us = frame_clock::duration_us(request.duration_ms.saturating_mul(1_000), &ranges);
  let frame_us = 1_000_000 / u64::from(fps);
  let frame_ms = 1_000.0 / fps as f32;
  let mut progress_pace = super::super::super::progress_pace::ProgressPace::default();
  let mut frames = 0_u64;
  for index in 0_u64.. {
    let output_us = index * 1_000_000 / u64::from(fps);
    if output_us >= duration_us {
      break;
    }
    if request.cancelled.load(Ordering::Acquire) {
      drop(sinks);
      for path in paths {
        let _ = std::fs::remove_file(path);
      }
      return Ok((ExportRunResult::Cancelled, frames));
    }
    let Some(source_us) = frame_clock::source_us(output_us, &ranges) else {
      continue;
    };
    // A gap's moments lie past the recording's end: nothing is read there,
    // and the frame in hand, which the gap hides, is composed at the gap's
    // own moment; a gap the video starts with takes the recording's first.
    let in_gap = source_us >= request.duration_ms.saturating_mul(1_000);
    let read_us = match (in_gap, screen.frame().is_some()) {
      (false, _) => Some(source_us),
      (true, false) => Some(0),
      (true, true) => None,
    };
    if let Some(read_us) = read_us {
      screen.show(read_us, frame_us)?;
      if let Some(camera) = camera.as_mut() {
        camera.show(read_us, frame_us)?;
      }
    }
    let Some(picture) = screen.frame() else {
      break;
    };
    let textures = canvas.compose(
      &mut draws,
      picture,
      camera.as_ref().and_then(Reader::frame),
      (source_us, frame_ms),
    )?;
    for (sink, texture) in sinks.iter_mut().zip(&textures) {
      sink.write(
        texture,
        i64::try_from(output_us.saturating_mul(10)).unwrap_or(i64::MAX),
        i64::try_from(frame_us * 10).unwrap_or(i64::MAX),
      )?;
    }
    frames += 1;
    let last = output_us + frame_us >= duration_us;
    if progress_pace.due(std::time::Instant::now(), last) {
      (request.on_progress)(
        output_us
          .div_ceil(1_000)
          .saturating_mul(GPU_PROGRESS_PERCENT)
          / 100,
      );
    }
  }
  for sink in &mut sinks {
    sink.finish()?;
  }
  Ok((ExportRunResult::Completed, frames))
}
