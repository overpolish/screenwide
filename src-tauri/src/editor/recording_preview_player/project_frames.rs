// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A saved recording's frame as its edit shows it, drawn without opening it
//! in the editor: its saved look, its saved edit and the annotations showing
//! then, composed as the editor composes its project's still. The camera is
//! drawn as it was recorded, its effects being the open editor's.

use std::path::Path;

use tauri::AppHandle;

use super::composed_frames::{compose_from, FrameLook};
use super::sources::project_sources;
use screenwide_imaging::CapturedImage;

/// The frame `source_ms` into the recording saved in the project whose
/// manifest is `project`, at most `max_width` wide. A recording last
/// edited before its look was kept in the project has none to draw with,
/// and shows its own picture, as its card does.
pub(crate) async fn compose_project_frame(
  app: &AppHandle,
  project: &Path,
  source_ms: u64,
  max_width: u32,
) -> Result<CapturedImage, String> {
  let sources = project_sources(app, project)?;
  let Some(look) = crate::editor::project_look::for_project(project) else {
    let (path, duration_ms) = (sources.screen_path.clone(), sources.duration_ms);
    return tauri::async_runtime::spawn_blocking(move || {
      screenwide_export::decode::source_frame_image(&path, source_ms, duration_ms)
        .and_then(|frame| screenwide_export::still_size::shrunk(frame, max_width))
    })
    .await
    .map_err(|error| error.to_string())?;
  };
  let mut recording_output = look.recording_output;
  recording_output.stamp_captures(sources.captures);
  let clips = sources
    .annotation_clips
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone();
  let ranges = sources
    .annotation_ranges
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone();
  let frame_look = FrameLook {
    bake_camera: look.bake_camera,
    camera_overlay: look.camera_overlay,
    cursor_effects: look.cursor_effects,
    keyboard_effects: look.keyboard_effects,
    recording_output,
  };
  compose_from(
    sources,
    frame_look,
    Some(clips),
    ranges,
    vec![source_ms],
    Some(max_width),
  )
  .await?
  .pop()
  .ok_or_else(|| "No frame was composed".to_owned())
}
