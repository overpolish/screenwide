// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The frame at `output_us` of `request`'s export, drawn and encoded at the
/// end of its stretch exactly as the export draws and encodes it, at `png`.
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
  let stretch = media_preview::PreviewStretch::leading_to(
    &super::super::super::edited_ranges(&request),
    output_us,
  )?;
  let video = gpu_video_path();
  let result = render_video(&mut request, &video, Some(&stretch)).and_then(|rendered| {
    if rendered == ExportRunResult::Completed {
      media_preview::write_last_frame(&video, None, png)?;
    }
    Ok(rendered)
  });
  let _ = std::fs::remove_file(&video);
  result
}
