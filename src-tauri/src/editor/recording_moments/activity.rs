// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What happened over a recording, for the timeline to find its beats in and
//! the project browser to draw and search: the clicks and typing the cursor
//! and key files hold, and what the picture did, from the readings the
//! capture wrote beside the movie. Each is read only if its file is there; a
//! recording made without one has none of it.

use std::path::{Path, PathBuf};

use screenwide_activity::PictureState;
use screenwide_records::cursor::{ButtonState, CursorRecord};
use screenwide_records::keyboard::KeyboardRecord;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::editor::{EditorArtifact, EditorState};

/// A press of the pointer, when it went down.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingClick {
  /// 2 for a double click, and so on.
  pub count: u8,
  /// On the recording's own timeline, before any edit.
  pub ms: u64,
}

/// A stretch where the picture was not at rest.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingPictureSpan {
  pub end_ms: u64,
  pub start_ms: u64,
  /// `moving` where something moved by itself, `new` where much of the
  /// picture was new: another page, another app.
  #[ts(type = "\"moving\" | \"new\"")]
  pub state: &'static str,
}

#[derive(Clone, Debug, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingActivity {
  pub clicks: Vec<RecordingClick>,
  pub picture: Vec<RecordingPictureSpan>,
  /// When text was typed, sorted; the key file says no more than that.
  pub typing: Vec<u64>,
}

struct Sources {
  cursor: Option<PathBuf>,
  keyboard: Option<PathBuf>,
  movie: PathBuf,
}

fn sources(app: &AppHandle, artifact_id: u64) -> Result<Sources, String> {
  let state = app.state::<EditorState>();
  let artifact = state
    .recording
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(EditorArtifact::Recording {
    cursor,
    id,
    keyboard,
    path,
    ..
  }) = artifact.as_ref()
  else {
    return Err("There is no recording open".to_owned());
  };
  if *id != artifact_id {
    return Err("That recording is no longer available in the editor".to_owned());
  }
  Ok(Sources {
    cursor: cursor.as_ref().map(|cursor| cursor.path.clone()),
    keyboard: keyboard.as_ref().map(|keyboard| keyboard.path.clone()),
    movie: path.clone(),
  })
}

#[tauri::command]
pub async fn get_recording_activity(
  app: AppHandle,
  artifact_id: u64,
) -> Result<RecordingActivity, String> {
  let sources = sources(&app, artifact_id)?;
  tauri::async_runtime::spawn_blocking(move || {
    read(
      sources.cursor.as_deref(),
      sources.keyboard.as_deref(),
      &sources.movie,
    )
  })
  .await
  .map_err(|error| error.to_string())
}

/// A file that cannot be read gives nothing rather than failing the rest:
/// each reading stands on its own.
pub(crate) fn read(
  cursor: Option<&Path>,
  keyboard: Option<&Path>,
  movie: &Path,
) -> RecordingActivity {
  RecordingActivity {
    clicks: cursor
      .and_then(|path| screenwide_records::cursor::read(path).ok())
      .map(|records| clicks(&records))
      .unwrap_or_default(),
    picture: picture(movie),
    typing: keyboard
      .and_then(|path| screenwide_records::keyboard::read(path).ok())
      .map(|records| typing(&records))
      .unwrap_or_default(),
  }
}

fn clicks(records: &[CursorRecord]) -> Vec<RecordingClick> {
  let mut clicks: Vec<RecordingClick> = records
    .iter()
    .filter_map(|record| match record {
      CursorRecord::Button {
        click_count,
        state: ButtonState::Down,
        timestamp_us,
        ..
      } => Some(RecordingClick {
        count: *click_count,
        ms: timestamp_us / 1_000,
      }),
      _ => None,
    })
    .collect();
  clicks.sort_by_key(|click| click.ms);
  clicks
}

fn typing(records: &[KeyboardRecord]) -> Vec<u64> {
  let mut typing: Vec<u64> = records
    .iter()
    .filter_map(|record| match record {
      KeyboardRecord::Typing { timestamp_us } => Some(timestamp_us / 1_000),
      _ => None,
    })
    .collect();
  typing.sort_unstable();
  typing
}

/// The picture's stretches away from rest, one per run of readings in the
/// same state.
fn picture(movie: &Path) -> Vec<RecordingPictureSpan> {
  let Ok(readings) = screenwide_activity::read_readings(movie) else {
    return Vec::new();
  };
  let states = screenwide_activity::states(&readings);
  let mut spans: Vec<RecordingPictureSpan> = Vec::new();
  for (index, (reading, state)) in readings.iter().zip(&states).enumerate() {
    let name = match state {
      PictureState::Still => continue,
      PictureState::Moving => "moving",
      PictureState::New => "new",
    };
    let end_ms = readings
      .get(index + 1)
      .map_or(reading.ms + 100, |next| next.ms);
    match spans.last_mut() {
      Some(last) if last.state == name && last.end_ms >= reading.ms => last.end_ms = end_ms,
      _ => spans.push(RecordingPictureSpan {
        end_ms,
        start_ms: reading.ms,
        state: name,
      }),
    }
  }
  spans
}
