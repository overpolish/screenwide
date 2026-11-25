// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every format of a recording exported at once. The formats whose picture
//! the compositor draws come from one decode of the recording, as many at a
//! time as the platform encodes; the others, and exports with no picture to
//! draw, are written one after another. A format that comes out the same as
//! the one before it is a copy of that file.

mod named;

use named::named_after;

use super::export_job::{
  captioned, finish_screen, prepare, save_recording_artifact, screen_progress_share,
  write_subtitles, Prepared, Route, Subtitles,
};
use super::*;
use crate::editor::export_picture::Run;
use screenwide_export::cursor_export;
use screenwide_scenes::CameraOverlaySettings;

/// What one format does differently from the others in an export of every
/// format.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatExport {
  pub file_stem: String,
  pub recording_output: RecordingOutputSettings,
  pub camera_overlay: CameraOverlaySettings,
  #[serde(default)]
  pub timeline_edit: Option<screenwide_timeline_edit::RecordingTimelineEdit>,
  /// Written as a copy of the format before it, which comes out the same.
  #[serde(default)]
  pub is_copy: bool,
}

impl FormatExport {
  fn options(&self, shared: &RecordingExportOptions) -> RecordingExportOptions {
    RecordingExportOptions {
      camera_overlay: self.camera_overlay,
      recording_output: self.recording_output.clone(),
      timeline_edit: self.timeline_edit.clone(),
      ..shared.clone()
    }
  }
}

/// A format prepared to be drawn with the others.
struct Drawn {
  index: usize,
  prepared: Prepared,
  subtitles: Option<Subtitles>,
  destination: PathBuf,
}

/// Exports every format of a recording at once: `options` as each format
/// shares them, and `formats` what each does differently, in the order
/// their files are written. A camera saved as a file of its own is written
/// once, named after `file_stem`. The finished island shows the last file.
#[tauri::command]
pub async fn save_format_exports(
  app: AppHandle,
  window: tauri::WebviewWindow,
  file_stem: String,
  options: RecordingExportOptions,
  formats: Vec<FormatExport>,
) -> Result<Option<super::export_command::ExportedFile>, String> {
  if formats.is_empty() {
    return Err("There is no format to export".to_owned());
  }
  let work: super::export_command::Work =
    Box::new(move |artifact, writing, app, cancelled, options| {
      save_formats(
        artifact,
        (writing, &file_stem),
        app,
        cancelled,
        (&options, &formats),
      )
    });
  super::export_command::run_export(app, window, options, work).await
}

/// Exports `formats`, each with `shared` and what it does differently,
/// answering every file written in their order, or none where cancelled.
/// One cancelled or failed takes back every file the export made, whichever
/// step made it: videos, cameras of their own, subtitles and copies.
pub(super) fn save_formats(
  artifact: &EditorArtifact,
  (writing, file_stem): (&Path, &str),
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
  (shared, formats): (&RecordingExportOptions, &[FormatExport]),
) -> Result<Option<Vec<PathBuf>>, String> {
  let unusable = || "That file name cannot be used".to_owned();
  let stem = sanitize_file_stem(file_stem).ok_or_else(unusable)?;
  let stems = formats
    .iter()
    .map(|format| sanitize_file_stem(&format.file_stem).ok_or_else(unusable))
    .collect::<Result<Vec<_>, _>>()?;
  let names = [stems.as_slice(), std::slice::from_ref(&stem)].concat();
  let before = named_after(writing, &names);
  let result = write_formats(
    artifact,
    (writing, &stem, &stems),
    progress_app,
    cancelled,
    (shared, formats),
  );
  if !matches!(result, Ok(Some(_))) {
    for path in named_after(writing, &names).difference(&before) {
      let _ = std::fs::remove_file(path);
    }
  }
  result
}

fn write_formats(
  artifact: &EditorArtifact,
  (writing, stem, stems): (&Path, &str, &[String]),
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
  (shared, formats): (&RecordingExportOptions, &[FormatExport]),
) -> Result<Option<Vec<PathBuf>>, String> {
  let mut written: Vec<Option<PathBuf>> = vec![None; formats.len()];
  if !draw_together(
    artifact,
    (writing, stem, stems),
    progress_app,
    cancelled,
    (shared, formats),
    &mut written,
  )? {
    return Ok(None);
  }
  for (index, format) in formats.iter().enumerate() {
    if written[index].is_some() {
      continue;
    }
    let stem = &stems[index];
    if format.is_copy {
      let Some(source) = written[..index].iter().rev().flatten().next() else {
        return Err(format!("{stem}: there is no file before it to copy"));
      };
      let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
      let path = unique_path(writing, stem, extension, &|candidate| candidate.exists());
      std::fs::copy(source, &path).map_err(|error| format!("{stem}: {error}"))?;
      written[index] = Some(path);
      continue;
    }
    let saved = save_recording_artifact(
      artifact,
      writing,
      stem,
      progress_app,
      cancelled,
      format.options(shared),
    )
    .map_err(|error| format!("{stem}: {error}"))?;
    let Some(saved) = saved else {
      return Ok(None);
    };
    written[index] = Some(saved);
  }
  Ok(Some(written.into_iter().flatten().collect()))
}

/// Draws every format the compositor draws from one decode of the
/// recording, filling their places in `written`. False where cancelled.
fn draw_together(
  artifact: &EditorArtifact,
  (writing, export_stem, stems): (&Path, &str, &[String]),
  progress_app: &AppHandle,
  cancelled: &AtomicBool,
  (shared, formats): (&RecordingExportOptions, &[FormatExport]),
  written: &mut [Option<PathBuf>],
) -> Result<bool, String> {
  let EditorArtifact::Recording { id, .. } = artifact else {
    return Ok(true);
  };
  // A GIF is drawn its own way, one format at a time.
  if shared.recording_format == crate::editor::recording_model::RecordingFormat::Gif {
    return Ok(true);
  }
  let mut claimed: Vec<PathBuf> = Vec::new();
  let mut drawn = Vec::new();
  for (index, format) in formats.iter().enumerate() {
    if format.is_copy {
      continue;
    }
    let (options, subtitles) = captioned(artifact, progress_app, format.options(shared));
    let prepared = prepare(artifact, progress_app, options)
      .map_err(|error| format!("{}: {error}", stems[index]))?;
    if prepared.drawn(artifact).is_none() {
      continue;
    }
    let destination = unique_path(writing, &stems[index], RECORDING_EXTENSION, &|candidate| {
      candidate.exists() || claimed.iter().any(|path| path == candidate)
    });
    claimed.push(destination.clone());
    drawn.push(Drawn {
      index,
      prepared,
      subtitles,
      destination,
    });
  }
  let Some(first) = drawn.first() else {
    return Ok(true);
  };
  let duration_ms = first.prepared.duration_ms(artifact);
  let share = screen_progress_share(&first.prepared, artifact);
  let passes = drawn.len().div_ceil(cursor_export::FORMATS_PER_PASS);
  for (pass, chunk) in drawn.chunks(cursor_export::FORMATS_PER_PASS).enumerate() {
    let start = share * pass as f64 / passes as f64;
    let mut reported = |processed_ms| {
      camera_save::emit_progress(
        progress_app,
        *id,
        "recording",
        processed_ms,
        duration_ms,
        start,
        share / passes as f64,
      );
    };
    let mut quiet = (1..chunk.len()).map(|_| |_: u64| {}).collect::<Vec<_>>();
    let progress = std::iter::once(&mut reported as &mut dyn FnMut(u64))
      .chain(quiet.iter_mut().map(|quiet| quiet as &mut dyn FnMut(u64)));
    let requests = chunk
      .iter()
      .zip(progress)
      .filter_map(|(format, on_progress)| {
        let picture = format.prepared.drawn(artifact)?;
        Some(picture.request(Run {
          cancelled,
          destination: &format.destination,
          on_progress,
        }))
      })
      .collect::<Vec<_>>();
    match cursor_export::export_formats(requests)? {
      screenwide_export::media_preview::ExportRunResult::Completed => {}
      screenwide_export::media_preview::ExportRunResult::Cancelled => return Ok(false),
    }
    for format in chunk {
      written[format.index] = Some(format.destination.clone());
    }
  }
  // A camera saved as a file of its own is written once, with the first
  // format's settings and named after the export itself, however many
  // formats there are: the camera is the same recording in each.
  for (turn, format) in drawn.into_iter().enumerate() {
    let stem = &stems[format.index];
    let camera = if turn == 0 {
      (Some(export_stem), (share, 99.0 - share))
    } else {
      (None, (99.0, 0.0))
    };
    let finished = match format.prepared.route {
      Route::Screen => finish_screen(
        &format.prepared,
        artifact,
        format.destination.clone(),
        writing,
        progress_app,
        cancelled,
        camera,
      ),
      _ if format.destination.is_file() => Ok(Some(format.destination.clone())),
      _ => Err("The exported recording did not finish publishing".to_owned()),
    }
    .map_err(|error| format!("{stem}: {error}"))?;
    let Some(video) = finished else {
      return Ok(false);
    };
    write_subtitles(&video, format.subtitles).map_err(|error| format!("{stem}: {error}"))?;
    written[format.index] = Some(video);
  }
  Ok(true)
}
