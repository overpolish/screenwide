// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::captions;

/// Exports the recording as [`save_recording_media`] does, with its captions
/// drawn into the picture and the subtitle file the export asks for beside
/// the video, where it has a transcript to make one from. Both are made
/// before the video; the file is written once the video is, and a video
/// whose subtitles cannot be written is taken back, as any half-finished
/// export is.
pub(in crate::editor::save) fn save_recording_artifact(
  artifact: &EditorArtifact,
  writing: &Path,
  stem: &str,
  progress_app: &AppHandle,
  job_cancellation: &AtomicBool,
  mut options: RecordingExportOptions,
) -> Result<Option<PathBuf>, String> {
  let mut subtitles = None;
  if let EditorArtifact::Recording {
    audio_tracks,
    duration_ms,
    id,
    project,
    ..
  } = artifact
  {
    if let Some(edit) =
      timeline_edit::export_edit(options.timeline_edit.as_ref(), project, *id, *duration_ms)
    {
      let recording = (project.as_path(), audio_tracks.as_slice(), *duration_ms);
      let output = &options.recording_output.primary;
      let frame = captions::frame_for(output.width, output.height);
      if let Some(kind) = options.subtitles.file() {
        let default = || crate::editor::text_templates::default_captions(progress_app);
        subtitles = captions::subtitles_for(recording, &edit, frame, kind, default);
      }
      options.captions = captions::track_for(recording, &edit, frame);
    }
  }
  let saved = save_recording_media(
    artifact,
    writing,
    stem,
    progress_app,
    job_cancellation,
    options,
  )?;
  if let (Some(video), Some(subtitles)) = (saved.as_ref(), subtitles) {
    if let Err(error) = captions::write_subtitles(video, subtitles) {
      let _ = std::fs::remove_file(video);
      return Err(error);
    }
  }
  Ok(saved)
}
