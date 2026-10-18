// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Real exports, read back frame by frame: the cursor, the keyboard, the
//! captions, the scenes, the camera and every timed annotation, drawn by the
//! shared frame drawer on whichever platform decodes and encodes them.

use std::path::PathBuf;
use std::sync::Mutex;

use super::*;

/// [`super::export`], one at a time across the tests. A hardware encoder
/// takes only a few sessions at once, and an app exports one video at a time
/// anyway, so the tests do too rather than failing on a busy encoder.
fn export(request: CursorExportRequest<'_>) -> Result<ExportRunResult, String> {
  static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
  let _turn = ONE_AT_A_TIME
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner);
  super::export(request)
}

mod camera_background_video_tests;
mod camera_effect_video_tests;
mod caption_video_tests;
mod cursor_order_video_tests;
mod highlight_tests;
mod keyboard_tests;
mod magnify_video_tests;
mod pinned_annotation_tests;
mod recenter_tests;
mod redact_video_tests;
mod reordered_video_tests;
mod scene_custom_video_tests;
mod scene_layout_video_tests;
mod scene_video_tests;
mod spotlight_video_tests;
mod tests;
mod timed_annotation_tests;
