// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The frame at `output_us` of `request`'s export, drawn and encoded at the
/// end of its stretch exactly as the export draws and encodes it, at `png`.
/// A drawn `picture` is encoded too: Media Foundation turns the frame from
/// RGB into YUV, and a frame read back from the compositor would skip the
/// colour conversion its encoded counterpart goes through.
pub(in super::super::super) fn preview_frame(
  request: CursorExportRequest<'_>,
  output_us: u64,
  _picture: media_preview::PreviewPicture,
  png: &Path,
) -> Result<ExportRunResult, String> {
  let output = export_output::scaled(&request);
  let mut request = CursorExportRequest {
    output: &output,
    on_progress: &mut *request.on_progress,
    ..request
  };
  let stretch = media_preview::PreviewStretch::leading_to(
    &super::super::super::edited_ranges(&request),
    output_us,
  )?;
  let video = gpu_video_path();
  let result = render_video(&mut request, &video, Some(&stretch)).and_then(|(rendered, frames)| {
    if rendered == ExportRunResult::Completed {
      media_preview::write_last_frame(&video, None, frames, png)?;
    }
    Ok(rendered)
  });
  let _ = std::fs::remove_file(&video);
  result
}
