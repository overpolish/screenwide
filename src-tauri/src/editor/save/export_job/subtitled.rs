// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::captions;

/// The subtitle file written beside a video: its kind and contents.
pub(in crate::editor::save) type Subtitles = (screenwide_text::captions::SubtitleFile, String);

/// `options` with the captions drawn into the picture, and the subtitle file
/// the export asks for beside the video, where the recording has a
/// transcript to make them from.
pub(in crate::editor::save) fn captioned(
  artifact: &EditorArtifact,
  progress_app: &AppHandle,
  mut options: RecordingExportOptions,
) -> (RecordingExportOptions, Option<Subtitles>) {
  let mut subtitles = None;
  let is_gif = options.recording_format == crate::editor::recording_model::RecordingFormat::Gif;
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
      // A GIF has its captions in the picture, and no file beside it.
      if let Some(kind) = options.subtitles.file().filter(|_| !is_gif) {
        let default = || crate::editor::text_templates::default_captions(progress_app);
        subtitles = captions::subtitles_for(recording, &edit, frame, kind, default);
      }
      options.captions = captions::track_for(recording, &edit, frame);
    }
  }
  (options, subtitles)
}

/// Writes `subtitles` beside `video`, taking the video back where they
/// cannot be written, as any half-finished export is.
pub(in crate::editor::save) fn write_subtitles(
  video: &Path,
  subtitles: Option<Subtitles>,
) -> Result<(), String> {
  let Some(subtitles) = subtitles else {
    return Ok(());
  };
  captions::write_subtitles(video, subtitles)
    .map(|_| ())
    .inspect_err(|_| {
      let _ = std::fs::remove_file(video);
    })
}

/// Exports the recording as [`save_recording_media`] does, with its captions
/// drawn into the picture and the subtitle file the export asks for beside
/// the video. Both are made before the video; the file is written once the
/// video is.
pub(in crate::editor::save) fn save_recording_artifact(
  artifact: &EditorArtifact,
  writing: &Path,
  stem: &str,
  progress_app: &AppHandle,
  job_cancellation: &AtomicBool,
  options: RecordingExportOptions,
) -> Result<Option<PathBuf>, String> {
  let (options, subtitles) = captioned(artifact, progress_app, options);
  if options.recording_format == crate::editor::recording_model::RecordingFormat::Gif {
    return super::gif::save_gif(
      artifact,
      writing,
      stem,
      progress_app,
      job_cancellation,
      options,
    );
  }
  let saved = save_recording_media(
    artifact,
    writing,
    stem,
    progress_app,
    job_cancellation,
    options,
  )?;
  if let Some(video) = saved.as_ref() {
    write_subtitles(video, subtitles)?;
  }
  Ok(saved)
}
