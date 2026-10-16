// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn export(_request: CursorExportRequest<'_>) -> Result<ExportRunResult, String> {
  Err("Cursor baking is not available on this platform yet".to_owned())
}

pub(super) fn estimate_video_bytes(_request: CursorExportRequest<'_>) -> Result<u64, String> {
  Err("Cursor baking is not available on this platform yet".to_owned())
}

pub(super) fn preview_frame(
  _request: CursorExportRequest<'_>,
  _output_us: u64,
  _picture: media_preview::PreviewPicture,
  _png: &Path,
) -> Result<ExportRunResult, String> {
  Err("Cursor baking is not available on this platform yet".to_owned())
}
