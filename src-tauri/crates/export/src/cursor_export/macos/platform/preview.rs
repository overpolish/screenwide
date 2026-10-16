// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The frame at `output_us` of `request`'s export, drawn and encoded at the
/// end of its stretch exactly as the export draws and encodes it, at `png`.
/// x264 writes the stretch as a raw stream, as it does for an estimate;
/// where FFmpeg cannot be started AVFoundation writes it, as the export would.
pub(in super::super::super) fn preview_frame(
  request: CursorExportRequest<'_>,
  output_us: u64,
  png: &Path,
) -> Result<ExportRunResult, String> {
  let output = export_output::scaled(&request);
  let mut request = CursorExportRequest {
    output: &output,
    on_progress: &mut *request.on_progress,
    ..request
  };
  let cursor_timeline = macos::native::evaluate(&request)?;
  let keyboard_timeline = macos::native::evaluate_keyboard(&request)?;
  let stretch = media_preview::PreviewStretch::leading_to(
    &super::super::super::edited_ranges(&request),
    output_us,
  )?;
  let fallback = gpu_video_path();
  let x264 = mux::X264Plan {
    output: mux::X264Output::Measured {
      path: media_preview::estimate_temp_path(),
      stretch_starts: None,
    },
    width: request.output.width,
    height: request.output.height,
    compression: request.video.compression,
    bitrate: super::super::super::video_bitrate(
      request.output.width,
      request.output.height,
      request.video.compression,
    ),
  };
  let cancelled = request.cancelled;
  let rendered = render_gpu_video(
    &mut request,
    cursor_timeline.as_ref(),
    keyboard_timeline.as_ref(),
    (&stretch.ranges, Some(stretch.source_span_ms())),
    &fallback,
    x264,
  );
  let result = match rendered {
    Ok((ExportRunResult::Completed, Some(stream))) => match stream.measure(cancelled) {
      Ok(Some(encoded)) => {
        let raw = media_preview::preview_temp_path("h264");
        let written = std::fs::write(&raw, encoded)
          .map_err(|error| error.to_string())
          .and_then(|()| media_preview::write_last_frame(&raw, Some("h264"), png));
        let _ = std::fs::remove_file(&raw);
        written.map(|()| ExportRunResult::Completed)
      }
      Ok(None) => Ok(ExportRunResult::Cancelled),
      Err(error) => Err(error),
    },
    Ok((ExportRunResult::Completed, None)) => {
      media_preview::write_last_frame(&fallback, None, png).map(|()| ExportRunResult::Completed)
    }
    Ok((ExportRunResult::Cancelled, _)) => Ok(ExportRunResult::Cancelled),
    Err(error) => Err(error),
  };
  let _ = std::fs::remove_file(&fallback);
  result
}
