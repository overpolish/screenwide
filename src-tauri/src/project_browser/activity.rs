// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happened in a recording, read from its project alone, so the browser
//! can draw a card's beats and lulls as the editor's strip does and search
//! what was said and flagged: the same readings the editor finds its beats
//! in, without opening it. Each file is read only if it is there; a
//! recording without one has none of it.

use std::path::{Path, PathBuf};

use serde::Serialize;
use ts_rs::TS;

use screenwide_speech::transcript::RecordingTranscriptWord;
use screenwide_timeline_edit::RecordingTimelineEdit;

use crate::editor::recording_moments::activity::{self, RecordingActivity};
use crate::editor::speech::transcript::read_cache;

/// A moment flagged while recording, as the browser searches it.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectMoment {
  /// The kind's colour as it was, `#rrggbb`.
  pub color: String,
  /// Which moment it is, counting from zero in the order they were placed.
  pub index: usize,
  pub name: String,
  /// What its voice note says, once transcribed.
  pub note: Option<String>,
  /// On the recording's own timeline, before any edit.
  pub source_ms: f64,
}

/// A recording's readings, for its card and for search.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectActivity {
  pub activity: RecordingActivity,
  /// How long it was recorded.
  pub duration_ms: u64,
  /// How it plays now; None while it plays as recorded.
  pub edit: Option<RecordingTimelineEdit>,
  pub moments: Vec<ProjectMoment>,
  /// What the microphone said, once transcribed.
  pub words: Vec<RecordingTranscriptWord>,
}

/// The readings of the recording whose manifest is `file`; None for a
/// screenshot, or a project that cannot be read.
#[tauri::command]
pub async fn get_project_activity(file: PathBuf) -> Option<ProjectActivity> {
  tauri::async_runtime::spawn_blocking(move || read(&file))
    .await
    .ok()
    .flatten()
}

fn read(file: &Path) -> Option<ProjectActivity> {
  let mut manifest = screenwide_project::read(file).ok()?;
  let timeline = manifest.timeline.take();
  let recording = manifest.recorded().ok()?;
  let duration_ms = recording.duration_ms?;
  let media = &recording.media;
  let resolve = |name: &Option<String>| {
    name
      .as_deref()
      .and_then(|name| screenwide_project::resolve(file, name).ok())
  };
  let movie = screenwide_project::resolve(file, &media.primary).ok()?;
  let cursor = resolve(&media.cursor);
  let keyboard = resolve(&media.keyboard);
  // The microphone follows the system audio among the movie's audio
  // streams, as the editor numbers them.
  let words = recording
    .has_microphone
    .then(|| {
      file
        .parent()
        .and_then(|folder| read_cache(folder, usize::from(recording.has_system_audio)))
    })
    .flatten()
    .map(|transcript| transcript.words)
    .unwrap_or_default();
  let moments = resolve(&media.moments)
    .and_then(|path| screenwide_moments::read(&path).ok())
    .unwrap_or_default()
    .into_iter()
    .enumerate()
    .map(|(index, moment)| ProjectMoment {
      color: moment.color,
      index,
      name: moment.name,
      note: moment.transcript.filter(|text| !text.trim().is_empty()),
      source_ms: moment.timestamp_us as f64 / 1_000.0,
    })
    .collect();
  let edit = timeline.and_then(|persisted| persisted.into_current(duration_ms));
  Some(ProjectActivity {
    activity: activity::read(cursor.as_deref(), keyboard.as_deref(), &movie),
    duration_ms,
    edit,
    moments,
    words,
  })
}
