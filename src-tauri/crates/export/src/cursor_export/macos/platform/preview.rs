// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The frame at `output_us` of `request`'s export, at `png`, as `picture`
/// asks: drawn and encoded at the end of its stretch exactly as the export
/// draws and encodes it, x264 writing the stretch as a raw stream as it does
/// for an estimate, or drawn alone and never encoded.
pub(in super::super::super) fn preview_frame(
  request: CursorExportRequest<'_>,
  output_us: u64,
  picture: media_preview::PreviewPicture,
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
  let whole = super::super::super::edited_ranges(&request);
  let (stretch, output_plan) = match picture {
    media_preview::PreviewPicture::Encoded => (
      media_preview::PreviewStretch::leading_to(&whole, output_us)?,
      mux::X264Output::Measured {
        path: media_preview::estimate_temp_path(),
        stretch_starts: None,
      },
    ),
    media_preview::PreviewPicture::Drawn => (
      media_preview::PreviewStretch::single(&whole, output_us)?,
      mux::X264Output::Picture {
        path: media_preview::preview_temp_path("ppm"),
      },
    ),
  };
  // AVFoundation writes here only where FFmpeg cannot be started, which
  // the preview then cannot read either.
  let fallback = gpu_video_path();
  let x264 = mux::X264Plan {
    output: output_plan,
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
  let _ = std::fs::remove_file(&fallback);
  let written = match rendered? {
    (ExportRunResult::Completed, Some(stream)) => stream.measure(cancelled)?,
    (ExportRunResult::Completed, None) => {
      return Err("The preview needs FFmpeg, which could not be started".to_owned());
    }
    (ExportRunResult::Cancelled, _) => None,
  };
  let Some(written) = written else {
    return Ok(ExportRunResult::Cancelled);
  };
  match picture {
    media_preview::PreviewPicture::Encoded => {
      let raw = media_preview::preview_temp_path("h264");
      let frames = media_preview::SampleFrames::of(&written).frames();
      let result = std::fs::write(&raw, written)
        .map_err(|error| error.to_string())
        .and_then(|()| media_preview::write_last_frame(&raw, Some("h264"), frames, png));
      let _ = std::fs::remove_file(&raw);
      result?;
    }
    media_preview::PreviewPicture::Drawn => media_preview::write_picture(&written, png)?,
  }
  Ok(ExportRunResult::Completed)
}
